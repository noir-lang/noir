#pragma once

#include <crypto/aes128/aes128.hpp>
#include <numeric/bitop/rotate.hpp>
#include <numeric/bitop/sparse_form.hpp>

#include "./types.hpp"

namespace waffle {

namespace aes128_tables {
static constexpr uint64_t AES_BASE = 9;

inline std::array<barretenberg::fr, 2> get_aes_sparse_values_from_key(const std::array<uint64_t, 2> key)
{
    const auto sparse = numeric::map_into_sparse_form<AES_BASE>(uint64_t(key[0]));
    return { barretenberg::fr(sparse), barretenberg::fr(0) };
}

inline PLookupTable generate_aes_sparse_table(PLookupTableId id, const size_t table_index)
{
    PLookupTable table;
    table.id = id;
    table.table_index = table_index;
    table.size = 256;
    table.use_twin_keys = true;
    for (uint64_t i = 0; i < table.size; ++i) {
        uint64_t left = i;
        const auto right = numeric::map_into_sparse_form<AES_BASE>((uint8_t)i);
        table.column_1.emplace_back(barretenberg::fr(left));
        table.column_2.emplace_back(barretenberg::fr(0));
        table.column_3.emplace_back(barretenberg::fr(right));
    }
    table.get_values_from_key = &get_aes_sparse_values_from_key;

    table.column_1_step_size = barretenberg::fr(256);
    table.column_2_step_size = barretenberg::fr(0);
    table.column_3_step_size = barretenberg::fr(0);
    return table;
}

inline std::array<barretenberg::fr, 2> get_aes_sparse_normalization_values_from_key(const std::array<uint64_t, 2> key)
{
    const auto byte = numeric::map_from_sparse_form<AES_BASE>(key[0]);
    return { barretenberg::fr(numeric::map_into_sparse_form<AES_BASE>(byte)), barretenberg::fr(0) };
}

inline PLookupTable generate_aes_sparse_normalization_table(PLookupTableId id, const size_t table_index)
{
    PLookupTable table;
    table.id = id;
    table.table_index = table_index;
    for (uint64_t i = 0; i < AES_BASE; ++i) {
        uint64_t i_raw = i * AES_BASE * AES_BASE * AES_BASE;
        uint64_t i_normalized = ((i & 1UL) == 1UL) * AES_BASE * AES_BASE * AES_BASE;
        for (uint64_t j = 0; j < AES_BASE; ++j) {
            uint64_t j_raw = j * AES_BASE * AES_BASE;
            uint64_t j_normalized = ((j & 1UL) == 1UL) * AES_BASE * AES_BASE;
            for (uint64_t k = 0; k < AES_BASE; ++k) {
                uint64_t k_raw = k * AES_BASE;
                uint64_t k_normalized = ((k & 1UL) == 1UL) * AES_BASE;
                for (uint64_t m = 0; m < AES_BASE; ++m) {
                    uint64_t m_raw = m;
                    uint64_t m_normalized = ((m & 1UL) == 1UL);
                    uint64_t left = i_raw + j_raw + k_raw + m_raw;
                    uint64_t right = i_normalized + j_normalized + k_normalized + m_normalized;
                    table.column_1.emplace_back(left);
                    table.column_2.emplace_back(barretenberg::fr(0));
                    table.column_3.emplace_back(right);
                }
            }
        }
    }
    table.size = table.column_1.size();
    table.use_twin_keys = true;
    table.get_values_from_key = &get_aes_sparse_normalization_values_from_key;

    table.column_1_step_size = barretenberg::fr(6561);
    table.column_2_step_size = barretenberg::fr(0);
    table.column_3_step_size = barretenberg::fr(6561);
    return table;
}

inline std::array<barretenberg::fr, 2> get_aes_sbox_values_from_key(const std::array<uint64_t, 2> key)
{
    const auto byte = numeric::map_from_sparse_form<AES_BASE>(key[0]);
    uint8_t sbox_value = crypto::aes128::sbox[(uint8_t)byte];
    uint8_t swizzled = ((uint8_t)(sbox_value << 1) ^ (uint8_t)(((sbox_value >> 7) & 1) * 0x1b));
    return { barretenberg::fr(numeric::map_into_sparse_form<AES_BASE>(sbox_value)),
             barretenberg::fr(numeric::map_into_sparse_form<AES_BASE>((uint8_t)(sbox_value ^ swizzled))) };
}

inline PLookupTable generate_aes_sbox_table(PLookupTableId id, const size_t table_index)
{
    PLookupTable table;
    table.id = id;
    table.table_index = table_index;
    table.size = 256;
    table.use_twin_keys = false;
    for (uint64_t i = 0; i < table.size; ++i) {
        const auto first = numeric::map_into_sparse_form<AES_BASE>((uint8_t)i);
        uint8_t sbox_value = crypto::aes128::sbox[(uint8_t)i];
        uint8_t swizzled = ((uint8_t)(sbox_value << 1) ^ (uint8_t)(((sbox_value >> 7) & 1) * 0x1b));
        const auto second = numeric::map_into_sparse_form<AES_BASE>(sbox_value);
        const auto third = numeric::map_into_sparse_form<AES_BASE>((uint8_t)(sbox_value ^ swizzled));

        table.column_1.emplace_back(barretenberg::fr(first));
        table.column_2.emplace_back(barretenberg::fr(second));
        table.column_3.emplace_back(barretenberg::fr(third));
    }
    table.get_values_from_key = get_aes_sbox_values_from_key;

    table.column_1_step_size = barretenberg::fr(0);
    table.column_2_step_size = barretenberg::fr(0);
    table.column_3_step_size = barretenberg::fr(0);
    return table;
}
} // namespace aes128_tables
} // namespace waffle