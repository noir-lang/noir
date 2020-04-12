#include "aes128.hpp"

#include <cstddef>
#include <cstdint>
#include <array>
#include "memory.h"

#include <iostream>
namespace crypto {
namespace aes128 {

namespace {


static constexpr uint8_t round_constants[11] = { 0x8d, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36 };

static constexpr uint64_t sparse_round_constants[11] = {
    map_into_sparse_form(round_constants[0]),  map_into_sparse_form(round_constants[1]),
    map_into_sparse_form(round_constants[2]),  map_into_sparse_form(round_constants[3]),
    map_into_sparse_form(round_constants[4]),  map_into_sparse_form(round_constants[5]),
    map_into_sparse_form(round_constants[6]),  map_into_sparse_form(round_constants[7]),
    map_into_sparse_form(round_constants[8]),  map_into_sparse_form(round_constants[9]),
    map_into_sparse_form(round_constants[10]),
};

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

inline uint64_t sparse_sbox(uint64_t input)
{
    return sparse_sbox_map[map_from_sparse_form(input)].first;
}

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

void expand_key(const uint8_t* key, uint64_t* round_key)
{
    uint64_t temp[4]{};
    uint64_t temp_add_counts[4]{};
    for (size_t i = 0; i < 16; i += 4) {
        round_key[i] = map_into_sparse_form(key[i]);
        round_key[i + 1] = map_into_sparse_form(key[i + 1]);
        round_key[i + 2] = map_into_sparse_form(key[i + 2]);
        round_key[i + 3] = map_into_sparse_form(key[i + 3]);
    }

    uint64_t add_counts[176];
    for (size_t i = 0; i < 176; ++i) {
        add_counts[i] = 1;
    }

    uint64_t normalize_count = 0;
    for (size_t i = 4; i < 44; ++i) {
        size_t k = (i - 1) * 4;

        temp_add_counts[0] = add_counts[k + 0];
        temp_add_counts[1] = add_counts[k + 1];
        temp_add_counts[2] = add_counts[k + 2];
        temp_add_counts[3] = add_counts[k + 3];

        temp[0] = round_key[k];
        temp[1] = round_key[k + 1];
        temp[2] = round_key[k + 2];
        temp[3] = round_key[k + 3];
        if ((i & 0x03) == 0) {
            const uint64_t t = temp[0];
            temp[0] = temp[1];
            temp[1] = temp[2];
            temp[2] = temp[3];
            temp[3] = t;

            temp[0] = sparse_sbox(temp[0]);
            temp[1] = sparse_sbox(temp[1]);
            temp[2] = sparse_sbox(temp[2]);
            temp[3] = sparse_sbox(temp[3]);

            temp[0] = temp[0] + sparse_round_constants[i >> 2];
            ++temp_add_counts[0];
        }
        size_t j = i * 4;
        k = (i - 4) * 4;
        round_key[j] = round_key[k] + temp[0];
        round_key[j + 1] = round_key[k + 1] + temp[1];
        round_key[j + 2] = round_key[k + 2] + temp[2];
        round_key[j + 3] = round_key[k + 3] + temp[3];

        add_counts[j] = add_counts[k] + temp_add_counts[0];
        add_counts[j + 1] = add_counts[k + 1] + temp_add_counts[1];
        add_counts[j + 2] = add_counts[k + 2] + temp_add_counts[2];
        add_counts[j + 3] = add_counts[k + 3] + temp_add_counts[3];

        constexpr uint64_t target = 3;
        if (add_counts[j] > target || (add_counts[j] > 1 && (j & 12) == 12)) {
            round_key[j] = normalize_sparse_form(round_key[j]);
            add_counts[j] = 1;
            ++normalize_count;
        }
        if (add_counts[j + 1] > target || (add_counts[j + 1] > 1 && ((j + 1) & 12) == 12)) {
            round_key[j + 1] = normalize_sparse_form(round_key[j + 1]);
            add_counts[j + 1] = 1;
            ++normalize_count;
        }
        if (add_counts[j + 2] > target || (add_counts[j + 2] > 1 && ((j + 2) & 12) == 12)) {
            round_key[j + 2] = normalize_sparse_form(round_key[j + 2]);
            add_counts[j + 2] = 1;
            ++normalize_count;
        }
        if (add_counts[j + 3] > target || (add_counts[j + 3] > 1 && ((j + 3) & 12) == 12)) {
            round_key[j + 3] = normalize_sparse_form(round_key[j + 3]);
            add_counts[j + 3] = 1;
            ++normalize_count;
        }

        // if (add_counts[j] > target || add_counts[j + 1] > target || add_counts[j + 2] > target ||
        //     add_counts[j + 3] > target) {
        //     std::cout << "normalizing at round i = " << i << std::endl;
        //     // std::cout << "normalizing indices " << j << ", " << j + 1 << ", " << j + 2 << ", " << j + 3 <<
        //     std::endl;

        //     round_key[j] = normalize_sparse_form(round_key[j]);
        //     round_key[j + 1] = normalize_sparse_form(round_key[j + 1]);
        //     round_key[j + 2] = normalize_sparse_form(round_key[j + 2]);
        //     round_key[j + 3] = normalize_sparse_form(round_key[j + 3]);

        //     add_counts[j] = 1;
        //     add_counts[j + 1] = 1;
        //     add_counts[j + 2] = 1;
        //     add_counts[j + 3] = 1;
        // }
        // if ((i & 0x02) == 2) {
        //     round_key[j] = normalize_sparse_form(round_key[j]);
        //     round_key[j + 1] = normalize_sparse_form(round_key[j + 1]);
        //     round_key[j + 2] = normalize_sparse_form(round_key[j + 2]);
        //     round_key[j + 3] = normalize_sparse_form(round_key[j + 3]);

        //     add_counts[j] = 1;
        //     add_counts[j + 1] = 1;
        //     add_counts[j + 2] = 1;
        //     add_counts[j + 3] = 1;
        // }
    }

    // for (size_t i = 0; i < 176; ++i) {
    //     std::cout << "add_counts[" << i << "] = " << add_counts[i] << std::endl;
    // }
    std::cout << "num normalizes = " << normalize_count << std::endl;
}

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

void aes128_cipher(uint8_t* input, uint64_t* sparse_round_key)
{
    sparse_sbox_pair state[16];

    for (size_t i = 0; i < 16; ++i) {
        state[i] = { map_into_sparse_form(input[i]), 0 };
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
    uint64_t round_key[176];
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