#pragma once
#include <common/mem.hpp>
#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include <proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/public_inputs/public_inputs.hpp>
#include <plonk/proof_system/utils/linearizer.hpp>

#include <transcript/transcript.hpp>
#include <polynomials/iterate_over_domain.hpp>
#include <polynomials/polynomial_arithmetic.hpp>

namespace waffle {

template <size_t program_width, bool idpolys, const size_t num_roots_cut_out_of_vanishing_polynomial>
ProverPermutationWidget<program_width, idpolys, num_roots_cut_out_of_vanishing_polynomial>::ProverPermutationWidget(
    proving_key* input_key)
    : ProverRandomWidget(input_key)
{}

template <size_t program_width, bool idpolys, const size_t num_roots_cut_out_of_vanishing_polynomial>
ProverPermutationWidget<program_width, idpolys, num_roots_cut_out_of_vanishing_polynomial>::ProverPermutationWidget(
    const ProverPermutationWidget& other)
    : ProverRandomWidget(other)
{}

template <size_t program_width, bool idpolys, const size_t num_roots_cut_out_of_vanishing_polynomial>
ProverPermutationWidget<program_width, idpolys, num_roots_cut_out_of_vanishing_polynomial>::ProverPermutationWidget(
    ProverPermutationWidget&& other)
    : ProverRandomWidget(other)
{}

template <size_t program_width, bool idpolys, const size_t num_roots_cut_out_of_vanishing_polynomial>
ProverPermutationWidget<program_width, idpolys, num_roots_cut_out_of_vanishing_polynomial>& ProverPermutationWidget<
    program_width,
    idpolys,
    num_roots_cut_out_of_vanishing_polynomial>::operator=(const ProverPermutationWidget& other)
{
    ProverRandomWidget::operator=(other);
    return *this;
}

template <size_t program_width, bool idpolys, const size_t num_roots_cut_out_of_vanishing_polynomial>
ProverPermutationWidget<program_width, idpolys, num_roots_cut_out_of_vanishing_polynomial>& ProverPermutationWidget<
    program_width,
    idpolys,
    num_roots_cut_out_of_vanishing_polynomial>::operator=(ProverPermutationWidget&& other)
{
    ProverRandomWidget::operator=(other);
    return *this;
}

/**
 * @brief:
 * - Computes the permutation polynomial z(X)
 * - Commits to z(X)
 * - Computes & stores the coset form of z(X) for later use in quotient polynomial calculation.
 */
template <size_t program_width, bool idpolys, const size_t num_roots_cut_out_of_vanishing_polynomial>
void ProverPermutationWidget<program_width, idpolys, num_roots_cut_out_of_vanishing_polynomial>::
    compute_round_commitments(transcript::StandardTranscript& transcript, const size_t round_number, work_queue& queue)
{
    if (round_number != 3) {
        return;
    }

    // Allocate scratch space in memory for computation of lagrange form of permutation polynomial
    // 'z_perm'. Elements 2,...,n of z_perm are constructed in place in accumulators[0]. (The first
    // element of z_perm is one, i.e. z_perm[0] == 1). The remaining accumulators are used only as scratch
    // space. All memory allocated for the accumulators is freed before termination of this function.
    size_t num_accumulators = (program_width == 1) ? 3 : program_width * 2;
    fr* accumulators[num_accumulators];
    // Allocate the required number of length n scratch space arrays
    for (size_t k = 0; k < num_accumulators; ++k) {
        accumulators[k] = static_cast<fr*>(aligned_alloc(64, sizeof(fr) * key->circuit_size));
    }

    barretenberg::fr beta = fr::serialize_from_buffer(transcript.get_challenge("beta").begin());
    barretenberg::fr gamma = fr::serialize_from_buffer(transcript.get_challenge("beta", 1).begin());

    std::array<const fr*, program_width> lagrange_base_wires;
    std::array<const fr*, program_width> lagrange_base_sigmas;
    [[maybe_unused]] std::array<const fr*, program_width> lagrange_base_ids;

    for (size_t i = 0; i < program_width; ++i) {
        lagrange_base_wires[i] =
            key->polynomial_cache.get("w_" + std::to_string(i + 1) + "_lagrange").get_coefficients();
        lagrange_base_sigmas[i] =
            key->polynomial_cache.get("sigma_" + std::to_string(i + 1) + "_lagrange").get_coefficients();

        // If idpolys = true, it implies that we do NOT use the identity permutation
        // S_ID1(X) = X, S_ID2(X) = k_1X, S_ID3(X) = k_2X.
        if constexpr (idpolys)
            lagrange_base_ids[i] =
                key->polynomial_cache.get("id_" + std::to_string(i + 1) + "_lagrange").get_coefficients();
    }

#ifndef NO_MULTITHREADING
#pragma omp parallel
#endif
    {
        // When we write w_i it means the evaluation of witness polynomial at i-th index.
        // When we write w^{i} it means the generator of the subgroup to the i-th power.
        //
        // step 1: compute the individual terms in the permutation poylnomial.
        //
        // Consider the case in which we use identity permutation polynomials and let program width = 3.
        // (extending it to the case when the permutation polynomials is not identity is trivial).
        //
        // coefficient of L_1: 1
        //
        // coefficient of L_2:
        //
        //  coeff_of_L1 *   (w_1 + γ + β.ω^{0}) . (w_{n+1} + γ + β.k_1.ω^{0}) . (w_{2n+1} + γ + β.k_2.ω^{0})
        //                  ---------------------------------------------------------------------------------
        //                  (w_1 + γ + β.σ(1) ) . (w_{n+1} + γ + β.σ(n+1)   ) . (w_{2n+1} + γ + β.σ(2n+1)  )
        //
        // coefficient of L_3:
        //
        //  coeff_of_L2 *   (w_2 + γ + β.ω^{1}) . (w_{n+2} + γ + β.k_1.ω^{1}) . (w_{2n+2} + γ + β.k_2.ω^{1})
        //                  --------------------------------------------------------------------------------
        //                  (w_2 + γ + β.σ(2) ) . (w_{n+2} + γ + β.σ(n+2)   ) . (w_{2n+2} + γ + β.σ(2n+2)  )
        // and so on...
        //
        // accumulator data structure:
        // numerators are stored in accumulator[0: program_width-1],
        // denominators are stored in accumulator[program_width:]
        //
        //      0                                1                                      (n-1)
        // 0 -> (w_1      + γ + β.ω^{0}    ),    (w_2      + γ + β.ω^{1}    ),    ...., (w_n      + γ + β.ω^{n-1}    )
        // 1 -> (w_{n+1}  + γ + β.k_1.ω^{0}),    (w_{n+1}  + γ + β.k_1.ω^{2}),    ...., (w_{n+1}  + γ + β.k_1.ω^{n-1})
        // 2 -> (w_{2n+1} + γ + β.k_2.ω^{0}),    (w_{2n+1} + γ + β.k_2.ω^{0}),    ...., (w_{2n+1} + γ + β.k_2.ω^{n-1})
        //
        // 3 -> (w_1      + γ + β.σ(1)     ),    (w_2      + γ + β.σ(2)     ),    ...., (w_n      + γ + β.σ(n)       )
        // 4 -> (w_{n+1}  + γ + β.σ(n+1)   ),    (w_{n+1}  + γ + β.σ{n+2}   ),    ...., (w_{n+1}  + γ + β.σ{n+n}     )
        // 5 -> (w_{2n+1} + γ + β.σ(2n+1)  ),    (w_{2n+1} + γ + β.σ(2n+2)  ),    ...., (w_{2n+1} + γ + β.σ(2n+n)    )
        //
        // Thus, to obtain coefficient_of_L2, we need to use accumulators[:][0]:
        //    acc[0][0]*acc[1][0]*acc[2][0] / acc[program_width][0]*acc[program_width+1][0]*acc[program_width+2][0]
        //
        // To obtain coefficient_of_L3, we need to use accumulator[:][0] and accumulator[:][1]
        // and so on upto coefficient_of_Ln.
#ifndef NO_MULTITHREADING
#pragma omp for
#endif
        // Recall: In a domain: num_threads * thread_size = size (= subgroup_size)
        //        |  0 |  1 |  2 |  3 |  4 |  5 |  6 |  7 |  8 |  9 | 10 | 11 | 12 | 13 | 14 | 15 | <-- n = 16
        //    j:  |    0    |    1    |    2    |    3    |    4    |    5    |    6    |    7    | num_threads = 8
        //    i:     0    1    0    1    0    1    0    1    0    1    0    1    0    1    0    1   thread_size = 2
        // So i will access a different element from 0..(n-1) each time.
        // Commented maths notation mirrors the indexing from the giant comment immediately above.
        for (size_t j = 0; j < key->small_domain.num_threads; ++j) {
            barretenberg::fr thread_root = key->small_domain.root.pow(
                static_cast<uint64_t>(j * key->small_domain.thread_size)); // effectively ω^{i} in inner loop
            [[maybe_unused]] barretenberg::fr cur_root_times_beta = thread_root * beta; // β.ω^{i}
            barretenberg::fr T0;
            barretenberg::fr wire_plus_gamma;
            size_t start = j * key->small_domain.thread_size;
            size_t end = (j + 1) * key->small_domain.thread_size;
            for (size_t i = start; i < end; ++i) {
                wire_plus_gamma = gamma + lagrange_base_wires[0][i]; // w_{i + 1} + γ
                                                                     // i in 0..(n-1)
                if constexpr (!idpolys) {
                    accumulators[0][i] = wire_plus_gamma + cur_root_times_beta; // w_{i + 1} + γ + β.ω^{i}
                }
                if constexpr (idpolys) {
                    T0 = lagrange_base_ids[0][i] * beta;       // β.id(i + 1)
                    accumulators[0][i] = T0 + wire_plus_gamma; // w_{i + 1} + γ + β.id(i + 1)
                }

                T0 = lagrange_base_sigmas[0][i] * beta;                // β.σ(i + 1)
                accumulators[program_width][i] = T0 + wire_plus_gamma; // w_{i + 1} + γ + β.σ(i + 1)

                for (size_t k = 1; k < program_width; ++k) {
                    wire_plus_gamma = gamma + lagrange_base_wires[k][i]; // w_{k.n + i + 1} + γ
                                                                         // i in 0..(n-1)
                    if constexpr (idpolys) {
                        T0 = lagrange_base_ids[k][i] * beta; // β.id(k.n + i + 1)
                    } else {
                        T0 = fr::coset_generator(k - 1) * cur_root_times_beta; // β.k_{k}.ω^{i}
                                                                               //   ^coset generator k
                    }
                    accumulators[k][i] = T0 + wire_plus_gamma; // w_{k.n + i + 1} + γ + β.id(k.n + i + 1)

                    T0 = lagrange_base_sigmas[k][i] * beta;                    // β.σ(k.n + i + 1)
                    accumulators[k + program_width][i] = T0 + wire_plus_gamma; // w_{k.n + i + 1} + γ + β.σ(k.n + i + 1)
                }
                if constexpr (!idpolys)
                    cur_root_times_beta *= key->small_domain.root; // β.ω^{i + 1}
            }
        }

        // Step 2: compute the constituent components of z(X). This is a small multithreading bottleneck, as we have
        // program_width * 2 non-parallelizable processes
        //
        // Update the accumulator matrix a[:][:] to contain the left products like so:
        //      0           1                     2                          (n-1)
        // 0 -> (a[0][0]),  (a[0][1] * a[0][0]),  (a[0][2] * a[0][1]), ...,  (a[0][n-1] * a[0][n-2])
        // 1 -> (a[1][0]),  (a[1][1] * a[1][0]),  (a[1][2] * a[1][1]), ...,  (a[1][n-1] * a[1][n-2])
        // 2 -> (a[2][0]),  (a[2][1] * a[2][0]),  (a[2][2] * a[2][1]), ...,  (a[2][n-1] * a[2][n-2])
        //
        // 3 -> (a[3][0]),  (a[3][1] * a[3][0]),  (a[3][2] * a[3][1]), ...,  (a[3][n-1] * a[3][n-2])
        // 4 -> (a[4][0]),  (a[4][1] * a[4][0]),  (a[4][2] * a[4][1]), ...,  (a[4][n-1] * a[4][n-2])
        // 5 -> (a[5][0]),  (a[5][1] * a[5][0]),  (a[5][2] * a[5][1]), ...,  (a[5][n-1] * a[5][n-2])
        //
        // and so on...
#ifndef NO_MULTITHREADING
#pragma omp for
#endif
        for (size_t i = 0; i < program_width * 2; ++i) {
            fr* coeffs = &accumulators[i][0]; // start from the beginning of a row
            for (size_t j = 0; j < key->small_domain.size - 1; ++j) {
                coeffs[j + 1] *= coeffs[j]; // iteratively update elements in subsequent columns
            }
        }

        // step 3: concatenate together the accumulator elements into z(X)
        //
        // Update each element of the accumulator row a[0] to be the product of itself with the 'numerator' rows beneath
        // it, and update each element of a[program_width] to be the product of itself with the 'denominator' rows
        // beneath it.
        //
        //       0                                     1                                           (n-1)
        // 0 ->  (a[0][0] * a[1][0] * a[2][0]),        (a[0][1] * a[1][1] * a[2][1]),        ...., (a[0][n-1] *
        // a[1][n-1] * a[2][n-1])
        //
        // pw -> (a[pw][0] * a[pw+1][0] * a[pw+2][0]), (a[pw][1] * a[pw+1][1] * a[pw+2][1]), ...., (a[pw][n-1] *
        // a[pw+1][n-1] * a[pw+2][n-1])
        //
        // Note that pw = program_width
        //
        // Hereafter, we can compute
        // coefficient_Lj = a[0][j]/a[pw][j]
        //
        // Naive way of computing these coefficients would result in n inversions, which is pretty expensive.
        // Instead we use Montgomery's trick for batch inversion.
        // Montgomery's trick documentation:
        // ./src/aztec/ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp/L286
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

                // N.B. accumulators[0][i] = z_perm[i + 1]
                // We can avoid fully reducing z_perm[i + 1] as the inverse fft will take care of that for us
                accumulators[0][i] = inversion_accumulator * inversion_coefficients[i];
                inversion_accumulator *= accumulators[program_width][i];
            }
        }
    }

    // Construct permutation polynomial 'z' in lagrange form as:
    // z = [1 accumulators[0][0] accumulators[0][1] ... accumulators[0][n-2]]
    polynomial z_perm(key->circuit_size, key->circuit_size);
    z_perm[0] = fr::one();
    barretenberg::polynomial_arithmetic::copy_polynomial(
        accumulators[0], &z_perm[1], key->circuit_size - 1, key->circuit_size - 1);

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
    // polynomial looks uniformly random.
    // z'(X) = z(X) + (r_3.X^2 + r_4.X + r_5)
    // where r_3, r_4, r_5 are uniformly random scalar field elements.
    //
    // Furthermore, instead of adding random polynomials, we could directly add random scalars in the lagrange-
    // basis forms of the witness and permutation polynomials. This is because we are using a modified vanishing
    // polynomial of the form
    //                           (X^n - 1)
    // Z*_H(X) = ------------------------------------------
    //           (X - ω^{n-1}).(X - ω^{n-2})...(X - ω^{n-k})
    // where ω = n-th root of unity, k = num_roots_cut_out_of_vanishing_polynomials.
    // Thus, the last k places in the lagrange basis form of z(X) are empty. We can therefore utilise them and
    // add random scalars as coefficients of L_{n-1}, L_{n-2},... and so on.
    //
    // Note: The number of coefficients in the permutation polynomial z(X) is (n - k + 1) DOCTODO: elaborate on why.
    // (refer to Round 2 in the PLONK paper). Hence, if we cut 3 roots out of the vanishing polynomial,
    // we are left with only 2 places (coefficients) in the z array to add randomness. To have the last 3 places
    // available for adding random scalars, we therefore need to cut at least 4 roots out of the vanishing polynomial.
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
        z_perm[(key->circuit_size - num_roots_cut_out_of_vanishing_polynomial) + 1 + k] = fr::random_element();
    }

    z_perm.ifft(key->small_domain);

    // free memory allocated for scratch space
    for (size_t k = 0; k < num_accumulators; ++k) {
        aligned_free(accumulators[k]);
    }

    // Commit to z:
    queue.add_to_queue({
        work_queue::WorkType::SCALAR_MULTIPLICATION,
        z_perm.get_coefficients(),
        "Z_PERM",
        barretenberg::fr(0),
        0,
    });

    // Compute coset-form of z:
    queue.add_to_queue({
        work_queue::WorkType::FFT,
        nullptr,
        "z_perm",
        barretenberg::fr(0),
        0,
    });

    key->polynomial_cache.put("z_perm", std::move(z_perm));
}

