#pragma once
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

#include <cstddef>
#include <cstdint>
#include <cstring>

#include "blake3s.hpp"

namespace blake3 {

// Right rotates 32 bit inputs
constexpr uint32_t rotr32(uint32_t w, uint32_t c)
{
    return (w >> c) | (w << (32 - c));
}

constexpr uint32_t load32(const uint8_t* src)
{
    return (static_cast<uint32_t>(src[0]) << 0) | (static_cast<uint32_t>(src[1]) << 8) |
           (static_cast<uint32_t>(src[2]) << 16) | (static_cast<uint32_t>(src[3]) << 24);
}

constexpr void load_key_words(const std::array<uint8_t, BLAKE3_KEY_LEN>& key, key_array& key_words)
{
    key_words[0] = load32(&key[0]);
    key_words[1] = load32(&key[4]);
    key_words[2] = load32(&key[8]);
    key_words[3] = load32(&key[12]);
    key_words[4] = load32(&key[16]);
    key_words[5] = load32(&key[20]);
    key_words[6] = load32(&key[24]);
    key_words[7] = load32(&key[28]);
}

constexpr void store32(uint8_t* dst, uint32_t w)
{
    dst[0] = static_cast<uint8_t>(w >> 0);
    dst[1] = static_cast<uint8_t>(w >> 8);
    dst[2] = static_cast<uint8_t>(w >> 16);
    dst[3] = static_cast<uint8_t>(w >> 24);
}

constexpr void store_cv_words(out_array& bytes_out, key_array& cv_words)
{
    store32(&bytes_out[0], cv_words[0]);
    store32(&bytes_out[4], cv_words[1]);
    store32(&bytes_out[8], cv_words[2]);
    store32(&bytes_out[12], cv_words[3]);
    store32(&bytes_out[16], cv_words[4]);
    store32(&bytes_out[20], cv_words[5]);
    store32(&bytes_out[24], cv_words[6]);
    store32(&bytes_out[28], cv_words[7]);
}

} // namespace blake3

#include "blake3s.tcc"

#endif