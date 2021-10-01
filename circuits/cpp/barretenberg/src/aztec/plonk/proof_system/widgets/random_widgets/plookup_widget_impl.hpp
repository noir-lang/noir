#pragma once

#include <plonk/proof_system/proving_key/proving_key.hpp>
#include <plonk/transcript/transcript.hpp>
#include <polynomials/iterate_over_domain.hpp>
#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include <polynomials/polynomial_arithmetic.hpp>
#include <common/mem.hpp>

namespace waffle {

template <const size_t num_roots_cut_out_of_vanishing_polynomial>
ProverPlookupWidget<num_roots_cut_out_of_vanishing_polynomial>::ProverPlookupWidget(proving_key* input_key,
                                                                                    program_witness* input_witness)
    : ProverRandomWidget(input_key, input_witness)
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

template <const size_t num_roots_cut_out_of_vanishing_polynomial>
void ProverPlookupWidget<num_roots_cut_out_of_vanishing_polynomial>::compute_sorted_list_commitment(
    transcript::StandardTranscript& transcript)
{
    auto& s_1 = witness->wires.at("s");
    fr* s_2 = &witness->wires.at("s_2")[0];
    fr* s_3 = &witness->wires.at("s_3")[0];
    fr* s_4 = &witness->wires.at("s_4")[0];

    const auto eta = fr::serialize_from_buffer(transcript.get_challenge("eta", 0).begin());

    ITERATE_OVER_DOMAIN_START(key->small_domain);
    fr T0 = s_4[i];
    T0 *= eta;
    T0 += s_3[i];
    T0 *= eta;
    T0 += s_2[i];
    T0 *= eta;
    s_1[i] += T0;
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
    // into `s`. Since we wish to add 3 random scalars, we need k >= 3. In our case, we have set k = 4.
    // Thus, we can add 3 random scalars as (s{n-k+1}, s{n-k+2}, s{n-k+3}).
    const size_t s_randomness = 3;
    ASSERT(s_randomness < num_roots_cut_out_of_vanishing_polynomial);
    for (size_t k = 0; k < s_randomness; ++k) {
        s_1[((key->n - num_roots_cut_out_of_vanishing_polynomial) + 1 + k)] = fr::random_element();
    }

    polynomial s_lagrange_base(s_1, key->small_domain.size);
    witness->wires.insert({ "s_lagrange_base", s_lagrange_base });
    s_1.ifft(key->small_domain);
}

template <const size_t num_roots_cut_out_of_vanishing_polynomial>
void ProverPlookupWidget<num_roots_cut_out_of_vanishing_polynomial>::compute_grand_product_commitment(
    transcript::StandardTranscript& transcript)
{
    const size_t n = key->n;
    polynomial& z = witness->wires.at("z_lookup");
    polynomial& s = witness->wires.at("s_lagrange_base");
    polynomial& z_fft = key->wire_ffts.at("z_lookup_fft");

    fr* accumulators[4];
    accumulators[0] = &z[1];
    accumulators[1] = &z_fft[0];
    accumulators[2] = &z_fft[n];
    accumulators[3] = &z_fft[n + n];

    fr* column_1_step_size = &key->constraint_selectors_lagrange_base.at("q_2")[0];
    fr* column_2_step_size = &key->constraint_selectors_lagrange_base.at("q_m")[0];
    fr* column_3_step_size = &key->constraint_selectors_lagrange_base.at("q_c")[0];

    fr eta = fr::serialize_from_buffer(transcript.get_challenge("eta").begin());
    fr eta_sqr = eta.sqr();
    fr eta_cube = eta_sqr * eta;

    fr beta = fr::serialize_from_buffer(transcript.get_challenge("beta").begin());
    fr gamma = fr::serialize_from_buffer(transcript.get_challenge("beta", 1).begin());
    // gamma = fr(1);
    // beta = fr(1);
    std::array<fr*, 3> lagrange_base_wires;
    std::array<fr*, 4> lagrange_base_tables{
        &key->constraint_selectors_lagrange_base.at("table_value_1")[0],
        &key->constraint_selectors_lagrange_base.at("table_value_2")[0],
        &key->constraint_selectors_lagrange_base.at("table_value_3")[0],
        &key->constraint_selectors_lagrange_base.at("table_value_4")[0],
    };

    fr* lookup_selector = &key->constraint_selectors_lagrange_base.at("table_type")[0];
    fr* lookup_index_selector = &key->constraint_selectors_lagrange_base.at("table_index")[0];
    for (size_t i = 0; i < 3; ++i) {
        lagrange_base_wires[i] = &key->wire_ffts.at("w_" + std::to_string(i + 1) + "_fft")[0];
    }

    const fr gamma_beta_constant = gamma * (fr(1) + beta);
    const fr beta_constant = beta + fr(1);

#ifndef NO_MULTITHREADING
#pragma omp parallel
#endif
    {
#ifndef NO_MULTITHREADING
#pragma omp for
#endif
        for (size_t j = 0; j < key->small_domain.num_threads; ++j) {
            fr T0;
            fr T1;
            fr T2;
            fr T3;
            size_t start = j * key->small_domain.thread_size;
            size_t end = (j + 1) * key->small_domain.thread_size;
            // fr accumulating_beta = beta_constant.pow(start + 1);
            const size_t block_mask = key->small_domain.size - 1;

            fr next_table = lagrange_base_tables[0][start] + lagrange_base_tables[1][start] * eta +
                            lagrange_base_tables[2][start] * eta_sqr + lagrange_base_tables[3][start] * eta_cube;
            for (size_t i = start; i < end; ++i) {
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

                accumulators[0][i] = T0;
                accumulators[0][i] += gamma;

                T0 = lagrange_base_tables[3][(i + 1) & block_mask];
                T0 *= eta;
                T0 += lagrange_base_tables[2][(i + 1) & block_mask];
                T0 *= eta;
                T0 += lagrange_base_tables[1][(i + 1) & block_mask];
                T0 *= eta;
                T0 += lagrange_base_tables[0][(i + 1) & block_mask];

                accumulators[1][i] = T0 * beta + next_table;
                next_table = T0;
                accumulators[1][i] += gamma_beta_constant;

                accumulators[2][i] = beta_constant;
                // accumulating_beta *= (beta_constant);
                accumulators[3][i] = s[(i + 1) & block_mask];
                accumulators[3][i] *= beta;
                accumulators[3][i] += s[i];
                accumulators[3][i] += gamma_beta_constant;
            }
        }

// step 2: compute the constituent components of Z(X). This is a small multithreading bottleneck, as we have
// only 4 parallelizable processes
#ifndef NO_MULTITHREADING
#pragma omp for
#endif
        for (size_t i = 0; i < 4; ++i) {
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
            const size_t end = (j == key->small_domain.num_threads - 1) ? (j + 1) * key->small_domain.thread_size - 1
                                                                        : (j + 1) * key->small_domain.thread_size;
            // const size_t end =
            //     ((j + 1) * key->small_domain.thread_size) - ((j == key->small_domain.num_threads - 1) ? 1 : 0);
            fr inversion_accumulator = fr::one();
            for (size_t i = start; i < end; ++i) {
                accumulators[0][i] *= accumulators[2][i];
                accumulators[0][i] *= accumulators[1][i];
                accumulators[0][i] *= inversion_accumulator;
                inversion_accumulator *= accumulators[3][i];
            }
            inversion_accumulator = inversion_accumulator.invert();
            for (size_t i = end - 1; i != start - 1; --i) {

                // N.B. accumulators[0][i] = z[i + 1]
                // We can avoid fully reducing z[i + 1] as the inverse fft will take care of that for us
                accumulators[0][i] *= inversion_accumulator;
                inversion_accumulator *= accumulators[3][i];
            }
        }
    }
    z[0] = fr::one();

    // Since `z_plookup` needs to be evaluated at 2 points in UltraPLONK, we need to add a degree-2 random
    // polynomial to `z_lookup` to make it "look" uniformly random. Alternatively, we can just add 3
    // random scalars into the lagrange form of `z_lookup`, rationale for which similar to that explained
    // for `s` polynomial.
    const size_t z_randomness = 3;
    ASSERT(z_randomness < num_roots_cut_out_of_vanishing_polynomial);
    for (size_t k = 0; k < z_randomness; ++k) {
        z[((n - num_roots_cut_out_of_vanishing_polynomial) + 1 + k)] = fr::random_element();
    }

    z.ifft(key->small_domain);
}

template <const size_t num_roots_cut_out_of_vanishing_polynomial>
void ProverPlookupWidget<num_roots_cut_out_of_vanishing_polynomial>::compute_round_commitments(
    transcript::StandardTranscript& transcript, const size_t round_number, work_queue& queue)
{
    if (round_number == 2) {
        compute_sorted_list_commitment(transcript);
        polynomial& s = witness->wires.at("s");

        queue.add_to_queue({
            work_queue::WorkType::SCALAR_MULTIPLICATION,
            s.get_coefficients(),
            "S",
            barretenberg::fr(0),
            0,
        });
        queue.add_to_queue({
            work_queue::WorkType::FFT,
            nullptr,
            "s",
            barretenberg::fr(0),
            0,
        });
        return;
    }
    if (round_number == 3) {
        compute_grand_product_commitment(transcript);
        polynomial& z = witness->wires.at("z_lookup");

        queue.add_to_queue({
            work_queue::WorkType::SCALAR_MULTIPLICATION,
            z.get_coefficients(),
            "Z_LOOKUP",
            barretenberg::fr(0),
            0,
        });
        queue.add_to_queue({
            work_queue::WorkType::FFT,
            nullptr,
            "z_lookup",
            barretenberg::fr(0),
            0,
        });
        return;
    }
}

template <const size_t num_roots_cut_out_of_vanishing_polynomial>
barretenberg::fr ProverPlookupWidget<num_roots_cut_out_of_vanishing_polynomial>::compute_quotient_contribution(
    const fr& alpha_base, const transcript::StandardTranscript& transcript)
{
    polynomial& z_fft = key->wire_ffts.at("z_lookup_fft");

    fr eta = fr::serialize_from_buffer(transcript.get_challenge("eta").begin());
    fr alpha = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());
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

