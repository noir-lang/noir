#pragma once

#include "../types.hpp"
#include "barretenberg/common/constexpr_utils.hpp"
#include "barretenberg/numeric/bitop/pow.hpp"

namespace plookup {
namespace keccak_tables {

/**
 * @brief Generates plookup tables required for CHI round of Keccak hash function
 *
 * Keccak has 25 hash lanes, each represented as 64-bit integers. The CHI round performs the following operation on 3
 * hash lanes:
 *
 * A ^ (~B & C)
 *
 * We evaluate in-circuit using a base-11 sparse integer representation:
 *
 * P = \sum_{j=0}^63 b_i * 11^i
 *
 * In this representation we evaluate CHI via the linear expression
 *
 * 2.A - B + C + Q
 *
 * Where Q is the precomputed constant \sum_{i=0}^63 11^i
 *
 * This can be represented via the 'truth table' for each base-11 quasi-bit:
 *
 * | A | B | C | Algebraic Output |
 * | - | - | - | ---------------- |
 * | 0 | 0 | 0 | 1                |
 * | 0 | 0 | 1 | 2                |
 * | 0 | 1 | 0 | 0                |
 * | 1 | 0 | 0 | 3                |
 * | 1 | 0 | 1 | 4                |
 * | 1 | 1 | 0 | 2                |
 * | 1 | 1 | 1 | 3                |
 *
 * CHI round uses a plookup table that normalizes the algebraic output back into the binary output.
 *
 * | Algebraic Output | Binary Output |
 * | ---------------- | ------------- |
 * | 0                | 0             |
 * | 1                | 0             |
 * | 2                | 1             |
 * | 3                | 1             |
 * | 4                | 0             |
 *
 * In addition we also use the CHI lookup table to determine the value of the most significant (63rd) bit of the output
 *
 * for all M in [0, ..., TABLE_BITS - 1] and K in [0, 1, 2, 3, 4], the column values of our lookup table are:
 *
 * Column1 value = \sum_{i \in M} \sum_{j \in K} 11^i * j]
 * Column2 value = \sum_{i \in M} \sum_{j \in K} 11^i * CHI_NORMALIZATION_TABLE[j]]
 * Column3 value = Column2 / 11^8
 *
 */
class Chi {
  public:
    // 1 + 2a - b + c => a xor (~b & c)
    static constexpr uint64_t CHI_NORMALIZATION_TABLE[5]{
        0, 0, 1, 1, 0,
    };

    static constexpr uint64_t BASE = 11;

    // effective base = maximum value each input 'quasi-bit' can reach at this stage of the Keccak algo
    // (we use base11 as it's a bit more efficient to use a consistent base across the entire Keccak hash algorithm.
    //  The THETA round requires base-11 in order to most efficiently convert XOR operations into algebraic operations)
    static constexpr uint64_t EFFECTIVE_BASE = 5;

    // TABLE_BITS defines table size. More bits = fewer lookups but larger tables!
    static constexpr uint64_t TABLE_BITS = 6;