template <size_t program_width, bool idpolys, const size_t num_roots_cut_out_of_vanishing_polynomial>
barretenberg::fr ProverPermutationWidget<program_width, idpolys, num_roots_cut_out_of_vanishing_polynomial>::
    compute_quotient_contribution(const fr& alpha_base, const transcript::StandardTranscript& transcript)
{
    const polynomial& z_perm_fft = key->polynomial_cache.get("z_perm_fft");

    barretenberg::fr alpha_squared = alpha_base.sqr();
    barretenberg::fr beta = fr::serialize_from_buffer(transcript.get_challenge("beta").begin());
    barretenberg::fr gamma = fr::serialize_from_buffer(transcript.get_challenge("beta", 1).begin());

    // Initialise the (n + 1)th coefficients of quotient parts so that reuse of proving
    // keys does not use some residual data from another proof.
    key->quotient_polynomial_parts[0][key->circuit_size] = 0;
    key->quotient_polynomial_parts[1][key->circuit_size] = 0;
    key->quotient_polynomial_parts[2][key->circuit_size] = 0;

    // Our permutation check boils down to two 'grand product' arguments, that we represent with a single polynomial
    // z(X). We want to test that z(X) has been constructed correctly. When evaluated at elements of ω ∈ H, the
    // numerator of z(ω) will equal the identity permutation grand product, and the denominator will equal the copy
    // permutation grand product.

    // The identity that we need to evaluate is: z(X.ω).(permutation grand product) == z(X).(identity grand product)
    // i.e. The next element of z is equal to the current element of z, multiplied by (identity grand product) /
    // (permutation grand product)

    // This method computes `(identity grand product).z(X).α`.
    // The random `alpha` is there to ensure our grand product polynomial identity is linearly independent from the
    // other polynomial identities that we are going to roll into the quotient polynomial T(X).

    // Specifically, we want to compute:
    // (w_l(X) + β.σ_1(X) + γ).(w_r(X) + β.σ_2(X) + γ).(w_o(X) + β.σ_3(X) + γ).z(X).α
    // Once we divide by the vanishing polynomial, this will be a degree 3n polynomial. (4 * (n-1) - (n-4)).

    std::array<const fr*, program_width> wire_ffts;
    std::array<const fr*, program_width> sigma_ffts;
    [[maybe_unused]] std::array<const fr*, program_width> id_ffts;

    for (size_t i = 0; i < program_width; ++i) {

        // wire_fft[0] contains the fft of the wire polynomial w_1
        // sigma_fft[0] contains the fft of the permutation selector polynomial \sigma_1
        wire_ffts[i] = key->polynomial_cache.get("w_" + std::to_string(i + 1) + "_fft").get_coefficients();
        sigma_ffts[i] = key->polynomial_cache.get("sigma_" + std::to_string(i + 1) + "_fft").get_coefficients();

        // idpolys is FALSE iff the "identity permutation" is used as a monomial
        // as a part of the permutation polynomial
        // <=> idpolys = FALSE
        if constexpr (idpolys)
            id_ffts[i] = key->polynomial_cache.get("id_" + std::to_string(i + 1) + "_fft").get_coefficients();
    }

    // we start with lagrange polynomial L_1(X)
    const polynomial& l_start = key->polynomial_cache.get("lagrange_1_fft");

    // Compute our public input component
    std::vector<barretenberg::fr> public_inputs = many_from_buffer<fr>(transcript.get_element("public_inputs"));

    barretenberg::fr public_input_delta =
        compute_public_input_delta<fr>(public_inputs, beta, gamma, key->small_domain.root);

    const size_t block_mask = key->large_domain.size - 1;
    // Step 4: Set the quotient polynomial to be equal to
#ifndef NO_MULTITHREADING
#pragma omp parallel for
#endif
    for (size_t j = 0; j < key->large_domain.num_threads; ++j) {
        const size_t start = j * key->large_domain.thread_size;
        const size_t end = (j + 1) * key->large_domain.thread_size;

        // Leverage multi-threading by computing quotient polynomial at points
        // (ω^{j * num_threads}, ω^{j * num_threads + 1}, ..., ω^{j * num_threads + num_threads})
        //
        // curr_root = ω^{j * num_threads} * g_{small} * β
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
                // start with (w_l(X) + β.X + γ)
                numerator = cur_root_times_beta + wire_plus_gamma;
            else
                numerator = id_ffts[0][i] * beta + wire_plus_gamma;

            // Denominator computation
            // start with (w_l(X) + β.σ_1(X) + γ)
            denominator = sigma_ffts[0][i] * beta;
            denominator += wire_plus_gamma;

            for (size_t k = 1; k < program_width; ++k) {
                wire_plus_gamma = gamma + wire_ffts[k][i];
                if constexpr (!idpolys)
                    // (w_r(X) + β.(k_{k}.X) + γ)
                    T0 = fr::coset_generator(k - 1) * cur_root_times_beta;
                if constexpr (idpolys)
                    T0 = id_ffts[k][i] * beta;

                T0 += wire_plus_gamma;
                numerator *= T0;

                // (w_r(X) + β.σ_{k}(X) + γ)
                T0 = sigma_ffts[k][i] * beta;
                T0 += wire_plus_gamma;
                denominator *= T0;
            }

            numerator *= z_perm_fft[i];
            denominator *= z_perm_fft[(i + 4) & block_mask];

            /**
             * Permutation bounds check
             * (z(X.w) - 1).(α^3).L_{end}(X) = T(X).Z*_H(X)
             *
             * where Z*_H(X) = (X^n - 1)/[(X - ω^{n-1})...(X - ω^{n - num_roots_cut_out_of_vanishing_polynomial})]
             * i.e. we remove some roots from the true vanishing polynomial to ensure that the overall degree
             * of the permutation polynomial is <= n.
             * Read more on this here: https://hackmd.io/1DaroFVfQwySwZPHMoMdBg
             *
             * Therefore, L_{end} = L_{n - num_roots_cut_out_of_vanishing_polynomial}
             **/
            // The α^3 term is so that we can subsume this polynomial into the quotient polynomial,
            // whilst ensuring the term is linearly independent form the other terms in the quotient polynomial

            // We want to verify that z(X) equals `1` when evaluated at `ω_n`, the 'last' element of our multiplicative
            // subgroup H. But PLONK's 'vanishing polynomial', Z*_H(X), isn't the true vanishing polynomial of subgroup
            // H. We need to cut a root of unity out of Z*_H(X), specifically `ω_n`, for our grand product argument.
            // When evaluating z(X) has been constructed correctly, we verify that z(X.ω).(identity permutation product)
            // = z(X).(sigma permutation product), for all X \in H. But this relationship breaks down for X = ω_n,
            // because z(X.ω) will evaluate to the *first* element of our grand product argument. The last element of
            // z(X) has a dependency on the first element, so the first element cannot have a dependency on the last
            // element.

            // TODO: With the reduction from 2 z polynomials to a single z(X), the above no longer applies
            // TODO: Fix this to remove the (z(X.ω) - 1).L_{n-1}(X) check

            // To summarise, we can't verify claims about z(X) when evaluated at `ω_n`.
            // But we can verify claims about z(X.ω) when evaluated at `ω_{n-1}`, which is the same thing

            // To summarise the summary: If z(ω_n) = 1, then (z(X.ω) - 1).L_{n-1}(X) will be divisible by Z_H*(X)
            // => add linearly independent term (z(X.ω) - 1).(α^3).L{n-1}(X) into the quotient polynomial to check
            // this

            // z_perm_fft already contains evaluations of Z(X).(\alpha^2)
            // at the (4n)'th roots of unity
            // => to get Z(X.w) instead of Z(X), index element (i+4) instead of i
            T0 = z_perm_fft[(i + 4) & block_mask] - public_input_delta; // T0 = (Z(X.w) - (delta)).(\alpha^2)
            T0 *= alpha_base;                                           // T0 = (Z(X.w) - (delta)).(\alpha^3)

            // T0 = (z(X.ω) - Δ).(α^3).L_{end}
            // where L_{end} = L{n - num_roots_cut_out_of_vanishing_polynomial}.
            //
            // Note that L_j(X) = L_1(X . ω^{-j}) = L_1(X . ω^{n-j})
            // => L_{end}= L_1(X . ω^{num_roots_cut_out_of_vanishing_polynomial + 1})
            // => fetch the value at index (i + (num_roots_cut_out_of_vanishing_polynomial + 1) * 4) in l_1
            // the factor of 4 is because l_1 is a 4n-size fft.
            //
            // Recall, we use l_start for l_1 for consistency in notation.
            T0 *= l_start[(i + 4 + 4 * num_roots_cut_out_of_vanishing_polynomial) & block_mask];
            numerator += T0;

            // Step 2: Compute (z(X) - 1).(α^4).L1(X)
            // We need to verify that z(X) equals `1` when evaluated at the first element of our subgroup H
            // i.e. z(X) starts at 1 and ends at 1
            // The `alpha^4` term is so that we can add this as a linearly independent term in our quotient polynomial
            T0 = z_perm_fft[i] - fr(1); // T0 = (Z(X) - 1).(\alpha^2)
            T0 *= alpha_squared;        // T0 = (Z(X) - 1).(\alpha^4)
            T0 *= l_start[i];           // T0 = (Z(X) - 1).(\alpha^2).L1(X)
            numerator += T0;

            // Combine into quotient polynomial
            T0 = numerator - denominator;
            key->quotient_polynomial_parts[i >> key->small_domain.log2_size][i & (key->circuit_size - 1)] =
                T0 * alpha_base;

            // Update our working root of unity
            cur_root_times_beta *= key->large_domain.root;
        }
    }
    return alpha_base.sqr().sqr();
}

