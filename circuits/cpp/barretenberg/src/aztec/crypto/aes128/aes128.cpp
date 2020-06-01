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

inline void add_round_key(uint8_t* state, const uint8_t* round_key, const size_t round)
{
    for (size_t i = 0; i < 16; i += 4) {
        for (size_t j = 0; j < 4; ++j) {
            state[i + j] ^= round_key[(round * 16U) + i + j];
        }
    }
}

inline void xor_with_iv(uint8_t* state, const uint8_t* iv)
{
    for (size_t i = 0; i < 16; ++i) {
        state[i] ^= iv[i];
    }
}

void sub_bytes(uint8_t* input)
{
    uint8_t i, j;
    for (i = 0; i < 4; ++i) {
        for (j = 0; j < 4; ++j) {
            input[j * 4 + i] = sbox[input[j * 4 + i]];
        }
    }
}

void inverse_sub_bytes(uint8_t* input)
{
    for (size_t i = 0; i < 4; ++i) {
        for (size_t j = 0; j < 4; ++j) {
            input[j * 4 + i] = sbox_inverse[input[j * 4 + i]];
        }
    }
}

void shift_rows(uint8_t* input)
{
    uint8_t temp;

    temp = input[0 * 4 + 1];
    input[0 * 4 + 1] = input[1 * 4 + 1];
    input[1 * 4 + 1] = input[2 * 4 + 1];
    input[2 * 4 + 1] = input[3 * 4 + 1];
    input[3 * 4 + 1] = temp;

    temp = input[0 * 4 + 2];
    input[0 * 4 + 2] = input[2 * 4 + 2];
    input[2 * 4 + 2] = temp;

    temp = input[1 * 4 + 2];
    input[1 * 4 + 2] = input[3 * 4 + 2];
    input[3 * 4 + 2] = temp;

    temp = input[0 * 4 + 3];
    input[0 * 4 + 3] = input[3 * 4 + 3];
    input[3 * 4 + 3] = input[2 * 4 + 3];
    input[2 * 4 + 3] = input[1 * 4 + 3];
    input[1 * 4 + 3] = temp;
}

static void inverse_shift_rows(uint8_t* input)
{
    uint8_t temp;

    temp = input[3 * 4 + 1];
    input[3 * 4 + 1] = input[2 * 4 + 1];
    input[2 * 4 + 1] = input[1 * 4 + 1];
    input[1 * 4 + 1] = input[0 * 4 + 1];
    input[0 * 4 + 1] = temp;

    temp = input[0 * 4 + 2];
    input[0 * 4 + 2] = input[2 * 4 + 2];
    input[2 * 4 + 2] = temp;

    temp = input[1 * 4 + 2];
    input[1 * 4 + 2] = input[3 * 4 + 2];
    input[3 * 4 + 2] = temp;

    temp = input[0 * 4 + 3];
    input[0 * 4 + 3] = input[1 * 4 + 3];
    input[1 * 4 + 3] = input[2 * 4 + 3];
    input[2 * 4 + 3] = input[3 * 4 + 3];
    input[3 * 4 + 3] = temp;
}

uint8_t xtime(const uint8_t x)
{
    return static_cast<uint8_t>((x << 1) ^ (((x >> 7) & 1) * 0x1b));
}

uint8_t gf2_8_mul(const uint8_t x, const uint8_t y)
{
    const uint8_t t0 = (uint8_t)((y & (uint8_t)1) * x);
    const uint8_t t1 = (uint8_t)(((y >> (uint8_t)1) & (uint8_t)1) * xtime(x));
    const uint8_t t2 = (uint8_t)(((y >> (uint8_t)2) & (uint8_t)1) * xtime(xtime(x)));
    const uint8_t t3 = (uint8_t)(((y >> (uint8_t)3) & (uint8_t)1) * xtime(xtime(xtime(x))));
    const uint8_t t4 = (uint8_t)(((y >> (uint8_t)4) & (uint8_t)1) * xtime(xtime(xtime(xtime(x)))));

    uint8_t out = t0 ^ t1 ^ t2 ^ t3 ^ t4;
    return out;
}

