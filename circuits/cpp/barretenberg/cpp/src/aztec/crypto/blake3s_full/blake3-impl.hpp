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

namespace blake3_full {

// This C implementation tries to support recent versions of GCC, Clang, and
// MSVC.
#if defined(_MSC_VER)
#define INLINE static __forceinline
#else
#define INLINE static inline __attribute__((always_inline))
#endif

#if defined(__x86_64__) || defined(_M_X64)
#define IS_X86
#define IS_X86_64
#endif

#if defined(__i386__) || defined(_M_IX86)
#define IS_X86
#define IS_X86_32
#endif

#if defined(IS_X86)
#if defined(_MSC_VER)
#include <intrin.h>
#endif
#include <immintrin.h>
#endif

// #if defined(IS_X86)
// #define MAX_SIMD_DEGREE 16
// #elif defined(BLAKE3_USE_NEON)
// #define MAX_SIMD_DEGREE 4
// #else
#define MAX_SIMD_DEGREE 1
// #endif

// There are some places where we want a static size that's equal to the
// MAX_SIMD_DEGREE, but also at least 2.
#define MAX_SIMD_DEGREE_OR_2 (MAX_SIMD_DEGREE > 2 ? MAX_SIMD_DEGREE : 2)

// The dynamically detected SIMD degree of the current platform.
/*
 * Commenting out unnecessary parts as we currently don't need SIMD fo
 * different hardwares. To be revisited later.
 *
 */
size_t blake3_simd_degree(void)
{
    return 1;
    // #if defined(IS_X86)
    //   const enum cpu_feature features = get_cpu_features();
    //   MAYBE_UNUSED(features);
    // #if !defined(BLAKE3_NO_AVX512)
    //   if ((features & (AVX512F|AVX512VL)) == (AVX512F|AVX512VL)) {
    //     return 16;
    //   }
    // #endif
    // #if !defined(BLAKE3_NO_AVX2)
    //   if (features & AVX2) {
    //     return 8;
    //   }
    // #endif
    // #if !defined(BLAKE3_NO_SSE41)
    //   if (features & SSE41) {
    //     return 4;
    //   }
    // #endif
    // #if !defined(BLAKE3_NO_SSE2)
    //   if (features & SSE2) {
    //     return 4;
    //   }
    // #endif
    // #endif
    // #if defined(BLAKE3_USE_NEON)
    //   return 4;
    // #endif
    //   return 1;
}

/*----------------------------------------------------------------
 *
 * Commenting out as we currently don't need SIMD for different hardwares.
 * To be revisited later.
 *

enum cpu_feature get_cpu_features() {
  if (g_cpu_features != UNDEFINED) {
    return g_cpu_features;
  } else {
#if defined(IS_X86)
    uint32_t regs[4] = {0};
    uint32_t *eax = &regs[0], *ebx = &regs[1], *ecx = &regs[2], *edx = &regs[3];
    (void)edx;
    enum cpu_feature features = 0;
    cpuid(regs, 0);
    const int max_id = *eax;
    cpuid(regs, 1);
#if defined(__amd64__) || defined(_M_X64)
    features |= SSE2;
#else
    if (*edx & (1UL << 26))
      features |= SSE2;
#endif
    if (*ecx & (1UL << 0))
      features |= SSSE3;
    if (*ecx & (1UL << 19))
      features |= SSE41;

    if (*ecx & (1UL << 27)) { // OSXSAVE
      const uint64_t mask = xgetbv();
      if ((mask & 6) == 6) { // SSE and AVX states
        if (*ecx & (1UL << 28))
          features |= AVX;
        if (max_id >= 7) {
          cpuidex(regs, 7, 0);
          if (*ebx & (1UL << 5))
            features |= AVX2;
          if ((mask & 224) == 224) { // Opmask, ZMM_Hi256, Hi16_Zmm
            if (*ebx & (1UL << 31))
              features |= AVX512VL;
            if (*ebx & (1UL << 16))
              features |= AVX512F;
          }
        }
      }
    }
    g_cpu_features = features;
    return features;
#else
    // How to detect NEON?
    return 0;
#endif
  }
}
----------------------------------------------------------------*/

/* Find index of the highest set bit */
/* x is assumed to be nonzero.       */
static unsigned int highest_one(uint64_t x)
{
#if defined(__GNUC__) || defined(__clang__)
    return uint32_t(63) ^ uint32_t(__builtin_clzll(x));
#elif defined(_MSC_VER) && defined(IS_X86_64)
    unsigned long index;
    _BitScanReverse64(&index, x);
    return index;
#elif defined(_MSC_VER) && defined(IS_X86_32)
    if (x >> 32) {
        unsigned long index;
        _BitScanReverse(&index, x >> 32);
        return 32 + index;
    } else {
        unsigned long index;
        _BitScanReverse(&index, x);
        return index;
    }
#else
    unsigned int c = 0;
    if (x & 0xffffffff00000000ULL) {
        x >>= 32;
        c += 32;
    }
    if (x & 0x00000000ffff0000ULL) {
        x >>= 16;
        c += 16;
    }
    if (x & 0x000000000000ff00ULL) {
        x >>= 8;
        c += 8;
    }
    if (x & 0x00000000000000f0ULL) {
        x >>= 4;
        c += 4;
    }
    if (x & 0x000000000000000cULL) {
        x >>= 2;
        c += 2;
    }
    if (x & 0x0000000000000002ULL) {
        c += 1;
    }
    return c;
#endif
}

// Count the number of 1 bits.
INLINE unsigned int popcnt(uint64_t x)
{
#if defined(__GNUC__) || defined(__clang__)
    return uint32_t(__builtin_popcountll(x));
#else
    unsigned int count = 0;
    while (x != 0) {
        count += 1;
        x &= x - 1;
    }
    return count;
#endif
}

// Right rotates 32 bit inputs
INLINE uint32_t rotr32(uint32_t w, uint32_t c)
{
    return (w >> c) | (w << (32 - c));
}

// Largest power of two less than or equal to x. As a special case, returns 1
// when x is 0.
INLINE uint64_t round_down_to_power_of_2(uint64_t x)
{
    return 1ULL << highest_one(x | 1);
}

INLINE uint32_t counter_low(uint64_t counter)
{
    return (uint32_t)counter;
}

INLINE uint32_t counter_high(uint64_t counter)
{
    return (uint32_t)(counter >> 32);
}

INLINE uint32_t load32(const void* src)
{
    const uint8_t* p = (const uint8_t*)src;
    return ((uint32_t)(p[0]) << 0) | ((uint32_t)(p[1]) << 8) | ((uint32_t)(p[2]) << 16) | ((uint32_t)(p[3]) << 24);
}

INLINE void load_key_words(const uint8_t key[BLAKE3_KEY_LEN], uint32_t key_words[8])
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

INLINE void store32(void* dst, uint32_t w)
{
    uint8_t* p = (uint8_t*)dst;
    p[0] = (uint8_t)(w >> 0);
    p[1] = (uint8_t)(w >> 8);
    p[2] = (uint8_t)(w >> 16);
    p[3] = (uint8_t)(w >> 24);
}

INLINE void store_cv_words(uint8_t bytes_out[32], uint32_t cv_words[8])
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

} // namespace blake3_full

#endif /* BLAKE3_IMPL_H */
