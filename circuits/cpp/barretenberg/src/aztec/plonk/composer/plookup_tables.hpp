#pragma once

#include <ecc/curves/bn254/fr.hpp>
#include <numeric/bitop/rotate.hpp>
#include <numeric/bitop/sparse_form.hpp>

namespace waffle {

enum PLookupTableId {
    XOR,
    AND,
    PEDERSEN,
    AES_SPARSE_MAP,
    AES_SBOX_MAP,
    AES_SPARSE_NORMALIZE,
    SHA256_BASE7_ROTATE6,
    SHA256_BASE7_ROTATE3,
    SHA256_BASE4_ROTATE2,
    SHA256_BASE4_ROTATE6,
    SHA256_BASE4_ROTATE7,
    SHA256_BASE4_ROTATE8,
    SHA256_BASE7_NORMALIZE,
    SHA256_PARTA_NORMALIZE,
    SHA256_BASE4_NORMALIZE,
    SHA256_PARTB_NORMALIZE,
};

struct PLookupTable {
    struct KeyEntry {
        std::array<uint64_t, 2> key{ 0, 0 };
        std::array<barretenberg::fr, 2> value{ barretenberg::fr(0), barretenberg::fr(0) };
        bool operator<(const KeyEntry& other) const
        {
            return key[0] < other.key[0] || ((key[0] == other.key[0]) && key[1] < other.key[1]);
        }

        std::array<barretenberg::fr, 3> to_sorted_list_components(const bool use_two_keys) const
        {
            return {
                barretenberg::fr(key[0]),
                use_two_keys ? barretenberg::fr(key[1]) : value[0],
                use_two_keys ? value[0] : value[1],
            };
        }
    };

    PLookupTableId id;
    size_t table_index;
    size_t size;
    bool use_twin_keys;

    barretenberg::fr column_1_step_size = barretenberg::fr(0);
    barretenberg::fr column_2_step_size = barretenberg::fr(0);
    barretenberg::fr column_3_step_size = barretenberg::fr(0);
    std::vector<barretenberg::fr> column_1;
    std::vector<barretenberg::fr> column_3;
    std::vector<barretenberg::fr> column_2;
    std::vector<KeyEntry> lookup_gates;

    std::array<barretenberg::fr, 2> (*get_values_from_key)(const std::array<uint64_t, 2>);
};

namespace aes_tables {
static constexpr uint64_t AES_BASE = 9;
static constexpr uint8_t sbox[256] = {
    // 0     1    2      3     4    5     6     7      8    9     A      B    C     D     E     F
    0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5, 0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab, 0x76, 0xca, 0x82, 0xc9,
    0x7d, 0xfa, 0x59, 0x47, 0xf0, 0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4, 0x72, 0xc0, 0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f,
    0xf7, 0xcc, 0x34, 0xa5, 0xe5, 0xf1, 0x71, 0xd8, 0x31, 0x15, 0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a, 0x07,
    0x12, 0x80, 0xe2, 0xeb, 0x27, 0xb2, 0x75, 0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0, 0x52, 0x3b, 0xd6, 0xb3,
    0x29, 0xe3, 0x2f, 0x84, 0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b, 0x6a, 0xcb, 0xbe, 0x39, 0x4a, 0x4c, 0x58,
    0xcf, 0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85, 0x45, 0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8, 0x51, 0xa3,
    0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5, 0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2, 0xcd, 0x0c, 0x13, 0xec, 0x5f,
    0x97, 0x44, 0x17, 0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73, 0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a, 0x90, 0x88,
    0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb, 0xe0, 0x32, 0x3a, 0x0a, 0x49, 0x06, 0x24, 0x5c, 0xc2, 0xd3, 0xac,
    0x62, 0x91, 0x95, 0xe4, 0x79, 0xe7, 0xc8, 0x37, 0x6d, 0x8d, 0xd5, 0x4e, 0xa9, 0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a,
    0xae, 0x08, 0xba, 0x78, 0x25, 0x2e, 0x1c, 0xa6, 0xb4, 0xc6, 0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a, 0x70,
    0x3e, 0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e, 0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e, 0xe1, 0xf8, 0x98, 0x11,
    0x69, 0xd9, 0x8e, 0x94, 0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf, 0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42,
    0x68, 0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb, 0x16,
};

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

    table.column_1_step_size = barretenberg::fr(0);
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