    std::array<fr*, 3> wire_ffts{
        &key->wire_ffts.at("w_1_fft")[0],
        &key->wire_ffts.at("w_2_fft")[0],
        &key->wire_ffts.at("w_3_fft")[0],
    };
    fr* s_fft = &key->wire_ffts.at("s_fft")[0];

    std::array<fr*, 4> table_ffts{
        &key->constraint_selector_ffts.at("table_value_1_fft")[0],
        &key->constraint_selector_ffts.at("table_value_2_fft")[0],
        &key->constraint_selector_ffts.at("table_value_3_fft")[0],
        &key->constraint_selector_ffts.at("table_value_4_fft")[0],
    };

    fr* column_1_step_size = &key->constraint_selector_ffts.at("q_2_fft")[0];
    fr* column_2_step_size = &key->constraint_selector_ffts.at("q_m_fft")[0];
    fr* column_3_step_size = &key->constraint_selector_ffts.at("q_c_fft")[0];

    fr* lookup_fft = &key->constraint_selector_ffts.at("table_type_fft")[0];
    fr* lookup_index_fft = &key->constraint_selector_ffts.at("table_index_fft")[0];

    polynomial& quotient_large = key->quotient_large;

    const fr gamma_beta_constant = gamma * (fr(1) + beta);

