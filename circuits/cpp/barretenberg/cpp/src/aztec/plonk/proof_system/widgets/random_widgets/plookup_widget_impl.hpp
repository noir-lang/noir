#pragma once

#include <proof_system/proving_key/proving_key.hpp>
#include <transcript/transcript.hpp>
#include <polynomials/iterate_over_domain.hpp>
#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include <polynomials/polynomial_arithmetic.hpp>
#include <common/mem.hpp>

namespace waffle {

template <const size_t num_roots_cut_out_of_vanishing_polynomial>
ProverPlookupWidget<num_roots_cut_out_of_vanishing_polynomial>::ProverPlookupWidget(proving_key* input_key)
    : ProverRandomWidget(input_key)
{}

template <const size_t num_roots_cut_out_of_vanishing_polynomial>
ProverPlookupWidget<num_roots_cut_out_of_vanishing_polynomial>::ProverPlookupWidget(const ProverPlookupWidget& other)
    : ProverRandomWidget(other)
{}

template <const size_t num_roots_cut_out_of_vanishing_polynomial>
ProverPlookupWidget<num_roots_cut_out_of_vanishing_polynomial>::ProverPlookupWidget(ProverPlookupWidget&& other)
    : ProverRandomWidget(other)
{}

template <const size_t num_roots_cut_out_of_vanishing_polynomial>
ProverPlookupWidget<num_roots_cut_out_of_vanishing_polynomial>& ProverPlookupWidget<
    num_roots_cut_out_of_vanishing_polynomial>::operator=(const ProverPlookupWidget& other)
{
    ProverRandomWidget::operator=(other);
    return *this;
}

template <const size_t num_roots_cut_out_of_vanishing_polynomial>
ProverPlookupWidget<num_roots_cut_out_of_vanishing_polynomial>& ProverPlookupWidget<
    num_roots_cut_out_of_vanishing_polynomial>::operator=(ProverPlookupWidget&& other)
{
    ProverRandomWidget::operator=(other);
    return *this;
}
/**
 * @brief Construct polynomial s and add blinding. Save s in both lagrange and monomial form.
 *
 * @tparam num_roots_cut_out_of_vanishing_polynomial
 * @param transcript
 *
 * @details Polynomial 's' is the sorted concatenation of witness values and lookup table values.
 * It is constructed as s = s_1 + η*s_2 + η²*s_3 + η³*s_4. Blinding is added by setting the last 3
 * elements in the lagrange representation to random values.
 */
template <const size_t num_roots_cut_out_of_vanishing_polynomial>
void ProverPlookupWidget<num_roots_cut_out_of_vanishing_polynomial>::compute_sorted_list_polynomial(
    transcript::StandardTranscript& transcript)
{
    barretenberg::polynomial s_1 = key->polynomial_cache.get("s_1_lagrange");
    const fr* s_2 = key->polynomial_cache.get("s_2_lagrange").get_coefficients();
    const fr* s_3 = key->polynomial_cache.get("s_3_lagrange").get_coefficients();
    const fr* s_4 = key->polynomial_cache.get("s_4_lagrange").get_coefficients();

    barretenberg::polynomial s_accum(key->circuit_size, key->circuit_size);
    barretenberg::polynomial_arithmetic::copy_polynomial(&s_1[0], &s_accum[0], key->circuit_size, key->circuit_size);

    // Get challenge η
    const auto eta = fr::serialize_from_buffer(transcript.get_challenge("eta", 0).begin());

    // Construct s = s_1 + η*s_2 + η²*s_3 + η³*s_4 via Horner, i.e. s = s_1 + η(s_2 + η(s_3 + η*s_4))
    // Note: we store 's' in the memory allocated for s_1
    ITERATE_OVER_DOMAIN_START(key->small_domain);
    fr T0 = s_4[i];
    T0 *= eta;
    T0 += s_3[i];
    T0 *= eta;
    T0 += s_2[i];
    T0 *= eta;
    s_accum[i] += T0;
    ITERATE_OVER_DOMAIN_END;

    // To make the plookup honest-verifier zero-knowledge, we need to ensure that the witness polynomials
    // look uniformly random. Since the `s` polynomial needs evaluation at 2 points in UltraPLONK, we need
    // to add a degree-2 random polynomial to `s`. Alternatively, we can add 3 random scalars in the lagrange basis
    // of `s`. Concretely, we wish to do this:
    // s'(X) = s(X) + (r0 + r1.X + r2.X^2)
    // In lagrange basis, suppose the coefficients of `s` are (s1, s2, s3, ..., s{n-k}) where `k` is the number
    // of roots cut out of the vanishing polynomial Z_H(X) = X^n - 1.
    // Thus, writing `s` into the usual coefficient form, we will have
    // s(X) = s1.L_1(X) + s2.L_2(X) + ... + s{n-k}.L_{n-k}(X)
    // Now, the coefficients of lagrange bases (L_{n-k+1}, ..., L_{n}) are empty. We can use them to add randomness
    // into `s`. Since we wish to add 3 random scalars, we need k >= 3. In our case, we have set
    // num_roots_cut_out_of_vanishing_polynomial = 4. Thus, we can add 3 random scalars as (s{n-k}, s{n-k+1},
    // s{n-k+2}).
    const size_t s_randomness = 3;
    ASSERT(s_randomness < num_roots_cut_out_of_vanishing_polynomial);
    for (size_t k = 0; k < s_randomness; ++k) {
        s_accum[((key->circuit_size - num_roots_cut_out_of_vanishing_polynomial) + 1 + k)] = fr::random_element();
    }

    // Save the lagrange base representation of s
    polynomial s_lagrange(s_accum, key->small_domain.size);
    key->polynomial_cache.put("s_lagrange", std::move(s_lagrange));

    // Compute the monomial coefficient representation of s
    s_accum.ifft(key->small_domain);
    key->polynomial_cache.put("s", std::move(s_accum));
}
/**
 * @brief Compute the blinded lookup grand product polynomial Z_lookup(X)
 *
 * @tparam num_roots_cut_out_of_vanishing_polynomial
 * @param transcript
 *
 * @brief The lookup grand product polynomial Z_lookup is of the form
 *
 *                   ∏(1 + β) ⋅ ∏(q_lookup*f_k + γ) ⋅ ∏(t_k + βt_{k+1} + γ(1 + β))
 * Z_lookup(g^j) = -----------------------------------------------------------------
 *                                   ∏(s_k + βs_{k+1} + γ(1 + β))
 *
 * where ∏ := ∏_{k<j}. This polynomial is constructed in evaluation form over the course
 * of three steps (descibed in more detail below). Blinding is added by setting the last 3
 * elements in the lagrange representation to random values. Finally, the monomial
 * coefficient form of Z_lookup is computed via an iFFT.
 */
template <const size_t num_roots_cut_out_of_vanishing_polynomial>
void ProverPlookupWidget<num_roots_cut_out_of_vanishing_polynomial>::compute_grand_product_polynomial(
    transcript::StandardTranscript& transcript)
{
    const size_t n = key->circuit_size;

    // Note: z_lookup ultimately is only only size 'n' but we allow 'n+1' for convenience
    // essentially as scratch space in the calculation to follow
    polynomial z_lookup(key->circuit_size + 1, key->circuit_size + 1);

    // Allocate 4 length n 'accumulators'. accumulators[0] points to the 1th index of
    // z_lookup and will be used to construct z_lookup (lagrange base) in place. The
    // remaining 3 are needed only locally in the construction of z_lookup. Note that
    // beyond this calculation we need only the monomial and coset FFT forms of z_lookup,
    // so z_lookup in lagrange base will not be added to the store.
    fr* accumulators[4];
    // Note: accumulators[0][i] = z[i + 1]
    accumulators[0] = &z_lookup[1];
    for (size_t k = 1; k < 4; ++k) {
        accumulators[k] = static_cast<fr*>(aligned_alloc(64, sizeof(fr) * n));
    }

    polynomial s_lagrange = key->polynomial_cache.get("s_lagrange");

    const fr* column_1_step_size = key->polynomial_cache.get("q_2_lagrange").get_coefficients();
    const fr* column_2_step_size = key->polynomial_cache.get("q_m_lagrange").get_coefficients();
    const fr* column_3_step_size = key->polynomial_cache.get("q_c_lagrange").get_coefficients();

    fr eta = fr::serialize_from_buffer(transcript.get_challenge("eta").begin());
    fr eta_sqr = eta.sqr();
    fr eta_cube = eta_sqr * eta;

    fr beta = fr::serialize_from_buffer(transcript.get_challenge("beta").begin());
    fr gamma = fr::serialize_from_buffer(transcript.get_challenge("beta", 1).begin());
    std::array<const fr*, 3> lagrange_base_wires;
    std::array<const fr*, 4> lagrange_base_tables{
        key->polynomial_cache.get("table_value_1_lagrange").get_coefficients(),
        key->polynomial_cache.get("table_value_2_lagrange").get_coefficients(),
        key->polynomial_cache.get("table_value_3_lagrange").get_coefficients(),
        key->polynomial_cache.get("table_value_4_lagrange").get_coefficients(),
    };

    const fr* lookup_selector = key->polynomial_cache.get("table_type_lagrange").get_coefficients();
    const fr* lookup_index_selector = key->polynomial_cache.get("q_3_lagrange").get_coefficients();
    for (size_t i = 0; i < 3; ++i) {
        lagrange_base_wires[i] =
            key->polynomial_cache.get("w_" + std::to_string(i + 1) + "_lagrange").get_coefficients();
    }

    const fr beta_constant = beta + fr(1);                // (1 + β)
    const fr gamma_beta_constant = gamma * beta_constant; // γ(1 + β)

#ifndef NO_MULTITHREADING
#pragma omp parallel
#endif
    {
#ifndef NO_MULTITHREADING
#pragma omp for
#endif
        // Step 1: Compute polynomials f, t and s and incorporate them into terms that are ultimately needed
        // to construct the grand product polynomial Z_lookup(X):
        // Note 1: In what follows, 't' is associated with table values (and is not to be confused with the
        // quotient polynomial, also refered to as 't' elsewhere). Polynomial 's' is the sorted  concatenation
        // of the witnesses and the table values.
        // Note 2: Evaluation at Xω is indicated explicitly, e.g. 'p(Xω)'; evaluation at X is simply omitted, e.g. 'p'
        //
        // 1a.   Compute f, then set accumulators[0] = (q_lookup*f + γ), where
        //
        //         f = (w_1 + q_2*w_1(Xω)) + η(w_2 + q_m*w_2(Xω)) + η²(w_3 + q_c*w_3(Xω)) + η³q_index.
        //      Note that q_2, q_m, and q_c are just the selectors from Standard Plonk that have been repurposed
        //      in the context of the plookup gate to represent 'shift' values. For example, setting each of the
        //      q_* in f to 2^8 facilitates operations on 32-bit values via four operations on 8-bit values. See
        //      Ultra documentation for details.
        //
        // 1b.   Compute t, then set accumulators[1] = (t + βt(Xω) + γ(1 + β)), where t = t_1 + ηt_2 + η²t_3 + η³t_4
        //
        // 1c.   Set accumulators[2] = (1 + β)
        //
        // 1d.   Compute s, then set accumulators[3] = (s + βs(Xω) + γ(1 + β)), where s = s_1 + ηs_2 + η²s_3 + η³s_4
        //
        for (size_t j = 0; j < key->small_domain.num_threads; ++j) {
            fr T0;

            size_t start = j * key->small_domain.thread_size;
            size_t end = (j + 1) * key->small_domain.thread_size;

            // Note: block_mask is used for efficient modulus, i.e. i % N := i & (N-1), for N = 2^k
            const size_t block_mask = key->small_domain.size - 1;

            // Initialize 't(X)' to be used in an expression of the form t(X) + β*t(Xω)
            fr next_table = lagrange_base_tables[0][start] + lagrange_base_tables[1][start] * eta +
                            lagrange_base_tables[2][start] * eta_sqr + lagrange_base_tables[3][start] * eta_cube;
            for (size_t i = start; i < end; ++i) {
                // Compute i'th element of f via Horner (see definition of f above)
                T0 = lookup_index_selector[i];
                T0 *= eta;
                T0 += lagrange_base_wires[2][(i + 1) & block_mask] * column_3_step_size[i];
                T0 += lagrange_base_wires[2][i];
                T0 *= eta;
                T0 += lagrange_base_wires[1][(i + 1) & block_mask] * column_2_step_size[i];
                T0 += lagrange_base_wires[1][i];
                T0 *= eta;
                T0 += lagrange_base_wires[0][(i + 1) & block_mask] * column_1_step_size[i];
                T0 += lagrange_base_wires[0][i];
                T0 *= lookup_selector[i];

                // Set i'th element of polynomial q_lookup*f + γ
                accumulators[0][i] = T0;
                accumulators[0][i] += gamma;

                // Compute i'th element of t via Horner
                T0 = lagrange_base_tables[3][(i + 1) & block_mask];
                T0 *= eta;
                T0 += lagrange_base_tables[2][(i + 1) & block_mask];
                T0 *= eta;
                T0 += lagrange_base_tables[1][(i + 1) & block_mask];
                T0 *= eta;
                T0 += lagrange_base_tables[0][(i + 1) & block_mask];

                // Set i'th element of polynomial (t + βt(Xω) + γ(1 + β))
                accumulators[1][i] = T0 * beta + next_table;
                next_table = T0;
                accumulators[1][i] += gamma_beta_constant;

                // Set value of this accumulator to (1 + β)
                accumulators[2][i] = beta_constant;

                // Set i'th element of polynomial (s + βs(Xω) + γ(1 + β))
                accumulators[3][i] = s_lagrange[(i + 1) & block_mask];
                accumulators[3][i] *= beta;
                accumulators[3][i] += s_lagrange[i];
                accumulators[3][i] += gamma_beta_constant;
            }
        }

// Step 2: Compute the constituent product components of Z_lookup(X).
// Let ∏ := Prod_{k<j}. Let f_k, t_k and s_k now represent the k'th component of the polynomials f,t and s
// defined above. We compute the following four product polynomials needed to construct the grand product
// Z_lookup(X).
// 1.   accumulators[0][j] = ∏ (q_lookup*f_k + γ)
// 2.   accumulators[1][j] = ∏ (t_k + βt_{k+1} + γ(1 + β))
// 3.   accumulators[2][j] = ∏ (1 + β)
// 4.   accumulators[3][j] = ∏ (s_k + βs_{k+1} + γ(1 + β))
// Note: This is a small multithreading bottleneck, as we have only 4 parallelizable processes.
#ifndef NO_MULTITHREADING
#pragma omp for
#endif
        for (size_t i = 0; i < 4; ++i) {
            fr* coeffs = &accumulators[i][0];
            for (size_t j = 0; j < key->small_domain.size - 1; ++j) {
                coeffs[j + 1] *= coeffs[j];
            }
        }

// Step 3: Combine the accumulator product elements to construct Z_lookup(X).
//
//                      ∏ (1 + β) ⋅ ∏ (q_lookup*f_k + γ) ⋅ ∏ (t_k + βt_{k+1} + γ(1 + β))
//  Z_lookup(g^j) = --------------------------------------------------------------------------
//                                      ∏ (s_k + βs_{k+1} + γ(1 + β))
//
// Note: Montgomery batch inversion is used to efficiently compute the coefficients of Z_lookup
// rather than peforming n individual inversions. I.e. we first compute the double product P_n:
//
// P_n := ∏_{j<n} ∏_{k<j} S_k, where S_k = (s_k + βs_{k+1} + γ(1 + β))
//
// and then compute the inverse on P_n. Then we work back to front to obtain terms of the form
// 1/∏_{k<i} S_i that appear in Z_lookup, using the fact that P_i/P_{i+1} = 1/∏_{k<i} S_i. (Note
// that once we have 1/P_n, we can compute 1/P_{n-1} as (1/P_n) * ∏_{k<n} S_i, and
// so on).
#ifndef NO_MULTITHREADING
#pragma omp for
#endif
        // Compute Z_lookup using Montgomery batch inversion
        // Note: This loop sets the values of z_lookup[i] for i = 1,...,(n-1), (Recall accumulators[0][i] = z_lookup[i +
        // 1])
        for (size_t j = 0; j < key->small_domain.num_threads; ++j) {
            const size_t start = j * key->small_domain.thread_size;
            // Set 'end' so its max value is (n-1) thus max value for 'i' is n-2 (N.B. accumulators[0][n-2] =
            // z_lookup[n-1])
            const size_t end = (j == key->small_domain.num_threads - 1) ? (j + 1) * key->small_domain.thread_size - 1
                                                                        : (j + 1) * key->small_domain.thread_size;

            // Compute <Z_lookup numerator> * ∏_{j<i}∏_{k<j}S_k
            fr inversion_accumulator = fr::one();
            for (size_t i = start; i < end; ++i) {
                accumulators[0][i] *= accumulators[2][i];
                accumulators[0][i] *= accumulators[1][i];
                accumulators[0][i] *= inversion_accumulator;
                inversion_accumulator *= accumulators[3][i];
            }
            inversion_accumulator = inversion_accumulator.invert(); // invert
            // Compute [Z_lookup numerator] * ∏_{j<i}∏_{k<j}S_k / ∏_{j<i+1}∏_{k<j}S_k = <Z_lookup numerator> /
            // ∏_{k<i}S_k
            for (size_t i = end - 1; i != start - 1; --i) {

                // N.B. accumulators[0][i] = z_lookup[i + 1]
                // We can avoid fully reducing z_lookup[i + 1] as the inverse fft will take care of that for us
                accumulators[0][i] *= inversion_accumulator;
                inversion_accumulator *= accumulators[3][i];
            }
        }
    }
    z_lookup[0] = fr::one();

    // Since `z_plookup` needs to be evaluated at 2 points in UltraPLONK, we need to add a degree-2 random
    // polynomial to `z_lookup` to make it "look" uniformly random. Alternatively, we can just add 3
    // random scalars into the lagrange form of `z_lookup`, rationale for which similar to that explained
    // for `s` polynomial.
    const size_t z_randomness = 3;
    ASSERT(z_randomness < num_roots_cut_out_of_vanishing_polynomial);
    for (size_t k = 0; k < z_randomness; ++k) {
        // Blinding:
        z_lookup[((n - num_roots_cut_out_of_vanishing_polynomial) + 1 + k)] = fr::random_element();
    }

    // Compute and add monomial form of z_lookup to the polynomial store
    z_lookup.ifft(key->small_domain);
    key->polynomial_cache.put("z_lookup", std::move(z_lookup));
}

/**
 * @brief Compute commitments and FFTs of 's' (round_number == 2) or 'Z_lookup' (round_number == 3)
 *
 * @tparam num_roots_cut_out_of_vanishing_polynomial
 * @param transcript
 * @param round_number
 * @param queue
 */
template <const size_t num_roots_cut_out_of_vanishing_polynomial>
void ProverPlookupWidget<num_roots_cut_out_of_vanishing_polynomial>::compute_round_commitments(
    transcript::StandardTranscript& transcript, const size_t round_number, work_queue& queue)
{
    if (round_number == 2) {
        compute_sorted_list_polynomial(transcript);
        const polynomial& s = key->polynomial_cache.get("s");

        // Commit to s:
        queue.add_to_queue({
            .work_type = work_queue::WorkType::SCALAR_MULTIPLICATION,
            .mul_scalars = s.get_coefficients(),
            .tag = "S",
            .constant = barretenberg::fr(0),
            .index = 0,
        });

        // Compute the coset FFT of 's' for use in quotient poly construction
        queue.add_to_queue({
            .work_type = work_queue::WorkType::FFT,
            .mul_scalars = nullptr,
            .tag = "s",
            .constant = barretenberg::fr(0),
            .index = 0,
        });

        return;
    }
    if (round_number == 3) {
        compute_grand_product_polynomial(transcript);
        const polynomial& z = key->polynomial_cache.get("z_lookup");

        // Commit to z_lookup:
        queue.add_to_queue({
            .work_type = work_queue::WorkType::SCALAR_MULTIPLICATION,
            .mul_scalars = z.get_coefficients(),
            .tag = "Z_LOOKUP",
            .constant = barretenberg::fr(0),
            .index = 0,
        });

        // Compute the coset FFT of 'z_lookup' for use in quotient poly construction
        queue.add_to_queue({
            .work_type = work_queue::WorkType::FFT,
            .mul_scalars = nullptr,
            .tag = "z_lookup",
            .constant = barretenberg::fr(0),
            .index = 0,
        });

        return;
    }
}

/**
 * @brief Add contibution of z_lookup grand product terms to the quotient polynomial
 *
 * @tparam num_roots_cut_out_of_vanishing_polynomial
 * @param alpha_base
 * @param transcript
 * @return barretenberg::fr
 *
 * @details The terms associated with the z_lookup grand product polynomial that must be added
 * to the quotient polynomial are as follows:
 *  z_lookup(X)*[(q_lookup*f + γ) * (t + βt(Xω) + γ(1 + β)) * (1 + β)] ...
 *      + (z_lookup - 1)*αL_1(X) ...
 *      - z_lookup(Xω)*(s + βs(Xω) + γ(1 + β)) ...
 *      + [z_lookup(Xω) - 1/γ(1 + β)^{n-k}]*α²L_1(Xω^k)
 *
 * These terms attest to the proper construction of Z_lookup. They are analogous to the terms
 * associated with the Standard Plonk grand product polynomial Z that also appear in the quotient
 * polynomial. See the comments there for more details. The contribution of these terms is
 * incorporated into the quotient polynomial via the coset evaluation form (i.e. the evaluation
 * on 4nth roots of unity).
 *
 */
template <const size_t num_roots_cut_out_of_vanishing_polynomial>
barretenberg::fr ProverPlookupWidget<num_roots_cut_out_of_vanishing_polynomial>::compute_quotient_contribution(
    const fr& alpha_base, const transcript::StandardTranscript& transcript)
{
    const polynomial& z_lookup_fft = key->polynomial_cache.get("z_lookup_fft");

    fr eta = fr::serialize_from_buffer(transcript.get_challenge("eta").begin());
    fr alpha = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());
    fr beta = fr::serialize_from_buffer(transcript.get_challenge("beta").begin());
    fr gamma = fr::serialize_from_buffer(transcript.get_challenge("beta", 1).begin());