template <size_t program_width, bool idpolys, const size_t num_roots_cut_out_of_vanishing_polynomial>
barretenberg::fr ProverPermutationWidget<program_width, idpolys, num_roots_cut_out_of_vanishing_polynomial>::
    compute_linear_contribution(const fr& alpha, const transcript::StandardTranscript& transcript, polynomial& r)
{
    // This method computes a part of the linearisation polynomial r(X) in round 4 of prover's algorithm in PLONK paper/
    // Since this is a member function of the `ProverPermutationWidget` class, we only compute the terms relevant to the
    // copy constraints. Concretely, we compute the terms:
    //
    // r(X) = (a_eval.b_eval.q_M(X) + a_eval.q_L(X) + b_eval.q_R(X) + c_eval.q_O(X) + q_C(X)) +
    // gate constraints
    //        ((a_eval + β.z + γ)(b_eval + β.k_1.z + γ)(c_eval + β.k_2.z + γ) z(X)).α -
    //        ((a_eval + β.sigma1_eval + γ)(b_eval + β.sigma2_eval + γ) β.z_eval_omega.S_{sigma3}(X)).α +
    //         z(X).L_1(z).α^{3} +
    //         −(a_eval+βS_σ1_eval+γ)(b_eval+βS_σ2_eval+γ)(c_eval+γ).zω.α −
    // α^3.L1(z) + α^2.(z_ω_eval−ΔPI).L_n(z) − Z_H(z).(t_lo(X)+z^n * t_mid(X)+z^{2n} * t_hi(X)).
    // copy constraints
    // Note here, we are only computing the `copy constraints` part and the `gate constraints` part need to be
    // done using Arithmetic widget. The term −Z_H(z)(t_lo(X)+z^n * t_mid(X)+z^{2n} * t_hi(X)) is added in prover.cpp.
    // A detailed discussion on this can be found at https://hackmd.io/vUGG8CO_Rk2iEjruBL_gGw?view
    // Also, the prover calls this function only when linearisation trick is used, so we
    // don't need to explicitly check the condition `use_linearisation`.
    //

    // Step 1: regenrate challenges, fetch evaluations wire polynomials
    const polynomial& z_perm = key->polynomial_cache.get("z_perm");
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

    barretenberg::fr z_1_shifted_eval = fr::serialize_from_buffer(&transcript.get_element("z_perm_omega")[0]);

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

    // Step 4: compute the multiplicand of S_{σ3}(X), the multiplicand of (c_eval+γ) i.e.
    // −(a_eval+βS_{σ1}_eval+γ)(b_eval+βS_{σ2}_eval+γ)z_ω_evalα, and (c_eval+γ). Note that
    // the multiplicand of S_{σ3}(X) = β * (multiplicand of (c_eval+γ))
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
    barretenberg::fr c_eval_gamma = wire_evaluations[program_width - 1] + gamma;
    barretenberg::fr c_eval_gamma_multiplicand = sigma_last_multiplicand;
    sigma_last_multiplicand *= beta;

    // Step 6: Fetching S_{σ3}(X), public input component
    const polynomial& sigma_last = key->polynomial_cache.get("sigma_" + std::to_string(program_width));
    std::vector<barretenberg::fr> public_inputs = many_from_buffer<fr>(transcript.get_element("public_inputs"));
    barretenberg::fr public_input_delta =
        compute_public_input_delta<fr>(public_inputs, beta, gamma, key->small_domain.root);
    // Compute pi_term = α^2(z_w_eval − Δ_PI)Ln(z)
    barretenberg::fr pi_term = z_1_shifted_eval - public_input_delta;
    pi_term *= lagrange_evals.l_end;
    pi_term *= alpha.sqr();
    // Step 7: add up the z(X), S_{sigma3}(X), c_eval_gamma, and l_one_z_alpha_cube terms into r(X)
    ITERATE_OVER_DOMAIN_START(key->small_domain);
    r[i] = (z_perm[i] * z_1_multiplicand) + (sigma_last[i] * sigma_last_multiplicand);
    ITERATE_OVER_DOMAIN_END;
    // We should add constant terms with r[0] only
    r[0] += (c_eval_gamma * c_eval_gamma_multiplicand) - lagrange_evals.l_start * alpha.sqr() * alpha + pi_term;

    return alpha.sqr().sqr();
}

