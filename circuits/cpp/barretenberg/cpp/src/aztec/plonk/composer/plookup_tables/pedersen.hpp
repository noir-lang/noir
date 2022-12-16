#pragma once

#include "./types.hpp"

#include <crypto/pedersen/pedersen_lookup.hpp>
#include <numeric/bitop/rotate.hpp>
#include <numeric/bitop/sparse_form.hpp>
#include <numeric/bitop/pow.hpp>

namespace plookup {
namespace pedersen_tables {
namespace basic {

template <size_t generator_index>
inline std::array<barretenberg::fr, 2> get_basic_pedersen_table_values(const std::array<uint64_t, 2> key)
{
    const auto& basic_table = crypto::pedersen::lookup::get_table(generator_index);
    const size_t index = static_cast<size_t>(key[0]);
    return { basic_table[index].x, basic_table[index].y };
}

inline std::array<barretenberg::fr, 2> get_pedersen_iv_table_values(const std::array<uint64_t, 2> key)
{
    const auto& iv_table = crypto::pedersen::lookup::get_iv_table();
    const size_t index = static_cast<size_t>(key[0]);
    return { iv_table[index].x, iv_table[index].y };
}

template <size_t generator_index, bool is_small = false>
inline BasicTable generate_basic_pedersen_table(BasicTableId id, const size_t table_index)
{
    BasicTable table;
    table.id = id;
    table.table_index = table_index;
    table.size =
        is_small ? crypto::pedersen::lookup::PEDERSEN_SMALL_TABLE_SIZE : crypto::pedersen::lookup::PEDERSEN_TABLE_SIZE;
    table.use_twin_keys = false;

    const auto& basic_table = crypto::pedersen::lookup::get_table(generator_index);

    for (size_t i = 0; i < table.size; ++i) {
        table.column_1.emplace_back(i);
        table.column_2.emplace_back(basic_table[i].x);
        table.column_3.emplace_back(basic_table[i].y);
    }

    table.get_values_from_key = &get_basic_pedersen_table_values<generator_index>;

    table.column_1_step_size = table.size;
    table.column_2_step_size = 0;
    table.column_3_step_size = 0;

    return table;
}

inline BasicTable generate_pedersen_iv_table(BasicTableId id)
{
    BasicTable table;
    table.id = id;
    table.table_index = 0;
    table.size = crypto::pedersen::lookup::PEDERSEN_IV_TABLE_SIZE;
    table.use_twin_keys = false;

    const auto& iv_table = crypto::pedersen::lookup::get_iv_table();

    for (size_t i = 0; i < table.size; ++i) {
        table.column_1.emplace_back(i);
        table.column_2.emplace_back(iv_table[i].x);
        table.column_3.emplace_back(iv_table[i].y);
    }

    table.get_values_from_key = &get_pedersen_iv_table_values;

    table.column_1_step_size = table.size;
    table.column_2_step_size = 0;
    table.column_3_step_size = 0;

    return table;
}

inline MultiTable get_pedersen_iv_table(const MultiTableId id = PEDERSEN_IV)
{
    MultiTable table(crypto::pedersen::lookup::PEDERSEN_IV_TABLE_SIZE, 0, 0, 1);
    table.id = id;
    table.slice_sizes.emplace_back(crypto::pedersen::lookup::PEDERSEN_IV_TABLE_SIZE);
    table.get_table_values.emplace_back(&get_pedersen_iv_table_values);
    table.lookup_ids = { PEDERSEN_IV_BASE };

    return table;
}

inline MultiTable get_pedersen_left_lo_table(const MultiTableId id = PEDERSEN_LEFT_LO)
{
    const size_t num_entries = 126 / crypto::pedersen::lookup::BITS_PER_TABLE;
    MultiTable table(crypto::pedersen::lookup::PEDERSEN_TABLE_SIZE, 0, 0, num_entries);

    table.id = id;
    for (size_t i = 0; i < num_entries; ++i) {
        table.slice_sizes.emplace_back(crypto::pedersen::lookup::PEDERSEN_TABLE_SIZE);
    }

    table.get_table_values = { &get_basic_pedersen_table_values<0>, &get_basic_pedersen_table_values<0>,
                               &get_basic_pedersen_table_values<1>, &get_basic_pedersen_table_values<1>,
                               &get_basic_pedersen_table_values<2>, &get_basic_pedersen_table_values<2>,
                               &get_basic_pedersen_table_values<3>, &get_basic_pedersen_table_values<3>,
                               &get_basic_pedersen_table_values<4>, &get_basic_pedersen_table_values<4>,
                               &get_basic_pedersen_table_values<5>, &get_basic_pedersen_table_values<5>,
                               &get_basic_pedersen_table_values<6>, &get_basic_pedersen_table_values<6> };

    table.lookup_ids = { PEDERSEN_0, PEDERSEN_0, PEDERSEN_1, PEDERSEN_1, PEDERSEN_2, PEDERSEN_2, PEDERSEN_3,
                         PEDERSEN_3, PEDERSEN_4, PEDERSEN_4, PEDERSEN_5, PEDERSEN_5, PEDERSEN_6, PEDERSEN_6 };
    return table;
}

inline MultiTable get_pedersen_left_hi_table(const MultiTableId id = PEDERSEN_LEFT_HI)
{
    const size_t num_entries =
        (128 + crypto::pedersen::lookup::BITS_PER_TABLE) / crypto::pedersen::lookup::BITS_PER_TABLE;
    MultiTable table(crypto::pedersen::lookup::PEDERSEN_TABLE_SIZE, 0, 0, num_entries);

    table.id = id;
    for (size_t i = 0; i < num_entries - 1; ++i) {
        table.slice_sizes.emplace_back(crypto::pedersen::lookup::PEDERSEN_TABLE_SIZE);
    }
    table.slice_sizes.emplace_back(crypto::pedersen::lookup::PEDERSEN_SMALL_TABLE_SIZE);

    table.get_table_values = { &get_basic_pedersen_table_values<7>,  &get_basic_pedersen_table_values<7>,
                               &get_basic_pedersen_table_values<8>,  &get_basic_pedersen_table_values<8>,
                               &get_basic_pedersen_table_values<9>,  &get_basic_pedersen_table_values<9>,
                               &get_basic_pedersen_table_values<10>, &get_basic_pedersen_table_values<10>,
                               &get_basic_pedersen_table_values<11>, &get_basic_pedersen_table_values<11>,
                               &get_basic_pedersen_table_values<12>, &get_basic_pedersen_table_values<12>,
                               &get_basic_pedersen_table_values<13>, &get_basic_pedersen_table_values<13>,
                               &get_basic_pedersen_table_values<14> };

    table.lookup_ids = { PEDERSEN_7,  PEDERSEN_7,  PEDERSEN_8,  PEDERSEN_8,  PEDERSEN_9,
                         PEDERSEN_9,  PEDERSEN_10, PEDERSEN_10, PEDERSEN_11, PEDERSEN_11,
                         PEDERSEN_12, PEDERSEN_12, PEDERSEN_13, PEDERSEN_13, PEDERSEN_14_SMALL };
    return table;
}

inline MultiTable get_pedersen_right_lo_table(const MultiTableId id = PEDERSEN_RIGHT_LO)
{
    const size_t num_entries = 126 / crypto::pedersen::lookup::BITS_PER_TABLE;
    MultiTable table(crypto::pedersen::lookup::PEDERSEN_TABLE_SIZE, 0, 0, num_entries);

    table.id = id;
    for (size_t i = 0; i < num_entries; ++i) {
        table.slice_sizes.emplace_back(crypto::pedersen::lookup::PEDERSEN_TABLE_SIZE);
    }

    table.get_table_values = { &get_basic_pedersen_table_values<15>, &get_basic_pedersen_table_values<15>,
                               &get_basic_pedersen_table_values<16>, &get_basic_pedersen_table_values<16>,
                               &get_basic_pedersen_table_values<17>, &get_basic_pedersen_table_values<17>,
                               &get_basic_pedersen_table_values<18>, &get_basic_pedersen_table_values<18>,
                               &get_basic_pedersen_table_values<19>, &get_basic_pedersen_table_values<19>,
                               &get_basic_pedersen_table_values<20>, &get_basic_pedersen_table_values<20>,
                               &get_basic_pedersen_table_values<21>, &get_basic_pedersen_table_values<21> };

    table.lookup_ids = { PEDERSEN_15, PEDERSEN_15, PEDERSEN_16, PEDERSEN_16, PEDERSEN_17, PEDERSEN_17, PEDERSEN_18,
                         PEDERSEN_18, PEDERSEN_19, PEDERSEN_19, PEDERSEN_20, PEDERSEN_20, PEDERSEN_21, PEDERSEN_21 };
    return table;
}

inline MultiTable get_pedersen_right_hi_table(const MultiTableId id = PEDERSEN_RIGHT_HI)
{
    const size_t num_entries =
        (128 + crypto::pedersen::lookup::BITS_PER_TABLE) / crypto::pedersen::lookup::BITS_PER_TABLE;
    MultiTable table(crypto::pedersen::lookup::PEDERSEN_TABLE_SIZE, 0, 0, num_entries);

    table.id = id;
    for (size_t i = 0; i < num_entries - 1; ++i) {
        table.slice_sizes.emplace_back(crypto::pedersen::lookup::PEDERSEN_TABLE_SIZE);
    }
    table.slice_sizes.emplace_back(crypto::pedersen::lookup::PEDERSEN_SMALL_TABLE_SIZE);

    table.get_table_values = { &get_basic_pedersen_table_values<22>, &get_basic_pedersen_table_values<22>,
                               &get_basic_pedersen_table_values<23>, &get_basic_pedersen_table_values<23>,
                               &get_basic_pedersen_table_values<24>, &get_basic_pedersen_table_values<24>,
                               &get_basic_pedersen_table_values<25>, &get_basic_pedersen_table_values<25>,
                               &get_basic_pedersen_table_values<26>, &get_basic_pedersen_table_values<26>,
                               &get_basic_pedersen_table_values<27>, &get_basic_pedersen_table_values<27>,
                               &get_basic_pedersen_table_values<28>, &get_basic_pedersen_table_values<28>,
                               &get_basic_pedersen_table_values<29> };

    table.lookup_ids = { PEDERSEN_22, PEDERSEN_22, PEDERSEN_23, PEDERSEN_23, PEDERSEN_24,
                         PEDERSEN_24, PEDERSEN_25, PEDERSEN_25, PEDERSEN_26, PEDERSEN_26,
                         PEDERSEN_27, PEDERSEN_27, PEDERSEN_28, PEDERSEN_28, PEDERSEN_29_SMALL };
    return table;
}
} // namespace basic
} // namespace pedersen_tables
} // namespace plookup