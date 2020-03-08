/* ethash: C/C++ implementation of Ethash, the Ethereum Proof of Work algorithm.
 * Copyright 2018-2019 Pawel Bylica.
 * Licensed under the Apache License, Version 2.0.
 */

#pragma once

#include "./hash_types.hpp"

#include <stddef.h>

#ifdef __cplusplus
#define NOEXCEPT noexcept
#else
#define NOEXCEPT
#endif

#ifdef __cplusplus
extern "C"
{
#endif

    /**
     * The Keccak-f[1600] function.
     *
     * The implementation of the Keccak-f function with 1600-bit width of the permutation (b).
     * The size of the state is also 1600 bit what gives 25 64-bit words.
     *
     * @param state  The state of 25 64-bit words on which the permutation is to be performed.
     */
    void ethash_keccakf1600(uint64_t state[25]) NOEXCEPT;

    struct keccak256 ethash_keccak256(const uint8_t* data, size_t size) NOEXCEPT;

    struct keccak256 hash_field_elements(const uint64_t* limbs, size_t num_elements);

    struct keccak256 hash_field_element(const uint64_t* limb);

#ifdef __cplusplus
}
#endif
