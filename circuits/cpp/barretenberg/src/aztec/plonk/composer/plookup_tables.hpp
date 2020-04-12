#pragma once

namespace waffle {

namespace aes_tables {
static constexpr uint64_t sparse_base = 9;
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

constexpr uint64_t map_into_sparse_form(const uint8_t input)
{
    uint64_t out = 0UL;
    uint64_t converted = (uint64_t)input;
    uint64_t base_accumulator = 1;
    for (uint64_t i = 0; i < 8; ++i) {
        uint64_t sparse_bit = ((converted >> i) & 1ULL);
        out += (sparse_bit * base_accumulator);
        base_accumulator *= sparse_base;
    }
    return out;
}

constexpr uint8_t map_from_sparse_form(const uint64_t input)
{
    uint64_t target = input;
    uint64_t output = 0;

    uint64_t count = 0;
    while (target > 0) {
        uint64_t slice = (target % sparse_base);
        uint64_t bit = slice & 1ULL;
        output += (bit << count);
        ++count;
        target -= slice;
        target /= sparse_base;
    }

    return (uint8_t)output;
}

inline void generate_aes_sparse_map(const size_t,
                                    std::vector<fr>& column_1,
                                    std::vector<fr>& column_2,
                                    std::vector<fr>& column_3)
{
    const uint64_t entries = 256;
    for (uint64_t i = 0; i < entries; ++i) {
        uint64_t left = i;
        uint64_t right = map_into_sparse_form((uint8_t)i);
        column_1.emplace_back(fr(left));
        column_2.emplace_back(fr(0));
        column_3.emplace_back(fr(right));
    }
}

inline void generate_aes_sparse_normalization_map(const size_t,
                                                  std::vector<fr>& column_1,
                                                  std::vector<fr>& column_2,
                                                  std::vector<fr>& column_3)
{
    for (uint64_t i = 0; i < sparse_base; ++i) {
        uint64_t i_raw = i * sparse_base * sparse_base * sparse_base;
        uint64_t i_normalized = ((i & 1UL) == 1UL) * sparse_base * sparse_base * sparse_base;
        for (uint64_t j = 0; j < sparse_base; ++j) {
            uint64_t j_raw = j * sparse_base * sparse_base;
            uint64_t j_normalized = ((j & 1UL) == 1UL) * sparse_base * sparse_base;
            for (uint64_t k = 0; k < sparse_base; ++k) {
                uint64_t k_raw = k * sparse_base;
                uint64_t k_normalized = ((k & 1UL) == 1UL) * sparse_base;
                for (uint64_t m = 0; m < sparse_base; ++m) {
                    uint64_t m_raw = m;
                    uint64_t m_normalized = ((m & 1UL) == 1UL);
                    uint64_t left = i_raw + j_raw + k_raw + m_raw;
                    uint64_t right = i_normalized + j_normalized + k_normalized + m_normalized;
                    column_1.emplace_back(left);
                    column_2.emplace_back(fr(0));
                    column_3.emplace_back(right);
                }
            }
        }
    }
}

inline void generate_aes_sbox_map(const size_t,
                                  std::vector<fr>& column_1,
                                  std::vector<fr>& column_2,
                                  std::vector<fr>& column_3)
{
    const uint64_t num_entries = 256;
    for (uint64_t i = 0; i < num_entries; ++i) {
        uint64_t first = map_into_sparse_form((uint8_t)i);
        uint8_t sbox_value = sbox[(uint8_t)i];
        uint8_t swizzled = ((uint8_t)(sbox_value << 1) ^ (uint8_t)(((sbox_value >> 7) & 1) * 0x1b));
        uint64_t second = map_into_sparse_form(sbox_value);
        uint64_t third = map_into_sparse_form((uint8_t)(sbox_value ^ swizzled));

        column_1.emplace_back(fr(first));
        column_2.emplace_back(fr(second));
        column_3.emplace_back(fr(third));
    }
}
} // namespace aes_tables
} // namespace waffle