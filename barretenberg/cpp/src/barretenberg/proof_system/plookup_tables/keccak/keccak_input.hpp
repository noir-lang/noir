#pragma once

#include "../types.hpp"
#include "barretenberg/common/constexpr_utils.hpp"
#include "barretenberg/numeric/bitop/pow.hpp"
#include "barretenberg/numeric/bitop/sparse_form.hpp"

namespace plookup {
namespace keccak_tables {

/**
 * @brief Generates plookup tables used convert 64-bit integers into a sparse representation used for Keccak hash
 * algorithm
 *
 * Keccak has 25 hash lanes, each represented as 64-bit integers.
 *
 * We evaluate in-circuit using a base-11 sparse integer representation for each lane:
 *
 * P = \sum_{j=0}^63 b_i * 11^i
 *
 * KeccakInput defines the plookup table that maps binary integer slices into base-11 integer slices.
 *
 * In addition, KeccakInput also is used to determine the value of the most significant (63rd) bit of the input
 * (which is used by stdlib::keccak to more efficiently left-rotate by 1 bit)
 */
class KeccakInput {

  public:
    static constexpr uint64_t BASE = 11;
    static constexpr size_t TABLE_BITS = 8;

    /**
     * @brief Given a table input value, return the table output value
     *
     * Used by the Plookup code to precompute lookup tables and generate witness values
     *
     * @param key (first element = table input. Second element is unused as this lookup does not have 2 keys per value)
     * @return std::array<bb::fr, 2> table output
     */
    static std::array<bb::fr, 2> get_keccak_input_values(const std::array<uint64_t, 2> key)
    {
        const uint256_t t0 = numeric::map_into_sparse_form<BASE>(key[0]);

        constexpr size_t msb_shift = (64 % TABLE_BITS == 0) ? TABLE_BITS - 1 : (64 % TABLE_BITS) - 1;
        const uint256_t t1 = key[0] >> msb_shift;
        return { bb::fr(t0), bb::fr(t1) };
    }

    /**
     * @brief Generate plookup table that maps a TABLE_BITS-slice of a base-2 integer into a base-11 representation
     *
     * @param id
     * @param table_index
     * @return BasicTable
     */
    static BasicTable generate_keccak_input_table(BasicTableId id, const size_t table_index)
    {
        BasicTable table;
        table.id = id;
        table.table_index = table_index;
        table.size = (1U << TABLE_BITS);
        table.use_twin_keys = false;
        constexpr size_t msb_shift = (64 % TABLE_BITS == 0) ? TABLE_BITS - 1 : (64 % TABLE_BITS) - 1;

        for (uint64_t i = 0; i < table.size; ++i) {
            const uint64_t source = i;
            const auto target = numeric::map_into_sparse_form<BASE>(source);
            table.column_1.emplace_back(bb::fr(source));
            table.column_2.emplace_back(bb::fr(target));
            table.column_3.emplace_back(bb::fr(source >> msb_shift));
        }

        table.get_values_from_key = &get_keccak_input_values;

        uint256_t sparse_step_size = 1;
        for (size_t i = 0; i < TABLE_BITS; ++i) {
            sparse_step_size *= BASE;
        }
        table.column_1_step_size = bb::fr((1 << TABLE_BITS));
        table.column_2_step_size = bb::fr(sparse_step_size);
        table.column_3_step_size = bb::fr(sparse_step_size);

        return table;
    }

    /**
     * @brief Create the KeccakInput MultiTable used by plookup to generate a sequence of lookups
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
     * Instead, e.g. for TABLE_BITS = 8 we have inputs A, B, C where
     *      A = \sum_{i=0}^7 A_i * 2^8
     *      B = \sum_{i=0}^7 B_i * 11^8
     *      C_i = B_i >> 7 (to get the most significant bit of B)
     *
     * Our plookup gates will produce a gates with the following wire values:
     *
     * | W1                      | W2                      | W3  |
     * | ----------------------- | ----------------------- | --- |
     * | \sum_{i=0}^7 A_i * 2^i  | \sum_{i=0}^7 B_i * 11^i | C_0 |
     * | \sum_{i=1}^7 A_i * 2^i  | \sum_{i=1}^7 B_i * 11^i | C_1 |
     * | \sum_{i=2}^7 A_i * 2^i  | \sum_{i=2}^7 B_i * 11^i | C_2 |
     * | ...                     | ...                     | ... |
     * | A^7                     | B^7                     | C^7 |
     *
     * The plookup protocol extracts the 1st and 2nd lookup column values by taking:
     *
     *      Colunn1 = W1[i] - 2^8 . W1[i + 1]
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
    static MultiTable get_keccak_input_table(const MultiTableId id = KECCAK_FORMAT_INPUT)
    {
        const size_t num_entries = 8;

        MultiTable table(1 << 8, uint256_t(11).pow(8), 0, num_entries);

        table.id = id;
        for (size_t i = 0; i < num_entries; ++i) {
            table.slice_sizes.emplace_back(1 << 8);
            table.lookup_ids.emplace_back(KECCAK_INPUT);
            table.get_table_values.emplace_back(&get_keccak_input_values);
        }
        return table;
    }
};

} // namespace keccak_tables
} // namespace plookup
