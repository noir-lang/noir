#pragma once

#include "../proving_key/proving_key.hpp"
#include "../public_inputs/public_inputs.hpp"
#include "../utils/linearizer.hpp"
#include <plonk/transcript/transcript.hpp>
#include <polynomials/iterate_over_domain.hpp>
#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include <polynomials/polynomial_arithmetic.hpp>
#include <common/mem.hpp>

using namespace barretenberg;

namespace waffle {

ProverPLookupWidget::ProverPLookupWidget(proving_key* input_key, program_witness* input_witness)
    : ProverBaseWidget(input_key, input_witness)
{}

ProverPLookupWidget::ProverPLookupWidget(const ProverPLookupWidget& other)
    : ProverBaseWidget(other)
{}

ProverPLookupWidget::ProverPLookupWidget(ProverPLookupWidget&& other)
    : ProverBaseWidget(other)
{}

ProverPLookupWidget& ProverPLookupWidget::operator=(const ProverPLookupWidget& other)
{
    ProverBaseWidget::operator=(other);
    return *this;
}

ProverPLookupWidget& ProverPLookupWidget::operator=(ProverPLookupWidget&& other)
{
    ProverBaseWidget::operator=(other);
    return *this;
}

struct lookup_entry {
    lookup_entry(const fr& a, const fr& b, const fr& c, const fr& d)
        : data{ a, b, c, d }
    {}

    lookup_entry() {}

    lookup_entry(const lookup_entry& other)
        : data{ other.data[0], other.data[1], other.data[2], other.data[3] }
    {}

    lookup_entry(lookup_entry&& other)
        : data{ other.data[0], other.data[1], other.data[2], other.data[3] }
    {}

    lookup_entry& operator=(const lookup_entry& other)
    {
        data[0] = other.data[0];
        data[1] = other.data[1];
        data[2] = other.data[2];
        data[3] = other.data[3];
        return *this;
    }

    lookup_entry& operator=(lookup_entry&& other)
    {
        data[0] = other.data[0];
        data[1] = other.data[1];
        data[2] = other.data[2];
        data[3] = other.data[3];
        return *this;
    }

