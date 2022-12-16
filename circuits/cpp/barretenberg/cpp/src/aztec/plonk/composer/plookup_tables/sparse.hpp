#pragma once

#include "./types.hpp"

#include <crypto/aes128/aes128.hpp>
#include <numeric/bitop/rotate.hpp>
#include <numeric/bitop/sparse_form.hpp>
#include <numeric/bitop/pow.hpp>

namespace plookup {
namespace sparse_tables {

template <uint64_t base, uint64_t num_rotated_bits>
inline std::array<barretenberg::fr, 2> get_sparse_table_with_rotation_values(const std::array<uint64_t, 2> key)
{
    const auto t0 = numeric::map_into_sparse_form<base>(key[0]);
    barretenberg::fr t1;
    if constexpr (num_rotated_bits > 0) {
        t1 = numeric::map_into_sparse_form<base>(numeric::rotate32((uint32_t)key[0], num_rotated_bits));
    } else {
        t1 = t0;
    }
    return { barretenberg::fr(t0), barretenberg::fr(t1) };
}

template <uint64_t base, uint64_t bits_per_slice, uint64_t num_rotated_bits>
inline BasicTable generate_sparse_table_with_rotation(BasicTableId id, const size_t table_index)
{
    BasicTable table;
    table.id = id;
    table.table_index = table_index;
    table.size = (1U << bits_per_slice);
    table.use_twin_keys = false;

    for (uint64_t i = 0; i < table.size; ++i) {
        const uint64_t source = i;
        const auto target = numeric::map_into_sparse_form<base>(source);
        table.column_1.emplace_back(barretenberg::fr(source));
        table.column_2.emplace_back(barretenberg::fr(target));

        if constexpr (num_rotated_bits > 0) {
            const auto rotated =
                numeric::map_into_sparse_form<base>(numeric::rotate32((uint32_t)source, num_rotated_bits));
            table.column_3.emplace_back(barretenberg::fr(rotated));
        } else {
            table.column_3.emplace_back(barretenberg::fr(target));
        }
    }

    table.get_values_from_key = &get_sparse_table_with_rotation_values<base, num_rotated_bits>;

    uint256_t sparse_step_size = 1;
    for (size_t i = 0; i < bits_per_slice; ++i) {
        sparse_step_size *= base;
    }
    table.column_1_step_size = barretenberg::fr((1 << 11));
    table.column_2_step_size = barretenberg::fr(sparse_step_size);
    table.column_3_step_size = barretenberg::fr(sparse_step_size);

    return table;
}

template <size_t base, const uint64_t* base_table>
inline std::array<barretenberg::fr, 2> get_sparse_normalization_values(const std::array<uint64_t, 2> key)
{
    uint64_t accumulator = 0;
    uint64_t input = key[0];
    uint64_t count = 0;
    while (input > 0) {
        uint64_t slice = input % base;
        uint64_t bit = base_table[static_cast<size_t>(slice)];
        accumulator += (bit << count);
        input -= slice;
        input /= base;
        ++count;
    }
    return { barretenberg::fr(accumulator), barretenberg::fr(0) };
}

template <size_t base, uint64_t num_bits, const uint64_t* base_table>
inline BasicTable generate_sparse_normalization_table(BasicTableId id, const size_t table_index)
{
    /**
     * If t = 7*((e >>> 6) + (e >>> 11) + (e >>> 25)) + e + 2f + 3g
     * we can create a mapping between the 28 distinct values, and the result of
     * (e >>> 6) ^ (e >>> 11) ^ (e >>> 25) + e + 2f + 3g
     */

    BasicTable table;
    table.id = id;
    table.table_index = table_index;
    table.use_twin_keys = false;
    table.size = numeric::pow64(static_cast<uint64_t>(base), num_bits);

    numeric::sparse_int<base, num_bits> accumulator(0);
    numeric::sparse_int<base, num_bits> to_add(1);
    for (size_t i = 0; i < table.size; ++i) {
        const auto& limbs = accumulator.get_limbs();
        uint64_t key = 0;
        for (size_t j = 0; j < num_bits; ++j) {
            const size_t table_idx = static_cast<size_t>(limbs[j]);
            key += ((base_table[table_idx]) << static_cast<uint64_t>(j));
        }

        table.column_1.emplace_back(accumulator.get_sparse_value());
        table.column_2.emplace_back(key);
        table.column_3.emplace_back(barretenberg::fr(0));
        accumulator += to_add;
    }

    table.get_values_from_key = &get_sparse_normalization_values<base, base_table>;

    table.column_1_step_size = barretenberg::fr(table.size);
    table.column_2_step_size = barretenberg::fr(((uint64_t)1 << num_bits));
    table.column_3_step_size = barretenberg::fr(0);
    return table;
}
} // namespace sparse_tables
} // namespace plookup