// ###

template <typename Field, typename Group, typename Transcript, const size_t num_roots_cut_out_of_vanishing_polynomial>
VerifierPermutationWidget<Field, Group, Transcript, num_roots_cut_out_of_vanishing_polynomial>::
    VerifierPermutationWidget()
{}

/**
 * @brief This function computes the part of the quotient polynomial evaluation relevant to PLONK's permutation
 argument.
 * Note that this is needed by the verifier in step 8 of verification algorithm. This has two flavours:
 *   1. The prover is using linearisation (use_linearisation = true)
 *      i.e. they're computing the polynomial r(X) and sending its evaluation r_eval = r(ʓ).
 *   2. The prover is not using linearisation and the overhead computation needs to be done by the verifier.
 *
 * @param key the verification key
 * @param alpha the quotient challenge (same name as in the Plonk paper)
 * @param transcript
 * @param r_0 this will be mutated by this function.
 * @param use_linearisation (see above)
 * @param idpolys describes whether we're using Vitalik's trick of using the trivial identity polys (idpolys=false), or
 *        whether the identity polys are circuit-specific and stored in the proving/verification key (idpolys=true).
 */
template <typename Field, typename Group, typename Transcript, const size_t num_roots_cut_out_of_vanishing_polynomial>
Field VerifierPermutationWidget<Field, Group, Transcript, num_roots_cut_out_of_vanishing_polynomial>::
    compute_quotient_evaluation_contribution(typename Transcript::Key* key,
                                             const Field& alpha,
                                             const Transcript& transcript,
                                             Field& r_0,
                                             const bool use_linearisation,
                                             const bool idpolys)