    bool operator<(const lookup_entry& other) const
    {
        bool result = (data[3].data[3] < other.data[3].data[3]);

        bool eq_check = data[3].data[3] == other.data[3].data[3];
        result = result || (eq_check && data[3].data[2] < other.data[3].data[2]);

        eq_check = eq_check && (data[3].data[2] == other.data[3].data[2]);
        result = result || (eq_check && data[3].data[1] < other.data[3].data[1]);

        eq_check = eq_check && (data[3].data[1] == other.data[3].data[1]);
        result = result || (eq_check && data[3].data[0] < other.data[3].data[0]);

        eq_check = eq_check && (data[3].data[0] == other.data[3].data[0]);
        result = result || (eq_check && data[1].data[3] < other.data[1].data[3]);

        eq_check = eq_check && (data[1].data[3] == other.data[1].data[3]);
        result = result || (eq_check && data[1].data[2] < other.data[1].data[2]);

        eq_check = eq_check && (data[1].data[2] == other.data[1].data[2]);
        result = result || (eq_check && data[1].data[1] < other.data[1].data[1]);

        eq_check = eq_check && (data[1].data[1] == other.data[1].data[1]);
        result = result || (eq_check && data[1].data[0] < other.data[1].data[0]);

        eq_check = eq_check && (data[1].data[0] == other.data[1].data[0]);
        result = result || (eq_check && data[0].data[3] < other.data[0].data[3]);

        eq_check = eq_check && (data[0].data[3] == other.data[0].data[3]);
        result = result || (eq_check && data[0].data[2] < other.data[0].data[2]);

        eq_check = eq_check && (data[0].data[2] == other.data[0].data[2]);
        result = result || (eq_check && data[0].data[1] < other.data[0].data[1]);

        eq_check = eq_check && (data[0].data[1] == other.data[0].data[1]);
        result = result || (eq_check && data[0].data[0] < other.data[0].data[0]);

        eq_check = eq_check && (data[0].data[0] == other.data[0].data[0]);
        result = result || (eq_check && data[2].data[3] < other.data[2].data[3]);

        eq_check = eq_check && (data[2].data[3] == other.data[2].data[3]);
        result = result || (eq_check && data[2].data[2] < other.data[2].data[2]);

        eq_check = eq_check && (data[2].data[2] == other.data[2].data[2]);
        result = result || (eq_check && data[2].data[1] < other.data[2].data[1]);

        eq_check = eq_check && (data[2].data[1] == other.data[2].data[1]);
        result = result || (eq_check && data[2].data[0] < other.data[2].data[0]);

        return result;
    }
    fr data[4];
};

void ProverPLookupWidget::compute_sorted_list_commitment(transcript::Transcript& transcript)
{
    polynomial& s = witness->wires.at("s");
    const auto& lookup_mapping = key->lookup_mapping;
    const auto& table_indices = key->table_indices;
    const auto step_size = key->lookup_table_step_size;
    const auto num_lookup_tables = key->num_lookup_tables;

    const auto eta = fr::serialize_from_buffer(transcript.get_challenge("eta", 0).begin());

    const auto eta_sqr = eta.sqr();
    const auto eta_cube = eta_sqr * eta;
    // const auto beta = fr::serialize_from_buffer(transcript.get_challenge("beta", 0).begin());
    // const auto gamma = fr::serialize_from_buffer(transcript.get_challenge("beta", 1).begin());

    std::vector<fr> table_values;
    for (size_t i = 0; i < num_lookup_tables + 1; ++i) {
        table_values.push_back(fr(i));
    }

    std::vector<std::vector<lookup_entry>> unsorted_lists;
    for (size_t i = 0; i < num_lookup_tables + 1; ++i) {
        unsorted_lists.emplace_back(std::vector<lookup_entry>());
    }

    fr* w_1 = &witness->wires.at("w_1")[0];
    fr* w_2 = &witness->wires.at("w_2")[0];
    fr* w_3 = &witness->wires.at("w_3")[0];

    for (size_t i = 0; i < key->small_domain.size; ++i) {
        switch (lookup_mapping[i]) {
        case LookupType::ABSOLUTE_LOOKUP: {
            unsorted_lists[table_indices[i]].emplace_back(
                lookup_entry(w_1[i], w_2[i], w_3[i], table_values[table_indices[i]]));
            break;
        }
        case LookupType::RELATIVE_LOOKUP: {
            auto t0 = w_1[i] - w_1[i + 1] * step_size;
            auto t1 = w_2[i] - w_2[i + 1] * step_size;
            auto t2 = w_3[i] - w_3[i + 1] * step_size;
            unsorted_lists[table_indices[i]].emplace_back(lookup_entry(t0, t1, t2, table_values[table_indices[i]]));
            break;
        }
        default: {
            unsorted_lists[0].emplace_back(fr::zero(), fr::zero(), fr::zero(), fr::zero());
            break;
        }
        }
    }

    std::array<fr*, 4> lagrange_base_tables{
        &key->permutation_selectors_lagrange_base.at("table_value_1")[0],
        &key->permutation_selectors_lagrange_base.at("table_value_2")[0],
        &key->permutation_selectors_lagrange_base.at("table_value_3")[0],
        &key->permutation_selectors_lagrange_base.at("table_value_4")[0],
    };

    for (size_t i = 0; i < key->small_domain.size; ++i) {
        uint256_t index(lagrange_base_tables[3][i]);
        if (index != uint256_t(0)) {
            unsorted_lists[static_cast<size_t>(index.data[0])].emplace_back(lagrange_base_tables[0][i],
                                                                            lagrange_base_tables[1][i],
                                                                            lagrange_base_tables[2][i],
                                                                            lagrange_base_tables[3][i]);
        }
    }

    for (size_t i = 1; i < num_lookup_tables + 1; ++i) {
        for (auto& item : unsorted_lists[i]) {
            item.data[0] = item.data[0].from_montgomery_form();
            item.data[1] = item.data[1].from_montgomery_form();
            item.data[2] = item.data[2].from_montgomery_form();
            item.data[3] = item.data[3].from_montgomery_form();
        }
        std::sort(unsorted_lists[i].begin(), unsorted_lists[i].end());
        for (auto& item : unsorted_lists[i]) {
            item.data[0] = item.data[0].to_montgomery_form();
            item.data[1] = item.data[1].to_montgomery_form();
            item.data[2] = item.data[2].to_montgomery_form();
            item.data[3] = item.data[3].to_montgomery_form();
        }
    }

    size_t num_set_union_elements = 0;
    for (size_t i = 1; i < unsorted_lists.size(); ++i) {
        num_set_union_elements += unsorted_lists[i].size();
    }
    size_t offset = key->small_domain.size - num_set_union_elements;
    size_t count = offset;

    for (size_t i = 1; i < unsorted_lists.size(); ++i) {
        auto list = unsorted_lists[i];

        for (auto element : list) {
            s[count] = element.data[0] + element.data[1] * eta + element.data[2] * eta_sqr + element.data[3] * eta_cube;
            ++count;
        }
    }
    for (size_t i = 0; i < offset; ++i) {
        s[i] = fr::zero();
    }

    s[key->small_domain.size] = s[0];

    polynomial s_lagrange_base(s, key->small_domain.size);
    witness->wires.insert({ "s_lagrange_base", s_lagrange_base });
    s.ifft(key->small_domain);
}

void ProverPLookupWidget::compute_grand_product_commitment(transcript::Transcript& transcript)
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

