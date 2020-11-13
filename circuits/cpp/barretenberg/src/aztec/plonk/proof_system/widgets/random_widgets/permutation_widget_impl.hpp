#pragma once

#include <common/mem.hpp>
#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include <plonk/proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/public_inputs/public_inputs.hpp>
#include <plonk/proof_system/utils/linearizer.hpp>

#include <plonk/transcript/transcript.hpp>
#include <polynomials/iterate_over_domain.hpp>
#include <polynomials/polynomial_arithmetic.hpp>

namespace waffle {

template <size_t program_width, bool idpolys, const size_t num_roots_cut_out_of_vanishing_polynomial>
ProverPermutationWidget<program_width, idpolys, num_roots_cut_out_of_vanishing_polynomial>::ProverPermutationWidget(proving_key* input_key,
                                                                         program_witness* input_witness)
    : ProverRandomWidget(input_key, input_witness)
{}

template <size_t program_width, bool idpolys, const size_t num_roots_cut_out_of_vanishing_polynomial>
ProverPermutationWidget<program_width, idpolys, num_roots_cut_out_of_vanishing_polynomial>::ProverPermutationWidget(const ProverPermutationWidget& other)
    : ProverRandomWidget(other)
{}

template <size_t program_width, bool idpolys, const size_t num_roots_cut_out_of_vanishing_polynomial>
ProverPermutationWidget<program_width, idpolys, num_roots_cut_out_of_vanishing_polynomial>::ProverPermutationWidget(ProverPermutationWidget&& other)
    : ProverRandomWidget(other)
{}

template <size_t program_width, bool idpolys, const size_t num_roots_cut_out_of_vanishing_polynomial>
ProverPermutationWidget<program_width, idpolys, num_roots_cut_out_of_vanishing_polynomial>& ProverPermutationWidget<program_width, idpolys, num_roots_cut_out_of_vanishing_polynomial>::operator=(
    const ProverPermutationWidget& other)
{
    ProverRandomWidget::operator=(other);
    return *this;
}

template <size_t program_width, bool idpolys, const size_t num_roots_cut_out_of_vanishing_polynomial>
ProverPermutationWidget<program_width, idpolys, num_roots_cut_out_of_vanishing_polynomial>& ProverPermutationWidget<program_width, idpolys, num_roots_cut_out_of_vanishing_polynomial>::operator=(
    ProverPermutationWidget&& other)
{
    ProverRandomWidget::operator=(other);
    return *this;
}

template <size_t program_width, bool idpolys, const size_t num_roots_cut_out_of_vanishing_polynomial>
void ProverPermutationWidget<program_width, idpolys,  num_roots_cut_out_of_vanishing_polynomial>::compute_round_commitments(
    transcript::StandardTranscript& transcript, const size_t round_number, work_queue& queue)
{
    if (round_number != 3) {
        return;
    }
    const size_t n = key->n;
    polynomial& z = witness->wires.at("z");
    polynomial& z_fft = key->wire_ffts.at("z_fft");

    fr* accumulators[(program_width == 1) ? 3 : program_width * 2];
    accumulators[0] = &z[1];
    accumulators[1] = &z_fft[0];
    accumulators[2] = &z_fft[n];

    if constexpr (program_width * 2 > 2) {      // program_width >= 2
        accumulators[3] = &z_fft[n + n];
    }
    if constexpr (program_width > 2) {          // program_width >= 3
        accumulators[4] = &z_fft[n + n + n];
        accumulators[5] = &key->opening_poly[0];
    }
    if constexpr (program_width > 3) {          // program_width >= 4
        accumulators[6] = &key->shifted_opening_poly[0];
        accumulators[7] = &key->quotient_large[0];
    }
    if constexpr (program_width > 4) {          // program_width >= 5
        accumulators[8] = &key->linear_poly[0];
        accumulators[9] = &key->quotient_large[n];
    }
    if constexpr (program_width > 5) {          // program_width >= 6
        accumulators[10] = &key->quotient_large[n + n];
        accumulators[11] = &key->quotient_large[n + n + n];
    }
    for (size_t k = 7; k < program_width; ++k) {// program_width >= 7
        // we're out of temporary memory!
        accumulators[(k - 1) * 2] = static_cast<fr*>(aligned_alloc(64, sizeof(fr) * n));
        accumulators[(k - 1) * 2 + 1] = static_cast<fr*>(aligned_alloc(64, sizeof(fr) * n));
    }

    barretenberg::fr beta = fr::serialize_from_buffer(transcript.get_challenge("beta").begin());
    barretenberg::fr gamma = fr::serialize_from_buffer(transcript.get_challenge("beta", 1).begin());

    std::array<fr*, program_width> lagrange_base_wires;
    std::array<fr*, program_width> lagrange_base_sigmas;
    [[maybe_unused]] std::array<fr*, program_width> lagrange_base_ids;

    for (size_t i = 0; i < program_width; ++i) {
        lagrange_base_wires[i] = &key->wire_ffts.at("w_" + std::to_string(i + 1) + "_fft")[0];
        lagrange_base_sigmas[i] = &key->permutation_selectors_lagrange_base.at("sigma_" + std::to_string(i + 1))[0];

        // if idpolys = true, it implies that we do NOT use the identity permutation 
        // S_ID1(X) = X, S_ID2(X) = k_1X, ...
        if constexpr (idpolys)
            lagrange_base_ids[i] = &key->permutation_selectors_lagrange_base.at("id_" + std::to_string(i + 1))[0];
    }

#ifndef NO_MULTITHREADING
#pragma omp parallel
#endif
    {
        // step 1: compute the individual terms in the permutation poylnomial.
        // 
        // Consider the case in which we use identity permutation polynomials and let program width = 3.
        // (extending it to the case when the permutation polynomials is not identity is trivial).
        //
        // coefficient of L_1: 1
        // coefficient of L_2:
        //                (w_1 + \gamma + \beta.w^{0}) . (w_{n+1} + \gamma + \beta.k_1.w^{0}) . (w_{2n+1} + \gamma + \beta.k_2.w^{0})
        //  coeff_of_L1 * --------------------------------------------------------------------------------------------------------------------
        //                (w_1 + \gamma + \beta.\sigma(1)) . (w_{n+1} + \gamma + \beta.\sigma(n+1)) . (w_{2n+1} + \gamma + \beta.\sigma(2n+1))
        // coefficient of L_3:
        //                (w_2 + \gamma + \beta.w^{1}) . (w_{n+2} + \gamma + \beta.k_1.w^{1}) . (w_{2n+2} + \gamma + \beta.k_2.w^{1})
        //  coeff_of_L2 * --------------------------------------------------------------------------------------------------------------------
        //                (w_2 + \gamma + \beta.\sigma(2)) . (w_{n+2} + \gamma + \beta.\sigma(n+2)) . (w_{2n+2} + \gamma + \beta.\sigma(2n+2))
        // and so on...
        //
        // accumulator data structure
        // numerators in accumulator[0: program_width-1], denominators in accumulator[program_width:]
        //      0                                         1                                               (n-1)
        // 0 -> (w_1 + \gamma + \beta.w^{0}),             (w_2 + \gamma + \beta.w^{1}),             ...., (w_n + \gamma + \beta.w^{n-1})
        // 1 -> (w_{n+1} + \gamma + \beta.k_1.w^{0}),     (w_{n+1} + \gamma + \beta.k_1.w^{2}),     ...., (w_{n+1} + \gamma + \beta.k_1.w^{n-1})
        // 2 -> (w_{2n+1} + \gamma + \beta.k_2.w^{0}),    (w_{2n+1} + \gamma + \beta.k_2.w^{0}),    ...., (w_{2n+1} + \gamma + \beta.k_2.w^{n-1})
        //
        // 3 -> (w_1 + \gamma + \beta.\sigma(1)),         (w_2 + \gamma + \beta.\sigma(2)),         ...., (w_n + \gamma + \beta.\sigma(n))
        // 4 -> (w_{n+1} + \gamma + \beta.\sigma(n+1)),   (w_{n+1} + \gamma + \beta.\sigma{n+2}),   ...., (w_{n+1} + \gamma + \beta.\sigma{n+n})
        // 5 -> (w_{2n+1} + \gamma + \beta.\sigma(2n+1)), (w_{2n+1} + \gamma + \beta.\sigma(2n+2)), ...., (w_{2n+1} + \gamma + \beta.\sigma(2n+n))
        //
        // Thus, to obtain coefficient_of_L2, we need to use accumulators[:][0].
        // To obtain coefficient_of_L3, we need to use accumulator[:][0] and accumulator[:][1]
        // and so on upto coefficient_of_Ln.
#ifndef NO_MULTITHREADING
#pragma omp for
#endif
        for (size_t j = 0; j < key->small_domain.num_threads; ++j) {
            barretenberg::fr thread_root =
                key->small_domain.root.pow(static_cast<uint64_t>(j * key->small_domain.thread_size));
            [[maybe_unused]] barretenberg::fr cur_root_times_beta = thread_root * beta;
            barretenberg::fr T0;
            barretenberg::fr wire_plus_gamma;
            size_t start = j * key->small_domain.thread_size;
            size_t end = (j + 1) * key->small_domain.thread_size;
            for (size_t i = start; i < end; ++i) {
                wire_plus_gamma = gamma + lagrange_base_wires[0][i];
                if constexpr (!idpolys) {
                    accumulators[0][i] = wire_plus_gamma + cur_root_times_beta;
                }
                if constexpr (idpolys) {
                    T0 = lagrange_base_ids[0][i] * beta;
                    accumulators[0][i] = T0 + wire_plus_gamma;
                }

                T0 = lagrange_base_sigmas[0][i] * beta;
                accumulators[program_width][i] = T0 + wire_plus_gamma;

                for (size_t k = 1; k < program_width; ++k) {
                    wire_plus_gamma = gamma + lagrange_base_wires[k][i];
                    if constexpr (idpolys) {
                        T0 = lagrange_base_ids[k][i] * beta;
                    } else {
                        T0 = fr::coset_generator(k - 1) * cur_root_times_beta;
                    }
                    accumulators[k][i] = T0 + wire_plus_gamma;

                    T0 = lagrange_base_sigmas[k][i] * beta;
                    accumulators[k + program_width][i] = T0 + wire_plus_gamma;
                }
                if constexpr (!idpolys)
                    cur_root_times_beta *= key->small_domain.root;
            }
        }

        // step 2: compute the constituent components of Z(X). This is a small multithreading bottleneck, as we have
        // program_width * 2 non-parallelizable processes
        //
        // update the accumulator matrix a[:][:] to:
        //      0          1                    2                         3
        // 0 -> (a[0][0]), (a[0][1] * a[0][0]), (a[0][2] * a[0][1]), ..., (a[0][n-1] * a[0][n-2])
        // 1 -> (a[1][0]), (a[1][1] * a[1][0]), (a[1][2] * a[1][1]), ..., (a[1][n-1] * a[1][n-2])
        //
        // and so on...
#ifndef NO_MULTITHREADING
#pragma omp for
#endif
        for (size_t i = 0; i < program_width * 2; ++i) {
            fr* coeffs = &accumulators[i][0];                           // start from the beginning of a row
            for (size_t j = 0; j < key->small_domain.size - 1; ++j) {
                coeffs[j + 1] *= coeffs[j];                             // iteratively update elements in subsequent columns
            }
        }

        // step 3: concatenate together the accumulator elements into Z(X)
        //
        // update the accumulator rows a[0] and a[program_width] to:
        //       0                                     1                                           (n-1)
        // 0 ->  (a[0][0] * a[1][0] * a[2][0]),        (a[0][1] * a[1][1] * a[2][1]),        ...., (a[0][n-1] * a[1][n-1] * a[2][n-1])
        // pw -> (a[pw][0] * a[pw+1][0] * a[pw+2][0]), (a[pw][1] * a[pw+1][1] * a[pw+2][1]), ...., (a[pw][n-1] * a[pw+1][n-1] * a[pw+2][n-1])
        //
        // note that pw = program_width
        // Hereafter, we can compute
        // coefficient_Lj = a[0][j]/a[pw][j]
        // 
        // Naive way of computing these coefficients would result in n inversions, which is pretty expensive.
        // Instead we use Montgomery's trick for batch inversion.
        // Montgomery's trick documentation: ./src/aztec/ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp/L286
#ifndef NO_MULTITHREADING
#pragma omp for
#endif
        for (size_t j = 0; j < key->small_domain.num_threads; ++j) {
            const size_t start = j * key->small_domain.thread_size;
            const size_t end =
                ((j + 1) * key->small_domain.thread_size) - ((j == key->small_domain.num_threads - 1) ? 1 : 0);
            barretenberg::fr inversion_accumulator = fr::one();
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

    // coefficient_L1 = 1
    z[0] = fr::one();

    /*
    Adding zero knowledge to the permutation polynomial.
    */
    // To ensure that PLONK is honest-verifier zero-knowledge, we need to ensure that the witness polynomials
    // and the permutation polynomial look uniformly random to an adversary. To make the witness polynomials
    // a(X), b(X) and c(X) uniformly random, we need to add 2 random blinding factors into each of them.
    // i.e. a'(X) = a(X) + (r_1X + r_2)
    // where r_1 and r_2 are uniformly random scalar field elements. A natural question is:
    // Why do we need 2 random scalars in witness polynomials? The reason is: our witness polynomials are
    // evaluated at only 1 point (\scripted{z}), so adding a random degree-1 polynomial suffices.
    //
    // NOTE: In TurboPlonk and UltraPlonk, the witness polynomials are evaluated at 2 points and thus 
    // we need to add 3 random scalars in them. 
    //
    // On the other hand, permutation polynomial z(X) is evaluated at two points, namely \scripted{z} and 
    // \scripted{z}.\omega. Hence, we need to add a random polynomial of degree 2 to ensure that the permutation
    // polynomials looks uniformly random.
    // z'(X) = z(X) + (r_3X^2 + r_4X + r_5)
    // where r_3, r_4, r_5 are uniformly random scalar field elements.
    //
    // Furthermore, instead of adding random polynomials, we could directly add random scalars in the lagrange-
    // basis forms of the witness and permutation polynomials. This is because we are using a modified vanishing
    // polynomial of the form
    //                           (X^n - 1)
    // Z*_H(X) = ------------------------------------------
    //           (X - w^{n-1}).(X - w^{n-2})...(X - w^{k})
    // where w = n-th root of unity, k = num_roots_cut_out_of_vanishing_polynomials.
    // Thus, the last k places in the lagrange basis form of z(X) are empty. We can therefore utilise them and 
    // add random scalars as coefficients of L_{n-1}, L_{n-2},... and so on.
    //
    // Note: The number of coefficients in the permutation polynomial z(X) are (n - k + 1)
    // (refer to Round 2 in the PLONK paper). Hence, if we cut 3 roots out of the vanishing polynomial,
    // we are left with only 2 places in the z array to add randomness. For having last 3 places available
    // for adding random scalars, we need to cut atleast 4 roots out of the vanishing polynomial.
    //  
    // Since we have valid z coefficients in positions from 0 to (n - k), we can start adding random scalars
    // from position (n - k + 1) upto (n - k + 3).
    //
    // NOTE: If in future there is a need to cut off more zeros off the vanishing polynomial, this method 
    // will not change. This must be changed only if the number of evaluations of permutation polynomial
    // changes.
    const size_t z_randomness = 3;
    ASSERT(z_randomness < num_roots_cut_out_of_vanishing_polynomial);
    for (size_t k = 0; k < z_randomness; ++k) {
        z[(n - num_roots_cut_out_of_vanishing_polynomial) + 1 + k] = fr::random_element();
    }

    z.ifft(key->small_domain);

    for (size_t k = 7; k < program_width; ++k) {
        aligned_free(accumulators[(k - 1) * 2]);
        aligned_free(accumulators[(k - 1) * 2 + 1]);
    }

    queue.add_to_queue({
        work_queue::WorkType::SCALAR_MULTIPLICATION,
        z.get_coefficients(),
        "Z",
        barretenberg::fr(0),
        0,
    });
    queue.add_to_queue({
        work_queue::WorkType::FFT,
        nullptr,
        "z",
        barretenberg::fr(0),
        0,
    });
}

template <size_t program_width, bool idpolys, const size_t num_roots_cut_out_of_vanishing_polynomial>
barretenberg::fr ProverPermutationWidget<program_width, idpolys, num_roots_cut_out_of_vanishing_polynomial>::compute_quotient_contribution(
    const fr& alpha_base, const transcript::StandardTranscript& transcript)
{
    polynomial& z_fft = key->wire_ffts.at("z_fft");

    barretenberg::fr alpha_squared = alpha_base.sqr();
    barretenberg::fr beta = fr::serialize_from_buffer(transcript.get_challenge("beta").begin());
    barretenberg::fr gamma = fr::serialize_from_buffer(transcript.get_challenge("beta", 1).begin());

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
    [[maybe_unused]] std::array<fr*, program_width> id_ffts;

    for (size_t i = 0; i < program_width; ++i) {
        
        // wire_fft[0] contains the fft of the wire polynomial w_1
        // sigma_fft[0] contains the fft of the permutation selector polynomial \sigma_1 
        wire_ffts[i] = &key->wire_ffts.at("w_" + std::to_string(i + 1) + "_fft")[0];
        sigma_ffts[i] = &key->permutation_selector_ffts.at("sigma_" + std::to_string(i + 1) + "_fft")[0];

        // idpolys is FALSE iff the "identity permutation" is used as a monomial
        // as a part of the permutation polynomial
        // <=> idpolys = FALSE
        if constexpr (idpolys)
            id_ffts[i] = &key->permutation_selector_ffts.at("id_" + std::to_string(i + 1) + "_fft")[0];
    }

    // we start with lagrange polynomial L_1(X)
    const polynomial& l_start = key->lagrange_1;

    // compute our public input component
    std::vector<barretenberg::fr> public_inputs = many_from_buffer<fr>(transcript.get_element("public_inputs"));

    barretenberg::fr public_input_delta =
        compute_public_input_delta<fr>(public_inputs, beta, gamma, key->small_domain.root);

    const size_t block_mask = key->large_domain.size - 1;
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

        // leverage multi-threading by computing quotient polynomial at points
        // (w^{j * num_threads}, w^{j * num_threads + 1}, ..., w^{j * num_threads + num_threads})
        // 
        // curr_root = w^{j * num_threads} * g_{small} * beta
        // curr_root will be used in denominator
        barretenberg::fr cur_root_times_beta =
            key->large_domain.root.pow(static_cast<uint64_t>(j * key->large_domain.thread_size));
        cur_root_times_beta *= key->small_domain.generator;
        cur_root_times_beta *= beta;

        barretenberg::fr wire_plus_gamma;
        barretenberg::fr T0;
        barretenberg::fr denominator;
        barretenberg::fr numerator;
        for (size_t i = start; i < end; ++i) {
            wire_plus_gamma = gamma + wire_ffts[0][i];

            // Numerator computation
            if constexpr (!idpolys)
                // identity polynomial used as a monomial: S_{id1} = x, S_{id2} = k_1.x, S_{id3} = k_2.x
                // start with (w_l(X) + \beta.X + \gamma)
                numerator = cur_root_times_beta + wire_plus_gamma;
            else
                numerator = id_ffts[0][i] * beta + wire_plus_gamma;

            // Denominator computation
            // start with (w_l(X) + \beta.\sigma1(X) + \gamma)
            denominator = sigma_ffts[0][i] * beta;
            denominator += wire_plus_gamma;

            for (size_t k = 1; k < program_width; ++k) {
                wire_plus_gamma = gamma + wire_ffts[k][i];
                if constexpr (!idpolys)
                    // (w_r(X) + \beta.(k_{k}.X) + \gamma)
                    T0 = fr::coset_generator(k - 1) * cur_root_times_beta;
                if constexpr (idpolys)
                    T0 = id_ffts[k][i] * beta;

                T0 += wire_plus_gamma;
                numerator *= T0;

                // (w_r(X) + \beta.\sigma_{k}(X) + \gamma)
                T0 = sigma_ffts[k][i] * beta;
                T0 += wire_plus_gamma;
                denominator *= T0;
            }

            numerator *= z_fft[i];
            denominator *= z_fft[(i + 4) & block_mask];

            /**
             * Permutation bounds check
             * (Z(X.w) - 1).(\alpha^3).L_{end}(X) = T(X)Z*_H(X)
             * 
             * where Z*_H(X) = (X^n - 1)/[(X - w^{n-1})...(X - w^{n - num_roots_cut_out_of_vanishing_polynomial})]
             * i.e. we remove some roots from the true vanishing polynomial to ensure that the overall degree
             * of the permutation polynomial is <= n. 
             * Read more on this here: https://hackmd.io/1DaroFVfQwySwZPHMoMdBg
             * 
             * Therefore, L_{end} = L_{n - num_roots_cut_out_of_vanishing_polynomial}
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
            // at the (4n)'th roots of unity
            // => to get Z(X.w) instead of Z(X), index element (i+4) instead of i
            T0 = z_fft[(i + 4) & block_mask] - public_input_delta; // T0 = (Z(X.w) - (delta)).(\alpha^2)
            T0 *= alpha_base;                                      // T0 = (Z(X.w) - (delta)).(\alpha^3)

            // T0 = (Z(X.w) - delta).(\alpha^3).L_{end}
            // where L_{end} = L{n - num_roots_cut_out_of_vanishing_polynomial}.
            //
            // Note that L_j(X) = L_1(X . w^{-j}) = L_1(X . w^{n-j})
            // => L_{end}= L_1(X . w^{num_roots_cut_out_of_vanishing_polynomial + 1})
            // => fetch the value at index (i + (num_roots_cut_out_of_vanishing_polynomial + 1) * 4) in l_1
            // the factor of 4 is because l_1 is a 4n-size fft.
            //
            // Recall, we use l_start for l_1 for consistency in notation.
            T0 *= l_start[(i + 4 + 4 * num_roots_cut_out_of_vanishing_polynomial) & block_mask];
            numerator += T0;

            // Step 2: Compute (Z(X) - 1).(\alpha^4).L1(X)
            // We need to verify that Z(X) equals `1` when evaluated at the first element of our subgroup H
            // i.e. Z(X) starts at 1 and ends at 1
            // The `alpha^4` term is so that we can add this as a linearly independent term in our quotient polynomial
            T0 = z_fft[i] - fr(1); // T0 = (Z(X) - 1).(\alpha^2)
            T0 *= alpha_squared;   // T0 = (Z(X) - 1).(\alpha^4)
            T0 *= l_start[i];          // T0 = (Z(X) - 1).(\alpha^2).L1(X)
            numerator += T0;

            // Combine into quotient polynomial
            T0 = numerator - denominator;
            quotient_large[i] = T0 * alpha_base;

            // Update our working root of unity
            cur_root_times_beta *= key->large_domain.root;
        }
    }
    return alpha_base.sqr().sqr();
}

template <size_t program_width, bool idpolys, const size_t num_roots_cut_out_of_vanishing_polynomial>
barretenberg::fr ProverPermutationWidget<program_width, idpolys, num_roots_cut_out_of_vanishing_polynomial>::compute_linear_contribution(
    const fr& alpha, const transcript::StandardTranscript& transcript, polynomial& r)
{
    // This method computes a part of the linearisation polynomial r(X) in round 4 of prover's algorithm in PLONK paper/
    // Since this is a member function of the `ProverPermutationWidget` class, we only compute the terms relevant to the
    // copy constraints. Concretely, we compute the terms:
    //
    // r(X) = (a_eval.b_eval.q_M(X) + a_eval.q_L(X) + b_eval.q_R(X) + c_eval.q_O(X) + q_C(X)) +            |-------> gate constraints
    //        ((a_eval + β.z + γ)(b_eval + β.k_1.z + γ)(c_eval + β.k_2.z + γ) z(X))α -                     |     
    //        ((a_eval + β.sigma1_eval + γ)(b_eval + β.sigma2_eval + γ) β.z_eval_omega.S_{sigma3}(X))α +    )------> copy constraints
    //        z(X).L_1(z).α^{3}                                                                            |    
    //
    // Note here, we are only trying to compute the `copy constraints` part and the `gate constraints` part need to be done 
    // using Arithmetic widget. Also, the prover calls this function only when linearisation trick is used, so we don't
    // need to explicitly check the condition `use_linearisation`.
    //

    // Step 1: regenrate challenges, fetch evaluations wire polynomials
    polynomial& z = witness->wires.at("z");
    barretenberg::fr z_challenge = fr::serialize_from_buffer(transcript.get_challenge("z").begin());

    barretenberg::polynomial_arithmetic::lagrange_evaluations lagrange_evals =
        barretenberg::polynomial_arithmetic::get_lagrange_evaluations(z_challenge, key->small_domain);

    barretenberg::fr alpha_cubed = alpha.sqr() * alpha;
    barretenberg::fr beta = fr::serialize_from_buffer(transcript.get_challenge("beta").begin());
    barretenberg::fr gamma = fr::serialize_from_buffer(transcript.get_challenge("beta", 1).begin());
    barretenberg::fr z_beta = z_challenge * beta;

    std::array<fr, program_width> wire_evaluations;
    for (size_t i = 0; i < program_width; ++i) {
        wire_evaluations[i] = fr::serialize_from_buffer(&transcript.get_element("w_" + std::to_string(i + 1))[0]);
    }

    barretenberg::fr z_1_shifted_eval = fr::serialize_from_buffer(&transcript.get_element("z_omega")[0]);

    // Step 2: compute scalar multiplicand of the permutation polynomial z(X)
    // This has two flavours, one in which we use identity polynomials for representing identity permutations
    // and the other one in which user defines polynomials to represent identity permutation.
    // The multiplicand of z(X) accordingly changes.
    barretenberg::fr T0;
    barretenberg::fr z_contribution = fr(1);

    if (!idpolys) {
        for (size_t i = 0; i < program_width; ++i) {
            barretenberg::fr coset_generator = (i == 0) ? fr(1) : fr::coset_generator(i - 1);
            T0 = z_beta * coset_generator;
            T0 += wire_evaluations[i];
            T0 += gamma;
            z_contribution *= T0;
        }
    } else {
        for (size_t i = 0; i < program_width; ++i) {
            barretenberg::fr id_evaluation =
                fr::serialize_from_buffer(&transcript.get_element("id_" + std::to_string(i + 1))[0]);
            T0 = id_evaluation * beta;
            T0 += wire_evaluations[i];
            T0 += gamma;
            z_contribution *= T0;
        }
    }

    // Step 3: add lagrange polynomial term to multiplicand of z(X)
    barretenberg::fr z_1_multiplicand = z_contribution * alpha;
    T0 = lagrange_evals.l_start * alpha_cubed;
    z_1_multiplicand += T0;

    // Step 4: compute the multiplicand of the copy permutation polynomial S_{sigma3}(X)
    barretenberg::fr sigma_contribution = fr(1);
    for (size_t i = 0; i < program_width - 1; ++i) {
        barretenberg::fr permutation_evaluation =
            fr::serialize_from_buffer(&transcript.get_element("sigma_" + std::to_string(i + 1))[0]);
        T0 = permutation_evaluation * beta;
        T0 += wire_evaluations[i];
        T0 += gamma;
        sigma_contribution *= T0;
    }
    sigma_contribution *= z_1_shifted_eval;
    barretenberg::fr sigma_last_multiplicand = -(sigma_contribution * alpha);
    sigma_last_multiplicand *= beta;

    // Step 5: add up the z(X) and S_{sigma3}(X) terms into r(X)
    const polynomial& sigma_last = key->permutation_selectors.at("sigma_" + std::to_string(program_width));
    ITERATE_OVER_DOMAIN_START(key->small_domain);
    r[i] = (z[i] * z_1_multiplicand) + (sigma_last[i] * sigma_last_multiplicand);
    ITERATE_OVER_DOMAIN_END;

    return alpha.sqr().sqr();
}

// ###

template <typename Field, typename Group, typename Transcript, const size_t num_roots_cut_out_of_vanishing_polynomial>
VerifierPermutationWidget<Field, Group, Transcript, num_roots_cut_out_of_vanishing_polynomial>::VerifierPermutationWidget()
{}

template <typename Field, typename Group, typename Transcript, const size_t num_roots_cut_out_of_vanishing_polynomial>
Field VerifierPermutationWidget<Field, Group, Transcript, num_roots_cut_out_of_vanishing_polynomial>::compute_quotient_evaluation_contribution(
    typename Transcript::Key* key,
    const Field& alpha,
    const Transcript& transcript,
    Field& t_eval,
    const bool use_linearisation,
    const bool idpolys)
{
    // This function computes the part of the quotient polynomial evaluation relevant to the PLONK's permutation argument.
    // Note that this is needed by the verifier in step 8 of verification algorithm. This has two flavours:
    //  1. The prover is using linearisation, i.e. she is computing polynomial r(X) and sending its evaluation r_eval 
    //  2. The prover is not using linearisation and the overhead computation needs to be done by the verifier.
    //
    Field alpha_cubed = alpha.sqr() * alpha;
    Field z = transcript.get_challenge_field_element("z");
    Field beta = transcript.get_challenge_field_element("beta", 0);
    Field gamma = transcript.get_challenge_field_element("beta", 1);
    Field z_beta = z * beta;

    // We need wire polynomials' and sigma polynomials' evaluations at zeta which we fetch from the transcript.
    // Fetch a_eval, b_eval, c_eval, sigma1_eval, sigma2_eval 
    std::vector<Field> wire_evaluations;
    std::vector<Field> sigma_evaluations;

    const size_t num_sigma_evaluations = (use_linearisation ? key->program_width - 1 : key->program_width);

    for (size_t i = 0; i < num_sigma_evaluations; ++i) {
        std::string index = std::to_string(i + 1);
        sigma_evaluations.emplace_back(transcript.get_field_element("sigma_" + index));
    }

    for (size_t i = 0; i < key->program_width; ++i) {
        wire_evaluations.emplace_back(transcript.get_field_element("w_" + std::to_string(i + 1)));
    }

    // Compute evaluations of lagrange polynomials L_1(X) and L_{n-k} at z (= zeta)
    // Recall, k is the number of roots cut out of the vanishing polynomial Z_H(X)
    // Note that 
    //                                  X^n - 1
    // L_i(X) = L_1(X.w^{-i + 1}) = -----------------
    //                               X.w^{-i + 1} - 1
    //
    Field numerator = key->z_pow_n - Field(1);

    numerator *= key->domain.domain_inverse;
    Field l_start = numerator / (z - Field(1));

    // compute w^{num_roots_cut_out_of_vanishing_polynomial + 1}
    Field l_end_root = (num_roots_cut_out_of_vanishing_polynomial & 1) ? key->domain.root.sqr() : key->domain.root;
    for (size_t i = 0; i < num_roots_cut_out_of_vanishing_polynomial / 2; ++i) {
        l_end_root *= key->domain.root.sqr();
    }
    Field l_end = numerator / ((z * l_end_root) - Field(1));

    Field z_1_shifted_eval = transcript.get_field_element("z_omega");

    // reconstruct evaluation of quotient polynomial from prover messages
    Field T1;
    Field T2;
    Field alpha_pow[4];
    alpha_pow[0] = alpha;
    for (size_t i = 1; i < 4; ++i) {
        alpha_pow[i] = alpha_pow[i - 1] * alpha_pow[0];
    }

    // Part 1: compute sigma contribution, i.e.
    // ((a_eval + β.sigma1_eval + γ)(b_eval + β.sigma2_eval + γ)(c_eval + γ) z_eval_omega)α
    Field sigma_contribution = Field(1);
    Field T0;
    for (size_t i = 0; i < key->program_width - 1; ++i) {
        T0 = sigma_evaluations[i] * beta;
        T1 = wire_evaluations[i] + gamma;
        T0 += T1;
        sigma_contribution *= T0;
    }

    T0 = wire_evaluations[key->program_width - 1] + gamma;
    sigma_contribution *= T0;
    sigma_contribution *= z_1_shifted_eval;
    sigma_contribution *= alpha_pow[0];

    // Part 2: compute the public-inputs term, i.e.
    // (z_eval_omega - ∆_{PI}).L_{n-k}(z).α^{2}
    std::vector<Field> public_inputs = transcript.get_field_element_vector("public_inputs");
    Field public_input_delta = compute_public_input_delta<Field>(public_inputs, beta, gamma, key->domain.root);

    T1 = z_1_shifted_eval - public_input_delta;
    T1 *= l_end;
    T1 *= alpha_pow[1];

    // Part 3: compute starting lagrange polynomial term, i.e.
    // L_1(z).α^{3}
    T2 = l_start * alpha_pow[2];

    // Combine parts 1, 2, 3. If linearisation is used, we need to add r_eval to T1 and we're done.
    T1 -= T2;
    T1 -= sigma_contribution;

    if (use_linearisation) {
        Field linear_eval = transcript.get_field_element("r");
        T1 += linear_eval;
    }

    t_eval += T1;

    // If linearisation is not used, the verifier needs to compute the evaluation of the linearisation polynomial r(X).
    // It has two terms, one due to the permutation argument (aka copy constraints) and the other due to the gate constraints.
    //        
    // r(X) = (a_eval.b_eval.q_M(X) + a_eval.q_L(X) + b_eval.q_R(X) + c_eval.q_O(X) + q_C(X)) +            |-------> gate constraints
    //        ((a_eval + β.z + γ)(b_eval + β.k_1.z + γ)(c_eval + β.k_2.z + γ) z(X))α -                     |     
    //        ((a_eval + β.sigma1_eval + γ)(b_eval + β.sigma2_eval + γ) β.z_eval_omega.S_{sigma3}(X))α +    )------> copy constraints
    //        z(X).L_1(z).α^{3}                                                                            |    
    //
    // Note here, we are only trying to compute the `copy constraints` part and the `gate constraints` part need to be done 
    // using Arithmetic widget.
    //
    if (!use_linearisation) {

        // Part 4: compute multiplicand of last sigma polynomial, i.e.
        // -(a_eval + β.sigma1_eval + γ)(b_eval + β.sigma2_eval + γ) β.z_eval_omega.α
        Field sigma_contribution = Field(1);
        for (size_t i = 0; i < key->program_width - 1; ++i) {
            Field permutation_evaluation = transcript.get_field_element("sigma_" + std::to_string(i + 1));
            T0 = permutation_evaluation * beta;
            T0 += wire_evaluations[i];
            T0 += gamma;
            sigma_contribution *= T0;
        }
        sigma_contribution *= z_1_shifted_eval;
        Field sigma_last_multiplicand = -(sigma_contribution * alpha);
        sigma_last_multiplicand *= beta;

        // add up part 4 to the t_eval term
        t_eval += (sigma_last_multiplicand * sigma_evaluations[key->program_width - 1]);

        Field z_eval = transcript.get_field_element("z");

        if (idpolys) {

            // Part 5.1: If idpolys is true, it indicates that we are not using the identity polynomials to 
            // represent identity permutations. In that case, we need to use the pre-defined values for
            // representing identity permutations and then compute the term
            // [(a_eval + β.id_1 + γ)(b_eval + β.id_2 + γ)(c_eval + β.id_3 + γ).α + L_1(z).α^{3}]
            Field id_contribution = Field(1);
            for (size_t i = 0; i < key->program_width; ++i) {
                Field id_evaluation = transcript.get_field_element("id_" + std::to_string(i + 1));
                T0 = id_evaluation * beta;
                T0 += wire_evaluations[i];
                T0 += gamma;
                id_contribution *= T0;
            }
            Field id_last_multiplicand = (id_contribution * alpha);
            T0 = l_start * alpha_cubed;
            id_last_multiplicand += T0;

            // add up part 5.1 to the t_eval term
            t_eval += (id_last_multiplicand * z_eval);
        }
        else {

            // Part 5.2: If idpolys is false, the identity permutations are identity polynomials. 
            // So we need to compute the following term
            // [(a_eval + β.z + γ)(b_eval + β.k_1.z + γ)(c_eval + β.k_2.z + γ).α + L_1(z).α^{3}]
            Field z_contribution = Field(1);
            for (size_t i = 0; i < key->program_width; ++i) {
                Field coset_generator = (i == 0) ? Field(1) : Field::coset_generator(i - 1);
                T0 = z_beta * coset_generator;
                T0 += wire_evaluations[i];
                T0 += gamma;
                z_contribution *= T0;
            }
            Field z_1_multiplicand = (z_contribution * alpha);
            T0 = l_start * alpha_cubed;
            z_1_multiplicand += T0;

            // add up part 5.2 to the t_eval term
            t_eval += (z_1_multiplicand * z_eval);
        }
    }

    return alpha.sqr().sqr();
}

template <typename Field, typename Group, typename Transcript, const size_t num_roots_cut_out_of_vanishing_polynomial>
Field VerifierPermutationWidget<Field, Group, Transcript, num_roots_cut_out_of_vanishing_polynomial>::append_scalar_multiplication_inputs(
    typename Transcript::Key* key,
    const Field& alpha_base,
    const Transcript& transcript,
    std::map<std::string, Field>& scalars,
    const bool use_linearisation,
    const bool idpolys)
{
    // In this method, we compute the scalars which are to be multiplied to the following
    //  1. commitment to the permutation polynomial z(X), i.e. [z]_1
    //  2. commitment to the last copy permutation S_{sigma3}(X), i.e. [sigma3]_1
    // To this end, we first compute the challenges from the transcript and fetch wire 
    // polynomials' evaluations at zeta (= z).
    //
    Field alpha_step = transcript.get_challenge_field_element("alpha");

    Field alpha_cubed = alpha_base * alpha_step.sqr();
    Field shifted_z_eval = transcript.get_field_element("z_omega");

    // We also need to compute the evaluation of lagrange polynomial L_1(X) at z.
    Field z = transcript.get_challenge_field_element("z");
    Field z_pow = z;
    for (size_t i = 0; i < key->domain.log2_size; ++i) {
        z_pow *= z_pow;
    }
    Field numerator = z_pow - Field(1);

    numerator *= key->domain.domain_inverse;
    Field l_start = numerator / (z - Field(1));

    Field beta = transcript.get_challenge_field_element("beta", 0);
    Field gamma = transcript.get_challenge_field_element("beta", 1);
    Field z_beta = z * beta;

    std::vector<Field> wire_evaluations;
    for (size_t i = 0; i < key->program_width; ++i) {
        wire_evaluations.emplace_back(transcript.get_field_element("w_" + std::to_string(i + 1)));
    }

    // Field z_omega_challenge = transcript.get_challenge_field_element_from_map("nu", "z_omega");
    //
    // Here, we start by computing the scalar multiplicand of [z]_1 while using linearisarion.
    // For standard PLONK's step 9 of the verifier, we wish to compute the term
    // [(a_eval + β.z + γ)(b_eval + β.k_1.z + γ)(c_eval + β.k_2.z + γ).α + L_1(z).α^{3}] * nu_{linear}
    //
    // If identity permutations are not represented by identity polynomials, we must accordingly compute
    // [(a_eval + β.id_1 + γ)(b_eval + β.id_2 + γ)(c_eval + β.id_3 + γ).α + L_1(z).α^{3}] * nu_{linear}
    //
    // Important: we don't add the term (u * nu_z_omega) to the scalar multiplicand of [z]_1 in this method,
    // where u = separator challenge. That is done in `populate_kate_element_map` function inside `kate_verification.hpp`.
    //
    if (use_linearisation) {
        Field linear_nu = transcript.get_challenge_field_element_from_map("nu", "r");
        Field T0;
        Field z_contribution = Field(1);
        if (!idpolys) {
            for (size_t i = 0; i < key->program_width; ++i) {
                Field coset_generator = (i == 0) ? Field(1) : Field::coset_generator(i - 1);
                T0 = z_beta * coset_generator;
                T0 += wire_evaluations[i];
                T0 += gamma;
                z_contribution *= T0;
            }
        } else {
            for (size_t i = 0; i < key->program_width; ++i) {
                Field id_evaluation = transcript.get_field_element("id_" + std::to_string(i + 1));
                Field T0 = id_evaluation * beta;
                T0 += wire_evaluations[i];
                T0 += gamma;
                z_contribution *= T0;
            }
        }
        Field z_1_multiplicand = z_contribution * alpha_base;
        T0 = l_start * alpha_cubed;
        z_1_multiplicand += T0;
        z_1_multiplicand *= linear_nu;
        scalars["Z"] += (z_1_multiplicand);
    }

    // Here, we compute the multiplicand of [sigma3]_1 as
    // -(a_eval + β.sigma1_eval + γ)(b_eval + β.sigma2_eval + γ)α.β.nu_{linear}.z_omega_eval
    if (use_linearisation) {
        Field linear_nu = transcript.get_challenge_field_element_from_map("nu", "r");
        Field sigma_contribution = Field(1);
        for (size_t i = 0; i < key->program_width - 1; ++i) {
            Field permutation_evaluation = transcript.get_field_element("sigma_" + std::to_string(i + 1));
            Field T0 = permutation_evaluation * beta;
            T0 += wire_evaluations[i];
            T0 += gamma;
            sigma_contribution *= T0;
        }
        sigma_contribution *= shifted_z_eval;
        Field sigma_last_multiplicand = -(sigma_contribution * alpha_base);
        sigma_last_multiplicand *= beta;
        sigma_last_multiplicand *= linear_nu;
        scalars["SIGMA_" + std::to_string(key->program_width)] += (sigma_last_multiplicand);
    }

    return alpha_base * alpha_step.sqr() * alpha_step;
}

template class VerifierPermutationWidget<barretenberg::fr,
                                         barretenberg::g1::affine_element,
                                         transcript::StandardTranscript>;

} // namespace waffle