    const polynomial& l_1 = key->lagrange_1;
    const fr delta_factor = gamma_beta_constant.pow(key->small_domain.size - num_roots_cut_out_of_vanishing_polynomial);
    const fr alpha_sqr = alpha.sqr();

    const fr beta_constant = beta + fr(1);

    const size_t block_mask = key->large_domain.size - 1;

    // Step 4: Set the quotient polynomial to be equal to
    // (w_l(X) + \beta.sigma1(X) + \gamma).(w_r(X) + \beta.sigma2(X) + \gamma).(w_o(X) + \beta.sigma3(X) +
    // \gamma).Z(X).alpha
#ifndef NO_MULTITHREADING
#pragma omp parallel for
#endif
    for (size_t j = 0; j < key->large_domain.num_threads; ++j) {
        const size_t start = j * key->large_domain.thread_size;
        const size_t end = (j + 1) * key->large_domain.thread_size;

        fr T0;
        fr T1;
        fr T2;
        fr denominator;
        fr numerator;

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

            numerator = T0;
            numerator *= lookup_fft[i];
            numerator += gamma;

            T0 = table_ffts[3][(i + 4) & block_mask];
            T0 *= eta;
            T0 += table_ffts[2][(i + 4) & block_mask];
            T0 *= eta;
            T0 += table_ffts[1][(i + 4) & block_mask];
            T0 *= eta;
            T0 += table_ffts[0][(i + 4) & block_mask];

            T1 = beta;
            T1 *= T0;
            T1 += next_ts[i & 0x03UL];
            T1 += gamma_beta_constant;

            next_ts[i & 0x03UL] = T0;

            numerator *= T1;
            numerator *= beta_constant;

            denominator = s_fft[(i + 4) & block_mask];
            denominator *= beta;
            denominator += s_fft[i];
            denominator += gamma_beta_constant;

            T0 = l_1[i] * alpha;
            T1 = l_1[(i + 4 + 4 * num_roots_cut_out_of_vanishing_polynomial) & block_mask] * alpha_sqr;

            numerator += T0;
            numerator *= z_fft[i];
            numerator -= T0;

            denominator -= T1;
            denominator *= z_fft[(i + 4) & block_mask];
            denominator += T1 * delta_factor;

            // Combine into quotient polynomial
            T0 = numerator - denominator;
            quotient_large[i] += T0 * alpha_base;
        }
    }
    return alpha_base * alpha.sqr() * alpha;
}