    /**
     * @brief Given a table input value, return the table output value
     *
     * Used by the Plookup code to precompute lookup tables and generate witness values
     *
     * @param key (first element = table input. Second element is unused as this lookup does not have 2 keys per value)
     * @return std::array<bb::fr, 2> table output (normalized input and normalized input / 11^8)
     */
    static std::array<bb::fr, 2> get_chi_renormalization_values(const std::array<uint64_t, 2> key)
    {
        uint64_t accumulator = 0;
        uint64_t input = key[0];
        uint64_t base_shift = 1;
        constexpr uint64_t divisor_exponent = (64 % TABLE_BITS == 0) ? (TABLE_BITS - 1) : (64 % TABLE_BITS) - 1;
        constexpr uint64_t divisor = numeric::pow64(BASE, divisor_exponent);
        while (input > 0) {
            uint64_t slice = input % BASE;
            uint64_t bit = CHI_NORMALIZATION_TABLE[static_cast<size_t>(slice)];
            accumulator += (bit * base_shift);
            input /= BASE;
            base_shift *= BASE;
        }

        return { bb::fr(accumulator), bb::fr(accumulator / divisor) };
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
     * (a bit tricky to compute because each quasi-bit ranges from [0, 1, 2, 3, 4],
     *  but we're working with base-11 numbers.
     *  i.e. unlike most of our lookup tables, the 1st column is not uniformly increasing by a constant value!)
     *
     * @param counts The current row value represented as an array of quasi-bits
     * @return std::array<uint64_t, 3> the columns of the plookup table
     */
    static std::array<uint64_t, 3> get_column_values_for_next_row(std::array<size_t, TABLE_BITS>& counts)
    {
        static constexpr auto scaled_bases = get_scaled_bases();

        // want the most significant bit of the 64-bit integer, when this table is used on the most significant slice
        constexpr uint64_t divisor_exponent = (64 % TABLE_BITS == 0) ? (TABLE_BITS - 1) : (64 % TABLE_BITS) - 1;
        constexpr uint64_t divisor = numeric::pow64(static_cast<uint64_t>(BASE), divisor_exponent);

        for (size_t i = 0; i < TABLE_BITS; ++i) {
            if (counts[i] == EFFECTIVE_BASE - 1) {
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
            normalized_value += (CHI_NORMALIZATION_TABLE[counts[i]]) * scaled_bases[i];
        }
        return { value, normalized_value, normalized_value / divisor };
    }

    /**
     * @brief Generate the CHI plookup table
     *
     * This table is used by Composer objects to generate plookup constraints
     *
     * @param id a compile-time ID defined via plookup_tables.hpp
     * @param table_index a circuit-specific ID for the table used by the circuit Composer
     * @return BasicTable
     */
    static BasicTable generate_chi_renormalization_table(BasicTableId id, const size_t table_index)
    {
        BasicTable table;
        table.id = id;
        table.table_index = table_index;
        table.use_twin_keys = false;
        table.size = numeric::pow64(static_cast<uint64_t>(EFFECTIVE_BASE), TABLE_BITS);

        std::array<size_t, TABLE_BITS> counts{};
        std::array<uint64_t, 3> column_values{ 0, 0, 0 };
        for (size_t i = 0; i < table.size; ++i) {
            table.column_1.emplace_back(column_values[0]);
            table.column_2.emplace_back(column_values[1]);
            table.column_3.emplace_back(column_values[2]);
            column_values = get_column_values_for_next_row(counts);
        }

        table.get_values_from_key = &get_chi_renormalization_values;

        constexpr uint64_t step_size = numeric::pow64(static_cast<uint64_t>(BASE), TABLE_BITS);
        table.column_1_step_size = bb::fr(step_size);
        table.column_2_step_size = bb::fr(step_size);
        table.column_3_step_size = bb::fr(0);
        return table;
    }

    /**
     * @brief Create the CHI MultiTable used by plookup to generate a sequence of lookups
     *
     * The CHI round operates on 64-bit integers, but the lookup table only indexes TABLE_BITS bits.
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
     * | \sum_{i=0}^7 A_i * 11^i | \sum_{i=0}^7 B_i * 11^i | C_0 |
     * | \sum_{i=1}^7 A_i * 11^i | \sum_{i=1}^7 B_i * 11^i | C_1 |
     * | \sum_{i=2}^7 A_i * 11^i | \sum_{i=2}^7 B_i * 11^i | C_2 |
     * | ...                     | ...                     | ... |
     * | A^7                     | B^7                     | C^7 |
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
    static MultiTable get_chi_output_table(const MultiTableId id = KECCAK_CHI_OUTPUT)
    {
        constexpr size_t num_tables_per_multitable =
            (64 / TABLE_BITS) + (64 % TABLE_BITS == 0 ? 0 : 1); // 64 bits, 8 bits per entry

        // column_multiplier is used to define the gap between rows when deriving colunn values via relative differences
        uint64_t column_multiplier = numeric::pow64(BASE, TABLE_BITS);
        MultiTable table(column_multiplier, column_multiplier, 0, num_tables_per_multitable);

        table.id = id;
        for (size_t i = 0; i < num_tables_per_multitable; ++i) {
            table.slice_sizes.emplace_back(numeric::pow64(BASE, TABLE_BITS));
            table.lookup_ids.emplace_back(KECCAK_CHI);
            table.get_table_values.emplace_back(&get_chi_renormalization_values);
        }
        return table;
    }
};
} // namespace keccak_tables
} // namespace plookup
