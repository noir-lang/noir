#pragma once

#include "./types.hpp"

#include <numeric/bitop/rotate.hpp>

namespace waffle {
namespace uint_tables {

template <uint64_t bits_per_slice, uint64_t num_rotated_output_bits>
inline std::array<barretenberg::fr, 2> get_xor_rotate_values_from_key(const std::array<uint64_t, 2> key)
{
    return { numeric::rotate64(key[0] ^ key[1], num_rotated_output_bits), 0ULL };
}

template <uint64_t bits_per_slice, uint64_t num_rotated_output_bits>
inline PlookupBasicTable generate_xor_rotate_table(PlookupBasicTableId id, const size_t table_index)
{
    const uint64_t base = 1UL << bits_per_slice;
    PlookupBasicTable table;
    table.id = id;
    table.table_index = table_index;
    table.size = base * base;
    table.use_twin_keys = true;

    for (uint64_t i = 0; i < base; ++i) {
        for (uint64_t j = 0; j < base; ++j) {
            table.column_1.emplace_back(i);
            table.column_2.emplace_back(j);
            table.column_3.emplace_back(numeric::rotate64(i ^ j, num_rotated_output_bits));
        }
    }

    table.get_values_from_key = &get_xor_rotate_values_from_key<bits_per_slice, num_rotated_output_bits>;

    table.column_1_step_size = base;
    table.column_2_step_size = base;
    table.column_3_step_size = base;

    return table;
}

template <uint64_t bits_per_slice, uint64_t num_rotated_output_bits>
inline std::array<barretenberg::fr, 2> get_and_rotate_values_from_key(const std::array<uint64_t, 2> key)
{
    return { numeric::rotate64(key[0] & key[1], num_rotated_output_bits), 0ULL };
}

template <uint64_t bits_per_slice, uint64_t num_rotated_output_bits>
inline PlookupBasicTable generate_and_rotate_table(PlookupBasicTableId id, const size_t table_index)
{
    const uint64_t base = 1UL << bits_per_slice;
    PlookupBasicTable table;
    table.id = id;
    table.table_index = table_index;
    table.size = base * base;
    table.use_twin_keys = true;

    for (uint64_t i = 0; i < base; ++i) {
        for (uint64_t j = 0; j < base; ++j) {
            table.column_1.emplace_back(i);
            table.column_2.emplace_back(j);
            table.column_3.emplace_back(numeric::rotate64(i & j, num_rotated_output_bits));
        }
    }

    table.get_values_from_key = &get_xor_rotate_values_from_key<bits_per_slice, num_rotated_output_bits>;

    table.column_1_step_size = base;
    table.column_2_step_size = base;
    table.column_3_step_size = base;

    return table;
}

inline PlookupMultiTable get_uint32_xor_table(const PlookupMultiTableId id = UINT32_XOR)
{
    const size_t num_entries = (32 + 5) / 6;
    const uint64_t base = 1 << 6;
    PlookupMultiTable table(base, base, base, num_entries);

    table.id = id;
    for (size_t i = 0; i < num_entries; ++i) {
        table.slice_sizes.emplace_back(base);
        table.lookup_ids.emplace_back(UINT_XOR_ROTATE0);
        table.get_table_values.emplace_back(&get_xor_rotate_values_from_key<6, 0>);
    }
    return table;
}

inline PlookupMultiTable get_uint32_and_table(const PlookupMultiTableId id = UINT32_AND)
{
    const size_t num_entries = (32 + 5) / 6;
    const uint64_t base = 1 << 6;
    PlookupMultiTable table(base, base, base, num_entries);

    table.id = id;
    for (size_t i = 0; i < num_entries; ++i) {
        table.slice_sizes.emplace_back(base);
        table.lookup_ids.emplace_back(UINT_AND_ROTATE0);
        table.get_table_values.emplace_back(&get_and_rotate_values_from_key<6, 0>);
    }
    return table;
}

} // namespace uint_tables
} // namespace waffle