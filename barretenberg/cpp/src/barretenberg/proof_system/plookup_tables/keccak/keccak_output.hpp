#pragma once

#include "barretenberg/common/constexpr_utils.hpp"
#include "barretenberg/numeric/bitop/pow.hpp"
#include "barretenberg/numeric/bitop/sparse_form.hpp"

#include "../sparse.hpp"
#include "../types.hpp"

namespace plookup {
namespace keccak_tables {

/**
 * @brief Converts a base-11 sparse integer representation into a regular base-2 binary integer.
 *        Used by the Keccak hash algorithm to convert the output of the algorithm into a regular integer.
 */
class KeccakOutput {

  public:
    static constexpr uint64_t BASE = 11;

    // effective base = maximum value each input 'quasi-bit' can reach
    // (the input is in base-11 representation, but at this point in the Keccak algorithm each 'quasi-bit' can only
    // take values [0, 1] not [0, ..., 10]
    static constexpr uint64_t EFFECTIVE_BASE = 2;
    static constexpr size_t TABLE_BITS = 8;

    static constexpr uint64_t OUTPUT_NORMALIZATION_TABLE[2]{ 0, 1 };

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
     * (a bit tricky to compute because each quasi-bit ranges from [0, 1],
     *  but we're working with base-11 numbers.
     *  i.e. unlike most of our lookup tables, the 1st column is not uniformly increasing by a constant value!)
     *
     * @param counts The current row value represented as an array of quasi-bits
     * @return std::array<uint64_t, uint64_t> first and second columns of plookup table (3rd column is 0)
     */
    static std::array<uint64_t, 2> get_column_values_for_next_row(std::array<size_t, TABLE_BITS>& counts)
    {
        static constexpr auto scaled_bases = get_scaled_bases();

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
            normalized_value += (OUTPUT_NORMALIZATION_TABLE[counts[i]]) << i;
        }
        return { value, normalized_value };
    }

    /**
     * @brief Generate plookup table that maps a TABLE_BITS-slice of a base-11 integer into a base-2 integer
     *
     * @param id
     * @param table_index
     * @return BasicTable
     */
    static BasicTable generate_keccak_output_table(BasicTableId id, const size_t table_index)
    {
        BasicTable table;
        table.id = id;
        table.table_index = table_index;
        table.use_twin_keys = false;
        table.size = numeric::pow64(static_cast<uint64_t>(EFFECTIVE_BASE), TABLE_BITS);

        std::array<size_t, TABLE_BITS> counts{};
        std::array<uint64_t, 2> column_values{ 0, 0 };

        for (size_t i = 0; i < table.size; ++i) {
            table.column_1.emplace_back(column_values[0]);
            table.column_2.emplace_back(column_values[1]);
            table.column_3.emplace_back(0);
            column_values = get_column_values_for_next_row(counts);
        }

        table.get_values_from_key = &sparse_tables::get_sparse_normalization_values<BASE, OUTPUT_NORMALIZATION_TABLE>;

        table.column_1_step_size = bb::fr(numeric::pow64(static_cast<size_t>(BASE), TABLE_BITS));
        table.column_2_step_size = bb::fr(((uint64_t)1 << TABLE_BITS));
        table.column_3_step_size = 0;
        return table;
    }

    /**
     * @brief Create the KeccakOutput MultiTable used by plookup to generate a sequence of lookups
     *
     * Keccak operates on 64-bit integers, but the lookup table only indexes TABLE_BITS bits.
     *
     * i.e. multiple lookups are required for a single 64-bit integer.
     *
     * If we group these lookups together, we can derive the plookup column values
     * from the relative difference between wire values.
     *
     * i.e. we do not need to split our 64-bit input into TABLE_BITS slices, perform the lookup and add together the
     * output slices
     *
     * Instead, e.g. for TABLE_BITS = 8 we have inputs A, B where
     *      A = \sum_{i=0}^7 A_i * 11^8
     *      B = \sum_{i=0}^7 B_i * 2^8
     *
     * Our plookup gates will produce a gates with the following wire values:
     *
     * | W1                      | W2                      | W3  |
     * | ----------------------- | ----------------------- | --- |
     * | \sum_{i=0}^7 A_i * 2^i  | \sum_{i=0}^7 B_i * 11^i | 0   |
     * | \sum_{i=1}^7 A_i * 2^i  | \sum_{i=1}^7 B_i * 11^i | 0   |
     * | \sum_{i=2}^7 A_i * 2^i  | \sum_{i=2}^7 B_i * 11^i | 0   |
     * | ...                     | ...                     | ... |
     * | A^7                     | B^7                     | 0   |
     *
     * The plookup protocol extracts the 1st and 2nd lookup column values by taking:
     *
     *      Colunn1 = W1[i] - 11^8 . W1[i + 1]
     *      Colunn2 = W2[i] - 2^8 . W2[i + 1]
     *
     * (where the -11^8, -2^8 coefficients are stored in a precomputed selector polynomial)
     *
     * This MultiTable construction defines the value of these precomputed selector polynomial values,
     * as well as defines how the column values are derived from a starting input value.
     *
     * @param id
     * @return MultiTable
     */
    static MultiTable get_keccak_output_table(const MultiTableId id = KECCAK_FORMAT_OUTPUT)
    {
        constexpr size_t num_tables_per_multitable =
            64 / TABLE_BITS + (64 % TABLE_BITS == 0 ? 0 : 1); // 64 bits, 8 bits per entry

        uint64_t column_multiplier = numeric::pow64(BASE, TABLE_BITS);
        MultiTable table(column_multiplier, (1ULL << TABLE_BITS), 0, num_tables_per_multitable);

        table.id = id;
        for (size_t i = 0; i < num_tables_per_multitable; ++i) {
            table.slice_sizes.emplace_back(numeric::pow64(BASE, TABLE_BITS));
            table.lookup_ids.emplace_back(KECCAK_OUTPUT);
            table.get_table_values.emplace_back(
                &sparse_tables::get_sparse_normalization_values<BASE, OUTPUT_NORMALIZATION_TABLE>);
        }
        return table;
    }
};

} // namespace keccak_tables
} // namespace plookup
