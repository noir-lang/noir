#pragma once

#include "poseidon2_params.hpp"

#include "barretenberg/common/throw_or_abort.hpp"

#include <array>
#include <cstddef>
#include <cstdint>

namespace bb::crypto {

/**
 * @brief Applies the Poseidon2 permutation function from https://eprint.iacr.org/2023/323 .
 * This algorithm was implemented using https://github.com/HorizenLabs/poseidon2 as a reference.
 *
 * @tparam Params
 */
template <typename Params> class Poseidon2Permutation {
  public:
    // t = sponge permutation size (in field elements)
    // t = rate + capacity
    // capacity = 1 field element (256 bits)
    // rate = number of field elements that can be compressed per permutation
    static constexpr size_t t = Params::t;
    // d = degree of s-box polynomials. For a given field, `d` is the smallest element of `p` such that gdc(d, p - 1) =
    // 1 (excluding 1) For bn254/grumpkin, d = 5
    static constexpr size_t d = Params::d;
    // sbox size = number of bits in p
    static constexpr size_t sbox_size = Params::sbox_size;
    // number of full sbox rounds
    static constexpr size_t rounds_f = Params::rounds_f;
    // number of partial sbox rounds
    static constexpr size_t rounds_p = Params::rounds_p;
    static constexpr size_t NUM_ROUNDS = Params::rounds_f + Params::rounds_p;

    using FF = typename Params::FF;
    using State = std::array<FF, t>;
    using RoundConstants = std::array<FF, t>;
    using MatrixDiagonal = std::array<FF, t>;
    using RoundConstantsContainer = std::array<RoundConstants, NUM_ROUNDS>;

    static constexpr MatrixDiagonal internal_matrix_diagonal =
        Poseidon2Bn254ScalarFieldParams::internal_matrix_diagonal;
    static constexpr RoundConstantsContainer round_constants = Poseidon2Bn254ScalarFieldParams::round_constants;

    static constexpr void matrix_multiplication_4x4(State& input)
    {
        /**
         * hardcoded algorithm that evaluates matrix multiplication using the following MDS matrix:
         * /         \
         * | 5 7 1 3 |
         * | 4 6 1 1 |
         * | 1 3 5 7 |
         * | 1 1 4 6 |
         * \         /
         *
         * Algorithm is taken directly from the Poseidon2 paper.
         */
        auto t0 = input[0] + input[1]; // A + B
        auto t1 = input[2] + input[3]; // C + D
        auto t2 = input[1] + input[1]; // 2B
        t2 += t1;                      // 2B + C + D
        auto t3 = input[3] + input[3]; // 2D
        t3 += t0;                      // 2D + A + B
        auto t4 = t1 + t1;
        t4 += t4;
        t4 += t3; // A + B + 4C + 6D
        auto t5 = t0 + t0;
        t5 += t5;
        t5 += t2;          // 4A + 6B + C + D
        auto t6 = t3 + t5; // 5A + 7B + C + 3D
        auto t7 = t2 + t4; // A + 3B + 5C + 7D
        input[0] = t6;
        input[1] = t5;
        input[2] = t7;
        input[3] = t4;
    }

    static constexpr void add_round_constants(State& input, const RoundConstants& rc)
    {
        for (size_t i = 0; i < t; ++i) {
            input[i] += rc[i];
        }
    }

    static constexpr void matrix_multiplication_internal(State& input)
    {
        // for t = 4
        auto sum = input[0];
        for (size_t i = 1; i < t; ++i) {
            sum += input[i];
        }
        for (size_t i = 0; i < t; ++i) {
            input[i] *= internal_matrix_diagonal[i];
            input[i] += sum;
        }
    }

    static constexpr void matrix_multiplication_external(State& input)
    {
        if constexpr (t == 4) {
            matrix_multiplication_4x4(input);
        } else {
            // erm panic
            throw_or_abort("not supported");
        }
    }

    static constexpr void apply_single_sbox(FF& input)
    {
        // hardcoded assumption that d = 5. should fix this or not make d configurable
        auto xx = input.sqr();
        auto xxxx = xx.sqr();
        input *= xxxx;
    }

    static constexpr void apply_sbox(State& input)
    {
        for (auto& in : input) {
            apply_single_sbox(in);
        }
    }

    /**
     * @brief Native form of Poseidon2 permutation from https://eprint.iacr.org/2023/323.
     * @details The permutation consists of one initial linear layer, then a set of external rounds, a set of internal
     * rounds, and a set of external rounds.
     * @param input
     * @return constexpr State
     */
    static constexpr State permutation(const State& input)
    {
        // deep copy
        State current_state(input);

        // Apply 1st linear layer
        matrix_multiplication_external(current_state);

        // First set of external rounds
        constexpr size_t rounds_f_beginning = rounds_f / 2;
        for (size_t i = 0; i < rounds_f_beginning; ++i) {
            add_round_constants(current_state, round_constants[i]);
            apply_sbox(current_state);
            matrix_multiplication_external(current_state);
        }

        // Internal rounds
        const size_t p_end = rounds_f_beginning + rounds_p;
        for (size_t i = rounds_f_beginning; i < p_end; ++i) {
            current_state[0] += round_constants[i][0];
            apply_single_sbox(current_state[0]);
            matrix_multiplication_internal(current_state);
        }

        // Remaining external rounds
        for (size_t i = p_end; i < NUM_ROUNDS; ++i) {
            add_round_constants(current_state, round_constants[i]);
            apply_sbox(current_state);
            matrix_multiplication_external(current_state);
        }
        return current_state;
    }
};
} // namespace bb::crypto