    fr eta = fr::serialize_from_buffer(transcript.get_challenge("eta").begin());
    fr eta_sqr = eta.sqr();
    fr eta_cube = eta_sqr * eta;

    fr beta = fr::serialize_from_buffer(transcript.get_challenge("beta").begin());
    fr gamma = fr::serialize_from_buffer(transcript.get_challenge("beta", 1).begin());
    // gamma = fr(1);
    // beta = fr(1);
    std::array<fr*, 3> lagrange_base_wires;
    std::array<fr*, 4> lagrange_base_tables{
        &key->permutation_selectors_lagrange_base.at("table_value_1")[0],
        &key->permutation_selectors_lagrange_base.at("table_value_2")[0],
        &key->permutation_selectors_lagrange_base.at("table_value_3")[0],
        &key->permutation_selectors_lagrange_base.at("table_value_4")[0],
    };

    fr* lookup_selector = &key->permutation_selectors_lagrange_base.at("table_type")[0];
    fr* lookup_index_selector = &key->permutation_selectors_lagrange_base.at("table_index")[0];
    for (size_t i = 0; i < 3; ++i) {
        lagrange_base_wires[i] = &key->wire_ffts.at("w_" + std::to_string(i + 1) + "_fft")[0];
    }

    const fr two(2);
    const fr half(two.invert());

    const fr gamma_beta_constant = gamma * (fr(1) + beta);
    const fr beta_constant = beta + fr(1);
    const auto step_size = key->lookup_table_step_size;

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

