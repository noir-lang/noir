#pragma once

#include "../types.hpp"
#include "barretenberg/common/constexpr_utils.hpp"
#include "barretenberg/numeric/bitop/pow.hpp"

namespace plookup {
namespace keccak_tables {

/**
 * @brief Generates plookup tables required for THETA round of Keccak hash function
 *
 * Keccak has 25 hash lanes, each represented as 64-bit integers. The THETA round performs the following operation on 3
 * hash lanes:
 *
 * C0 = A0 ^ A1 ^ A2 ^ A3 ^ A4
 * C1 = B0 ^ B1 ^ B2 ^ B3 ^ B4
 * D  = C0 ^ ROTATE_LEFT(C1, 1)
 *
 * We evaluate in-circuit using a base-11 sparse integer representation:
 *
 * P = \sum_{j=0}^63 b_i * 11^i
 *
 * In this representation we evaluate CHI via the linear expression
 *
 * C0 = A0 + A1 + A2 + A3 + A4
 * C1 = B0 + B1 + B2 + B3 + B4
 * D  = C0 + ROTATE_LEFT(C1, 1)
 *
 * We use base-11 spare representation because the maximum value of each 'quasi-bit' of D is 10
 *
 * THETA round uses a plookup table that normalizes the algebraic output.
 *
 * This can be represented via the 'truth table' for each base-11 quasi-bit:
 *
 * | Algebraic Output | Binary Output |
 * | ---------------- | ------------- |
 * | 0                | 0             |
 * | 1                | 1             |
 * | 2                | 0             |
 * | 3                | 1             |
 * | 4                | 0             |
 * | 5                | 1             |
 * | 6                | 0             |
 * | 7                | 1             |
 * | 8                | 0             |
 * | 9                | 1             |
 * | 10               | 0             |
 *
 * i.e. even = 0, odd = 1
 *
 */
class Theta {
  public:
    static constexpr size_t TABLE_BITS = 4;
    static constexpr uint64_t BASE = 11;

    static constexpr uint64_t THETA_NORMALIZATION_TABLE[11]{
        0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
    };

    // template <size_t i> static std::pair<uint64_t, uint64_t> update_counts(std::array<size_t, TABLE_BITS>& counts)
    // {
    //     ASSERT(i <= TABLE_BITS);
    //     if constexpr (i >= TABLE_BITS) {
    //         // TODO use concepts or template metaprogramming to put this condition in method declaration
    //         return std::make_pair(0, 0);
    //     } else {
    //         if (counts[i] == BASE - 1) {
    //             counts[i] = 0;
    //             return update_counts<i + 1>(counts);
    //         } else {
    //             counts[i] += 1;
    //         }

    //         uint64_t value = 0;
    //         uint64_t normalized_value = 0;
    //         uint64_t cumulative_base = 1;
    //         for (size_t j = 0; j < TABLE_BITS; ++j) {
    //             value += counts[j] * cumulative_base;
    //             normalized_value += (THETA_NORMALIZATION_TABLE[counts[j]]) * cumulative_base;
    //             cumulative_base *= BASE;
    //         }
    //         return std::make_pair(value, normalized_value);
    //     }
    // }

    /**
     * @brief Given a table input value, return the table output value
     *
     * Used by the Plookup code to precompute lookup tables and generate witness values
     *
     * @param key (first element = table input. Second element is unused as this lookup does not have 2 keys per value)
     * @return std::array<bb::fr, 2> table output (normalized input and normalized input / 11^TABLE_BITS - 1)
     */
    static std::array<bb::fr, 2> get_theta_renormalization_values(const std::array<uint64_t, 2> key)
    {
        uint64_t accumulator = 0;
        uint64_t input = key[0];
        uint64_t base_shift = 1;
        while (input > 0) {
            uint64_t slice = input % BASE;
            uint64_t bit = THETA_NORMALIZATION_TABLE[static_cast<size_t>(slice)];
            accumulator += (bit * base_shift);
            input /= BASE;
            base_shift *= BASE;
        }
        return { bb::fr(accumulator), bb::fr(0) };
    }

    /**
     * @brief Precompute an array of base multipliers (11^i for i = [0, ..., TABLE_BITS - 1])
     * Code is slightly faster at runtime if we compute this at compile time
     *
     * @return constexpr std::array<uint64_t, TABLE_BITS>
     */
    static constexpr std::array<uint64_t, TABLE_BITS> get_scaled_bases()
    {
        std::array<uint64_t, TABLE_BITS> result;
        uint64_t acc = 1;
        for (size_t i = 0; i < TABLE_BITS; ++i) {
            result[i] = acc;
            acc *= BASE;
        }
        return result;
    }

