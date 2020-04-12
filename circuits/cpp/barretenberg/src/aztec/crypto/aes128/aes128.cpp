#include "aes128.hpp"

#include <cstddef>
#include <cstdint>
#include <array>
#include "memory.h"

#include <iostream>
namespace crypto {
namespace aes128 {

namespace {
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

static constexpr uint8_t round_constants[11] = { 0x8d, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36 };

constexpr std::array<uint64_t, 256> compute_sparse_map()
{
    std::array<uint64_t, 256> out{};
    for (size_t i = 0; i < 256; ++i) {
        out[i] = map_into_sparse_form((uint8_t)i);
    }
    return out;
}

static constexpr std::array<uint64_t, 256> sparse_map = compute_sparse_map();

struct sparse_sbox_pair {
    uint64_t first;
    uint64_t second;
};

constexpr std::array<sparse_sbox_pair, 256> compute_sparse_sbox_map()
{
    std::array<sparse_sbox_pair, 256> result{};
    for (size_t i = 0; i < 256; ++i) {
        uint8_t left = sbox[i];
        uint8_t right = ((uint8_t)(left << 1) ^ (uint8_t)(((left >> 7) & 1) * 0x1b));
        result[i] = { map_into_sparse_form(left), map_into_sparse_form((uint8_t)(left ^ right)) };
    }
    return result;
}

static constexpr std::array<sparse_sbox_pair, 256> sparse_sbox_map = compute_sparse_sbox_map();

inline void mix_column(sparse_sbox_pair* column_pairs)
{
    uint64_t t0 = column_pairs[0].second + column_pairs[1].second + column_pairs[2].first + column_pairs[3].first;
    uint64_t t1 = column_pairs[1].second + column_pairs[2].second + column_pairs[0].first + column_pairs[3].first;
    uint64_t t2 = column_pairs[2].second + column_pairs[3].second + column_pairs[0].first + column_pairs[1].first;
    uint64_t t3 = column_pairs[3].second + column_pairs[0].second + column_pairs[1].first + column_pairs[2].first;
    column_pairs[0].first += t0;
    column_pairs[1].first += t1;
    column_pairs[2].first += t2;
    column_pairs[3].first += t3;
}

inline void mix_columns(sparse_sbox_pair* state_pairs)
{
    mix_column(state_pairs);
    mix_column(state_pairs + 4);
    mix_column(state_pairs + 8);
    mix_column(state_pairs + 12);
}

inline void shift_rows(sparse_sbox_pair* state)
{
    sparse_sbox_pair temp = state[1];
    state[1] = state[5];
    state[5] = state[9];
    state[9] = state[13];
    state[13] = temp;

    temp = state[2];
    state[2] = state[10];
    state[10] = temp;
    temp = state[6];
    state[6] = state[14];
    state[14] = temp;

    temp = state[3];
    state[3] = state[15];
    state[15] = state[11];
    state[11] = state[7];
    state[7] = temp;
}

inline void sub_bytes(uint64_t* sparse_state, sparse_sbox_pair* new_state)
{
    for (size_t i = 0; i < 4; ++i) {
        for (size_t j = 0; j < 16; j += 4) {
            auto converted = map_from_sparse_form(sparse_state[j + i]);
            new_state[j + i] = sparse_sbox_map[converted];
        }
    }
}

inline void sub_bytes(sparse_sbox_pair* state)
{
    for (size_t i = 0; i < 4; ++i) {
        for (size_t j = 0; j < 16; j += 4) {
            auto converted = map_from_sparse_form(state[j + i].first);
            state[j + i] = sparse_sbox_map[converted];
        }
    }
}

inline void add_round_key(sparse_sbox_pair* sparse_state, uint64_t* sparse_round_key, uint8_t round)
{
    for (size_t i = 0; i < 16; i += 4) {
        for (size_t j = 0; j < 4; ++j) {
            sparse_state[i + j].first += sparse_round_key[(round * 16U) + i + j];
        }
    }
}

struct sbox_pair {
    uint8_t first;
    uint8_t second;
};

constexpr std::array<sbox_pair, 256> compute_sbox_map()
{
    std::array<sbox_pair, 256> result{};
    for (size_t i = 0; i < 256; ++i) {
        uint8_t left = sbox[i];
        uint8_t right = ((uint8_t)(left << 1) ^ (uint8_t)(((left >> 7) & 1) * 0x1b));
        result[i] = { left, (uint8_t)(left ^ right) };
    }
    return result;
}

static constexpr std::array<sbox_pair, 256> sbox_map = compute_sbox_map();

inline void mix_column(sbox_pair* column_pairs)
{
    uint8_t t0 = column_pairs[0].second ^ column_pairs[1].second ^ column_pairs[2].first ^ column_pairs[3].first;
    uint8_t t1 = column_pairs[1].second ^ column_pairs[2].second ^ column_pairs[0].first ^ column_pairs[3].first;
    uint8_t t2 = column_pairs[2].second ^ column_pairs[3].second ^ column_pairs[0].first ^ column_pairs[1].first;
    uint8_t t3 = column_pairs[3].second ^ column_pairs[0].second ^ column_pairs[1].first ^ column_pairs[2].first;
    column_pairs[0].first ^= t0;
    column_pairs[1].first ^= t1;
    column_pairs[2].first ^= t2;
    column_pairs[3].first ^= t3;
}

inline void mix_columns(sbox_pair* state_pairs)
{
    mix_column(state_pairs);
    mix_column(state_pairs + 4);
    mix_column(state_pairs + 8);
    mix_column(state_pairs + 12);
}

inline void shift_rows(sbox_pair* state)
{
    sbox_pair temp = state[1];
    state[1] = state[5];
    state[5] = state[9];
    state[9] = state[13];
    state[13] = temp;

    temp = state[2];
    state[2] = state[10];
    state[10] = temp;
    temp = state[6];
    state[6] = state[14];
    state[14] = temp;

    temp = state[3];
    state[3] = state[15];
    state[15] = state[11];
    state[11] = state[7];
    state[7] = temp;
}

inline void sub_bytes(uint8_t* state, sbox_pair* new_state)
{
    for (size_t i = 0; i < 4; ++i) {
        for (size_t j = 0; j < 16; j += 4) {
            new_state[j + i] = sbox_map[state[j + i]];
        }
    }
}

inline void add_round_key(uint8_t* state, uint8_t* round_key, uint8_t round)
{
    for (size_t i = 0; i < 16; i += 4) {
        for (size_t j = 0; j < 4; ++j) {
            state[i + j] ^= round_key[(round * 16U) + i + j];
        }
    }
}

inline void add_round_key(uint8_t* state, sbox_pair* state_pairs, uint8_t* round_key, uint8_t round)
{
    for (size_t i = 0; i < 16; i += 4) {
        for (size_t j = 0; j < 4; ++j) {
            state_pairs[i + j].first ^= round_key[(round * 16U) + i + j];
        }
    }
    for (size_t i = 0; i < 16; ++i) {
        state[i] = state_pairs[i].first;
    }
}

inline void xor_with_iv(uint8_t* state, const uint8_t* iv)
{
    for (size_t i = 0; i < 16; ++i) {
        state[i] ^= iv[i];
    }
}

} // namespace

void expand_key(const uint8_t* key, uint8_t* round_key)
{
    uint8_t temp[4]{};

    for (size_t i = 0; i < 16; i += 4) {
        round_key[i] = key[i];
        round_key[i + 1] = key[i + 1];
        round_key[i + 2] = key[i + 2];
        round_key[i + 3] = key[i + 3];
    }

    for (size_t i = 4; i < 44; ++i) {
        size_t k = (i - 1) * 4;
        temp[0] = round_key[k];
        temp[1] = round_key[k + 1];
        temp[2] = round_key[k + 2];
        temp[3] = round_key[k + 3];

        if ((i & 0x03) == 0) {
            const uint8_t t = temp[0];
            temp[0] = temp[1];
            temp[1] = temp[2];
            temp[2] = temp[3];
            temp[3] = t;

            temp[0] = sbox[temp[0]];
            temp[1] = sbox[temp[1]];
            temp[2] = sbox[temp[2]];
            temp[3] = sbox[temp[3]];

            temp[0] = temp[0] ^ round_constants[i >> 2];
        }
        size_t j = i * 4;
        k = (i - 4) * 4;
        round_key[j] = round_key[k] ^ temp[0];
        round_key[j + 1] = round_key[k + 1] ^ temp[1];
        round_key[j + 2] = round_key[k + 2] ^ temp[2];
        round_key[j + 3] = round_key[k + 3] ^ temp[3];
    }
}

void aes128_cipher(uint8_t* input, uint8_t* round_key)
{
    uint64_t sparse_round_key[176];
    sparse_sbox_pair state[16];

    for (size_t i = 0; i < 16; ++i) {
        state[i] = { map_into_sparse_form(input[i]), 0 };
    }
    for (size_t i = 0; i < 176; ++i) {
        sparse_round_key[i] = map_into_sparse_form(round_key[i]);
    }

    add_round_key(state, sparse_round_key, 0);
    for (size_t i = 0; i < 16; ++i) {
        state[i].first = normalize_sparse_form(state[i].first);
    }

    for (uint8_t round = 1; round < 10; ++round) {
        sub_bytes(state);
        shift_rows(state);
        mix_columns(state);
        add_round_key(state, sparse_round_key, round);
        for (size_t i = 0; i < 16; ++i) {
            state[i].first = normalize_sparse_form(state[i].first);
        }
    }

    sub_bytes(state);
    shift_rows(state);
    add_round_key(state, sparse_round_key, 10);

    for (size_t i = 0; i < 16; ++i) {
        input[i] = map_from_sparse_form(state[i].first);
    }
}

// void aes128_cipher(uint8_t* state, uint8_t* round_key)
// {
//     uint64_t sparse_round_key[176]{};
//     for (size_t i = 0; i < 176; ++i) {
//         sparse_round_key[i] = map_into_sparse_form(round_key[i]);
//     }

//     uint64_t sparse_state[16];
//     sparse_sbox_pair sparse_pairs[16];
//     add_round_key(state, round_key, 0);

//     sbox_pair state_pairs[16];
//     for (uint8_t round = 1; round < 10; ++round) {
//         for (size_t i = 0; i < 16; ++i) {
//             sparse_state[i] = map_into_sparse_form(state[i]);
//         }
//         sub_bytes(sparse_state, sparse_pairs);
//         // for (size_t i = 0; i < 16; ++i) {
//         //     sparse_pairs[i] = { map_into_sparse_form(state_pairs[i].first),
//         //                         map_into_sparse_form(state_pairs[i].second) };
//         // }
//         shift_rows(sparse_pairs);

//         mix_columns(sparse_pairs);
//         // for (size_t i = 0; i < 16; ++i) {
//         //     sparse_state[i] = sparse_pairs[i].first;
//         // }
//         add_round_key(sparse_pairs, sparse_round_key, round);

//         for (size_t i = 0; i < 16; ++i) {
//             state_pairs[i] = { map_from_sparse_form(sparse_pairs[i].first),
//                                map_from_sparse_form(sparse_pairs[i].second) };
//         }
//         for (size_t i = 0; i < 16; ++i) {
//             state[i] = state_pairs[i].first;
//         }
//     }

//     sub_bytes(state, state_pairs);
//     shift_rows(state_pairs);
//     add_round_key(state, state_pairs, round_key, 10);
// }

// void aes128_cipher(uint8_t* state, uint8_t* round_key)
// {
//     sparse_sbox_pair sparse_pairs[16];
//     add_round_key(state, round_key, 0);

//     sbox_pair state_pairs[16];
//     for (uint8_t round = 1; round < 10; ++round) {
//         sub_bytes(state, state_pairs);
//         shift_rows(state_pairs);

//         for (size_t i = 0; i < 16; ++i)
//         {
//             sparse_pairs[i] = { map_into_sparse_form(state_pairs[i].first),
//             map_into_sparse_form(state_pairs[i].second) };
//         }
//         mix_columns(state_pairs);
//         add_round_key(state, state_pairs, round_key, round);
//     }

//     sub_bytes(state, state_pairs);
//     shift_rows(state_pairs);
//     add_round_key(state, state_pairs, round_key, 10);
// }

void encrypt_buffer_cbc(uint8_t* buffer, uint8_t* iv, const uint8_t* key, const size_t length)
{
    uint8_t round_key[176];
    expand_key(key, round_key);

    uint8_t block_state[16]{};

    const size_t num_blocks = (length / 16);

    for (size_t i = 0; i < num_blocks; ++i) {
        memcpy((void*)block_state, (void*)(buffer + (i * 16)), 16);
        xor_with_iv(block_state, iv);

        aes128_cipher(block_state, round_key);

        memcpy((void*)(buffer + (i * 16)), (void*)block_state, 16);
        memcpy((void*)iv, (void*)block_state, 16);
    }
}

} // namespace aes128
} // namespace crypto