            fr next_f = lagrange_base_wires[2][start] * eta_sqr + lagrange_base_wires[1][start] * eta +
                        lagrange_base_wires[0][start];
            fr next_table = lagrange_base_tables[0][start] + lagrange_base_tables[1][start] * eta +
                            lagrange_base_tables[2][start] * eta_sqr + lagrange_base_tables[3][start] * eta_cube;
            for (size_t i = start; i < end; ++i) {
                // T0 = lookup_index_selector[i + 1];
                // T0 *= eta;
                T0 = lagrange_base_wires[2][i + 1];
                T0 *= eta;
                T0 += lagrange_base_wires[1][i + 1];
                T0 *= eta;
                T0 += lagrange_base_wires[0][i + 1];

                T3 = next_f + (lookup_index_selector[i] * eta_cube);
                T2 = (T3 - T0 * step_size) * half;
                T1 = T3;

                next_f = T0;

                // T1 *= (lookup_selector[i] - one);
                accumulators[0][i] = (T2 - T1) * lookup_selector[i];
                accumulators[0][i] += T1;
                accumulators[0][i] += T1;
                accumulators[0][i] -= T2;
                accumulators[0][i] *= lookup_selector[i];

                // std::cout << "accumulators[0][" << i << "] = " << accumulators[0][i] << std::endl;
                accumulators[0][i] += gamma;

                T0 = lagrange_base_tables[3][i + 1];
                T0 *= eta;
                T0 += lagrange_base_tables[2][i + 1];
                T0 *= eta;
                T0 += lagrange_base_tables[1][i + 1];
                T0 *= eta;
                T0 += lagrange_base_tables[0][i + 1];

                accumulators[1][i] = T0 * beta + next_table;
                next_table = T0;
                accumulators[1][i] += gamma_beta_constant;

                accumulators[2][i] = beta_constant;
                // accumulating_beta *= (beta_constant);
                accumulators[3][i] = s[i + 1];
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
            const size_t end = (j + 1) * key->small_domain.thread_size;
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

    z.ifft(key->small_domain);
}

void ProverPLookupWidget::compute_round_commitments(transcript::Transcript& transcript,
                                                    const size_t round_number,
                                                    work_queue& queue)
{
    if (round_number == 2) {
        compute_sorted_list_commitment(transcript);
        polynomial& s = witness->wires.at("s");

        queue.add_to_queue({
            work_queue::WorkType::SCALAR_MULTIPLICATION,
            s.get_coefficients(),
            "S",
        });
        queue.add_to_queue({
            work_queue::WorkType::FFT,
            nullptr,
            "s",
        });
        return;
    }
    if (round_number == 3) {
        compute_grand_product_commitment(transcript);
        polynomial& z = witness->wires.at("z_lookup");

        queue.add_to_queue({
            work_queue::WorkType::SCALAR_MULTIPLICATION,
            z.get_coefficients(),
            "Z_lookup",
        });
        queue.add_to_queue({
            work_queue::WorkType::FFT,
            nullptr,
            "z_lookup",
        });
        return;
    }
}

fr ProverPLookupWidget::compute_quotient_contribution(const fr& alpha_base, const transcript::Transcript& transcript)
{
    polynomial& z_fft = key->wire_ffts.at("z_lookup_fft");

    fr eta = fr::serialize_from_buffer(transcript.get_challenge("eta").begin());
    fr eta_cube = eta.sqr() * eta;
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
        &key->permutation_selector_ffts.at("table_value_1_fft")[0],
        &key->permutation_selector_ffts.at("table_value_2_fft")[0],
        &key->permutation_selector_ffts.at("table_value_3_fft")[0],
        &key->permutation_selector_ffts.at("table_value_4_fft")[0],
    };
    fr* lookup_fft = &key->permutation_selector_ffts.at("table_type_fft")[0];
    fr* lookup_index_fft = &key->permutation_selector_ffts.at("table_index_fft")[0];

    polynomial& quotient_large = key->quotient_large;

    const fr one(1);
    const fr half = fr(2).invert();
    const fr gamma_beta_constant = gamma * (fr(1) + beta);

    const polynomial& l_1 = key->lagrange_1;
    const fr delta_factor = gamma_beta_constant.pow(key->small_domain.size - 1);
    const fr alpha_sqr = alpha.sqr();

    const fr beta_constant = beta + fr(1);
    const auto step_size = key->lookup_table_step_size;

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

