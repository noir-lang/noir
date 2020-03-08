/* ethash: C/C++ implementation of Ethash, the Ethereum Proof of Work algorithm.
 * Copyright 2018-2019 Pawel Bylica.
 * Licensed under the Apache License, Version 2.0.
 */

#include "keccak.hpp"

#include "./hash_types.hpp"

#if _MSC_VER
#include <string.h>
#define __builtin_memcpy memcpy
#endif

#if _WIN32
/* On Windows assume little endian. */
#define __LITTLE_ENDIAN 1234
#define __BIG_ENDIAN 4321
#define __BYTE_ORDER __LITTLE_ENDIAN
#elif __APPLE__
#include <machine/endian.h>
#else
#include <endian.h>
#endif

#if __BYTE_ORDER == __LITTLE_ENDIAN
#define to_le64(X) X
#else
#define to_le64(X) __builtin_bswap64(X)
#endif

#if __BYTE_ORDER == __LITTLE_ENDIAN
#define to_be64(X) __builtin_bswap64(X)
#else
#define to_be64(X) X
#endif

/** Loads 64-bit integer from given memory location as little-endian number. */
static inline uint64_t load_le(const uint8_t* data)
{
    /* memcpy is the best way of expressing the intention. Every compiler will
       optimize is to single load instruction if the target architecture
       supports unaligned memory access (GCC and clang even in O0).
       This is great trick because we are violating C/C++ memory alignment
       restrictions with no performance penalty. */
    uint64_t word;
    __builtin_memcpy(&word, data, sizeof(word));
    return to_le64(word);
}

static inline void keccak(uint64_t* out, size_t bits, const uint8_t* data, size_t size)
{
    static const size_t word_size = sizeof(uint64_t);
    const size_t hash_size = bits / 8;
    const size_t block_size = (1600 - bits * 2) / 8;

    size_t i;
    uint64_t* state_iter;
    uint64_t last_word = 0;
    uint8_t* last_word_iter = (uint8_t*)&last_word;

    uint64_t state[25] = { 0 };

    while (size >= block_size)
    {
        for (i = 0; i < (block_size / word_size); ++i)
        {
            state[i] ^= load_le(data);
            data += word_size;
        }

        ethash_keccakf1600(state);

        size -= block_size;
    }

    state_iter = state;

    while (size >= word_size)
    {
        *state_iter ^= load_le(data);
        ++state_iter;
        data += word_size;
        size -= word_size;
    }

    while (size > 0)
    {
        *last_word_iter = *data;
        ++last_word_iter;
        ++data;
        --size;
    }
    *last_word_iter = 0x01;
    *state_iter ^= to_le64(last_word);

    state[(block_size / word_size) - 1] ^= 0x8000000000000000;

    ethash_keccakf1600(state);

    for (i = 0; i < (hash_size / word_size); ++i)
        out[i] = to_le64(state[i]);
}

struct keccak256 ethash_keccak256(const uint8_t* data, size_t size) NOEXCEPT
{
    struct keccak256 hash;
    keccak(hash.word64s, 256, data, size);
    return hash;
}

struct keccak256 hash_field_elements(const uint64_t* limbs, size_t num_elements)
{
    uint8_t input_buffer[num_elements * 32];

    for (size_t i = 0; i < num_elements; ++i)
    {
        for (size_t j = 0; j < 4; ++j)
        {
            uint64_t word = (limbs[i * 4 + j]);
            size_t idx = i * 32 + j * 8;
            input_buffer[idx] = (uint8_t)((word >> 56) & 0xff);
            input_buffer[idx + 1] = (uint8_t)((word >> 48) & 0xff);
            input_buffer[idx + 2] = (uint8_t)((word >> 40) & 0xff);
            input_buffer[idx + 3] = (uint8_t)((word >> 32) & 0xff);
            input_buffer[idx + 4] = (uint8_t)((word >> 24) & 0xff);
            input_buffer[idx + 5] = (uint8_t)((word >> 16) & 0xff);
            input_buffer[idx + 6] = (uint8_t)((word >> 8) & 0xff);
            input_buffer[idx + 7] = (uint8_t)(word & 0xff);
        }
    }

    return ethash_keccak256(input_buffer, num_elements * 32);
}

struct keccak256 hash_field_element(const uint64_t* limb)
{
    return hash_field_elements(limb, 1);
}
