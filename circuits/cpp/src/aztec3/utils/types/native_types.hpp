#pragma once

#include <barretenberg/barretenberg.hpp>
namespace aztec3::utils::types {

struct NativeTypes {
    using boolean = bool;

    using uint8 = uint8_t;
    using uint16 = uint16_t;
    using uint32 = uint32_t;
    using uint64 = uint64_t;
    using uint256 = uint256_t;

    using fr = barretenberg::fr;
    using address = proof_system::plonk::stdlib::address;

    using fq = barretenberg::fq;

    // typedef fq grumpkin_fr;
    // typedef fr grumpkin_fq;
    using grumpkin_point = grumpkin::g1::affine_element;
    // typedef grumpkin::g1::element grumpkin_jac_point;
    using grumpkin_group = grumpkin::g1;

    using bn254_point = barretenberg::g1::affine_element;
    // typedef barretenberg::g1::element bn254_jac_point;
    // typedef barretenberg::g1 bn254_group;

    using bit_array = std::vector<bool>;
    using byte_array = std::vector<uint8_t>;
    using packed_byte_array = std::string;

    using schnorr_signature = crypto::schnorr::signature;
    using ecdsa_signature = crypto::ecdsa::signature;

    using AggregationObject = proof_system::plonk::stdlib::recursion::native_aggregation_state;
    using VKData = plonk::verification_key_data;
    using VK = plonk::verification_key;
    using Proof = plonk::proof;

    /// TODO: lots of these compress / commit functions aren't actually used: remove them.

    // Define the 'native' version of the function `compress`, with the name `compress`:
    static fr compress(const std::vector<fr>& inputs, const size_t hash_index = 0)
    {
        return crypto::pedersen_commitment::compress_native(inputs, hash_index);
    }

    template <size_t SIZE> static fr compress(std::array<fr, SIZE> const& inputs, const size_t hash_index = 0)
    {
        std::vector<fr> const inputs_vec(std::begin(inputs), std::end(inputs));
        return crypto::pedersen_commitment::compress_native(inputs_vec, hash_index);
    }

    static fr compress(const std::vector<std::pair<fr, crypto::generators::generator_index_t>>& input_pairs)
    {
        return crypto::pedersen_commitment::compress_native(input_pairs);
    }

    /**
     * @brief Compute the hash for a pair of left and right nodes in a merkle tree.
     *
     * @details Compress the two nodes using the default/0-generator which is reserved
     * for internal merkle hashing.
     *
     * @param left The left child node
     * @param right The right child node
     * @return The computed Merkle tree hash for the given pair of nodes
     */
    static fr merkle_hash(fr left, fr right)
    {
        // use 0-generator for internal merkle hashing
        // use lookup namespace since we now use ultraplonk
        return crypto::pedersen_hash::lookup::hash_multiple({ left, right }, 0);
    }

    static grumpkin_point commit(const std::vector<fr>& inputs, const size_t hash_index = 0)
    {
        return crypto::pedersen_commitment::commit_native(inputs, hash_index);
    }

    static grumpkin_point commit(const std::vector<std::pair<fr, crypto::generators::generator_index_t>>& input_pairs)
    {
        return crypto::pedersen_commitment::commit_native(input_pairs);
    }

    static byte_array blake2s(const byte_array& input)
    {
        auto res = blake2::blake2s(input);
        return byte_array(res.begin(), res.end());
    }

    static byte_array blake3s(const byte_array& input) { return blake3::blake3s(input); }
};

}  // namespace aztec3::utils::types