        std::array<fr, 4> next_fs;
        std::array<fr, 4> next_ts;
        for (size_t i = 0; i < 4; ++i) {
            next_fs[i] = wire_ffts[2][start + i];
            next_fs[i] *= eta;
            next_fs[i] += wire_ffts[1][start + i];
            next_fs[i] *= eta;
            next_fs[i] += wire_ffts[0][start + i];
            next_ts[i] = table_ffts[3][start + i];
            next_ts[i] *= eta;
            next_ts[i] += table_ffts[2][start + i];
            next_ts[i] *= eta;
            next_ts[i] += table_ffts[1][start + i];
            next_ts[i] *= eta;
            next_ts[i] += table_ffts[0][start + i];
        }
        for (size_t i = start; i < end; ++i) {

            T0 = wire_ffts[2][i + 4];
            T0 *= eta;
            T0 += wire_ffts[1][i + 4];
            T0 *= eta;
            T0 += wire_ffts[0][i + 4];

            T1 = lookup_index_fft[i];
            T1 *= eta_cube;
            T1 += next_fs[i & 0x03UL];

            T2 = T1;
            T2 -= T0 * step_size;
            T2 *= half;

            next_fs[i & 0x03UL] = T0;

            numerator = (T2 - T1) * lookup_fft[i];
            numerator += T1;
            numerator += T1;
            numerator -= T2;

            numerator *= lookup_fft[i];
            numerator += gamma;

            T0 = table_ffts[3][i + 4];
            T0 *= eta;
            T0 += table_ffts[2][i + 4];
            T0 *= eta;
            T0 += table_ffts[1][i + 4];
            T0 *= eta;
            T0 += table_ffts[0][i + 4];

            T1 = beta;
            T1 *= T0;
            T1 += next_ts[i & 0x03UL];
            T1 += gamma_beta_constant;

            next_ts[i & 0x03UL] = T0;

            numerator *= T1;
            numerator *= beta_constant;

            denominator = s_fft[i + 4];
            denominator *= beta;
            denominator += s_fft[i];
            denominator += gamma_beta_constant;

            T0 = l_1[i] * alpha;
            T1 = l_1[i + 8] * alpha_sqr;

            numerator += T0;
            numerator *= z_fft[i];
            numerator -= T0;

            denominator -= T1;
            denominator *= z_fft[i + 4];
            denominator += T1 * delta_factor;

            // Combine into quotient polynomial
            T0 = numerator - denominator;
            quotient_large[i] = T0 * alpha_base;
        }
    }
    return alpha_base * alpha.sqr() * alpha;
}

fr ProverPLookupWidget::compute_linear_contribution(const fr& alpha_base,
                                                    const transcript::Transcript& transcript,
                                                    polynomial&)
{
    fr alpha = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());

    return alpha_base * alpha.sqr() * alpha;
}

void ProverPLookupWidget::compute_opening_poly_contribution(const transcript::Transcript& transcript, const bool)
{
    fr* opening_poly = &key->opening_poly[0];
    fr* shifted_opening_poly = &key->shifted_opening_poly[0];

    fr* z_lookup = &witness->wires.at("z_lookup")[0];
    fr* s = &witness->wires.at("s")[0];

    std::array<fr*, 4> tables{
        &key->permutation_selectors.at("table_value_1")[0],
        &key->permutation_selectors.at("table_value_2")[0],
        &key->permutation_selectors.at("table_value_3")[0],
        &key->permutation_selectors.at("table_value_4")[0],
    };
    fr* table_index_selector = &key->permutation_selectors.at("table_index")[0];
    fr* table_selector = &key->permutation_selectors.at("table_type")[0];

    // const size_t num_challenges = num_sigma_evaluations + (!use_linearisation ? 1 : 0);
    std::array<barretenberg::fr, 8> nu_challenges{
        fr::serialize_from_buffer(&transcript.get_challenge_from_map("nu", "table_value_1")[0]),
        fr::serialize_from_buffer(&transcript.get_challenge_from_map("nu", "table_value_2")[0]),
        fr::serialize_from_buffer(&transcript.get_challenge_from_map("nu", "table_value_3")[0]),
        fr::serialize_from_buffer(&transcript.get_challenge_from_map("nu", "table_value_4")[0]),
        fr::serialize_from_buffer(&transcript.get_challenge_from_map("nu", "table_index")[0]),
        fr::serialize_from_buffer(&transcript.get_challenge_from_map("nu", "table_type")[0]),
        fr::serialize_from_buffer(&transcript.get_challenge_from_map("nu", "s")[0]),
        fr::serialize_from_buffer(&transcript.get_challenge_from_map("nu", "z_lookup")[0]),
    };

    // barretenberg::fr shifted_nu_challenge =
    //     fr::serialize_from_buffer(&transcript.get_challenge_from_map("nu", "z_omega")[0]);

    ITERATE_OVER_DOMAIN_START(key->small_domain);

    opening_poly[i] += tables[0][i] * nu_challenges[0];
    opening_poly[i] += tables[1][i] * nu_challenges[1];
    opening_poly[i] += tables[2][i] * nu_challenges[2];
    opening_poly[i] += tables[3][i] * nu_challenges[3];
    opening_poly[i] += table_index_selector[i] * nu_challenges[4];
    opening_poly[i] += table_selector[i] * nu_challenges[5];
    opening_poly[i] += s[i] * nu_challenges[6];
    opening_poly[i] += z_lookup[i] * nu_challenges[7];

    shifted_opening_poly[i] += tables[0][i] * nu_challenges[0];
    shifted_opening_poly[i] += tables[1][i] * nu_challenges[1];
    shifted_opening_poly[i] += tables[2][i] * nu_challenges[2];
    shifted_opening_poly[i] += tables[3][i] * nu_challenges[3];
    shifted_opening_poly[i] += s[i] * nu_challenges[6];
    shifted_opening_poly[i] += z_lookup[i] * nu_challenges[7];
    ITERATE_OVER_DOMAIN_END;
}

