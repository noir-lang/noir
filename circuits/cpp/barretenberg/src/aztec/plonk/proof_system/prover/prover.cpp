#include "prover.hpp"
#include "../public_inputs/public_inputs.hpp"
#include "../utils/linearizer.hpp"
#include <chrono>
#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include <polynomials/iterate_over_domain.hpp>
#include <polynomials/polynomial_arithmetic.hpp>

using namespace barretenberg;

namespace waffle {

template <typename settings>
ProverBase<settings>::ProverBase(std::shared_ptr<proving_key> input_key,
                                 std::shared_ptr<program_witness> input_witness,
                                 const transcript::Manifest& input_manifest)
    : n(input_key == nullptr ? 0 : input_key->n)
    , transcript(input_manifest, settings::hash_type, settings::num_challenge_bytes)
    , key(input_key)
    , witness(input_witness)
    , queue(key.get(), witness.get(), &transcript)
{
    if (witness->wires.count("z") == 0) {
        witness->wires.insert({ "z", polynomial(n, n) });
    }
}

template <typename settings>
ProverBase<settings>::ProverBase(ProverBase<settings>&& other)
    : n(other.n)
    , transcript(other.transcript)
    , key(std::move(other.key))
    , witness(std::move(other.witness))
    , queue(key.get(), witness.get(), &transcript)
{
    for (size_t i = 0; i < other.widgets.size(); ++i) {
        widgets.emplace_back(std::move(other.widgets[i]));
    }
}

template <typename settings> ProverBase<settings>& ProverBase<settings>::operator=(ProverBase<settings>&& other)
{
    n = other.n;

    widgets.resize(0);
    for (size_t i = 0; i < other.widgets.size(); ++i) {
        widgets.emplace_back(std::move(other.widgets[i]));
    }

    transcript = other.transcript;
    key = std::move(other.key);
    witness = std::move(other.witness);
    queue = work_queue(key.get(), witness.get(), &transcript);
    return *this;
}

template <typename settings> void ProverBase<settings>::compute_wire_pre_commitments()
{
    for (size_t i = 0; i < settings::program_width; ++i) {
        std::string wire_tag = "w_" + std::to_string(i + 1);
        queue.add_to_queue({
            work_queue::WorkType::SCALAR_MULTIPLICATION,
            witness->wires.at(wire_tag).get_coefficients(),
            "W_" + std::to_string(i + 1),
        });
    }

    // add public inputs
    const polynomial& public_wires_source = key->wire_ffts.at("w_2_fft");
    std::vector<fr> public_wires;
    for (size_t i = 0; i < key->num_public_inputs; ++i) {
        public_wires.push_back(public_wires_source[i]);
    }
    transcript.add_element("public_inputs", fr::to_buffer(public_wires));
}

template <typename settings> void ProverBase<settings>::compute_quotient_pre_commitment()
{
    for (size_t i = 0; i < settings::program_width; ++i) {
        const size_t offset = n * i;
        queue.add_to_queue({
            work_queue::WorkType::SCALAR_MULTIPLICATION,
            &key->quotient_large.get_coefficients()[offset],
            "T_" + std::to_string(i + 1),
        });
    }
}

// TODO: FROM LOCAL BRANCH, INTEGRATE....
// template <typename settings> void ProverBase<settings>::compute_permutation_grand_product_coefficients()
// {
//     polynomial& z_fft = key->z_fft;

//     fr alpha = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());
//     fr neg_alpha = -alpha;
//     fr alpha_squared = alpha.sqr();
//     fr beta = fr::serialize_from_buffer(transcript.get_challenge("beta").begin());
//     fr gamma = fr::serialize_from_buffer(transcript.get_challenge("beta", 1).begin());

//     // Our permutation check boils down to two 'grand product' arguments,
//     // that we represent with a single polynomial Z(X).
//     // We want to test that Z(X) has been constructed correctly.
//     // When evaluated at elements of w \in H, the numerator of Z(w) will equal the
//     // identity permutation grand product, and the denominator will equal the copy permutation grand product.

//     // The identity that we need to evaluate is: Z(X.w).(permutation grand product) = Z(X).(identity grand product)
//     // i.e. The next element of Z is equal to the current element of Z, multiplied by (identity grand product) /
//     // (permutation grand product)

//     // This method computes `Z(X).(identity grand product).{alpha}`.
//     // The random `alpha` is there to ensure our grand product polynomial identity is linearly independent from the
//     // other polynomial identities that we are going to roll into the quotient polynomial T(X).

//     // Specifically, we want to compute:
//     // (w_l(X) + \beta.sigma1(X) + \gamma).(w_r(X) + \beta.sigma2(X) + \gamma).(w_o(X) + \beta.sigma3(X) +
//     // \gamma).Z(X).alpha Once we divide by the vanishing polynomial, this will be a degree 3n polynomial.

//     // Multiply Z(X) by \alpha^2 when performing fft transform - we get this for free if we roll \alpha^2 into the
//     // multiplicative generator
//     z_fft.coset_fft_with_constant(key->large_domain, alpha);

//     // We actually want Z(X.w) as well as Z(X)! But that's easy to get. z_fft contains Z(X) evaluated at the 4n'th roots
//     // of unity. So z_fft(i) = Z(w^{i/4}) i.e. z_fft(i + 4) = Z(w^{i/4}.w)
//     // => if virtual term 'foo' contains a 4n fft of Z(X.w), then z_fft(i + 4) = foo(i)
//     // So all we need to do, to get Z(X.w) is to offset indexes to z_fft by 4.
//     // If `i >= 4n  4`, we need to wrap around to the start - so just append the 4 starting elements to the end of z_fft
//     z_fft.add_lagrange_base_coefficient(z_fft[0]);
//     z_fft.add_lagrange_base_coefficient(z_fft[1]);
//     z_fft.add_lagrange_base_coefficient(z_fft[2]);
//     z_fft.add_lagrange_base_coefficient(z_fft[3]);

//     std::array<fr*, settings::program_width> wire_ffts;
//     std::array<fr*, settings::program_width> sigma_ffts;

//     for (size_t i = 0; i < settings::program_width; ++i) {
//         wire_ffts[i] = &key->wire_ffts.at("w_" + std::to_string(i + 1) + "_fft")[0];
//         sigma_ffts[i] = &key->permutation_selector_ffts.at("sigma_" + std::to_string(i + 1) + "_fft")[0];
//     }

//     const polynomial& l_1 = key->lagrange_1;

//     // compute our public input component
//     std::vector<barretenberg::fr> public_inputs =
//         barretenberg::fr::from_buffer(transcript.get_element("public_inputs"));

//     fr public_input_delta =
//         compute_public_input_delta<barretenberg::fr>(public_inputs, beta, gamma, key->small_domain.root);
//     public_input_delta *= alpha;

//     polynomial& quotient_large = key->quotient_large;
//     // Step 4: Set the quotient polynomial to be equal to
//     // (w_l(X) + \beta.sigma1(X) + \gamma).(w_r(X) + \beta.sigma2(X) + \gamma).(w_o(X) + \beta.sigma3(X) +
//     // \gamma).Z(X).alpha
// #ifndef NO_MULTITHREADING
// #pragma omp parallel for
// #endif
//     for (size_t j = 0; j < key->large_domain.num_threads; ++j) {
//         const size_t start = j * key->large_domain.thread_size;
//         const size_t end = (j + 1) * key->large_domain.thread_size;

//         fr work_root = key->large_domain.root.pow(static_cast<uint64_t>(j * key->large_domain.thread_size));
//         work_root *= key->small_domain.generator;
//         // work_root *= fr::coset_generator(0);
//         work_root *= beta;

//         fr wire_plus_gamma;
//         fr T0;
//         fr denominator;
//         fr numerator;
//         for (size_t i = start; i < end; ++i) {
//             wire_plus_gamma = gamma + wire_ffts[0][i];

//             // Numerator computation
//             numerator = work_root + wire_plus_gamma;

//             // Denominator computation
//             denominator = sigma_ffts[0][i] * beta;
//             denominator += wire_plus_gamma;

//             for (size_t k = 1; k < settings::program_width; ++k) {
//                 wire_plus_gamma = gamma + wire_ffts[k][i];
//                 T0 = fr::coset_generator(k - 1) * work_root;
//                 T0 += wire_plus_gamma;
//                 numerator *= T0;

//                 T0 = sigma_ffts[k][i] * beta;
//                 T0 += wire_plus_gamma;
//                 denominator *= T0;
//             }

//             numerator *= z_fft[i];
//             denominator *= z_fft[i + 4];

//             /**
//              * Permutation bounds check
//              * (Z(X.w) - 1).(\alpha^3).L{n-1}(X) = T(X)Z_H(X)
//              **/
//             // The \alpha^3 term is so that we can subsume this polynomial into the quotient polynomial,
//             // whilst ensuring the term is linearly independent form the other terms in the quotient polynomial

//             // We want to verify that Z(X) equals `1` when evaluated at `w_n`, the 'last' element of our multiplicative
//             // subgroup H. But PLONK's 'vanishing polynomial', Z*_H(X), isn't the true vanishing polynomial of subgroup
//             // H. We need to cut a root of unity out of Z*_H(X), specifically `w_n`, for our grand product argument.
//             // When evaluating Z(X) has been constructed correctly, we verify that Z(X.w).(identity permutation product)
//             // = Z(X).(sigma permutation product), for all X \in H. But this relationship breaks down for X = w_n,
//             // because Z(X.w) will evaluate to the *first* element of our grand product argument. The last element of
//             // Z(X) has a dependency on the first element, so the first element cannot have a dependency on the last
//             // element.

//             // TODO: With the reduction from 2 Z polynomials to a single Z(X), the above no longer applies
//             // TODO: Fix this to remove the (Z(X.w) - 1).L_{n-1}(X) check

//             // To summarise, we can't verify claims about Z(X) when evaluated at `w_n`.
//             // But we can verify claims about Z(X.w) when evaluated at `w_{n-1}`, which is the same thing

//             // To summarise the summary: If Z(w_n) = 1, then (Z(X.w) - 1).L_{n-1}(X) will be divisible by Z_H*(X)
//             // => add linearly independent term (Z(X.w) - 1).(\alpha^3).L{n-1}(X) into the quotient polynomial to check
//             // this

//             // z_fft already contains evaluations of Z(X).(\alpha^2)
//             // at the (2n)'th roots of unity
//             // => to get Z(X.w) instead of Z(X), index element (i+2) instead of i
//             T0 = z_fft[i + 4] - public_input_delta; // T0 = (Z(X.w) - (delta)).(\alpha^2)
//             T0 *= alpha;                            // T0 = (Z(X.w) - (delta)).(\alpha^3)
//             T0 *= l_1[i + 8];                       // T0 = (Z(X.w)-delta).(\alpha^3).L{n-1}
//             numerator += T0;

//             // Step 2: Compute (Z(X) - 1).(\alpha^4).L1(X)
//             // We need to verify that Z(X) equals `1` when evaluated at the first element of our subgroup H
//             // i.e. Z(X) starts at 1 and ends at 1
//             // The `alpha^4` term is so that we can add this as a linearly independent term in our quotient polynomial
//             T0 = z_fft[i] + neg_alpha; // T0 = (Z(X) - 1).(\alpha^2)
//             T0 *= alpha_squared;       // T0 = (Z(X) - 1).(\alpha^4)
//             T0 *= l_1[i];              // T0 = (Z(X) - 1).(\alpha^2).L1(X)
//             numerator += T0;

//             // Combine into quotient polynomial
//             T0 = numerator - denominator;
//             quotient_large[i] = T0;

//             // Update our working root of unity
//             work_root *= key->large_domain.root;
//         }
//     }
// }

// template <typename settings> void ProverBase<settings>::init_quotient_polynomials()
// {
//     n = key->n;
// }

template <typename settings> void ProverBase<settings>::execute_preamble_round()
{
    queue.flush_queue();
    transcript.add_element("circuit_size",
                           { static_cast<uint8_t>(n >> 24),
                             static_cast<uint8_t>(n >> 16),
                             static_cast<uint8_t>(n >> 8),
                             static_cast<uint8_t>(n) });
    transcript.add_element("public_input_size",
                           { static_cast<uint8_t>(key->num_public_inputs >> 24),
                             static_cast<uint8_t>(key->num_public_inputs >> 16),
                             static_cast<uint8_t>(key->num_public_inputs >> 8),
                             static_cast<uint8_t>(key->num_public_inputs) });
    transcript.apply_fiat_shamir("init");

    for (size_t i = 0; i < settings::program_width; ++i) {
        std::string wire_tag = "w_" + std::to_string(i + 1);
        barretenberg::polynomial& wire = witness->wires.at(wire_tag);
        barretenberg::polynomial& wire_fft = key->wire_ffts.at(wire_tag + "_fft");
        barretenberg::polynomial_arithmetic::copy_polynomial(&wire[0], &wire_fft[0], n, n);
        queue.add_to_queue({ work_queue::WorkType::IFFT, nullptr, wire_tag });
    }
}

template <typename settings> void ProverBase<settings>::execute_first_round()
{
    queue.flush_queue();
#ifdef DEBUG_TIMING
    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
#endif
#ifdef DEBUG_TIMING
    std::chrono::steady_clock::time_point end = std::chrono::steady_clock::now();
    std::chrono::milliseconds diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "init quotient polys: " << diff.count() << "ms" << std::endl;
#endif
#ifdef DEBUG_TIMING
    start = std::chrono::steady_clock::now();
#endif
#ifdef DEBUG_TIMING
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "compute wire coefficients: " << diff.count() << "ms" << std::endl;
#endif
#ifdef DEBUG_TIMING
    start = std::chrono::steady_clock::now();
#endif
    compute_wire_pre_commitments();
    for (auto& widget : widgets) {
        widget->compute_round_commitments(transcript, 1, queue);
    }
#ifdef DEBUG_TIMING
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "compute wire commitments: " << diff.count() << "ms" << std::endl;
#endif
}

template <typename settings> void ProverBase<settings>::execute_second_round()
{
    queue.flush_queue();
    transcript.apply_fiat_shamir("beta");
#ifdef DEBUG_TIMING
    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
#endif
#ifdef DEBUG_TIMING
    std::chrono::steady_clock::time_point end = std::chrono::steady_clock::now();
    std::chrono::milliseconds diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "compute z coefficients: " << diff.count() << "ms" << std::endl;
#endif
#ifdef DEBUG_TIMING
    start = std::chrono::steady_clock::now();
#endif
    for (auto& widget : widgets) {
        widget->compute_round_commitments(transcript, 2, queue);
    }

    for (size_t i = 0; i < settings::program_width; ++i) {
        std::string wire_tag = "w_" + std::to_string(i + 1);
        queue.add_to_queue({ work_queue::WorkType::FFT, nullptr, wire_tag });
    }
#ifdef DEBUG_TIMING
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "compute z commitment: " << diff.count() << "ms" << std::endl;
#endif
}

template <typename settings> void ProverBase<settings>::execute_third_round()
{
    queue.flush_queue();
    transcript.apply_fiat_shamir("alpha");
#ifdef DEBUG_TIMING
    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
#endif
#ifdef DEBUG_TIMING
    std::chrono::steady_clock::time_point end = std::chrono::steady_clock::now();
    std::chrono::milliseconds diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "compute wire ffts: " << diff.count() << "ms" << std::endl;
#endif

#ifdef DEBUG_TIMING
    start = std::chrono::steady_clock::now();
#endif
#ifdef DEBUG_TIMING
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "copy z: " << diff.count() << "ms" << std::endl;
#endif
#ifdef DEBUG_TIMING
    start = std::chrono::steady_clock::now();
#endif
#ifdef DEBUG_TIMING
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "compute permutation grand product coeffs: " << diff.count() << "ms" << std::endl;
#endif
    fr alpha_base = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());
    // fr alpha_base = alpha.sqr().sqr();

