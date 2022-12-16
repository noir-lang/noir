#pragma once

#include <numeric/bitop/rotate.hpp>

#include "types.hpp"
#include "sparse.hpp"

namespace plookup {
namespace blake2s_tables {

static constexpr size_t BITS_IN_LAST_SLICE = 5UL;
static constexpr size_t SIZE_OF_LAST_SLICE = (1UL << BITS_IN_LAST_SLICE);

/**
 * This functions performs the operation ROTR^{k}(a ^ b) when filter is false and
 * ROTR^{k}((a % 4) ^ (a % 4)) when filter is true. In other words, (filter = true) implies
 * that the XOR operation works only on the two least significant bits.
 */
template <uint64_t bits_per_slice, uint64_t num_rotated_output_bits, bool filter = false>
inline std::array<barretenberg::fr, 2> get_xor_rotate_values_from_key(const std::array<uint64_t, 2> key)
{
    uint64_t filtered_key0 = filter ? key[0] & 3ULL : key[0];
    uint64_t filtered_key1 = filter ? key[1] & 3ULL : key[1];
    return { uint256_t(numeric::rotate32(uint32_t(filtered_key0) ^ uint32_t(filtered_key1),
                                         uint32_t(num_rotated_output_bits))),
             0ULL };
}

/**
 * Generates a basic 32-bit (XOR + ROTR) lookup table.
 */
template <uint64_t bits_per_slice, uint64_t num_rotated_output_bits, bool filter = false>
inline BasicTable generate_xor_rotate_table(BasicTableId id, const size_t table_index)
{
    const uint64_t base = 1UL << bits_per_slice;
    BasicTable table;
    table.id = id;
    table.table_index = table_index;
    table.size = base * base;
    table.use_twin_keys = true;

    for (uint64_t i = 0; i < base; ++i) {
        for (uint64_t j = 0; j < base; ++j) {
            table.column_1.emplace_back(i);
            table.column_2.emplace_back(j);
            uint64_t i_copy = i;
            uint64_t j_copy = j;
            if (filter) {
                i_copy &= 3ULL;
                j_copy &= 3ULL;
            }
            table.column_3.emplace_back(
                uint256_t(numeric::rotate32(uint32_t(i_copy) ^ uint32_t(j_copy), uint32_t(num_rotated_output_bits))));
        }
    }

    table.get_values_from_key = &get_xor_rotate_values_from_key<bits_per_slice, num_rotated_output_bits, filter>;

    table.column_1_step_size = base;
    table.column_2_step_size = base;
    table.column_3_step_size = base;

    return table;
}

/**
 * Generates a multi-lookup-table with 5 slices for 32-bit XOR operation (a ^ b).
 *
 * Details:
 *
 * The following table summarizes the shifts required for each slice for different operations.
 * We need to ensure that the coefficient of s0 always is 1, so we need adjust other coefficients
 * accordingly. For example, the coefficient of slice s4 for ROTR_16 should be set to
 * (2^8 / 2^{16}) = 2^{-8}.
 *
 * -----------------------------------------------
 * | Slice | ROTR_16 | ROTR_12 | ROTR_8 | ROTR_7 |
 * |-------|---------|---------|--------|--------|
 * | s0    | 16      | 20      | 24     | 25     |
 * | s1    | 22      | 26      | 0      | 0      |
 * | s2    | 0       | 0       | 4      | 5      |
 * | s3    | 2       | 6       | 10     | 11     |
 * | s4    | 8       | 12      | 16     | 17     |
 * | s5    | 14      | 18      | 22     | 23     |
 * -----------------------------------------------
 *
 * We don't need to have a separate table for ROTR_12 as its output can be derived from an XOR table.
 * Thus, we have a blake2s_xor_table function below.
 */
inline MultiTable get_blake2s_xor_table(const MultiTableId id = BLAKE_XOR)
{
    const size_t num_entries = (32 + 2) / 6 + 1;
    const uint64_t base = 1 << 6;
    MultiTable table(base, base, base, num_entries);

    table.id = id;
    for (size_t i = 0; i < num_entries - 1; ++i) {
        table.slice_sizes.emplace_back(base);
        table.lookup_ids.emplace_back(BLAKE_XOR_ROTATE0);
        table.get_table_values.emplace_back(&get_xor_rotate_values_from_key<6, 0>);
    }

    table.slice_sizes.emplace_back(SIZE_OF_LAST_SLICE);
    table.lookup_ids.emplace_back(BLAKE_XOR_ROTATE0_SLICE5_MOD4);
    table.get_table_values.emplace_back(&get_xor_rotate_values_from_key<BITS_IN_LAST_SLICE, 0, true>);

    return table;
}

/**
 * Generates a multi-lookup-table with 5 slices for 32-bit operation ROTR^{16}(a ^ b).
 */
inline MultiTable get_blake2s_xor_rotate_16_table(const MultiTableId id = BLAKE_XOR_ROTATE_16)
{
    const uint64_t base = 1 << 6;
    constexpr barretenberg::fr coefficient_16 = barretenberg::fr(1) / barretenberg::fr(1 << 16);

    std::vector<barretenberg::fr> column_1_coefficients{ barretenberg::fr(1),       barretenberg::fr(1 << 6),
                                                         barretenberg::fr(1 << 12), barretenberg::fr(1 << 18),
                                                         barretenberg::fr(1 << 24), barretenberg::fr(1 << 30) };

    std::vector<barretenberg::fr> column_3_coefficients{ barretenberg::fr(1),
                                                         barretenberg::fr(1 << 6),
                                                         coefficient_16,
                                                         coefficient_16 * barretenberg::fr(1 << 2),
                                                         coefficient_16 * barretenberg::fr(1 << 8),
                                                         coefficient_16 * barretenberg::fr(1 << 14) };

    MultiTable table(column_1_coefficients, column_1_coefficients, column_3_coefficients);

    table.id = id;
    table.slice_sizes = { base, base, base, base, base, SIZE_OF_LAST_SLICE };
    table.lookup_ids = { BLAKE_XOR_ROTATE0, BLAKE_XOR_ROTATE0, BLAKE_XOR_ROTATE4,
                         BLAKE_XOR_ROTATE0, BLAKE_XOR_ROTATE0, BLAKE_XOR_ROTATE0_SLICE5_MOD4 };

    table.get_table_values.emplace_back(&get_xor_rotate_values_from_key<6, 0>);
    table.get_table_values.emplace_back(&get_xor_rotate_values_from_key<6, 0>);
    table.get_table_values.emplace_back(&get_xor_rotate_values_from_key<6, 4>);
    table.get_table_values.emplace_back(&get_xor_rotate_values_from_key<6, 0>);
    table.get_table_values.emplace_back(&get_xor_rotate_values_from_key<6, 0>);
    table.get_table_values.emplace_back(&get_xor_rotate_values_from_key<BITS_IN_LAST_SLICE, 0, true>);

    return table;
}

/**
 * Generates a multi-lookup-table with 5 slices for 32-bit operation ROTR^{8}(a ^ b).
 */
inline MultiTable get_blake2s_xor_rotate_8_table(const MultiTableId id = BLAKE_XOR_ROTATE_8)
{
    const uint64_t base = 1 << 6;
    constexpr barretenberg::fr coefficient_24 = barretenberg::fr(1) / barretenberg::fr(1 << 24);

    std::vector<barretenberg::fr> column_1_coefficients{ barretenberg::fr(1),       barretenberg::fr(1 << 6),
                                                         barretenberg::fr(1 << 12), barretenberg::fr(1 << 18),
                                                         barretenberg::fr(1 << 24), barretenberg::fr(1 << 30) };

    std::vector<barretenberg::fr> column_3_coefficients{ barretenberg::fr(1),
                                                         coefficient_24,
                                                         coefficient_24 * barretenberg::fr(1 << 4),
                                                         coefficient_24 * barretenberg::fr(1 << (4 + 6)),
                                                         coefficient_24 * barretenberg::fr(1 << (4 + 12)),
                                                         coefficient_24 * barretenberg::fr(1 << (4 + 18)) };

    MultiTable table(column_1_coefficients, column_1_coefficients, column_3_coefficients);

    table.id = id;
    table.slice_sizes = { base, base, base, base, base, SIZE_OF_LAST_SLICE };
    table.lookup_ids = { BLAKE_XOR_ROTATE0, BLAKE_XOR_ROTATE2, BLAKE_XOR_ROTATE0,
                         BLAKE_XOR_ROTATE0, BLAKE_XOR_ROTATE0, BLAKE_XOR_ROTATE0_SLICE5_MOD4 };

    table.get_table_values.emplace_back(&get_xor_rotate_values_from_key<6, 0>);
    table.get_table_values.emplace_back(&get_xor_rotate_values_from_key<6, 2>);
    table.get_table_values.emplace_back(&get_xor_rotate_values_from_key<6, 0>);
    table.get_table_values.emplace_back(&get_xor_rotate_values_from_key<6, 0>);
    table.get_table_values.emplace_back(&get_xor_rotate_values_from_key<6, 0>);
    table.get_table_values.emplace_back(&get_xor_rotate_values_from_key<BITS_IN_LAST_SLICE, 0, true>);

    return table;
}

/**
 * Generates a multi-lookup-table with 5 slices for 32-bit operation ROTR^{7}(a ^ b).
 */
inline MultiTable get_blake2s_xor_rotate_7_table(const MultiTableId id = BLAKE_XOR_ROTATE_7)
{
    const uint64_t base = 1 << 6;
    constexpr barretenberg::fr coefficient_25 = barretenberg::fr(1) / barretenberg::fr(1 << 25);

    std::vector<barretenberg::fr> column_1_coefficients{ barretenberg::fr(1),       barretenberg::fr(1 << 6),
                                                         barretenberg::fr(1 << 12), barretenberg::fr(1 << 18),
                                                         barretenberg::fr(1 << 24), barretenberg::fr(1 << 30) };

    std::vector<barretenberg::fr> column_3_coefficients{ barretenberg::fr(1),
                                                         coefficient_25,
                                                         coefficient_25 * barretenberg::fr(1 << 5),
                                                         coefficient_25 * barretenberg::fr(1 << (5 + 6)),
                                                         coefficient_25 * barretenberg::fr(1 << (5 + 12)),
                                                         coefficient_25 * barretenberg::fr(1 << (5 + 18)) };

    MultiTable table(column_1_coefficients, column_1_coefficients, column_3_coefficients);

    table.id = id;
    table.slice_sizes = { base, base, base, base, base, SIZE_OF_LAST_SLICE };
    table.lookup_ids = { BLAKE_XOR_ROTATE0, BLAKE_XOR_ROTATE1, BLAKE_XOR_ROTATE0,
                         BLAKE_XOR_ROTATE0, BLAKE_XOR_ROTATE0, BLAKE_XOR_ROTATE0_SLICE5_MOD4 };

    table.get_table_values.emplace_back(&get_xor_rotate_values_from_key<6, 0>);
    table.get_table_values.emplace_back(&get_xor_rotate_values_from_key<6, 1>);
    table.get_table_values.emplace_back(&get_xor_rotate_values_from_key<6, 0>);
    table.get_table_values.emplace_back(&get_xor_rotate_values_from_key<6, 0>);
    table.get_table_values.emplace_back(&get_xor_rotate_values_from_key<6, 0>);
    table.get_table_values.emplace_back(&get_xor_rotate_values_from_key<BITS_IN_LAST_SLICE, 0, true>);

    return table;
}

} // namespace blake2s_tables
} // namespace plookup