void ProverPLookupWidget::compute_transcript_elements(transcript::Transcript& transcript, const bool)
{
    // iterate over permutations, skipping the last one as we use the linearisation trick to avoid including it in
    // the transcript
    const size_t n = key->n;
    fr z_challenge = fr::serialize_from_buffer(transcript.get_challenge("z").begin());
    fr shifted_z = z_challenge * key->small_domain.root;

    fr evaluation = key->permutation_selectors.at("table_value_1").evaluate(z_challenge, n);
    transcript.add_element("table_value_1", evaluation.to_buffer());

    evaluation = key->permutation_selectors.at("table_value_2").evaluate(z_challenge, n);
    transcript.add_element("table_value_2", evaluation.to_buffer());

    evaluation = key->permutation_selectors.at("table_value_3").evaluate(z_challenge, n);
    transcript.add_element("table_value_3", evaluation.to_buffer());

    evaluation = key->permutation_selectors.at("table_value_4").evaluate(z_challenge, n);
    transcript.add_element("table_value_4", evaluation.to_buffer());

    evaluation = key->permutation_selectors.at("table_value_1").evaluate(shifted_z, n);
    transcript.add_element("table_value_1_omega", evaluation.to_buffer());

    evaluation = key->permutation_selectors.at("table_value_2").evaluate(shifted_z, n);
    transcript.add_element("table_value_2_omega", evaluation.to_buffer());

    evaluation = key->permutation_selectors.at("table_value_3").evaluate(shifted_z, n);
    transcript.add_element("table_value_3_omega", evaluation.to_buffer());

    evaluation = key->permutation_selectors.at("table_value_4").evaluate(shifted_z, n);
    transcript.add_element("table_value_4_omega", evaluation.to_buffer());

    evaluation = key->permutation_selectors.at("table_index").evaluate(z_challenge, n);
    transcript.add_element("table_index", evaluation.to_buffer());

    evaluation = key->permutation_selectors.at("table_type").evaluate(z_challenge, n);
    transcript.add_element("table_type", evaluation.to_buffer());

    evaluation = witness->wires.at("z_lookup").evaluate(z_challenge, n);
    transcript.add_element("z_lookup", evaluation.to_buffer());

    evaluation = witness->wires.at("z_lookup").evaluate(shifted_z, n);
    transcript.add_element("z_lookup_omega", evaluation.to_buffer());

    evaluation = witness->wires.at("s").evaluate(z_challenge, n);
    transcript.add_element("s", evaluation.to_buffer());

    evaluation = witness->wires.at("s").evaluate(shifted_z, n);
    transcript.add_element("s_omega", evaluation.to_buffer());
    return;
}

// ###

template <typename Field, typename Group, typename Transcript>
VerifierPLookupWidget<Field, Group, Transcript>::VerifierPLookupWidget()
{}