    for (size_t i = 0; i < widgets.size(); ++i) {
#ifdef DEBUG_TIMING
        std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
#endif
        alpha_base = widgets[i]->compute_quotient_contribution(alpha_base, transcript);
#ifdef DEBUG_TIMING
        std::chrono::steady_clock::time_point end = std::chrono::steady_clock::now();
        std::chrono::milliseconds diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
        std::cout << "widget " << i << " quotient compute time: " << diff.count() << "ms" << std::endl;
#endif
    }

    fr* q_mid = &key->quotient_mid[0];
    fr* q_large = &key->quotient_large[0];

#ifdef DEBUG_TIMING
    start = std::chrono::steady_clock::now();
#endif
    if constexpr (settings::uses_quotient_mid) {
        barretenberg::polynomial_arithmetic::divide_by_pseudo_vanishing_polynomial(
            key->quotient_mid.get_coefficients(), key->small_domain, key->mid_domain);
    }
    barretenberg::polynomial_arithmetic::divide_by_pseudo_vanishing_polynomial(
        key->quotient_large.get_coefficients(), key->small_domain, key->large_domain);
#ifdef DEBUG_TIMING
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "divide by vanishing polynomial: " << diff.count() << "ms" << std::endl;
#endif
#ifdef DEBUG_TIMING
    start = std::chrono::steady_clock::now();
#endif
    if (settings::uses_quotient_mid) {
        key->quotient_mid.coset_ifft(key->mid_domain);
    }
    key->quotient_large.coset_ifft(key->large_domain);
#ifdef DEBUG_TIMING
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "final inverse fourier transforms: " << diff.count() << "ms" << std::endl;
#endif
    if (settings::uses_quotient_mid) {
        ITERATE_OVER_DOMAIN_START(key->mid_domain);
        q_large[i] += q_mid[i];
        ITERATE_OVER_DOMAIN_END;
    }
#ifdef DEBUG_TIMING
    start = std::chrono::steady_clock::now();
#endif
    compute_quotient_pre_commitment();
#ifdef DEBUG_TIMING
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "compute quotient commitment: " << diff.count() << "ms" << std::endl;
#endif
}