{
    Field alpha_squared = alpha.sqr();
    Field alpha_cubed = alpha_squared * alpha;
    Field z = transcript.get_challenge_field_element("z"); // a.k.a. zeta or ʓ
    Field beta = transcript.get_challenge_field_element("beta", 0);
    Field gamma = transcript.get_challenge_field_element("beta", 1);
    Field z_beta = z * beta;

    // We need wire polynomials' and sigma polynomials' evaluations at zeta which we fetch from the transcript.
    // Fetch a_eval, b_eval, c_eval, sigma1_eval, sigma2_eval
    std::vector<Field> wire_evaluations;
    std::vector<Field> sigma_evaluations;

    // E.g. in standard plonk, S_σ_3(X) is not evaluated when computing the linearisation polynomial:
    const size_t num_sigma_evaluations = (use_linearisation ? key->program_width - 1 : key->program_width);

    for (size_t i = 0; i < num_sigma_evaluations; ++i) {
        std::string index = std::to_string(i + 1);
        sigma_evaluations.emplace_back(transcript.get_field_element("sigma_" + index)); // S_σ_i(ʓ)
    }

    for (size_t i = 0; i < key->program_width; ++i) {
        wire_evaluations.emplace_back(transcript.get_field_element(
            "w_" + std::to_string(
                       i + 1))); // w_i(ʓ)
                                 // (Note: in the Plonk paper, these polys are called a, b, c. We interchangeably call
                                 // them a,b,c or w_l, w_r, w_o, or w_1, w_2, w_3,... depending on the context).
    }

    // Compute evaluations of lagrange polynomials L_1(X) and L_{n-k} at ʓ.
    // Recall, k is the number of roots cut out of the vanishing polynomial Z_H(X), to yield Z_H*(X).
    // Note that
    //                                  X^n - 1
    // L_i(X) = L_1(X.ω^{-i + 1}) = -----------------
    //                               X.ω^{-i + 1} - 1
    //
    Field numerator = key->z_pow_n - Field(1); // ʓ^n - 1

    numerator *= key->domain.domain_inverse;
    Field l_start = numerator / (z - Field(1)); // [ʓ^n - 1] / [n.(ʓ - 1)] =: L_1(ʓ)

    // Compute ω^{num_roots_cut_out_of_vanishing_polynomial + 1}
    Field l_end_root = (num_roots_cut_out_of_vanishing_polynomial & 1) ? key->domain.root.sqr() : key->domain.root;
    for (size_t i = 0; i < num_roots_cut_out_of_vanishing_polynomial / 2; ++i) {
        l_end_root *= key->domain.root.sqr();
    }
    Field l_end = numerator / ((z * l_end_root) - Field(1)); // [ʓ^n - 1] / [n.(ʓ.ω^{k+1} - 1)] =: L_{n-k}(ʓ)

    Field z_1_shifted_eval = transcript.get_field_element("z_perm_omega");

    // Reconstruct the evaluation of the quotient polynomial, t(ʓ), from prover messages.
    // Recall:
    //
    // t(X) = 1/Z_H*(X) * (
    //         [   a(X).b(X).qm(X) + a(X).ql(X) + b(X).qr(X) + c(X).qo(X) + qc(X) ]
    //   +   α.[
    //             [ a(X) + β.X + γ)(b(X) + β.k_1.X + γ)(c(X) + β.k_2.X + γ).z(X) ]
    //           - [ a(X) + β.Sσ1(X) + γ)(b(X) + β.Sσ2(X) + γ)(c(X) + β.Sσ3(X) + γ).z(X.ω) ]
    //         ]
    //   + α^3.[ (z(X) - 1).L_1(X) ]
    //   + α^2.[ (z(X.ω) - ∆_PI).L_{n-k}(X) ]
    // )
    //
    Field T1;
    Field T2;

    // Part 1: compute the sigma contribution, i.e.
    //
    // sigma_contribution = (a_eval + β.sigma1_eval + γ)(b_eval + β.sigma2_eval + γ)(c_eval + γ).z(ʓ.ω).α
    //
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
    sigma_contribution *= alpha;

    // Part 2: compute the public-inputs term, i.e.
    //
    // (z(ʓ.ω) - ∆_{PI}).L_{n-k}(ʓ).α^2
    //
    // (See the separate paper which alters the 'public inputs' component of the plonk protocol)
    std::vector<Field> public_inputs = transcript.get_field_element_vector("public_inputs");
    Field public_input_delta = compute_public_input_delta<Field>(public_inputs, beta, gamma, key->domain.root);

    T1 = z_1_shifted_eval - public_input_delta;
    T1 *= l_end;
    T1 *= alpha_squared;

    // Part 3: compute starting lagrange polynomial term, i.e.
    //
    // L_1(ʓ).α^3
    //
    T2 = l_start * alpha_cubed;

    // Combine parts 1, 2, 3. If linearisation is used, we need to add r_eval to T1 and we're done.
    //
    // r_0 =   α^2.(z(ʓ.ω) - ∆_{PI}).L_{n-k}(ʓ)
    //       - α^3.L_1(ʓ)
    //       -   α.(a_eval + β.sigma1_eval + γ)(b_eval + β.sigma2_eval + γ)(c_eval + γ).z(ʓ.ω)
    //
    T1 -= T2;
    T1 -= sigma_contribution;
    r_0 += T1;

    // If we use linearisation, then r_0 computed at this point is equal to the constant contribution of r(X).
    if (use_linearisation) {
        return alpha_squared.sqr();
    }

    // If linearisation is not used, the verifier needs to compute the evaluation of the linearisation polynomial r(X)
    // at ʓ (i.e. r(ʓ)).
    //
    // r(X) has two terms:
    //   - one due to the permutation argument (aka copy constraints);
    //   - and the other due to the gate constraints.
    // (See the separate paper which alters the 'public inputs' component of the plonk protocol)
    //
    // r(X) =     (a_eval.b_eval.q_M(X) + a_eval.q_L(X) + b_eval.q_R(X) + c_eval.q_O(X) + q_C(X))        |-> gate
    //                                                                                                       constraints
    //      +   α.(a_eval + β.ʓ + γ)(b_eval + β.k_1.ʓ + γ)(c_eval + β.k_2.ʓ + γ).z(X)                    |
    //      -   α.(a_eval + β.sigma1_eval + γ)(b_eval + β.sigma2_eval + γ).β.z(ʓ.ω).S_{sigma3}(X)        |-> copy
    //      + α^3.L_1(ʓ).z(X)                                                                            |   constraints
    //
    // Note here, we are only trying to compute the `copy constraints` part. The `gate constraints` part is calculated
    // in the Arithmetic widget.
    //
    else {

        // Part 4: compute multiplicand of last sigma polynomial S_{sigma3}(X), i.e.
        //
        // - α.(a_eval + β.sigma1_eval + γ)(b_eval + β.sigma2_eval + γ).β.z(ʓ.ω)
        //
        Field sigma_contribution = Field(1);
        for (size_t i = 0; i < key->program_width - 1; ++i) {
            T0 = sigma_evaluations[i] * beta;
            T0 += wire_evaluations[i];
            T0 += gamma;
            sigma_contribution *= T0;
        }
        sigma_contribution *= z_1_shifted_eval;
        Field sigma_last_multiplicand = -(sigma_contribution * alpha);
        sigma_last_multiplicand *= beta;

        // Add up part 4 to the r_0 term
        //
        // At this intermediate stage, r_0 will be:
        //
        // r_0 =   α^2.(z(ʓ.ω) - ∆_{PI}).L_{n-k}(ʓ)                                                      |
        //       - α^3.L_1(ʓ)                                                                            |-> r_0 from
        //       -   α.(a_eval + β.sigma1_eval + γ)(b_eval + β.sigma2_eval + γ)(c_eval + γ).z(ʓ.ω)       |   before
        //
        //       -   α.(a_eval + β.sigma1_eval + γ)(b_eval + β.sigma2_eval + γ).β.z(ʓ.ω).S_{sigma3}(ʓ)
        //                                                                               ^^^^^^^^^^^^^
        //                                                                               Evaluated at X=ʓ
        //     =   α^2.(z(ʓ.ω) - ∆_{PI}).L_{n-k}(ʓ)
        //       - α^3.L_1(ʓ)
        //       -   α.(a_eval + β.sigma1_eval + γ)(b_eval + β.sigma2_eval + γ)(c_eval + β.S_{sigma3}(ʓ) + γ).z(ʓ.ω)
        //
        r_0 += (sigma_last_multiplicand * sigma_evaluations[key->program_width - 1]);

        Field z_eval = transcript.get_field_element("z_perm");

        if (idpolys) {

            // Part 5.1: If idpolys = true, it indicates that we are not using the identity polynomials to
            // represent identity permutations. In that case, we need to use the pre-defined values for
            // representing identity permutations and then compute the coefficient of the z(X) component of r(X):
            //
            // [
            //       α.(a_eval + β.id_1 + γ)(b_eval + β.id_2 + γ)(c_eval + β.id_3 + γ)
            //   + α^3.L_1(ʓ)
            // ].z(X)
            //
            Field id_contribution = Field(1);
            for (size_t i = 0; i < key->program_width; ++i) {
                Field id_evaluation = transcript.get_field_element("id_" + std::to_string(i + 1));
                T0 = id_evaluation * beta;
                T0 += wire_evaluations[i];
                T0 += gamma;
                id_contribution *= T0;
            }
            Field id_last_multiplicand = id_contribution * alpha;
            T0 = l_start * alpha_cubed;
            id_last_multiplicand += T0;

            // Add up part 5.1 to the r_0 term, so r_0 will be:
            //
            // r_0 = α^2.(z(ʓ.ω) - ∆_{PI}).L_{n-k}(ʓ)
            //     - α^3.L_1(ʓ)
            //     -   α.(a_eval + β.sigma1_eval + γ)(b_eval + β.sigma2_eval + γ)(c_eval + β.S_{sigma3}(ʓ) + γ).z(ʓ.ω)
            //     + [
            //             α.(a_eval + β.id_1 + γ)(b_eval + β.id_2 + γ)(c_eval + β.id_3 + γ)
            //         + α^3.L_1(ʓ)
            //       ].z(ʓ)
            //         ^^^^
            //         Evaluated at X=ʓ

            r_0 += (id_last_multiplicand * z_eval);
        } else {

            // Part 5.2: If idpolys is false, the identity permutations are identity polynomials.
            // So we need to compute the following term
            //
            // [
            //       α.(a_eval + β.ʓ + γ)(b_eval + β.k_1.ʓ + γ)(c_eval + β.k_2.ʓ + γ)
            //   + α^3.L_1(ʓ)
            // ].z(ʓ)
            //
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

            // add up part 5.2 to the r_0 term
            r_0 += (z_1_multiplicand * z_eval);
        }
        return alpha_squared.sqr();
    }
}

template <typename Field, typename Group, typename Transcript, const size_t num_roots_cut_out_of_vanishing_polynomial>
Field VerifierPermutationWidget<Field, Group, Transcript, num_roots_cut_out_of_vanishing_polynomial>::
    append_scalar_multiplication_inputs(typename Transcript::Key* key,
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
    Field shifted_z_eval = transcript.get_field_element("z_perm_omega");

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
    // where u = separator challenge. That is done in `populate_kate_element_map` function inside
    // `kate_verification.hpp`.
    //
    if (use_linearisation) {
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
        scalars["Z_PERM"] += (z_1_multiplicand);
    }

    // Here, we compute the multiplicand of [sigma3]_1 as
    // -(a_eval + β.sigma1_eval + γ)(b_eval + β.sigma2_eval + γ)α.β.nu_{linear}.z_omega_eval
    if (use_linearisation) {
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
        scalars["SIGMA_" + std::to_string(key->program_width)] += (sigma_last_multiplicand);
    }

    return alpha_base * alpha_step.sqr() * alpha_step;
}

template class VerifierPermutationWidget<barretenberg::fr,
                                         barretenberg::g1::affine_element,
                                         transcript::StandardTranscript>;

} // namespace waffle