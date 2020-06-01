#pragma once

#include "./types.hpp"

#include <crypto/pedersen/sidon_pedersen.hpp>
#include <numeric/bitop/rotate.hpp>
#include <numeric/bitop/sparse_form.hpp>
#include <numeric/bitop/pow.hpp>

namespace waffle {
namespace pedersen_tables {

template <size_t generator_index>
inline std::array<barretenberg::fr, 2> get_sidon_pedersen_table_values(const std::array<uint64_t, 2> key)
{
    const auto& sidon_table = crypto::pedersen::sidon::get_table(generator_index);
    const size_t index = static_cast<size_t>(key[0]);
    return { sidon_table[index].x, sidon_table[index].y };
}

template <size_t generator_index>
inline PLookupBasicTable generate_sidon_pedersen_table(PLookupBasicTableId id, const size_t table_index)
{
    PLookupBasicTable table;
    table.id = id;
    table.table_index = table_index;
    table.size = crypto::pedersen::sidon::PEDERSEN_TABLE_SIZE;
    table.use_twin_keys = false;

    const auto& sidon_table = crypto::pedersen::sidon::get_table(generator_index);

    for (size_t i = 0; i < table.size; ++i) {
        table.column_1.emplace_back(i);
        table.column_2.emplace_back(sidon_table[i].x);
        table.column_3.emplace_back(sidon_table[i].y);
    }

    table.get_values_from_key = &get_sidon_pedersen_table_values<generator_index>;

    table.column_1_step_size = table.size;
    table.column_2_step_size = 0;
    table.column_3_step_size = 0;

    return table;
}

inline PLookupMultiTable get_pedersen_left_table(const PLookupMultiTableId id = PEDERSEN_LEFT)
{
    const size_t num_entries =
        (256 + crypto::pedersen::sidon::BITS_PER_TABLE - 1) / crypto::pedersen::sidon::BITS_PER_TABLE;
    PLookupMultiTable table(crypto::pedersen::sidon::PEDERSEN_TABLE_SIZE, 0, 0, num_entries);

    table.id = id;
    for (size_t i = 0; i < num_entries; ++i) {
        table.slice_sizes.emplace_back(crypto::pedersen::sidon::PEDERSEN_TABLE_SIZE);
    }

    table.get_table_values = {
        &get_sidon_pedersen_table_values<0>, &get_sidon_pedersen_table_values<0>, &get_sidon_pedersen_table_values<0>,
        &get_sidon_pedersen_table_values<1>, &get_sidon_pedersen_table_values<1>, &get_sidon_pedersen_table_values<1>,
        &get_sidon_pedersen_table_values<2>, &get_sidon_pedersen_table_values<2>, &get_sidon_pedersen_table_values<2>,
        &get_sidon_pedersen_table_values<3>, &get_sidon_pedersen_table_values<3>, &get_sidon_pedersen_table_values<3>,
        &get_sidon_pedersen_table_values<4>, &get_sidon_pedersen_table_values<4>, &get_sidon_pedersen_table_values<4>,
        &get_sidon_pedersen_table_values<5>, &get_sidon_pedersen_table_values<5>, &get_sidon_pedersen_table_values<5>,
        &get_sidon_pedersen_table_values<6>, &get_sidon_pedersen_table_values<6>, &get_sidon_pedersen_table_values<6>,
        &get_sidon_pedersen_table_values<7>, &get_sidon_pedersen_table_values<7>, &get_sidon_pedersen_table_values<7>,
        &get_sidon_pedersen_table_values<8>, &get_sidon_pedersen_table_values<8>,
    };

    table.lookup_ids = {
        PEDERSEN_0, PEDERSEN_0, PEDERSEN_0, PEDERSEN_1, PEDERSEN_1, PEDERSEN_1, PEDERSEN_2, PEDERSEN_2, PEDERSEN_2,
        PEDERSEN_3, PEDERSEN_3, PEDERSEN_3, PEDERSEN_4, PEDERSEN_4, PEDERSEN_4, PEDERSEN_5, PEDERSEN_5, PEDERSEN_5,
        PEDERSEN_6, PEDERSEN_6, PEDERSEN_6, PEDERSEN_7, PEDERSEN_7, PEDERSEN_7, PEDERSEN_8, PEDERSEN_8,
    };
    return table;
}

inline PLookupMultiTable get_pedersen_right_table(const PLookupMultiTableId id = PEDERSEN_RIGHT)
{
    const size_t num_entries =
        (256 + crypto::pedersen::sidon::BITS_PER_TABLE) / crypto::pedersen::sidon::BITS_PER_TABLE;
    PLookupMultiTable table(crypto::pedersen::sidon::PEDERSEN_TABLE_SIZE, 0, 0, num_entries);

    table.id = id;
    for (size_t i = 0; i < num_entries; ++i) {
        table.slice_sizes.emplace_back(crypto::pedersen::sidon::PEDERSEN_TABLE_SIZE);
    }

    table.get_table_values = {
        &get_sidon_pedersen_table_values<9>,  &get_sidon_pedersen_table_values<9>,
        &get_sidon_pedersen_table_values<9>,  &get_sidon_pedersen_table_values<10>,
        &get_sidon_pedersen_table_values<10>, &get_sidon_pedersen_table_values<10>,
        &get_sidon_pedersen_table_values<11>, &get_sidon_pedersen_table_values<11>,
        &get_sidon_pedersen_table_values<11>, &get_sidon_pedersen_table_values<12>,
        &get_sidon_pedersen_table_values<12>, &get_sidon_pedersen_table_values<12>,
        &get_sidon_pedersen_table_values<13>, &get_sidon_pedersen_table_values<13>,
        &get_sidon_pedersen_table_values<13>, &get_sidon_pedersen_table_values<14>,
        &get_sidon_pedersen_table_values<14>, &get_sidon_pedersen_table_values<14>,
        &get_sidon_pedersen_table_values<15>, &get_sidon_pedersen_table_values<15>,
        &get_sidon_pedersen_table_values<15>, &get_sidon_pedersen_table_values<16>,
        &get_sidon_pedersen_table_values<16>, &get_sidon_pedersen_table_values<16>,
        &get_sidon_pedersen_table_values<17>, &get_sidon_pedersen_table_values<17>,
    };

    table.lookup_ids = { PEDERSEN_9,  PEDERSEN_9,  PEDERSEN_9,  PEDERSEN_10, PEDERSEN_10, PEDERSEN_10, PEDERSEN_11,
                         PEDERSEN_11, PEDERSEN_11, PEDERSEN_12, PEDERSEN_12, PEDERSEN_12, PEDERSEN_13, PEDERSEN_13,
                         PEDERSEN_13, PEDERSEN_14, PEDERSEN_14, PEDERSEN_14, PEDERSEN_15, PEDERSEN_15, PEDERSEN_15,
                         PEDERSEN_16, PEDERSEN_16, PEDERSEN_16, PEDERSEN_17, PEDERSEN_17 };
    return table;
}
} // namespace pedersen_tables
} // namespace waffle