template <typename settings> void ProverBase<settings>::execute_fourth_round()
{
    queue.flush_queue();
    transcript.apply_fiat_shamir("z"); // end of 3rd round
#ifdef DEBUG_TIMING
    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
#endif
    compute_linearisation_coefficients();
#ifdef DEBUG_TIMING
    std::chrono::steady_clock::time_point end = std::chrono::steady_clock::now();
    std::chrono::milliseconds diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "compute linearisation coefficients: " << diff.count() << "ms" << std::endl;
#endif
}

template <typename settings> void ProverBase<settings>::execute_fifth_round()
{
    queue.flush_queue();
    transcript.apply_fiat_shamir("nu");
#ifdef DEBUG_TIMING
    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
#endif
    std::vector<fr> nu_challenges;
    std::vector<fr> shifted_nu_challenges;

    if constexpr (settings::use_linearisation) {
        nu_challenges.emplace_back(fr::serialize_from_buffer(transcript.get_challenge_from_map("nu", "r").begin()));
    }
    for (size_t i = 0; i < settings::program_width; ++i) {
        if (settings::requires_shifted_wire(settings::wire_shift_settings, i)) {
            shifted_nu_challenges.emplace_back(fr::serialize_from_buffer(
                transcript.get_challenge_from_map("nu", "w_" + std::to_string(i + 1)).begin()));
        }
        nu_challenges.emplace_back(
            fr::serialize_from_buffer(transcript.get_challenge_from_map("nu", "w_" + std::to_string(i + 1)).begin()));
    }

    fr z_challenge = fr::serialize_from_buffer(transcript.get_challenge("z").begin());
    fr* r = key->linear_poly.get_coefficients();

    std::array<fr*, settings::program_width> wires;
    for (size_t i = 0; i < settings::program_width; ++i) {
        wires[i] = &witness->wires.at("w_" + std::to_string(i + 1))[0];
    }

    // Next step: compute the two Kate polynomial commitments, and associated opening proofs
    // We have two evaluation points: z and z.omega
    // We need to create random linear combinations of each individual polynomial and combine them

    polynomial& opening_poly = key->opening_poly;
    polynomial& shifted_opening_poly = key->shifted_opening_poly;

    std::array<fr, settings::program_width> z_powers;
    z_powers[0] = z_challenge;
    for (size_t i = 1; i < settings::program_width; ++i) {
        z_powers[i] = z_challenge.pow(static_cast<uint64_t>(n * i));
    }

    polynomial& quotient_large = key->quotient_large;

    ITERATE_OVER_DOMAIN_START(key->small_domain);

    fr T0;
    fr quotient_temp = fr::zero();
    if constexpr (settings::use_linearisation) {
        quotient_temp = r[i] * nu_challenges[0];
    }
    for (size_t k = 1; k < settings::program_width; ++k) {
        T0 = quotient_large[i + (k * n)] * z_powers[k];
        quotient_temp += T0;
    }
    for (size_t k = 0; k < settings::program_width; ++k) {
        T0 = wires[k][i] * nu_challenges[k + settings::use_linearisation];
        quotient_temp += T0;
    }
    shifted_opening_poly[i] = 0;

    opening_poly[i] = quotient_large[i] + quotient_temp;

    ITERATE_OVER_DOMAIN_END;

    if constexpr (settings::wire_shift_settings > 0) {
        ITERATE_OVER_DOMAIN_START(key->small_domain);
        size_t nu_ptr = 0;
        if constexpr (settings::requires_shifted_wire(settings::wire_shift_settings, 0)) {
            fr T0;
            T0 = shifted_nu_challenges[nu_ptr++] * wires[0][i];
            shifted_opening_poly[i] += T0;
        }
        if constexpr (settings::requires_shifted_wire(settings::wire_shift_settings, 1)) {
            fr T0;
            T0 = shifted_nu_challenges[nu_ptr++] * wires[1][i];
            shifted_opening_poly[i] += T0;
        }
        if constexpr (settings::requires_shifted_wire(settings::wire_shift_settings, 2)) {
            fr T0;
            T0 = shifted_nu_challenges[nu_ptr++] * wires[2][i];
            shifted_opening_poly[i] += T0;
        }
        if constexpr (settings::requires_shifted_wire(settings::wire_shift_settings, 3)) {
            fr T0;
            T0 = shifted_nu_challenges[nu_ptr++] * wires[3][i];
            shifted_opening_poly[i] += T0;
        }
        for (size_t k = 4; k < settings::program_width; ++k) {
            if (settings::requires_shifted_wire(settings::wire_shift_settings, k)) {
                fr T0;
                T0 = shifted_nu_challenges[nu_ptr++] * wires[k][i];
                shifted_opening_poly[i] += T0;
            }
        }
        ITERATE_OVER_DOMAIN_END;
    }

#ifdef DEBUG_TIMING
    std::chrono::steady_clock::time_point end = std::chrono::steady_clock::now();
    std::chrono::milliseconds diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "compute base opening poly contribution: " << diff.count() << "ms" << std::endl;
#endif
#ifdef DEBUG_TIMING
    start = std::chrono::steady_clock::now();
#endif
    // Currently code assumes permutation_widget is first.
    for (size_t i = 0; i < widgets.size(); ++i) {
        widgets[i]->compute_opening_poly_contribution(transcript, settings::use_linearisation);
    }
#ifdef DEBUG_TIMING
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "compute widget opening poly contributions: " << diff.count() << "ms" << std::endl;
#endif
    fr shifted_z;
    shifted_z = z_challenge * key->small_domain.root;
#ifdef DEBUG_TIMING
    start = std::chrono::steady_clock::now();
#endif
    opening_poly.compute_kate_opening_coefficients(z_challenge);

    shifted_opening_poly.compute_kate_opening_coefficients(shifted_z);
#ifdef DEBUG_TIMING
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "compute kate opening poly coefficients: " << diff.count() << "ms" << std::endl;
#endif
#ifdef DEBUG_TIMING
    start = std::chrono::steady_clock::now();
#endif

    queue.add_to_queue({
        work_queue::WorkType::SCALAR_MULTIPLICATION,
        opening_poly.get_coefficients(),
        "PI_Z",
    });
    queue.add_to_queue({
        work_queue::WorkType::SCALAR_MULTIPLICATION,
        shifted_opening_poly.get_coefficients(),
        "PI_Z_OMEGA",
    });
#ifdef DEBUG_TIMING
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "compute opening commitment: " << diff.count() << "ms" << std::endl;
#endif
}