// This part comes computes r_plookup terms in r(X). More on this can be found in
// https://hackmd.io/vUGG8CO_Rk2iEjruBL_gGw?view#Note-A-Mind-Boggling-Issue-with-Ultra-Plonk
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
    fr table_index_eval = transcript.get_field_element("table_index");

    fr s_eval = transcript.get_field_element("s");
    fr shifted_s_eval = transcript.get_field_element("s_omega");

    fr z_eval = transcript.get_field_element("z_lookup");
    fr shifted_z_eval = transcript.get_field_element("z_lookup_omega");

    fr z = transcript.get_challenge_field_element("z");
    // fr alpha = transcript.get_challenge_field_element("alpha", 0);
    fr beta = transcript.get_challenge_field_element("beta", 0);
    fr gamma = transcript.get_challenge_field_element("beta", 1);
    fr eta = transcript.get_challenge_field_element("eta", 0);
    fr l_numerator = z.pow(key->n) - fr(1);

    l_numerator *= key->small_domain.domain_inverse;
    fr l_1 = l_numerator / (z - fr(1));

    // compute w^{num_roots_cut_out_of_vanishing_polynomial + 1}
    fr l_end_root =
        (num_roots_cut_out_of_vanishing_polynomial & 1) ? key->small_domain.root.sqr() : key->small_domain.root;
    for (size_t i = 0; i < num_roots_cut_out_of_vanishing_polynomial / 2; ++i) {
        l_end_root *= key->small_domain.root.sqr();
    }
    fr l_end = l_numerator / ((z * l_end_root) - fr(1));

    const fr one(1);
    const fr gamma_beta_constant = gamma * (one + beta);

    const fr delta_factor = gamma_beta_constant.pow(key->small_domain.size - num_roots_cut_out_of_vanishing_polynomial);
    const fr alpha_sqr = alpha.sqr();

    const fr beta_constant = beta + one;

    fr T0;
    fr T1;
    fr T2;
    fr denominator;
    fr numerator;

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

    fr table_eval = table_evaluations[3];
    table_eval *= eta;
    table_eval += table_evaluations[2];
    table_eval *= eta;
    table_eval += table_evaluations[1];
    table_eval *= eta;
    table_eval += table_evaluations[0];

    numerator = f_eval * table_type_eval;
    numerator += gamma;

    T0 = shifted_table_evaluations[3];
    T0 *= eta;
    T0 += shifted_table_evaluations[2];
    T0 *= eta;
    T0 += shifted_table_evaluations[1];
    T0 *= eta;
    T0 += shifted_table_evaluations[0];

    T1 = beta;
    T1 *= T0;
    T1 += table_eval;
    T1 += gamma_beta_constant;

    numerator *= T1;
    numerator *= beta_constant;

    denominator = shifted_s_eval;
    denominator *= beta;
    denominator += s_eval;
    denominator += gamma_beta_constant;

    T0 = l_1 * alpha;
    T1 = l_end * alpha_sqr;

    numerator += T0;
    numerator *= z_eval;
    numerator -= T0;

    denominator -= T1;
    denominator *= shifted_z_eval;
    denominator += T1 * delta_factor;

    // We need to add the constant term of plookup permutation polynomial in the linearisation
    // polynomial to ensure that r(z) = 0.
    T0 = numerator - denominator;
    r[0] += T0 * alpha_base;

    return alpha_base * alpha.sqr() * alpha;
}