    std::array<const fr*, 3> wire_ffts{
        key->polynomial_cache.get("w_1_fft").get_coefficients(),
        key->polynomial_cache.get("w_2_fft").get_coefficients(),
        key->polynomial_cache.get("w_3_fft").get_coefficients(),
    };

    const fr* s_fft = key->polynomial_cache.get("s_fft").get_coefficients();

    std::array<const fr*, 4> table_ffts{
        key->polynomial_cache.get("table_value_1_fft").get_coefficients(),
        key->polynomial_cache.get("table_value_2_fft").get_coefficients(),
        key->polynomial_cache.get("table_value_3_fft").get_coefficients(),
        key->polynomial_cache.get("table_value_4_fft").get_coefficients(),
    };

    const fr* column_1_step_size = key->polynomial_cache.get("q_2_fft").get_coefficients();
    const fr* column_2_step_size = key->polynomial_cache.get("q_m_fft").get_coefficients();
    const fr* column_3_step_size = key->polynomial_cache.get("q_c_fft").get_coefficients();

    const fr* lookup_fft = key->polynomial_cache.get("table_type_fft").get_coefficients();
    const fr* lookup_index_fft = key->polynomial_cache.get("q_3_fft").get_coefficients();

    const fr gamma_beta_constant = gamma * (fr(1) + beta); // γ(1 + β)