template <typename settings> barretenberg::fr ProverBase<settings>::compute_linearisation_coefficients()
{

    fr z_challenge = fr::serialize_from_buffer(transcript.get_challenge("z").begin());
    fr shifted_z = z_challenge * key->small_domain.root;

    polynomial& r = key->linear_poly;
    // ok... now we need to evaluate polynomials. Jeepers

    // evaluate the prover and instance polynomials.
    // (we don't need to evaluate the quotient polynomial, that can be derived by the verifier)
    for (size_t i = 0; i < settings::program_width; ++i) {
        std::string wire_key = "w_" + std::to_string(i + 1);
        const polynomial& wire = witness->wires.at(wire_key);
        fr wire_eval;
        wire_eval = wire.evaluate(z_challenge, n);
        transcript.add_element(wire_key, wire_eval.to_buffer());

        if (settings::requires_shifted_wire(settings::wire_shift_settings, i)) {
            fr shifted_wire_eval;
            shifted_wire_eval = wire.evaluate(shifted_z, n);
            transcript.add_element(wire_key + "_omega", shifted_wire_eval.to_buffer());
        }
    }

    for (size_t i = 0; i < widgets.size(); ++i) {
        widgets[i]->compute_transcript_elements(transcript, settings::use_linearisation);
    }

    fr t_eval = key->quotient_large.evaluate(z_challenge, 4 * n);

    if constexpr (settings::use_linearisation) {
        // TODO LOCAL BRANCH CHANGES INTEGRATE
        // barretenberg::polynomial_arithmetic::lagrange_evaluations lagrange_evals =
        //     barretenberg::polynomial_arithmetic::get_lagrange_evaluations(z_challenge, key->small_domain);
        // plonk_linear_terms linear_terms =
        //     compute_linear_terms<barretenberg::fr, transcript::StandardTranscript, settings>(transcript,
        //                                                                                      lagrange_evals.l_1);

        // const polynomial& sigma_last =
        //     key->permutation_selectors.at("sigma_" + std::to_string(settings::program_width));
        // ITERATE_OVER_DOMAIN_START(key->small_domain);
        // r[i] = (z[i] * linear_terms.z_1) + (sigma_last[i] * linear_terms.sigma_last);
        // ITERATE_OVER_DOMAIN_END;

        // fr alpha_base = alpha.sqr().sqr();
        fr alpha_base = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());
        for (size_t i = 0; i < widgets.size(); ++i) {
            alpha_base = widgets[i]->compute_linear_contribution(alpha_base, transcript, r);
        }

        fr linear_eval = r.evaluate(z_challenge, n);
        transcript.add_element("r", linear_eval.to_buffer());
    }
    transcript.add_element("t", t_eval.to_buffer());
    return t_eval;
}

template <typename settings> waffle::plonk_proof& ProverBase<settings>::export_proof()
{
    proof.proof_data = transcript.export_transcript();
    return proof;
}

template <typename settings> waffle::plonk_proof& ProverBase<settings>::construct_proof()
{
    execute_preamble_round();
    queue.process_queue();
    execute_first_round();
    queue.process_queue();
    execute_second_round();
    queue.process_queue();
    execute_third_round();
    queue.process_queue();
    execute_fourth_round();
    execute_fifth_round();
    queue.process_queue();
    return export_proof();
}

template <typename settings> void ProverBase<settings>::reset()
{
    transcript::Manifest manifest = transcript.get_manifest();
    transcript = transcript::StandardTranscript(manifest, settings::hash_type, settings::num_challenge_bytes);
}

template class ProverBase<unrolled_standard_settings>;
template class ProverBase<unrolled_turbo_settings>;
template class ProverBase<standard_settings>;
template class ProverBase<turbo_settings>;

} // namespace waffle