void mix_columns(uint8_t* input)
{
    for (uint8_t i = 0; i < 4; ++i) {
        uint8_t t = input[i * 4 + 0];
        uint8_t Tmp = input[i * 4 + 0] ^ input[i * 4 + 1] ^ input[i * 4 + 2] ^ input[i * 4 + 3];
        uint8_t Tm = input[i * 4 + 0] ^ input[i * 4 + 1];
        Tm = xtime(Tm);
        input[i * 4 + 0] ^= Tm ^ Tmp;
        Tm = input[i * 4 + 1] ^ input[i * 4 + 2];
        Tm = xtime(Tm);
        input[i * 4 + 1] ^= Tm ^ Tmp;
        Tm = input[i * 4 + 2] ^ input[i * 4 + 3];
        Tm = xtime(Tm);
        input[i * 4 + 2] ^= Tm ^ Tmp;
        Tm = input[i * 4 + 3] ^ t;
        Tm = xtime(Tm);
        input[i * 4 + 3] ^= Tm ^ Tmp;
    }
}

void inverse_mix_columns(uint8_t* input)
{
    for (uint8_t i = 0; i < 4; ++i) {
        uint8_t a = input[i * 4 + 0];
        uint8_t b = input[i * 4 + 1];
        uint8_t c = input[i * 4 + 2];
        uint8_t d = input[i * 4 + 3];

        input[i * 4 + 0] = gf2_8_mul(a, 0x0e) ^ gf2_8_mul(b, 0x0b) ^ gf2_8_mul(c, 0x0d) ^ gf2_8_mul(d, 0x09);
        input[i * 4 + 1] = gf2_8_mul(a, 0x09) ^ gf2_8_mul(b, 0x0e) ^ gf2_8_mul(c, 0x0b) ^ gf2_8_mul(d, 0x0d);
        input[i * 4 + 2] = gf2_8_mul(a, 0x0d) ^ gf2_8_mul(b, 0x09) ^ gf2_8_mul(c, 0x0e) ^ gf2_8_mul(d, 0x0b);
        input[i * 4 + 3] = gf2_8_mul(a, 0x0b) ^ gf2_8_mul(b, 0x0d) ^ gf2_8_mul(c, 0x09) ^ gf2_8_mul(d, 0x0e);
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

void aes128_inverse_cipher(uint8_t* input, const uint8_t* round_key)
{

    add_round_key(input, round_key, 10);

    for (size_t round = 9; round > 0; --round) {
        inverse_shift_rows(input);
        inverse_sub_bytes(input);
        add_round_key(input, round_key, round);
        inverse_mix_columns(input);
    }
    inverse_shift_rows(input);
    inverse_sub_bytes(input);
    add_round_key(input, round_key, 0);
}

void aes128_cipher(uint8_t* state, const uint8_t* round_key)
{
    add_round_key(state, round_key, 0);

    for (uint8_t round = 1; round < 10; ++round) {
        sub_bytes(state);
        shift_rows(state);
        mix_columns(state);
        add_round_key(state, round_key, round);
    }

    sub_bytes(state);
    shift_rows(state);
    add_round_key(state, round_key, 10);
}

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

void decrypt_buffer_cbc(uint8_t* buffer, uint8_t* iv, const uint8_t* key, const size_t length)
{
    uint8_t round_key[176];
    expand_key(key, round_key);
    uint8_t block_state[16]{};
    const size_t num_blocks = (length / 16);

    uint8_t next_iv[16]{};
    for (size_t i = 0; i < num_blocks; ++i) {
        memcpy((void*)block_state, (void*)(buffer + (i * 16)), 16);
        memcpy((void*)next_iv, (void*)block_state, 16);
        aes128_inverse_cipher(block_state, round_key);
        xor_with_iv(block_state, iv);
        memcpy((void*)(buffer + (i * 16)), (void*)block_state, 16);
        memcpy((void*)iv, (void*)next_iv, 16);
    }
}

} // namespace aes128
} // namespace crypto