    const polynomial& l_1 = key->polynomial_cache.get("lagrange_1_fft");
    // delta_factor = [γ(1 + β)]^{n-k}
    const fr delta_factor = gamma_beta_constant.pow(key->small_domain.size - num_roots_cut_out_of_vanishing_polynomial);
    const fr alpha_sqr = alpha.sqr();

    const fr beta_constant = beta + fr(1); // (1 + β)

    const size_t block_mask = key->large_domain.size - 1;

#ifndef NO_MULTITHREADING
#pragma omp parallel for
#endif
    // Add to the quotient polynomial the components associated with z_lookup
    for (size_t j = 0; j < key->large_domain.num_threads; ++j) {
        const size_t start = j * key->large_domain.thread_size;
        const size_t end = (j + 1) * key->large_domain.thread_size;

        fr T0;
        fr T1;
        fr denominator;
        fr numerator;

        // Initialize first four t(X) = t_table(X) for expression t + βt(Xω) + γ(1 + β)
        std::array<fr, 4> next_ts;
        for (size_t i = 0; i < 4; ++i) {
            next_ts[i] = table_ffts[3][(start + i) & block_mask];
            next_ts[i] *= eta;
            next_ts[i] += table_ffts[2][(start + i) & block_mask];
            next_ts[i] *= eta;
            next_ts[i] += table_ffts[1][(start + i) & block_mask];
            next_ts[i] *= eta;
            next_ts[i] += table_ffts[0][(start + i) & block_mask];
        }
        for (size_t i = start; i < end; ++i) {
            // Set T0 = f := (w_1 + q_2*w_1(Xω)) + η(w_2 + q_m*w_2(Xω)) + η²(w_3 + q_c*w_3(Xω)) + η³q_index
            T0 = lookup_index_fft[i];
            T0 *= eta;
            T0 += wire_ffts[2][(i + 4) & block_mask] * column_3_step_size[i];
            T0 += wire_ffts[2][i];
            T0 *= eta;
            T0 += wire_ffts[1][(i + 4) & block_mask] * column_2_step_size[i];
            T0 += wire_ffts[1][i];
            T0 *= eta;
            T0 += wire_ffts[0][(i + 4) & block_mask] * column_1_step_size[i];
            T0 += wire_ffts[0][i];

            // Set numerator = q_lookup*f + γ
            numerator = T0;
            numerator *= lookup_fft[i];
            numerator += gamma;

            // Set T0 = t(Xω) := t_1(Xω) + ηt_2(Xω) + η²t_3(Xω) + η³t_4(Xω)
            T0 = table_ffts[3][(i + 4) & block_mask];
            T0 *= eta;
            T0 += table_ffts[2][(i + 4) & block_mask];
            T0 *= eta;
            T0 += table_ffts[1][(i + 4) & block_mask];
            T0 *= eta;
            T0 += table_ffts[0][(i + 4) & block_mask];

            // Set T1 = (t + βt(Xω) + γ(1 + β))
            T1 = beta;
            T1 *= T0;
            T1 += next_ts[i & 0x03UL];
            T1 += gamma_beta_constant;

            // Set t(X) = t(Xω) for the next time around
            next_ts[i & 0x03UL] = T0;

            // numerator = (q_lookup*f + γ) * (t + βt(Xω) + γ(1 + β)) * (1 + β)
            numerator *= T1;
            numerator *= beta_constant;

            // Set denominator = (s + βs(Xω) + γ(1 + β))
            denominator = s_fft[(i + 4) & block_mask];
            denominator *= beta;
            denominator += s_fft[i];
            denominator += gamma_beta_constant;

            // Set T0 = αL_1(X)
            T0 = l_1[i] * alpha;
            // Set T1 = α²L_{n-k}(X) = α²L_1(Xω^{-(n-k)+1}) = α²L_1(Xω^{k+1}), k = num roots cut out of Z_H
            T1 = l_1[(i + 4 + 4 * num_roots_cut_out_of_vanishing_polynomial) & block_mask] * alpha_sqr;

            // Set numerator = z_lookup(X)*[(q_lookup*f + γ) * (t + βt(Xω) + γ(1 + β)) * (1 + β)] + (z_lookup -
            // 1)*αL_1(X)
            numerator += T0;
            numerator *= z_lookup_fft[i];
            numerator -= T0;

            // Set denominator = z_lookup(Xω)*(s + βs(Xω) + γ(1 + β)) - [z_lookup(Xω) - [γ(1 + β)]^{n-k}]*α²L_{n-k}(X)
            denominator -= T1;
            denominator *= z_lookup_fft[(i + 4) & block_mask];
            denominator += T1 * delta_factor;

            // Combine into quotient polynomial contribution
            // T0 = z_lookup(X)*[(q_lookup*f + γ) * (t + βt(Xω) + γ(1 + β)) * (1 + β)] + (z_lookup - 1)*αL_1(X) ...
            //      - z_lookup(Xω)*(s + βs(Xω) + γ(1 + β)) + [z_lookup(Xω) - [γ(1 + β)]^{n-k}]*α²L_{n-k}(X)
            T0 = numerator - denominator;
            // key->quotient_large[i] += T0 * alpha_base; // CODY: Luke did this while documenting
            key->quotient_polynomial_parts[i >> key->small_domain.log2_size][i & (key->circuit_size - 1)] +=
                T0 * alpha_base;
        }
    }
    return alpha_base * alpha.sqr() * alpha;
}