template <typename Field, typename Group, typename Transcript>
Field VerifierPLookupWidget<Field, Group, Transcript>::compute_quotient_evaluation_contribution(
    verification_key* key,
    const Field& alpha,
    const Transcript& transcript,
    Field& t_eval,
    const bool use_linearisation)
{

    Field alpha_cubed = alpha.sqr() * alpha;
    Field z = transcript.get_challenge_field_element("z");
    Field beta = transcript.get_challenge_field_element("beta", 0);
    Field gamma = transcript.get_challenge_field_element("beta", 1);
    Field z_beta = z * beta;

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

    Field z_pow = z;
    for (size_t i = 0; i < key->domain.log2_size; ++i) {
        z_pow *= z_pow;
    }
    Field numerator = z_pow - Field(1);

    numerator *= key->domain.domain_inverse;
    Field l_1 = numerator / (z - Field(1));
    Field l_n_minus_1 = numerator / ((z * key->domain.root.sqr()) - Field(1));

    Field z_1_shifted_eval = transcript.get_field_element("z_omega");

    Field T0;
    Field z_contribution = Field(1);
    for (size_t i = 0; i < key->program_width; ++i) {
        Field coset_generator = (i == 0) ? Field(1) : Field::coset_generator(i - 1);
        T0 = z_beta * coset_generator;
        T0 += wire_evaluations[i];
        T0 += gamma;
        z_contribution *= T0;
    }
    Field z_1_multiplicand = z_contribution * alpha;
    T0 = l_1 * alpha_cubed;
    z_1_multiplicand += T0;

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

    // reconstruct evaluation of quotient polynomial from prover messages
    Field T1;
    Field T2;
    Field alpha_pow[4];
    alpha_pow[0] = alpha;
    for (size_t i = 1; i < 4; ++i) {
        alpha_pow[i] = alpha_pow[i - 1] * alpha_pow[0];
    }

    sigma_contribution = Field(1);

    for (size_t i = 0; i < key->program_width - 1; ++i) {
        T0 = sigma_evaluations[i] * beta;
        T1 = wire_evaluations[i] + gamma;
        T0 += T1;
        sigma_contribution *= T0;
    }

    std::vector<Field> public_inputs = transcript.get_field_element_vector("public_inputs");

    Field public_input_delta = compute_public_input_delta<Field>(public_inputs, beta, gamma, key->domain.root);
    T0 = wire_evaluations[key->program_width - 1] + gamma;
    sigma_contribution *= T0;
    sigma_contribution *= z_1_shifted_eval;
    sigma_contribution *= alpha_pow[0];

    T1 = z_1_shifted_eval - public_input_delta;
    T1 *= l_n_minus_1;
    T1 *= alpha_pow[1];

    T2 = l_1 * alpha_pow[2];
    T1 -= T2;
    T1 -= sigma_contribution;

    if (use_linearisation) {
        Field linear_eval = transcript.get_field_element("r");
        T1 += linear_eval;
    }

    t_eval += T1;

    if (!use_linearisation) {
        Field z_eval = transcript.get_field_element("z");
        t_eval += (z_1_multiplicand * z_eval);
        t_eval += (sigma_last_multiplicand * sigma_evaluations[key->program_width - 1]);
    }

    return alpha.sqr().sqr();
}

template <typename Field, typename Group, typename Transcript>
void VerifierPLookupWidget<Field, Group, Transcript>::compute_batch_evaluation_contribution(
    verification_key* key, Field& batch_eval, const Transcript& transcript, const bool use_linearisation)
{
    Field u = transcript.get_challenge_field_element("separator");
    Field shifted_z_eval = transcript.get_field_element("z_omega");

    const size_t num_sigma_evaluations = use_linearisation ? key->program_width - 1 : key->program_width;
    std::vector<Field> sigmas(num_sigma_evaluations);
    for (size_t i = 0; i < num_sigma_evaluations; ++i) {
        sigmas[i] = transcript.get_field_element("sigma_" + std::to_string(i + 1));
    }

    Field T0;
    Field quotient_temp = Field(0);

    for (size_t k = 0; k < key->program_width - 1; ++k) {
        T0 = sigmas[k] * transcript.get_challenge_field_element_from_map("nu", "sigma_" + std::to_string(k + 1));
        quotient_temp += T0;
    }

    if (!use_linearisation) {
        Field z_eval = transcript.get_field_element("z");
        T0 = sigmas[key->program_width - 1] *
             transcript.get_challenge_field_element_from_map("nu", "sigma_" + std::to_string(key->program_width));
        quotient_temp += T0;
        T0 = z_eval * transcript.get_challenge_field_element_from_map("nu", "z_omega");
        quotient_temp += T0;
    }

    Field shifted_batch_eval = shifted_z_eval; // * transcript.get_challenge_field_element_from_map("nu", "z_omega");

    batch_eval += quotient_temp;
    batch_eval += (shifted_batch_eval * u);
};