    /**
     * @brief Get column values for next row of plookup table. Used to generate plookup table row values
     *
     * Input `counts` is an array of quasi-bits that represent the current row.
     * Method increases `counts` by 1 and returns the plookup table column values.
     *
     * @param counts The current row value represented as an array of quasi-bits
     * @return std::array<uint64_t, 2> the columns of the plookup table
     */
    static std::array<uint64_t, 2> get_column_values_for_next_row(std::array<size_t, TABLE_BITS>& counts)
    {
        static constexpr auto scaled_bases = get_scaled_bases();

        for (size_t i = 0; i < TABLE_BITS; ++i) {
            if (counts[i] == BASE - 1) {
                counts[i] = 0;
            } else {
                counts[i] += 1;
                break;
            }
        }

        uint64_t value = 0;
        uint64_t normalized_value = 0;
        for (size_t i = 0; i < TABLE_BITS; ++i) {
            value += counts[i] * scaled_bases[i];
            normalized_value += (THETA_NORMALIZATION_TABLE[counts[i]]) * scaled_bases[i];
        }
        return { value, normalized_value };
    }

    /**
     * @brief Generate plookup table that normalizes a TABLE_BITS-slice of a base-11 integer
     *
     * @param id
     * @param table_index
     * @return BasicTable
     */
    static BasicTable generate_theta_renormalization_table(BasicTableId id, const size_t table_index)
    {
        // max_base_value_plus_one sometimes may not equal base iff this is an intermediate lookup table
        // (e.g. keccak, we have base11 values that need to be normalized where the actual values-per-base only range
        // from [0, 1, 2])
        BasicTable table;
        table.id = id;
        table.table_index = table_index;
        table.use_twin_keys = false;
        table.size = numeric::pow64(static_cast<uint64_t>(BASE), TABLE_BITS);

        std::array<size_t, TABLE_BITS> counts{};
        std::array<uint64_t, 2> column_values{ 0, 0 };

        for (size_t i = 0; i < table.size; ++i) {
            table.column_1.emplace_back(column_values[0]);
            table.column_2.emplace_back(column_values[1]);
            table.column_3.emplace_back(0);
            column_values = get_column_values_for_next_row(counts);
        }

        table.get_values_from_key = &get_theta_renormalization_values;

        constexpr uint64_t step_size = numeric::pow64(static_cast<uint64_t>(BASE), TABLE_BITS);
        table.column_1_step_size = bb::fr(step_size);
        table.column_2_step_size = bb::fr(step_size);
        table.column_3_step_size = bb::fr(0);
        return table;
    }

    /**
     * @brief Create the THETA MultiTable used by plookup to generate a sequence of lookups
     *
     * The THETA round operates on 64-bit integers, but the lookup table only indexes TABLE_BITS bits.
     *
     * i.e. multiple lookups are required for a single 64-bit integer.
     *
     * If we group these lookups together, we can derive the plookup column values
     * from the relative difference between wire values.
     *
     * i.e. we do not need to split our 64-bit input into TABLE_BITS slices, perform the lookup and add together the
     * output slices
     *
     * Instead, e.g. for TABLE_BITS = 8 we have inputs A, B, C where
     *      A = \sum_{i=0}^7 A_i * 11^8
     *      B = \sum_{i=0}^7 B_i * 11^8
     *      C_i = B_i / 11^8 (to get the most significant bit of B)
     *
     * Our plookup gates will produce a gates with the following wire values:
     *
     * | W1                      | W2                      | W3  |
     * | ----------------------- | ----------------------- | --- |
     * | \sum_{i=0}^7 A_i * 11^i | \sum_{i=0}^7 B_i * 11^i | --- |
     * | \sum_{i=1}^7 A_i * 11^i | \sum_{i=1}^7 B_i * 11^i | --- |
     * | \sum_{i=2}^7 A_i * 11^i | \sum_{i=2}^7 B_i * 11^i | --- |
     * | ...                     | ...                     | ... |
     * | A^7                     | B^7                     | --- |
     *
     * The plookup protocol extracts the 1st and 2nd lookup column values by taking:
     *
     *      Colunn1 = W1[i] - 11^8 . W1[i + 1]
     *      Colunn2 = W2[i] - 11^8 . W2[i + 1]
     *
     * (where the -11^8 coefficient is stored in a precomputed selector polynomial)
     *
     * This MultiTable construction defines the value of these precomputed selector polynomial values,
     * as well as defines how the column values are derived from a starting input value.
     *
     * @param id
     * @return MultiTable
     */
    static MultiTable get_theta_output_table(const MultiTableId id = KECCAK_THETA_OUTPUT)
    {
        constexpr size_t num_tables_per_multitable =
            (64 / TABLE_BITS) + (64 % TABLE_BITS == 0 ? 0 : 1); // 64 bits, 5 bits per entry

        uint64_t column_multiplier = numeric::pow64(BASE, TABLE_BITS);
        MultiTable table(column_multiplier, column_multiplier, 0, num_tables_per_multitable);

        table.id = id;
        for (size_t i = 0; i < num_tables_per_multitable; ++i) {
            table.slice_sizes.emplace_back(numeric::pow64(BASE, TABLE_BITS));
            table.lookup_ids.emplace_back(KECCAK_THETA);
            table.get_table_values.emplace_back(&get_theta_renormalization_values);
        }
        return table;
    }
};
} // namespace keccak_tables
} // namespace plookup