/**
 * @brief Computes the evaluation at challenge point 'z' of the terms in the linearization polynomial
 * associated with z_lookup.
 *
 * @tparam num_roots_cut_out_of_vanishing_polynomial
 * @param alpha_base
 * @param transcript
 * @param r
 * @return barretenberg::fr
 *
 * @details This is used in computation of the 'linearization' polynomial r(X).
 * Note, however, that the components computed here are not 'linearized'; all terms are
 * simply evaluated at 'z'. More on this function can be found in
 * https://hackmd.io/vUGG8CO_Rk2iEjruBL_gGw?view#Note-A-Mind-Boggling-Issue-with-Ultra-Plonk
 */
template <const size_t num_roots_cut_out_of_vanishing_polynomial>
barretenberg::fr ProverPlookupWidget<num_roots_cut_out_of_vanishing_polynomial>::compute_linear_contribution(
    const fr& alpha_base, const transcript::StandardTranscript& transcript, polynomial& r)

{
    fr alpha = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());
    std::array<fr, 3> wire_evaluations{
        transcript.get_field_element("w_1"),
        transcript.get_field_element("w_2"),
        transcript.get_field_element("w_3"),
    };
    std::array<fr, 3> shifted_wire_evaluations{
        transcript.get_field_element("w_1_omega"),
        transcript.get_field_element("w_2_omega"),
        transcript.get_field_element("w_3_omega"),
    };

    std::array<fr, 4> table_evaluations{
        transcript.get_field_element("table_value_1"),
        transcript.get_field_element("table_value_2"),
        transcript.get_field_element("table_value_3"),
        transcript.get_field_element("table_value_4"),
    };

    std::array<fr, 4> shifted_table_evaluations{
        transcript.get_field_element("table_value_1_omega"),
        transcript.get_field_element("table_value_2_omega"),
        transcript.get_field_element("table_value_3_omega"),
        transcript.get_field_element("table_value_4_omega"),
    };

    fr column_1_step_size = transcript.get_field_element("q_2");
    fr column_2_step_size = transcript.get_field_element("q_m");
    fr column_3_step_size = transcript.get_field_element("q_c");
    fr table_type_eval = transcript.get_field_element("table_type");
    fr table_index_eval = transcript.get_field_element("q_3");

    fr s_eval = transcript.get_field_element("s");
    fr shifted_s_eval = transcript.get_field_element("s_omega");

    fr z_eval = transcript.get_field_element("z_lookup");
    fr shifted_z_eval = transcript.get_field_element("z_lookup_omega");

    fr z = transcript.get_challenge_field_element("z");
    fr beta = transcript.get_challenge_field_element("beta", 0);
    fr gamma = transcript.get_challenge_field_element("beta", 1);
    fr eta = transcript.get_challenge_field_element("eta", 0);
    fr l_numerator = z.pow(key->circuit_size) - fr(1);

    // Compute evaluation L_1(z)
    l_numerator *= key->small_domain.domain_inverse;
    fr l_1 = l_numerator / (z - fr(1));

    // Compute evaluation L_end(z) = L_{n-k}(z) using ω^{-(n - k) + 1} = ω^{k + 1} where
    // k = num roots cut out of Z_H
    fr l_end_root =
        (num_roots_cut_out_of_vanishing_polynomial & 1) ? key->small_domain.root.sqr() : key->small_domain.root;
    for (size_t i = 0; i < num_roots_cut_out_of_vanishing_polynomial / 2; ++i) {
        l_end_root *= key->small_domain.root.sqr();
    }
    fr l_end = l_numerator / ((z * l_end_root) - fr(1));

    const fr one(1);
    const fr gamma_beta_constant = gamma * (one + beta); // γ(β + 1)

    // delta_factor = γ(β + 1)^{n-k}
    const fr delta_factor = gamma_beta_constant.pow(key->small_domain.size - num_roots_cut_out_of_vanishing_polynomial);
    const fr alpha_sqr = alpha.sqr();

    const fr beta_constant = beta + one;

    fr T0;
    fr T1;
    fr denominator;
    fr numerator;

    // Set f_eval = f(z) := (w_1(z) + q_2*w_1(zω)) + η(w_2(z) + q_m*w_2(zω)) + η²(w_3(z) + q_c*w_3(zω)) + η³q_index(z)
    fr f_eval = table_index_eval;
    f_eval *= eta;
    f_eval += shifted_wire_evaluations[2] * column_3_step_size;
    f_eval += wire_evaluations[2];
    f_eval *= eta;
    f_eval += shifted_wire_evaluations[1] * column_2_step_size;
    f_eval += wire_evaluations[1];
    f_eval *= eta;
    f_eval += shifted_wire_evaluations[0] * column_1_step_size;
    f_eval += wire_evaluations[0];

    // Set table_eval = t(z)
    fr table_eval = table_evaluations[3];
    table_eval *= eta;
    table_eval += table_evaluations[2];
    table_eval *= eta;
    table_eval += table_evaluations[1];
    table_eval *= eta;
    table_eval += table_evaluations[0];

    // Set numerator = q_index(z)*f(z) + γ
    numerator = f_eval * table_type_eval;
    numerator += gamma;

    // Set T0 = t(zω)
    T0 = shifted_table_evaluations[3];
    T0 *= eta;
    T0 += shifted_table_evaluations[2];
    T0 *= eta;
    T0 += shifted_table_evaluations[1];
    T0 *= eta;
    T0 += shifted_table_evaluations[0];

    // Set T1 = t(z) + βt(zω) + γ(β + 1)
    T1 = beta;
    T1 *= T0;
    T1 += table_eval;
    T1 += gamma_beta_constant;

    // Set numerator = (q_index*f(z) + γ) * (t(z) + βt(zω) + γ(β + 1)) * (β + 1)
    numerator *= T1;
    numerator *= beta_constant;

    // Set denominator = s(z) + βs(zω) + γ(β + 1)
    denominator = shifted_s_eval;
    denominator *= beta;
    denominator += s_eval;
    denominator += gamma_beta_constant;

    // Set T0 = αL_1(z), T1 = α²L_end(z)
    T0 = l_1 * alpha;
    T1 = l_end * alpha_sqr;

    // Set numerator = z_lookup(z)*[(q_index*f(z) + γ) * (t(z) + βt(zω) + γ(β + 1)) * (β + 1)] + (z_lookup(z) -
    // 1)*αL_1(z)
    numerator += T0;
    numerator *= z_eval;
    numerator -= T0;

    // Set denominator = z_lookup(zω)*[s(z) + βs(zω) + γ(1 + β)] - [z_lookup(zω) - [γ(1 + β)]^{n-k}]*α²L_end(z)
    denominator -= T1;
    denominator *= shifted_z_eval;
    denominator += T1 * delta_factor;

    // Set T0 = z_lookup(z)*[(q_index*f(z) + γ) * (t(z) + βt(zω) + γ(β + 1)) * (β + 1)] + (z_lookup(z) - 1)*αL_1(z) ...
    //      - z_lookup(zω)*[s(z) + βs(zω) + γ(1 + β)] + [z_lookup(zω) - [γ(1 + β)]^{n-k}]*α²L_end(z)
    T0 = numerator - denominator;
    // We need to add the constant term of plookup permutation polynomial in the linearisation
    // polynomial to ensure that r(z) = 0.
    r[0] += T0 * alpha_base;

    return alpha_base * alpha.sqr() * alpha;
}