// ###

template <typename Field, typename Group, typename Transcript, const size_t num_roots_cut_out_of_vanishing_polynomial>
VerifierPlookupWidget<Field, Group, Transcript, num_roots_cut_out_of_vanishing_polynomial>::VerifierPlookupWidget()
{}

template <typename Field, typename Group, typename Transcript, const size_t num_roots_cut_out_of_vanishing_polynomial>
Field VerifierPlookupWidget<Field, Group, Transcript, num_roots_cut_out_of_vanishing_polynomial>::
    compute_quotient_evaluation_contribution(typename Transcript::Key* key,
                                             const Field& alpha_base,
                                             const Transcript& transcript,
                                             Field& t_eval,
                                             Field& r_0,
                                             const bool use_linearisation)

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
    Field table_index_eval = transcript.get_field_element("table_index");

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

    // compute w^{num_roots_cut_out_of_vanishing_polynomial + 1}
    Field l_end_root = (num_roots_cut_out_of_vanishing_polynomial & 1) ? key->domain.root.sqr() : key->domain.root;
    for (size_t i = 0; i < num_roots_cut_out_of_vanishing_polynomial / 2; ++i) {
        l_end_root *= key->domain.root.sqr();
    }
    Field l_end = l_numerator / ((z * l_end_root) - Field(1));

    const Field one(1);
    const Field gamma_beta_constant = gamma * (one + beta);

    const Field delta_factor = gamma_beta_constant.pow(key->domain.size - num_roots_cut_out_of_vanishing_polynomial);
    const Field alpha_sqr = alpha.sqr();

    const Field beta_constant = beta + one;

    Field T0;
    Field T1;
    Field T2;
    Field denominator;
    Field numerator;

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

    Field table_eval = table_evaluations[3];
    table_eval *= eta;
    table_eval += table_evaluations[2];
    table_eval *= eta;
    table_eval += table_evaluations[1];
    table_eval *= eta;
    table_eval += table_evaluations[0];

    numerator = f_eval * table_type_eval;
    numerator += gamma;

    T0 = shifted_table_evaluations[3];
    T0 *= eta;
    T0 += shifted_table_evaluations[2];
    T0 *= eta;
    T0 += shifted_table_evaluations[1];
    T0 *= eta;
    T0 += shifted_table_evaluations[0];

    T1 = beta;
    T1 *= T0;
    T1 += table_eval;
    T1 += gamma_beta_constant;

    numerator *= T1;
    numerator *= beta_constant;

    denominator = shifted_s_eval;
    denominator *= beta;
    denominator += s_eval;
    denominator += gamma_beta_constant;

    T0 = l_1 * alpha;
    T1 = l_end * alpha_sqr;

    numerator += T0;
    numerator *= z_eval;
    numerator -= T0;

    denominator -= T1;
    denominator *= shifted_z_eval;
    denominator += T1 * delta_factor;

    // Combine into quotient polynomial
    T0 = numerator - denominator;
    t_eval += T0 * alpha_base;
    if (use_linearisation) {
        r_0 += (T0 * alpha_base);
    }

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