#include "permutation_widget.hpp"
#include "../proving_key/proving_key.hpp"
#include "../public_inputs/public_inputs.hpp"
#include "../utils/linearizer.hpp"
#include <plonk/transcript/transcript.hpp>
#include <polynomials/iterate_over_domain.hpp>
#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include <polynomials/polynomial_arithmetic.hpp>

using namespace barretenberg;

namespace waffle {

template <size_t program_width>
ProverPermutationWidget<program_width>::ProverPermutationWidget(proving_key* input_key, program_witness* input_witness)
    : ProverBaseWidget(input_key, input_witness)
{}

template <size_t program_width>
ProverPermutationWidget<program_width>::ProverPermutationWidget(const ProverPermutationWidget& other)
    : ProverBaseWidget(other)
{}

template <size_t program_width>
ProverPermutationWidget<program_width>::ProverPermutationWidget(ProverPermutationWidget&& other)
    : ProverBaseWidget(other)
{}

template <size_t program_width>
ProverPermutationWidget<program_width>& ProverPermutationWidget<program_width>::operator=(
    const ProverPermutationWidget& other)
{
    ProverBaseWidget::operator=(other);
    return *this;
}

template <size_t program_width>
ProverPermutationWidget<program_width>& ProverPermutationWidget<program_width>::operator=(
    ProverPermutationWidget&& other)
{
    ProverBaseWidget::operator=(other);
    return *this;
}

template <size_t program_width>
void ProverPermutationWidget<program_width>::compute_round_commitments(transcript::Transcript& transcript,
                                                                       const size_t round_number,
                                                                       work_queue& queue)
{
    if (round_number != 2) {
        return;
    }
    const size_t n = key->n;
    polynomial& z = witness->wires.at("z");
    polynomial& z_fft = key->wire_ffts.at("z_fft");

    fr* accumulators[(program_width == 1) ? 3 : program_width * 2];
    accumulators[0] = &z[1];
    accumulators[1] = &z_fft[0];
    accumulators[2] = &z_fft[n];

    if constexpr (program_width * 2 > 2) {
        accumulators[3] = &z_fft[n + n];
    }
    if constexpr (program_width > 2) {
        accumulators[4] = &z_fft[n + n + n];
        accumulators[5] = &key->opening_poly[0];
    }
    if constexpr (program_width > 3) {
        accumulators[6] = &key->shifted_opening_poly[0];
        accumulators[7] = &key->quotient_large[0];
    }
    if constexpr (program_width > 4) {
        accumulators[8] = &key->linear_poly[0];
        accumulators[9] = &key->quotient_large[n];
    }
    if constexpr (program_width > 5) {
        accumulators[10] = &key->quotient_large[n + n];
        accumulators[11] = &key->quotient_large[n + n + n];
    }
    for (size_t k = 7; k < program_width; ++k) {
        // we're out of temporary memory!
        accumulators[(k - 1) * 2] = static_cast<fr*>(aligned_alloc(64, sizeof(fr) * n));
        accumulators[(k - 1) * 2 + 1] = static_cast<fr*>(aligned_alloc(64, sizeof(fr) * n));
    }

    fr beta = fr::serialize_from_buffer(transcript.get_challenge("beta").begin());
    fr gamma = fr::serialize_from_buffer(transcript.get_challenge("beta", 1).begin());

    std::array<fr*, program_width> lagrange_base_wires;
    std::array<fr*, program_width> lagrange_base_sigmas;

    for (size_t i = 0; i < program_width; ++i) {
        lagrange_base_wires[i] = &key->wire_ffts.at("w_" + std::to_string(i + 1) + "_fft")[0];
        lagrange_base_sigmas[i] = &key->permutation_selectors_lagrange_base.at("sigma_" + std::to_string(i + 1))[0];
    }

#ifndef NO_MULTITHREADING
#pragma omp parallel
#endif
    {
#ifndef NO_MULTITHREADING
#pragma omp for
#endif
        for (size_t j = 0; j < key->small_domain.num_threads; ++j) {
            fr thread_root = key->small_domain.root.pow(static_cast<uint64_t>(j * key->small_domain.thread_size));
            fr work_root = thread_root * beta;
            fr T0;
            fr wire_plus_gamma;
            size_t start = j * key->small_domain.thread_size;
            size_t end = (j + 1) * key->small_domain.thread_size;
            for (size_t i = start; i < end; ++i) {
                wire_plus_gamma = gamma + lagrange_base_wires[0][i];
                accumulators[0][i] = wire_plus_gamma + work_root;

                T0 = lagrange_base_sigmas[0][i] * beta;
                accumulators[program_width][i] = T0 + wire_plus_gamma;

                for (size_t k = 1; k < program_width; ++k) {
                    wire_plus_gamma = gamma + lagrange_base_wires[k][i];
                    T0 = fr::coset_generator(k - 1) * work_root;
                    accumulators[k][i] = T0 + wire_plus_gamma;

                    T0 = lagrange_base_sigmas[k][i] * beta;
                    accumulators[k + program_width][i] = T0 + wire_plus_gamma;
                }

                work_root *= key->small_domain.root;
            }
        }

        // step 2: compute the constituent components of Z(X). This is a small multithreading bottleneck, as we have
        // program_width * 2 non-parallelizable processes
#ifndef NO_MULTITHREADING
#pragma omp for
#endif
        for (size_t i = 0; i < program_width * 2; ++i) {
            fr* coeffs = &accumulators[i][0];
            for (size_t j = 0; j < key->small_domain.size - 1; ++j) {
                coeffs[j + 1] *= coeffs[j];
            }
        }

        // step 3: concatenate together the accumulator elements into Z(X)
#ifndef NO_MULTITHREADING
#pragma omp for
#endif
        for (size_t j = 0; j < key->small_domain.num_threads; ++j) {
            const size_t start = j * key->small_domain.thread_size;
            const size_t end =
                ((j + 1) * key->small_domain.thread_size) - ((j == key->small_domain.num_threads - 1) ? 1 : 0);
            fr inversion_accumulator = fr::one();
            constexpr size_t inversion_index = (program_width == 1) ? 2 : program_width * 2 - 1;
            fr* inversion_coefficients = &accumulators[inversion_index][0];
            for (size_t i = start; i < end; ++i) {

                for (size_t k = 1; k < program_width; ++k) {
                    accumulators[0][i] *= accumulators[k][i];
                    accumulators[program_width][i] *= accumulators[program_width + k][i];
                }
                inversion_coefficients[i] = accumulators[0][i] * inversion_accumulator;
                inversion_accumulator *= accumulators[program_width][i];
            }
            inversion_accumulator = inversion_accumulator.invert();
            for (size_t i = end - 1; i != start - 1; --i) {

                // N.B. accumulators[0][i] = z[i + 1]
                // We can avoid fully reducing z[i + 1] as the inverse fft will take care of that for us
                accumulators[0][i] = inversion_accumulator * inversion_coefficients[i];
                inversion_accumulator *= accumulators[program_width][i];
            }
        }
    }
    z[0] = fr::one();
    z.ifft(key->small_domain);
    for (size_t k = 7; k < program_width; ++k) {
        aligned_free(accumulators[(k - 1) * 2]);
        aligned_free(accumulators[(k - 1) * 2 + 1]);
    }

    queue.add_to_queue({
        work_queue::WorkType::SCALAR_MULTIPLICATION,
        z.get_coefficients(),
        "Z",
    });
    queue.add_to_queue({
        work_queue::WorkType::FFT,
        nullptr,
        "z",
    });
}

template <size_t program_width>
fr ProverPermutationWidget<program_width>::compute_quotient_contribution(const fr& alpha_base,
                                                                         const transcript::Transcript& transcript)
{
    polynomial& z_fft = key->wire_ffts.at("z_fft");

    fr alpha_squared = alpha_base.sqr();
    fr beta = fr::serialize_from_buffer(transcript.get_challenge("beta").begin());
    fr gamma = fr::serialize_from_buffer(transcript.get_challenge("beta", 1).begin());

    // Our permutation check boils down to two 'grand product' arguments,
    // that we represent with a single polynomial Z(X).
    // We want to test that Z(X) has been constructed correctly.
    // When evaluated at elements of w \in H, the numerator of Z(w) will equal the
    // identity permutation grand product, and the denominator will equal the copy permutation grand product.

    // The identity that we need to evaluate is: Z(X.w).(permutation grand product) = Z(X).(identity grand product)
    // i.e. The next element of Z is equal to the current element of Z, multiplied by (identity grand product) /
    // (permutation grand product)

    // This method computes `Z(X).(identity grand product).{alpha}`.
    // The random `alpha` is there to ensure our grand product polynomial identity is linearly independent from the
    // other polynomial identities that we are going to roll into the quotient polynomial T(X).

    // Specifically, we want to compute:
    // (w_l(X) + \beta.sigma1(X) + \gamma).(w_r(X) + \beta.sigma2(X) + \gamma).(w_o(X) + \beta.sigma3(X) +
    // \gamma).Z(X).alpha Once we divide by the vanishing polynomial, this will be a degree 3n polynomial.

    std::array<fr*, program_width> wire_ffts;
    std::array<fr*, program_width> sigma_ffts;

    for (size_t i = 0; i < program_width; ++i) {
        wire_ffts[i] = &key->wire_ffts.at("w_" + std::to_string(i + 1) + "_fft")[0];
        sigma_ffts[i] = &key->permutation_selector_ffts.at("sigma_" + std::to_string(i + 1) + "_fft")[0];
    }

    const polynomial& l_1 = key->lagrange_1;

    // compute our public input component
    std::vector<barretenberg::fr> public_inputs =
        barretenberg::fr::from_buffer(transcript.get_element("public_inputs"));

    fr public_input_delta = compute_public_input_delta(public_inputs, beta, gamma, key->small_domain.root);

    polynomial& quotient_large = key->quotient_large;
    // Step 4: Set the quotient polynomial to be equal to
    // (w_l(X) + \beta.sigma1(X) + \gamma).(w_r(X) + \beta.sigma2(X) + \gamma).(w_o(X) + \beta.sigma3(X) +
    // \gamma).Z(X).alpha
#ifndef NO_MULTITHREADING
#pragma omp parallel for
#endif
    for (size_t j = 0; j < key->large_domain.num_threads; ++j) {
        const size_t start = j * key->large_domain.thread_size;
        const size_t end = (j + 1) * key->large_domain.thread_size;

        fr work_root = key->large_domain.root.pow(static_cast<uint64_t>(j * key->large_domain.thread_size));
        work_root *= key->small_domain.generator;
        // work_root *= fr::coset_generator(0);
        work_root *= beta;

        fr wire_plus_gamma;
        fr T0;
        fr denominator;
        fr numerator;
        for (size_t i = start; i < end; ++i) {
            wire_plus_gamma = gamma + wire_ffts[0][i];

            // Numerator computation
            numerator = work_root + wire_plus_gamma;

            // Denominator computation
            denominator = sigma_ffts[0][i] * beta;
            denominator += wire_plus_gamma;

            for (size_t k = 1; k < program_width; ++k) {
                wire_plus_gamma = gamma + wire_ffts[k][i];
                T0 = fr::coset_generator(k - 1) * work_root;
                T0 += wire_plus_gamma;
                numerator *= T0;

                T0 = sigma_ffts[k][i] * beta;
                T0 += wire_plus_gamma;
                denominator *= T0;
            }

            numerator *= z_fft[i];
            denominator *= z_fft[i + 4];

            /**
             * Permutation bounds check
             * (Z(X.w) - 1).(\alpha^3).L{n-1}(X) = T(X)Z_H(X)
             **/
            // The \alpha^3 term is so that we can subsume this polynomial into the quotient polynomial,
            // whilst ensuring the term is linearly independent form the other terms in the quotient polynomial

            // We want to verify that Z(X) equals `1` when evaluated at `w_n`, the 'last' element of our multiplicative
            // subgroup H. But PLONK's 'vanishing polynomial', Z*_H(X), isn't the true vanishing polynomial of subgroup
            // H. We need to cut a root of unity out of Z*_H(X), specifically `w_n`, for our grand product argument.
            // When evaluating Z(X) has been constructed correctly, we verify that Z(X.w).(identity permutation product)
            // = Z(X).(sigma permutation product), for all X \in H. But this relationship breaks down for X = w_n,
            // because Z(X.w) will evaluate to the *first* element of our grand product argument. The last element of
            // Z(X) has a dependency on the first element, so the first element cannot have a dependency on the last
            // element.

            // TODO: With the reduction from 2 Z polynomials to a single Z(X), the above no longer applies
            // TODO: Fix this to remove the (Z(X.w) - 1).L_{n-1}(X) check

            // To summarise, we can't verify claims about Z(X) when evaluated at `w_n`.
            // But we can verify claims about Z(X.w) when evaluated at `w_{n-1}`, which is the same thing

            // To summarise the summary: If Z(w_n) = 1, then (Z(X.w) - 1).L_{n-1}(X) will be divisible by Z_H*(X)
            // => add linearly independent term (Z(X.w) - 1).(\alpha^3).L{n-1}(X) into the quotient polynomial to check
            // this

            // z_fft already contains evaluations of Z(X).(\alpha^2)
            // at the (2n)'th roots of unity
            // => to get Z(X.w) instead of Z(X), index element (i+2) instead of i
            T0 = z_fft[i + 4] - public_input_delta; // T0 = (Z(X.w) - (delta)).(\alpha^2)
            T0 *= alpha_base;                       // T0 = (Z(X.w) - (delta)).(\alpha^3)
            T0 *= l_1[i + 8];                       // T0 = (Z(X.w)-delta).(\alpha^3).L{n-1}
            numerator += T0;

            // Step 2: Compute (Z(X) - 1).(\alpha^4).L1(X)
            // We need to verify that Z(X) equals `1` when evaluated at the first element of our subgroup H
            // i.e. Z(X) starts at 1 and ends at 1
            // The `alpha^4` term is so that we can add this as a linearly independent term in our quotient polynomial
            T0 = z_fft[i] - fr(1); // T0 = (Z(X) - 1).(\alpha^2)
            T0 *= alpha_squared;   // T0 = (Z(X) - 1).(\alpha^4)
            T0 *= l_1[i];          // T0 = (Z(X) - 1).(\alpha^2).L1(X)
            numerator += T0;

            // Combine into quotient polynomial
            T0 = numerator - denominator;
            quotient_large[i] = T0 * alpha_base;

            // Update our working root of unity
            work_root *= key->large_domain.root;
        }
    }
    return alpha_base.sqr().sqr();
}

template <size_t program_width>
fr ProverPermutationWidget<program_width>::compute_linear_contribution(const fr& alpha,
                                                                       const transcript::Transcript& transcript,
                                                                       polynomial& r)
{
    polynomial& z = witness->wires.at("z");
    fr z_challenge = fr::serialize_from_buffer(transcript.get_challenge("z").begin());

    barretenberg::polynomial_arithmetic::lagrange_evaluations lagrange_evals =
        barretenberg::polynomial_arithmetic::get_lagrange_evaluations(z_challenge, key->small_domain);

    fr alpha_cubed = alpha.sqr() * alpha;
    fr beta = fr::serialize_from_buffer(transcript.get_challenge("beta").begin());
    fr gamma = fr::serialize_from_buffer(transcript.get_challenge("beta", 1).begin());
    fr z_beta = z_challenge * beta;

    std::array<fr, program_width> wire_evaluations;
    for (size_t i = 0; i < program_width; ++i) {
        wire_evaluations[i] = fr::serialize_from_buffer(&transcript.get_element("w_" + std::to_string(i + 1))[0]);
    }

    fr z_1_shifted_eval = fr::serialize_from_buffer(&transcript.get_element("z_omega")[0]);

    fr T0;
    fr z_contribution = fr(1);
    for (size_t i = 0; i < program_width; ++i) {
        fr coset_generator = (i == 0) ? fr(1) : fr::coset_generator(i - 1);
        T0 = z_beta * coset_generator;
        T0 += wire_evaluations[i];
        T0 += gamma;
        z_contribution *= T0;
    }
    fr z_1_multiplicand = z_contribution * alpha;
    T0 = lagrange_evals.l_1 * alpha_cubed;
    z_1_multiplicand += T0;

    fr sigma_contribution = fr(1);
    for (size_t i = 0; i < program_width - 1; ++i) {
        fr permutation_evaluation =
            fr::serialize_from_buffer(&transcript.get_element("sigma_" + std::to_string(i + 1))[0]);
        T0 = permutation_evaluation * beta;
        T0 += wire_evaluations[i];
        T0 += gamma;
        sigma_contribution *= T0;
    }
    sigma_contribution *= z_1_shifted_eval;
    fr sigma_last_multiplicand = -(sigma_contribution * alpha);
    sigma_last_multiplicand *= beta;

    const polynomial& sigma_last = key->permutation_selectors.at("sigma_" + std::to_string(program_width));
    ITERATE_OVER_DOMAIN_START(key->small_domain);
    r[i] = (z[i] * z_1_multiplicand) + (sigma_last[i] * sigma_last_multiplicand);
    ITERATE_OVER_DOMAIN_END;

    return alpha.sqr().sqr();
}

template <size_t program_width>
size_t ProverPermutationWidget<program_width>::compute_opening_poly_contribution(
    const size_t nu_index,
    const transcript::Transcript& transcript,
    barretenberg::fr* opening_poly,
    barretenberg::fr* shifted_opening_poly,
    const bool use_linearisation)
{
    polynomial& z = witness->wires.at("z");

    const size_t num_sigma_evaluations = use_linearisation ? program_width - 1 : program_width;
    std::vector<fr*> sigmas(num_sigma_evaluations);
    for (size_t i = 0; i < num_sigma_evaluations; ++i) {
        sigmas[i] = &key->permutation_selectors.at("sigma_" + std::to_string(i + 1))[0];
    }

    size_t nu_shifted_z_offset = (use_linearisation) ? program_width - 1 : program_width + 1;
    std::vector<barretenberg::fr> nu_challenges(nu_shifted_z_offset + 1);
    for (size_t i = 0; i < nu_shifted_z_offset + 1; i++) {
        nu_challenges[i] = fr::serialize_from_buffer(&transcript.get_challenge("nu", nu_index + i)[0]);
    }

    ITERATE_OVER_DOMAIN_START(key->small_domain);

    fr T0;
    fr quotient_temp = fr::zero();

    for (size_t k = 0; k < program_width - 1; ++k) {
        T0 = sigmas[k][i] * nu_challenges[k];
        quotient_temp += T0;
    }

    if (!use_linearisation) {
        // TODO: fix overlapping nu_powers
        T0 = sigmas[program_width - 1][i] * nu_challenges[program_width - 1];
        quotient_temp += T0;
        T0 = z[i] * nu_challenges[program_width];
        quotient_temp += T0;
    }

    shifted_opening_poly[i] += z[i] * nu_challenges[nu_shifted_z_offset];

    opening_poly[i] += quotient_temp;

    ITERATE_OVER_DOMAIN_END;

    return nu_index + nu_shifted_z_offset + 1;
}

template <size_t program_width>
void ProverPermutationWidget<program_width>::compute_transcript_elements(transcript::Transcript& transcript,
                                                                         const bool use_linearisation)
{
    // iterate over permutations, skipping the last one as we use the linearisation trick to avoid including it in the
    // transcript
    const size_t n = key->n;
    fr z_challenge = fr::serialize_from_buffer(transcript.get_challenge("z").begin());
    fr shifted_z = z_challenge * key->small_domain.root;
    polynomial& z = witness->wires.at("z");

    for (size_t i = 0; i < program_width - 1; ++i) {
        std::string permutation_key = "sigma_" + std::to_string(i + 1);
        const polynomial& sigma = key->permutation_selectors.at(permutation_key);
        fr permutation_eval = sigma.evaluate(z_challenge, n);
        transcript.add_element(permutation_key, permutation_eval.to_buffer());
    }

    if (!use_linearisation) {
        fr z_eval = z.evaluate(z_challenge, n);
        std::string sigma_last_key = "sigma_" + std::to_string(program_width);
        fr sigma_last_eval = key->permutation_selectors.at(sigma_last_key).evaluate(z_challenge, n);
        transcript.add_element("z", z_eval.to_buffer());
        transcript.add_element(sigma_last_key, sigma_last_eval.to_buffer());
    }

    fr z_shifted_eval = z.evaluate(shifted_z, n);
    transcript.add_element("z_omega", z_shifted_eval.to_buffer());
    return;
}

template class ProverPermutationWidget<3>;
template class ProverPermutationWidget<4>;

// ###

template <typename Field, typename Group, typename Transcript>
VerifierPermutationWidget<Field, Group, Transcript>::VerifierPermutationWidget()
{}

template <typename Field, typename Group, typename Transcript>
Field VerifierPermutationWidget<Field, Group, Transcript>::compute_quotient_evaluation_contribution(
    verification_key*, const Field& alpha_base, const Transcript&, Field&, const bool)
{

    return alpha_base;
}

template <typename Field, typename Group, typename Transcript>
size_t VerifierPermutationWidget<Field, Group, Transcript>::compute_batch_evaluation_contribution(
    verification_key*, Field&, const size_t nu_index, const Transcript&, const bool)
{
    return nu_index;
};

template <typename Field, typename Group, typename Transcript>
VerifierBaseWidget::challenge_coefficients<Field> VerifierPermutationWidget<Field, Group, Transcript>::
    append_scalar_multiplication_inputs(verification_key*,
                                        const VerifierBaseWidget::challenge_coefficients<Field>& challenge,
                                        const Transcript&,
                                        std::vector<Group>&,
                                        std::vector<Field>&,
                                        const bool)
{
    return challenge;
}

template class VerifierPermutationWidget<barretenberg::fr,
                                         barretenberg::g1::affine_element,
                                         transcript::StandardTranscript>;

} // namespace waffle