#pragma once

#include <array>
#include <cstddef>
#include <cstdint>
#include <span>

#include "barretenberg/numeric/uint256/uint256.hpp"

namespace bb::crypto {

/**
 * @brief Implements a cryptographic sponge over prime fields.
 *        Implements the sponge specification from the Community Cryptographic Specification Project
 *        see https://github.com/C2SP/C2SP/blob/792c1254124f625d459bfe34417e8f6bdd02eb28/poseidon-sponge.md
 *        (Note: this spec was not accepted into the C2SP repo, we might want to reference something else!)
 *
 *        Note: If we ever use this sponge class for more than 1 hash functions, we should move this out of `poseidon2`
 *              and into its own directory
 * @tparam FF
 * @tparam rate
 * @tparam capacity
 * @tparam t
 * @tparam Permutation
 */
template <typename FF, size_t rate, size_t capacity, size_t t, typename Permutation> class FieldSponge {
  public:
    /**
     * @brief Defines what phase of the sponge algorithm we are in.
     *
     *        ABSORB: 'absorbing' field elements into the sponge
     *        SQUEEZE: compressing the sponge and extracting a field element
     *
     */
    enum Mode {
        ABSORB,
        SQUEEZE,
    };

    // sponge state. t = rate + capacity. capacity = 1 field element (~256 bits)
    std::array<FF, t> state;

    // cached elements that have been absorbed.
    std::array<FF, rate> cache;
    size_t cache_size = 0;
    Mode mode = Mode::ABSORB;

    FieldSponge(FF domain_iv = 0)
    {
        for (size_t i = 0; i < rate; ++i) {
            state[i] = 0;
        }
        state[rate] = domain_iv;
    }

    std::array<FF, rate> perform_duplex()
    {
        // zero-pad the cache
        for (size_t i = cache_size; i < rate; ++i) {
            cache[i] = 0;
        }
        // add the cache into sponge state
        for (size_t i = 0; i < rate; ++i) {
            state[i] += cache[i];
        }
        state = Permutation::permutation(state);
        // return `rate` number of field elements from the sponge state.
        std::array<FF, rate> output;
        for (size_t i = 0; i < rate; ++i) {
            output[i] = state[i];
        }
        return output;
    }

    void absorb(const FF& input)
    {
        if (mode == Mode::ABSORB && cache_size == rate) {
            // If we're absorbing, and the cache is full, apply the sponge permutation to compress the cache
            perform_duplex();
            cache[0] = input;
            cache_size = 1;
        } else if (mode == Mode::ABSORB && cache_size < rate) {
            // If we're absorbing, and the cache is not full, add the input into the cache
            cache[cache_size] = input;
            cache_size += 1;
        } else if (mode == Mode::SQUEEZE) {
            // If we're in squeeze mode, switch to absorb mode and add the input into the cache.
            // N.B. I don't think this code path can be reached?!
            cache[0] = input;
            cache_size = 1;
            mode = Mode::ABSORB;
        }
    }

    FF squeeze()
    {
        if (mode == Mode::SQUEEZE && cache_size == 0) {
            // If we're in squeze mode and the cache is empty, there is nothing left to squeeze out of the sponge!
            // Switch to absorb mode.
            mode = Mode::ABSORB;
            cache_size = 0;
        }
        if (mode == Mode::ABSORB) {
            // If we're in absorb mode, apply sponge permutation to compress the cache, populate cache with compressed
            // state and switch to squeeze mode. Note: this code block will execute if the previous `if` condition was
            // matched
            auto new_output_elements = perform_duplex();
            mode = Mode::SQUEEZE;
            for (size_t i = 0; i < rate; ++i) {
                cache[i] = new_output_elements[i];
            }
            cache_size = rate;
        }
        // By this point, we should have a non-empty cache. Pop one item off the top of the cache and return it.
        FF result = cache[0];
        for (size_t i = 1; i < cache_size; ++i) {
            cache[i - 1] = cache[i];
        }
        cache_size -= 1;
        cache[cache_size] = 0;
        return result;
    }

    /**
     * @brief Use the sponge to hash an input string
     *
     * @tparam out_len
     * @tparam is_variable_length. Distinguishes between hashes where the preimage length is constant/not constant
     * @param input
     * @return std::array<FF, out_len>
     */
    template <size_t out_len, bool is_variable_length> static std::array<FF, out_len> hash_internal(std::span<FF> input)
    {
        size_t in_len = input.size();
        const uint256_t iv = (static_cast<uint256_t>(in_len) << 64) + out_len - 1;
        FieldSponge sponge(iv);

        for (size_t i = 0; i < in_len; ++i) {
            sponge.absorb(input[i]);
        }

        // In the case where the hash preimage is variable-length, we append `1` to the end of the input, to distinguish
        // from fixed-length hashes. (the combination of this additional field element + the hash IV ensures
        // fixed-length and variable-length hashes do not collide)
        if constexpr (is_variable_length) {
            sponge.absorb(1);
        }

        std::array<FF, out_len> output;
        for (size_t i = 0; i < out_len; ++i) {
            output[i] = sponge.squeeze();
        }
        return output;
    }

    template <size_t out_len> static std::array<FF, out_len> hash_fixed_length(std::span<FF> input)
    {
        return hash_internal<out_len, false>(input);
    }
    static FF hash_fixed_length(std::span<FF> input) { return hash_fixed_length<1>(input)[0]; }

    template <size_t out_len> static std::array<FF, out_len> hash_variable_length(std::span<FF> input)
    {
        return hash_internal<out_len, true>(input);
    }
    static FF hash_variable_length(std::span<FF> input) { return hash_variable_length<1>(input)[0]; }
};
} // namespace bb::crypto