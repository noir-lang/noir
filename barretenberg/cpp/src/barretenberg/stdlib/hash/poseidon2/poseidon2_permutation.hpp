#pragma once
#include <array>
#include <cstddef>
#include <cstdint>

#include "barretenberg/crypto/poseidon2/poseidon2_permutation.hpp"
#include "barretenberg/stdlib/primitives/circuit_builders/circuit_builders.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"

namespace bb::stdlib {

using namespace bb;
template <typename Params, typename Builder> class Poseidon2Permutation {
  public:
    using NativePermutation = crypto::Poseidon2Permutation<Params>;
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
    using State = std::array<field_t<Builder>, t>;
    using NativeState = std::array<FF, t>;

    using RoundConstants = std::array<FF, t>;
    using RoundConstantsContainer = std::array<RoundConstants, NUM_ROUNDS>;
    static constexpr RoundConstantsContainer round_constants = Params::round_constants;

    /**
     * @brief Circuit form of Poseidon2 permutation from https://eprint.iacr.org/2023/323.
     * @details The permutation consists of one initial linear layer, then a set of external rounds, a set of internal
     * rounds, and a set of external rounds.
     * @param builder
     * @param input
     * @return State
     */
    static State permutation(Builder* builder, const State& input)
        requires IsGoblinUltraBuilder<Builder>;
    static State permutation(Builder* builder, const State& input)
        requires IsNotGoblinUltraBuilder<Builder>;

    static void add_round_constants(State& input, const RoundConstants& rc)
        requires IsNotGoblinUltraBuilder<Builder>;
    static void apply_sbox(State& input)
        requires IsNotGoblinUltraBuilder<Builder>;
    static void apply_single_sbox(field_t<Builder>& input)
        requires IsNotGoblinUltraBuilder<Builder>;
    static void matrix_multiplication_internal(State& input)
        requires IsNotGoblinUltraBuilder<Builder>;

    /**
     * @brief Separate function to do just the first linear layer (equivalent to external matrix mul).
     * @details We use 6 arithmetic gates to implement:
     *          gate 1: Compute tmp1 = state[0] + state[1] + 2 * state[3]
     *          gate 2: Compute tmp2 = 2 * state[1] + state[2] + state[3]
     *          gate 3: Compute v2 = 4 * state[0] + 4 * state[1] + tmp2
     *          gate 4: Compute v1 = v2 + tmp1
     *          gate 5: Compute v4 = tmp1 + 4 * state[2] + 4 * state[3]
     *          gate 6: Compute v3 = v4 + tmp2
     *          output state is [v1, v2, v3, v4]
     * @param builder
     * @param state
     */
    static void matrix_multiplication_external(Builder* builder, State& state);
};

} // namespace bb::stdlib