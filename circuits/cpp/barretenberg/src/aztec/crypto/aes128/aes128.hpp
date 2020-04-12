#pragma once

/**
 * AES-128 Cipher
 *
 * Implements AES-128 block cipher, and buffer encryption using cbc cipher chaining
 *
 * Implementation is not optimized and mimics a planned Plonk implementation using Plookups
 *
 * Originally based off of tiny-AES by @kokke : https://github.com/kokke/tiny-AES-c
 **/

#include <cstddef>
#include <cstdint>
#include <array>
#include "memory.h"

#include <iostream>
namespace crypto {
namespace aes128 {

void expand_key(const uint8_t* key, uint8_t* round_key);
void aes128_cipher(uint8_t* state, uint8_t* round_key);
void encrypt_buffer_cbc(uint8_t* buffer, uint8_t* iv, const uint8_t* key, const size_t length);

constexpr uint64_t map_into_sparse_form(const uint8_t input)
{
    uint64_t out = 0UL;
    uint64_t converted = (uint64_t)input;
    for (uint64_t i = 0; i < 8; ++i) {
        uint64_t sparse_bit = ((converted >> i) & 1ULL) << (i * 8);
        out += sparse_bit;
    }
    return out;
}

constexpr uint8_t map_from_sparse_form(const uint64_t input)
{
    uint64_t output = 0;
    for (uint64_t i = 0; i < 8; ++i) {
        uint64_t byte = (input >> (i * 8)) & 0x255ULL;
        output += (byte & 1ULL) << i;
    }
    return (uint8_t)output;
}

constexpr uint64_t normalize_sparse_form(const uint64_t input)
{
    uint64_t output = 0;
    for (uint64_t i = 0; i < 8; ++i) {
        uint64_t byte = (input >> (i * 8)) & 0x255ULL;
        output += ((byte & 1ULL) << (i * 8));
    }
    return output;
}
} // namespace aes128
} // namespace crypto