// ###

template <typename Field, typename Group, typename Transcript, const size_t num_roots_cut_out_of_vanishing_polynomial>
VerifierPlookupWidget<Field, Group, Transcript, num_roots_cut_out_of_vanishing_polynomial>::VerifierPlookupWidget()
{}

/**
 * @brief Computes the evaluation at challenge point 'z' of the terms in the quotient polynomial
 * associated with z_lookup.
 *
 * @tparam Field
 * @tparam Group
 * @tparam Transcript
 * @tparam num_roots_cut_out_of_vanishing_polynomial
 * @param key
 * @param alpha_base
 * @param transcript
 * @param r_0
 * @return Field
 *
 * @brief Used by verifier in computation of quotient polynomial evaluation at challenge point 'z'.
 *
 */
template <typename Field, typename Group, typename Transcript, const size_t num_roots_cut_out_of_vanishing_polynomial>
Field VerifierPlookupWidget<Field, Group, Transcript, num_roots_cut_out_of_vanishing_polynomial>::
    compute_quotient_evaluation_contribution(
        typename Transcript::Key* key, const Field& alpha_base, const Transcript& transcript, Field& r_0, const bool)
{

    std::array<Field, 3> wire_evaluations{
        transcript.get_field_element("w_1"),
        transcript.get_field_element("w_2"),
        transcript.get_field_element("w_3"),
    };
    std::array<Field, 3> shifted_wire_evaluations{
        transcript.get_field_element("w_1_omega"),
        transcript.get_field_element("w_2_omega"),
        transcript.get_field_element("w_3_omega"),
    };

    std::array<Field, 4> table_evaluations{
        transcript.get_field_element("table_value_1"),
        transcript.get_field_element("table_value_2"),
        transcript.get_field_element("table_value_3"),
        transcript.get_field_element("table_value_4"),
    };

    std::array<Field, 4> shifted_table_evaluations{
        transcript.get_field_element("table_value_1_omega"),
        transcript.get_field_element("table_value_2_omega"),
        transcript.get_field_element("table_value_3_omega"),
        transcript.get_field_element("table_value_4_omega"),
    };

    Field column_1_step_size = transcript.get_field_element("q_2");
    Field column_2_step_size = transcript.get_field_element("q_m");
    Field column_3_step_size = transcript.get_field_element("q_c");
    Field table_type_eval = transcript.get_field_element("table_type");
    Field table_index_eval = transcript.get_field_element("q_3");

    Field s_eval = transcript.get_field_element("s");
    Field shifted_s_eval = transcript.get_field_element("s_omega");

    Field z_eval = transcript.get_field_element("z_lookup");
    Field shifted_z_eval = transcript.get_field_element("z_lookup_omega");

    Field z = transcript.get_challenge_field_element("z");
    Field alpha = transcript.get_challenge_field_element("alpha", 0);
    Field beta = transcript.get_challenge_field_element("beta", 0);
    Field gamma = transcript.get_challenge_field_element("beta", 1);
    Field eta = transcript.get_challenge_field_element("eta", 0);
    Field l_numerator = key->z_pow_n - Field(1);

    l_numerator *= key->domain.domain_inverse;
    Field l_1 = l_numerator / (z - Field(1));

    // Compute evaluation L_end(z) = L_{n-k}(z) using ω^{-(n - k) + 1} = ω^{k + 1} where
    // k = num roots cut out of Z_H
    Field l_end_root = (num_roots_cut_out_of_vanishing_polynomial & 1) ? key->domain.root.sqr() : key->domain.root;
    for (size_t i = 0; i < num_roots_cut_out_of_vanishing_polynomial / 2; ++i) {
        l_end_root *= key->domain.root.sqr();
    }
    Field l_end = l_numerator / ((z * l_end_root) - Field(1)); // L_{n-k}(z)

    const Field one(1);
    const Field gamma_beta_constant = gamma * (one + beta); // γ(1 + β)

    // [γ(1 + β)]^{n-k}
    const Field delta_factor = gamma_beta_constant.pow(key->domain.domain - num_roots_cut_out_of_vanishing_polynomial);

    const Field alpha_sqr = alpha.sqr();

    const Field beta_constant = beta + one; // (1 + β)

    Field T0;
    Field T1;
    Field denominator;
    Field numerator;

    // Set f_eval = f(z) := (w_1(z) + q_2*w_1(zω)) + η(w_2(z) + q_m*w_2(zω)) + η²(w_3(z) + q_c*w_3(zω)) + η³q_index(z)
    Field f_eval = table_index_eval;
    f_eval *= eta;
    f_eval += shifted_wire_evaluations[2] * column_3_step_size;
    f_eval += wire_evaluations[2];
    f_eval *= eta;
    f_eval += shifted_wire_evaluations[1] * column_2_step_size;
    f_eval += wire_evaluations[1];
    f_eval *= eta;
    f_eval += shifted_wire_evaluations[0] * column_1_step_size;
    f_eval += wire_evaluations[0];

    // Set table_eval = t(z)
    Field table_eval = table_evaluations[3];
    table_eval *= eta;
    table_eval += table_evaluations[2];
    table_eval *= eta;
    table_eval += table_evaluations[1];
    table_eval *= eta;
    table_eval += table_evaluations[0];

    // Set numerator = q_index(z)*f(z) + γ
    numerator = f_eval * table_type_eval;
    numerator += gamma;

    // Set T0 = t(zω)
    T0 = shifted_table_evaluations[3];
    T0 *= eta;
    T0 += shifted_table_evaluations[2];
    T0 *= eta;
    T0 += shifted_table_evaluations[1];
    T0 *= eta;
    T0 += shifted_table_evaluations[0];

    // Set T1 = t(z) + βt(zω) + γ(β + 1)
    T1 = beta;
    T1 *= T0;
    T1 += table_eval;
    T1 += gamma_beta_constant;

    // Set numerator = (q_index*f(z) + γ) * (t(z) + βt(zω) + γ(β + 1)) * (β + 1)
    numerator *= T1;
    numerator *= beta_constant;

    // Set denominator = s(z) + βs(zω) + γ(β + 1)
    denominator = shifted_s_eval;
    denominator *= beta;
    denominator += s_eval;
    denominator += gamma_beta_constant;

    // Set T0 = αL_1(z), T1 = α²L_end(z)
    T0 = l_1 * alpha;
    T1 = l_end * alpha_sqr;

    // Set numerator = z_lookup(z)*[(q_index*f(z) + γ) * (t(z) + βt(zω) + γ(β + 1)) * (β + 1)] + ...
    //  (z_lookup(z) - 1)*αL_1(z)
    numerator += T0;
    numerator *= z_eval;
    numerator -= T0;

    // Set denominator = z_lookup(zω)*[s(z) + βs(zω) + γ(1 + β)] - [z_lookup(zω) - [γ(1 + β)]^{n-k}]*α²L_end(z)
    denominator -= T1;
    denominator *= shifted_z_eval;
    denominator += T1 * delta_factor;

    // Set T0 = z_lookup(z)*[(q_index*f(z) + γ) * (t(z) + βt(zω) + γ(β + 1)) * (β + 1)] + (z_lookup(z) - 1)*αL_1(z) ...
    //      - z_lookup(zω)*[s(z) + βs(zω) + γ(1 + β)] + [z_lookup(zω) - [γ(1 + β)]^{n-k}]*α²L_end(z)
    T0 = numerator - denominator;
    r_0 += T0 * alpha_base;
    return alpha_base * alpha.sqr() * alpha;
} // namespace waffle

template <typename Field, typename Group, typename Transcript, const size_t num_roots_cut_out_of_vanishing_polynomial>
Field VerifierPlookupWidget<Field, Group, Transcript, num_roots_cut_out_of_vanishing_polynomial>::
    append_scalar_multiplication_inputs(typename Transcript::Key*,
                                        const Field& alpha_base,
                                        const Transcript& transcript,
                                        std::map<std::string, Field>&,
                                        const bool)
{
    Field alpha = transcript.get_challenge_field_element("alpha");
    return alpha_base * alpha.sqr() * alpha;
}

template class VerifierPlookupWidget<barretenberg::fr,
                                     barretenberg::g1::affine_element,
                                     transcript::StandardTranscript>;

} // namespace waffle