    table.column_1_step_size = barretenberg::fr(-6561);
    table.column_2_step_size = barretenberg::fr(0);
    table.column_3_step_size = barretenberg::fr(-16);
    return table;
}

inline std::array<barretenberg::fr, 2> get_aes_sbox_values_from_key(const std::array<uint64_t, 2> key)
{
    const auto byte = sbox[numeric::map_from_sparse_form<AES_BASE>(key[0])];
    uint8_t sbox_value = sbox[(uint8_t)byte];
    uint8_t swizzled = ((uint8_t)(sbox_value << 1) ^ (uint8_t)(((sbox_value >> 7) & 1) * 0x1b));
    return { barretenberg::fr(sbox_value), barretenberg::fr(swizzled) };
}

inline PLookupTable generate_aes_sbox_table(PLookupTableId id, const size_t table_index)
{
    PLookupTable table;
    table.id = id;
    table.table_index = table_index;
    table.size = 256;
    table.use_twin_keys = true;
    for (uint64_t i = 0; i < table.size; ++i) {
        const auto first = numeric::map_into_sparse_form<AES_BASE>((uint8_t)i);
        uint8_t sbox_value = sbox[(uint8_t)i];
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
} // namespace aes_tables

namespace sha256_tables {

template <uint64_t base, uint64_t num_rotated_bits>
inline std::array<barretenberg::fr, 2> get_sha256_sparse_map_values_from_key(const std::array<uint64_t, 2> key)
{
    const auto t1 = numeric::map_into_sparse_form<base>(numeric::rotate32((uint32_t)key[0], num_rotated_bits));
    return { barretenberg::fr(t1), barretenberg::fr(0) };
}

template <uint64_t base, uint64_t num_rotated_bits>
inline PLookupTable generate_sparse_table_with_rotation(PLookupTableId id, const size_t table_index)
{
    constexpr uint64_t bits_per_slice = 11;
    PLookupTable table;
    table.id = id;
    table.table_index = table_index;
    table.size = (1U << bits_per_slice);
    table.use_twin_keys = true;

    for (uint64_t i = 0; i < table.size; ++i) {
        const uint64_t source = i;
        const auto target = numeric::map_into_sparse_form<base>(source);
        const auto rotated = numeric::map_into_sparse_form<base>(numeric::rotate32((uint32_t)source, num_rotated_bits));
        table.column_1.emplace_back(barretenberg::fr(source));
        table.column_2.emplace_back(barretenberg::fr(target));
        table.column_3.emplace_back(barretenberg::fr(rotated));
    }

    table.get_values_from_key = &get_sha256_sparse_map_values_from_key<base, num_rotated_bits>;

    uint256_t sparse_step_size = 1;
    for (size_t i = 0; i < bits_per_slice; ++i) {
        sparse_step_size *= base;
    }
    table.column_1_step_size = barretenberg::fr(-(1 << 11));
    table.column_2_step_size = barretenberg::fr(-sparse_step_size);
    table.column_3_step_size = barretenberg::fr(-sparse_step_size);

    return table;
}

template <uint64_t base>
inline std::array<barretenberg::fr, 2> get_output_sparse_map_values_from_key(const std::array<uint64_t, 2> key)
{
    const uint64_t target = numeric::map_from_sparse_form<base>(key[0]);
    return { barretenberg::fr(target), barretenberg::fr(0) };
}

template <uint64_t base, uint64_t bits_per_slice>
inline PLookupTable generate_sparse_normalization_table(PLookupTableId id, const size_t table_index)
{
    PLookupTable table;
    table.size = 1;
    table.use_twin_keys = true;
    table.id = id;
    table.table_index = table_index;
    for (uint64_t i = 0; i < bits_per_slice; ++i) {
        table.size *= base;
    }

    for (size_t i = 0; i < table.size; ++i) {
        table.column_1.emplace_back(barretenberg::fr(i));
        table.column_2.emplace_back(barretenberg::fr(0));
        table.column_3.emplace_back(barretenberg::fr(numeric::map_from_sparse_form<base>(i)));
    }

    table.get_values_from_key = &get_output_sparse_map_values_from_key<base>;

    table.column_1_step_size = barretenberg::fr(-(1 << bits_per_slice));
    table.column_2_step_size = barretenberg::fr(0);
    table.column_3_step_size = barretenberg::fr(-table.size);

    return table;
}

inline std::array<barretenberg::fr, 2> get_sha256_part_a_output_values_from_key(const std::array<uint64_t, 2> key)
{
    constexpr uint64_t base = 7;
    constexpr uint64_t bit_table[7]{
        0, // e + 2f + 3g = 0 => e = 0, f = 0, g = 0 => t = 0
        0, // e + 2f + 3g = 1 => e = 1, f = 0, g = 0 => t = 0
        0, // e + 2f + 3g = 2 => e = 0, f = 1, g = 0 => t = 0
        1, // e + 2f + 3g = 3 => e = 0, f = 0, g = 1 OR e = 1, f = 1, g = 0 => t = 1
        0, // e + 2f + 3g = 4 => e = 1, f = 0, g = 1 => t = 0
        1, // e + 2f + 3g = 5 => e = 0, f = 1, g = 1 => t = 1
        1, // e + 2f + 3g = 6 => e = 1, f = 1, g = 1 => t = 1
    };

    uint64_t accumulator = 0;
    uint64_t input = key[0];
    uint64_t count = 0;
    while (input > 0) {
        uint64_t slice = input % base;
        uint64_t bit = bit_table[static_cast<size_t>(slice)];
        accumulator += (bit << count);
        input -= slice;
        input /= base;
        ++count;
    }
    return { barretenberg::fr(accumulator), barretenberg::fr(0) };
}

inline PLookupTable generate_sha256_part_a_normalization_table(PLookupTableId id, const size_t table_index)
{
    constexpr uint64_t bit_table[7]{
        0, // e + 2f + 3g = 0 => e = 0, f = 0, g = 0 => t = 0
        0, // e + 2f + 3g = 1 => e = 1, f = 0, g = 0 => t = 0
        0, // e + 2f + 3g = 2 => e = 0, f = 1, g = 0 => t = 0
        1, // e + 2f + 3g = 3 => e = 0, f = 0, g = 1 OR e = 1, f = 1, g = 0 => t = 1
        0, // e + 2f + 3g = 4 => e = 1, f = 0, g = 1 => t = 0
        1, // e + 2f + 3g = 5 => e = 0, f = 1, g = 1 => t = 1
        1, // e + 2f + 3g = 6 => e = 1, f = 1, g = 1 => t = 1
    };

    constexpr uint64_t base = 7;
    constexpr uint64_t base_sqr = base * base;
    constexpr uint64_t base_cube = base * base * base;

    PLookupTable table;
    table.id = id;
    table.table_index = table_index;
    table.use_twin_keys = true;
    table.size = base_cube * base;

    /**
     * t = (e & f) ^ (~e & g)
     *
     * For a given bit of e, f, g, we can create a unique mapping between t and e + 2f + 3g
     *
     * | e | f | g | t | e + 2f + 3g |
     * -------------------------------
     * | 0 | 0 | 0 | 0 |           0 |
     * | 0 | 0 | 1 | 1 |           3 |
     * | 0 | 1 | 0 | 0 |           2 |
     * | 0 | 1 | 1 | 1 |           5 |
     * | 1 | 0 | 0 | 0 |           1 |
     * | 1 | 0 | 1 | 0 |           4 |
     * | 1 | 1 | 0 | 1 |           3 |
     * | 1 | 1 | 1 | 1 |           6 |
     *
     * More generally, we want to create a unique encoding of 't', using some arithmetic relationship
     * between e, f and g.
     *
     * Let's define e', f', g' to be the arithmetic encodings of e, f, g. In the above case where
     * t' = e + 2f + 3g, then e' = e, f' = 2f, g' = 3g.
     *
     * We need e', f', g' to be distinct as the operation (e & f) ^ (~e & g) is not associative.
     * However we do not need the output to be distinct for each combination of e, f and g.
     *
     * We can exploit symmetries that arise when any of e, f, g are fixed
     *
     * For example, we can have e' + f' = g', as (e & f) = g when e is non-zero.
     *
     * However it is not possible to define relationships that remove more than 1 degree of freedom.
     * Which gives us seven unique states that t can take.
     *
     * The wider context to this, is that we want to create a lookup table that can be read from to
     * determine the value of `ch`.
     *
     * One approach would be to split the algorithm into its logical components (addition, xor, rotate)
     * and to perform lookups for each operation.
     *
     * E.g. if we map a slice of input bits into a 'sparse' form, we can approximate logical operations with arithmetic
     *operations. For example, if we map a binary integer into a base-3 form, then XOR can be represented with
     *additions.
     *
     *  1. Map a, b into base-3 form a', b' (where each bit can take 3 values)
     *  2. compute c' = a' + b'
     *  3. Map c' out of sparse form, where every 'trit' reduces to a bit using the map (0 = 0, 1 = 1, 2 = 0)
     *
     *
     * But this approach can be extended to cover more complex algorithms, at the cost of increasing the base.
     *
     * If we map e, f, g into a base-7 sparse form, then we can evaluate (e & f) ^ (~e & g) as described above.
     *
     **/
    for (size_t i = 0; i < base; ++i) {
        const uint64_t i_value = i * base_cube;
        const uint64_t i_bit = bit_table[static_cast<size_t>(i)] << 3;
        for (size_t j = 0; j < base; ++j) {
            const uint64_t j_value = j * base_sqr;
            const uint64_t j_bit = bit_table[static_cast<size_t>(j)] << 2;
            for (size_t k = 0; k < base; ++k) {
                const uint64_t k_value = k * base;
                const uint64_t k_bit = bit_table[static_cast<size_t>(k)] << 1;
                for (size_t m = 0; m < base; ++m) {
                    const uint64_t m_value = m;
                    const uint64_t m_bit = bit_table[static_cast<size_t>(m)];

                    const uint64_t input = m_value + k_value + j_value + i_value;
                    const uint64_t output = m_bit + k_bit + j_bit + i_bit;
                    table.column_1.emplace_back(barretenberg::fr(input));
                    table.column_2.emplace_back(barretenberg::fr(0));
                    table.column_3.emplace_back(barretenberg::fr(output));
                }
            }
        }
    }

    table.get_values_from_key = &get_sha256_part_a_output_values_from_key;

    table.column_1_step_size = barretenberg::fr(-(1 << 4));
    table.column_2_step_size = barretenberg::fr(0);
    table.column_3_step_size = barretenberg::fr(-2401);
    return table;
}

inline barretenberg::fr get_sha256_part_a_output_values_from_key(const uint256_t key)
{
    constexpr uint256_t base = 7;
    constexpr uint64_t bit_table[7]{
        0, // e + 2f + 3g = 0 => e = 0, f = 0, g = 0 => t = 0
        0, // e + 2f + 3g = 1 => e = 1, f = 0, g = 0 => t = 0
        0, // e + 2f + 3g = 2 => e = 0, f = 1, g = 0 => t = 0
        1, // e + 2f + 3g = 3 => e = 0, f = 0, g = 1 OR e = 1, f = 1, g = 0 => t = 1
        0, // e + 2f + 3g = 4 => e = 1, f = 0, g = 1 => t = 0
        1, // e + 2f + 3g = 5 => e = 0, f = 1, g = 1 => t = 1
        1, // e + 2f + 3g = 6 => e = 1, f = 1, g = 1 => t = 1
    };

    uint64_t accumulator = 0;
    uint256_t input = key;
    uint64_t count = 0;
    while (input > 0) {
        uint256_t slice = input % base;
        uint64_t bit = bit_table[static_cast<size_t>(slice.data[0])];
        accumulator += (bit << count);
        input -= slice;
        input /= base;
        ++count;
    }
    return barretenberg::fr(accumulator);
}

inline std::array<barretenberg::fr, 2> get_sha256_part_b_output_values_from_key(const std::array<uint64_t, 2> key)
{
    constexpr uint64_t base = 4;
    constexpr uint64_t bit_table[4]{
        0, // a + b + c = 0 => (a & b) ^ (a & c) ^ (b & c) = 0
        0, // a + b + c = 1 => (a & b) ^ (a & c) ^ (b & c) = 0
        1, // a + b + c = 2 => (a & b) ^ (a & c) ^ (b & c) = 1
        1, // a + b + c = 3 => (a & b) ^ (a & c) ^ (b & c) = 0
    };

    uint64_t accumulator = 0;
    uint64_t input = key[0];
    uint64_t count = 0;
    while (input > 0) {
        uint64_t slice = input % base;
        uint64_t bit = bit_table[static_cast<size_t>(slice)];
        accumulator += (bit << count);
        input -= slice;
        input /= base;
        ++count;
    }
    return { barretenberg::fr(accumulator), barretenberg::fr(0) };
}

inline PLookupTable generate_sha256_part_b_normalization_table(PLookupTableId id, const size_t table_index)
{
    /**
     * v = (a & b) ^ (a & c) ^ (b & c)
     *
     * For a given bit of a, b, c, we can create a unique mapping between s and a + b + c
     *
     * | a | b | c | s |  a + b + c  |
     * -------------------------------
     * | 0 | 0 | 0 | 0 |           0 |
     * | 0 | 0 | 1 | 0 |           1 |
     * | 0 | 1 | 0 | 0 |           1 |
     * | 0 | 1 | 1 | 1 |           2 |
     * | 1 | 0 | 0 | 0 |           1 |
     * | 1 | 0 | 1 | 1 |           2 |
     * | 1 | 1 | 0 | 1 |           2 |
     * | 1 | 1 | 1 | 0 |           3 |
     *
     * i.e. we map 0 to 0, 1 to 0, 2 to 1, 3 to 2.
     *
     *
     **/

    constexpr uint64_t bit_table[4]{
        0, // a + b + c = 0 => (a & b) ^ (a & c) ^ (b & c) = 0
        0, // a + b + c = 1 => (a & b) ^ (a & c) ^ (b & c) = 0
        1, // a + b + c = 2 => (a & b) ^ (a & c) ^ (b & c) = 1
        1, // a + b + c = 3 => (a & b) ^ (a & c) ^ (b & c) = 0
    };

    constexpr uint64_t base = 4;
    constexpr uint64_t base_sqr = base * base;
    constexpr uint64_t base_cube = base * base * base;
    constexpr uint64_t base_quad = base * base * base * base;
    constexpr uint64_t base_pent = base * base * base * base * base;

    PLookupTable table;
    table.use_twin_keys = true;
    table.size = base_pent * base;
    table.id = id;
    table.table_index = table_index;
    for (size_t i = 0; i < base; ++i) {
        const uint64_t i_value = i * base_pent;
        const uint64_t i_bit = bit_table[static_cast<uint64_t>(i)] << 5;
        for (size_t j = 0; j < base; ++j) {
            const uint64_t j_value = j * base_quad;
            const uint64_t j_bit = bit_table[static_cast<uint64_t>(j)] << 4;
            for (size_t k = 0; k < base; ++k) {
                const uint64_t k_value = k * base_cube;
                const uint64_t k_bit = bit_table[static_cast<uint64_t>(k)] << 3;
                for (size_t m = 0; m < base; ++m) {
                    const uint64_t m_value = m * base_sqr;
                    const uint64_t m_bit = bit_table[static_cast<uint64_t>(m)] << 2;
                    for (size_t l = 0; l < base; ++l) {
                        const uint64_t l_value = l * base;
                        const uint64_t l_bit = bit_table[static_cast<uint64_t>(l)] << 1;
                        for (size_t p = 0; p < base; ++p) {
                            const uint64_t p_value = p;
                            const uint64_t p_bit = bit_table[static_cast<uint64_t>(p)];

                            const uint64_t input = p_value + l_value + m_value + k_value + j_value + i_value;
                            const uint64_t output = p_bit + l_bit + m_bit + k_bit + j_bit + i_bit;

                            table.column_1.emplace_back(barretenberg::fr(input));
                            table.column_2.emplace_back(barretenberg::fr(0));
                            table.column_3.emplace_back(barretenberg::fr(output));
                        }
                    }
                }
            }
        }
    }

    table.column_1_step_size = barretenberg::fr(-(1 << 4));
    table.column_2_step_size = barretenberg::fr(0);
    table.column_3_step_size = barretenberg::fr(-table.size);

    table.get_values_from_key = &get_sha256_part_b_output_values_from_key;
    return table;
}

inline barretenberg::fr get_sha256_part_b_output_values_from_key(const uint256_t key)
{
    constexpr uint256_t base = 4;
    constexpr uint64_t bit_table[4]{
        0, // a + b + c = 0 => (a & b) ^ (a & c) ^ (b & c) = 0
        0, // a + b + c = 1 => (a & b) ^ (a & c) ^ (b & c) = 0
        1, // a + b + c = 2 => (a & b) ^ (a & c) ^ (b & c) = 1
        1, // a + b + c = 3 => (a & b) ^ (a & c) ^ (b & c) = 0
    };

    uint64_t accumulator = 0;
    uint64_t count = 0;
    uint256_t input = key;
    while (input > 0) {
        uint256_t slice = input % base;
        uint64_t bit = bit_table[static_cast<size_t>(slice.data[0])];
        accumulator += (bit << count);
        input -= slice;
        input /= base;
        ++count;
    }
    return barretenberg::fr(accumulator);
}

} // namespace sha256_tables
} // namespace waffle