#pragma once
#include "barretenberg/stdlib/primitives/byte_array/byte_array.hpp"
#include "barretenberg/stdlib/primitives/packed_byte_array/packed_byte_array.hpp"
#include "barretenberg/stdlib/primitives/uint/uint.hpp"
#include <array>

namespace bb::plonk {
namespace stdlib {
template <typename Builder> class bit_array;

/**
 * @brief KECCAAAAAAAAAAK
 *
 * Creates constraints that evaluate the Keccak256 hash algorithm.
 *
 * UltraPlonk only due to heavy lookup table use.
 *
 * Current cost 17,329 constraints for a 1-block hash
 * using small(ish) lookup tables (total size < 2^64)
 *
 * @tparam Builder
 */
template <typename Builder> class keccak {
  public:
    using witness_ct = stdlib::witness_t<Builder>;
    using field_ct = stdlib::field_t<Builder>;
    using bool_ct = stdlib::bool_t<Builder>;
    using byte_array_ct = stdlib::byte_array<Builder>;
    using uint32_ct = stdlib::uint32<Builder>;

    // base of extended representation we use for efficient logic operations
    static constexpr uint256_t BASE = 11;

    // number of bits of hash output
    static constexpr size_t BITS = 256;

    // word size of hash lane
    static constexpr size_t WORD_SIZE = 8;

    // block size. We only support keccak256 with a 1088-bit rate! This is what Ethereum uses
    static constexpr size_t BLOCK_SIZE = (1600 - BITS * 2) / WORD_SIZE;

    // how many limbs fit into a block (17)
    static constexpr size_t LIMBS_PER_BLOCK = BLOCK_SIZE / 8;

    static constexpr size_t NUM_KECCAK_ROUNDS = 24;

    // 1 "lane" = 64 bits. Instead of interpreting the keccak sponge as 1,600 bits, it's easier to work over 64-bit
    // "lanes". 1,600 / 64 = 25.
    static constexpr size_t NUM_KECCAK_LANES = 25;

    // round constants. Used in IOTA round
    static constexpr std::array<uint64_t, NUM_KECCAK_ROUNDS> RC = {
        0x0000000000000001, 0x0000000000008082, 0x800000000000808a, 0x8000000080008000, 0x000000000000808b,
        0x0000000080000001, 0x8000000080008081, 0x8000000000008009, 0x000000000000008a, 0x0000000000000088,
        0x0000000080008009, 0x000000008000000a, 0x000000008000808b, 0x800000000000008b, 0x8000000000008089,
        0x8000000000008003, 0x8000000000008002, 0x8000000000000080, 0x000000000000800a, 0x800000008000000a,
        0x8000000080008081, 0x8000000000008080, 0x0000000080000001, 0x8000000080008008
    };

    // Rotation offsets, y vertically, x horizontally: r[y * 5 + x]
    static constexpr std::array<size_t, NUM_KECCAK_LANES> ROTATIONS = {
        0, 1, 62, 28, 27, 36, 44, 6, 55, 20, 3, 10, 43, 25, 39, 41, 45, 15, 21, 8, 18, 2, 61, 56, 14,
    };

    /**
     * @brief Convert a binary integer into a base11 integer
     *
     * Input  = \sum_{i=0}^63 b_i * 2^i
     * Output = \sum_{i=0}^63 b_i * 11^i
     *
     * @param input
     * @return constexpr uint256_t sparse form of input
     */
    static constexpr uint256_t convert_to_sparse(uint256_t input)
    {
        std::array<uint64_t, 64> out_bits;
        size_t count = 0;
        while (input > 0) {
            uint64_t bit = static_cast<uint64_t>(input & 1);
            out_bits[count++] = bit;
            input = input >> 1;
        }
        uint256_t output = 0;
        for (size_t i = 0; i < count; ++i) {
            output *= BASE;
            output += out_bits[count - 1 - i];
        }
        return output;
    };

    /**
     * @brief Normalize a base-11 integer where each base value can be > 1
     *
     * Input  = \sum_{i=0}^63 b_i * 11^i
     * Output = \sum_{i=0}^63 (b_i & 1) * 11^i
     *
     * (XORs are evaluated by adding integers in sparse-form and normalizing. Even = 0, Odd = 1)
     *
     * @param input
     * @return constexpr uint256_t
     */
    static constexpr uint256_t normalize_sparse(uint256_t input)
    {
        std::array<uint64_t, 64> out_bits;
        size_t count = 0;
        while (input > 0) {
            const auto [quotient, slice] = input.divmod(BASE);
            uint64_t bit = static_cast<uint64_t>(slice) & 1;
            out_bits[count++] = bit;
            input = quotient;
        }
        uint256_t out;
        for (size_t i = 0; i < count; ++i) {
            out *= BASE;
            out += out_bits[count - 1 - i];
        }
        return out;
    }

    /**
     * @brief Get the sparse round constants object
     *
     * @return constexpr std::array<uint256_t, 24>
     */
    static constexpr std::array<uint256_t, NUM_KECCAK_ROUNDS> get_sparse_round_constants()
    {
        std::array<uint256_t, 24> output;
        for (size_t i = 0; i < 24; ++i) {
            output[i] = convert_to_sparse(RC[i]);
        }
        return output;
    }
    static constexpr std::array<uint256_t, NUM_KECCAK_ROUNDS> SPARSE_RC = get_sparse_round_constants();

    /**
     * @brief Compute the constant offset added in the `Chi` round
     *
     * We want to compute, for each bit slice of the inputs A, B, C...
     *  1 + 2A - B + C
     *
     * i.e. we need to add the constant value \sum_{i=0}^63 11^i
     *
     * @return constexpr uint256_t
     */
    static constexpr uint256_t get_chi_offset()
    {
        uint256_t result = 0;
        for (size_t i = 0; i < 64; ++i) {
            result *= 11;
            result += 1;
        }
        return result;
    }
    static constexpr uint256_t CHI_OFFSET = get_chi_offset();

    struct keccak_state {
        std::array<field_ct, NUM_KECCAK_LANES> state;
        std::array<field_ct, NUM_KECCAK_LANES> state_msb;
        std::array<field_ct, NUM_KECCAK_LANES> twisted_state;
        Builder* context;
    };

    template <size_t lane_index> static field_t<Builder> normalize_and_rotate(const field_ct& limb, field_ct& msb);
    static void compute_twisted_state(keccak_state& internal);
    static void theta(keccak_state& state);
    static void rho(keccak_state& state);
    static void pi(keccak_state& state);
    static void chi(keccak_state& state);
    static void iota(keccak_state& state, size_t round);
    static void sponge_absorb(keccak_state& internal,
                              const std::vector<field_ct>& input_buffer,
                              const std::vector<field_ct>& msb_buffer,
                              const field_ct& num_blocks_with_data);
    static byte_array_ct sponge_squeeze(keccak_state& internal);
    static void keccakf1600(keccak_state& state);
    static byte_array_ct hash(byte_array_ct& input, const uint32_ct& num_bytes);
    static byte_array_ct hash(byte_array_ct& input) { return hash(input, static_cast<uint32_t>(input.size())); };

    static std::vector<field_ct> format_input_lanes(byte_array_ct& input, const uint32_ct& num_bytes);

    static std::vector<uint8_t> hash_native(const std::vector<uint8_t>& data)
    {
        auto hash_result = ethash_keccak256(&data[0], data.size());

        std::vector<uint8_t> output;
        output.resize(32);

        memcpy((void*)&output[0], (void*)&hash_result.word64s[0], 32);
        return output;
    }

    // exposing keccak f1600 permutation
    static byte_array_ct hash_using_permutation_opcode(byte_array_ct& input, const uint32_ct& num_bytes);
    static std::array<field_ct, NUM_KECCAK_LANES> permutation_opcode(std::array<field_ct, NUM_KECCAK_LANES> state,
                                                                     Builder* context);
    static void sponge_absorb_with_permutation_opcode(keccak_state& internal,
                                                      std::vector<field_ct>& input_buffer,
                                                      const size_t input_size);
    static std::array<field_ct, NUM_KECCAK_LANES> extended_2_normal(keccak_state& internal);
    static byte_array_ct sponge_squeeze_for_permutation_opcode(std::array<field_ct, NUM_KECCAK_LANES> lanes,
                                                               Builder* context);
};

} // namespace stdlib
} // namespace bb::plonk