template <typename Field, typename Group, typename Transcript>
Field VerifierPLookupWidget<Field, Group, Transcript>::append_scalar_multiplication_inputs(verification_key* key,
                                                                                           const Field& alpha_base,
                                                                                           const Transcript& transcript,
                                                                                           std::vector<Group>& elements,
                                                                                           std::vector<Field>& scalars,
                                                                                           const bool use_linearisation)
{
    Field alpha_step = transcript.get_challenge_field_element("alpha");

    Field alpha_cubed = alpha_base * alpha_step.sqr();
    Field u = transcript.get_challenge_field_element("separator");
    Field shifted_z_eval = transcript.get_field_element("z_omega");

    Field z = transcript.get_challenge_field_element("z");
    Field z_pow = z;
    for (size_t i = 0; i < key->domain.log2_size; ++i) {
        z_pow *= z_pow;
    }
    Field numerator = z_pow - Field(1);

    numerator *= key->domain.domain_inverse;
    Field l_1 = numerator / (z - Field(1));

    Field beta = transcript.get_challenge_field_element("beta", 0);
    Field gamma = transcript.get_challenge_field_element("beta", 1);
    Field z_beta = z * beta;

    std::vector<Field> wire_evaluations;
    for (size_t i = 0; i < key->program_width; ++i) {
        wire_evaluations.emplace_back(transcript.get_field_element("w_" + std::to_string(i + 1)));
    }

    g1::affine_element Z_1 = transcript.get_group_element("Z");

    elements.emplace_back(Z_1);
    // Field z_omega_challenge = transcript.get_challenge_field_element_from_map("nu", "z_omega");
    if (use_linearisation) {
        Field linear_nu = transcript.get_challenge_field_element_from_map("nu", "r");
        Field T0;
        Field z_contribution = Field(1);
        for (size_t i = 0; i < key->program_width; ++i) {
            Field coset_generator = (i == 0) ? Field(1) : Field::coset_generator(i - 1);
            T0 = z_beta * coset_generator;
            T0 += wire_evaluations[i];
            T0 += gamma;
            z_contribution *= T0;
        }
        Field z_1_multiplicand = z_contribution * alpha_base;
        T0 = l_1 * alpha_cubed;
        z_1_multiplicand += T0;
        z_1_multiplicand *= linear_nu;
        z_1_multiplicand += u; // (u * z_omega_challenge);
        scalars.emplace_back(z_1_multiplicand);
    } else {
        Field T0 = u + transcript.get_challenge_field_element_from_map("nu", "z_omega");
        //        Field T0 = z_omega_challenge * u + transcript.get_challenge_field_element_from_map("nu",
        //        "z_omega");
        scalars.emplace_back(T0);
    }

    for (size_t i = 0; i < key->program_width - 1; ++i) {
        elements.emplace_back(key->permutation_selectors.at("SIGMA_" + std::to_string(i + 1)));
        scalars.emplace_back(transcript.get_challenge_field_element_from_map("nu", "sigma_" + std::to_string(i + 1)));
    }

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
        elements.emplace_back(key->permutation_selectors.at("SIGMA_" + std::to_string(key->program_width)));
        sigma_last_multiplicand *= linear_nu;
        scalars.emplace_back(sigma_last_multiplicand);
    } else {
        elements.emplace_back(key->permutation_selectors.at("SIGMA_" + std::to_string(key->program_width)));
        scalars.emplace_back(
            transcript.get_challenge_field_element_from_map("nu", "sigma_" + std::to_string(key->program_width)));
    }

    return alpha_base * alpha_step.sqr() * alpha_step;
}

template class VerifierPLookupWidget<barretenberg::fr,
                                     barretenberg::g1::affine_element,
                                     transcript::StandardTranscript>;

} // namespace waffle