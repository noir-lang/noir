/*
    BLAKE3 reference source code package - C implementations

    Intellectual property:

    The Rust code is copyright Jack O'Connor, 2019-2020.
    The C code is copyright Samuel Neves and Jack O'Connor, 2019-2020.
    The assembly code is copyright Samuel Neves, 2019-2020.

    This work is released into the public domain with CC0 1.0. Alternatively, it is licensed under the Apache
   License 2.0.

    - CC0 1.0 Universal : http://creativecommons.org/publicdomain/zero/1.0
    - Apache 2.0        : http://www.apache.org/licenses/LICENSE-2.0

    More information about the BLAKE3 hash function can be found at
    https://github.com/BLAKE3-team/BLAKE3.
*/

#ifndef BLAKE3_IMPL_H
#define BLAKE3_IMPL_H

#include <assert.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include "blake3s.hpp"

namespace blake3 {

// Right rotates 32 bit inputs
uint32_t rotr32(uint32_t w, uint32_t c)
{
    return (w >> c) | (w << (32 - c));
}

uint32_t load32(const void* src)
{
    const uint8_t* p = (const uint8_t*)src;
    return ((uint32_t)(p[0]) << 0) | ((uint32_t)(p[1]) << 8) | ((uint32_t)(p[2]) << 16) | ((uint32_t)(p[3]) << 24);
}

void load_key_words(const uint8_t key[BLAKE3_KEY_LEN], uint32_t key_words[8])
{
    key_words[0] = load32(&key[0 * 4]);
    key_words[1] = load32(&key[1 * 4]);
    key_words[2] = load32(&key[2 * 4]);
    key_words[3] = load32(&key[3 * 4]);
    key_words[4] = load32(&key[4 * 4]);
    key_words[5] = load32(&key[5 * 4]);
    key_words[6] = load32(&key[6 * 4]);
    key_words[7] = load32(&key[7 * 4]);
}

void store32(void* dst, uint32_t w)
{
    uint8_t* p = (uint8_t*)dst;
    p[0] = (uint8_t)(w >> 0);
    p[1] = (uint8_t)(w >> 8);
    p[2] = (uint8_t)(w >> 16);
    p[3] = (uint8_t)(w >> 24);
}

void store_cv_words(uint8_t bytes_out[32], uint32_t cv_words[8])
{
    store32(&bytes_out[0 * 4], cv_words[0]);
    store32(&bytes_out[1 * 4], cv_words[1]);
    store32(&bytes_out[2 * 4], cv_words[2]);
    store32(&bytes_out[3 * 4], cv_words[3]);
    store32(&bytes_out[4 * 4], cv_words[4]);
    store32(&bytes_out[5 * 4], cv_words[5]);
    store32(&bytes_out[6 * 4], cv_words[6]);
    store32(&bytes_out[7 * 4], cv_words[7]);
}

} // namespace blake3

#endif /* BLAKE3_IMPL_H */
