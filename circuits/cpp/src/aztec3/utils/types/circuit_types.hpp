#pragma once

// TODO(dbanks12) consider removing this include which is used by consumers of circuit_types.hpp
#include "native_types.hpp"

#include <barretenberg/barretenberg.hpp>

using namespace proof_system::plonk;

// Note: Inner proving system type for recursion is inflexibly set to UltraPlonk.
namespace aztec3::utils::types {

template <typename Builder> struct CircuitTypes {
    // Basic uint types
    using boolean = stdlib::bool_t<Builder>;
    using uint8 = stdlib::uint8<Builder>;
    using uint16 = stdlib::uint16<Builder>;
    using uint32 = stdlib::uint32<Builder>;
    using uint64 = stdlib::uint64<Builder>;

    // Types related to the bn254 curve
    using fr = stdlib::field_t<Builder>;
    using safe_fr = stdlib::safe_uint_t<Builder>;
    using address = stdlib::address_t<Builder>;
    using fq = stdlib::bigfield<Builder, barretenberg::Bn254FqParams>;

    using witness = stdlib::witness_t<Builder>;

    // typedef fq grumpkin_fr;
    // typedef fr grumpkin_fq;
    using grumpkin_point = stdlib::point<Builder>;  // affine
    using grumpkin_group = stdlib::group<Builder>;

    using bn254 = stdlib::bn254<Builder>;
    // typedef bn254::g1_ct bn254_point;
    using bn254_point = stdlib::element<Builder, fq, fr, barretenberg::g1>;  // affine

    using bit_array = stdlib::bit_array<Builder>;
    using byte_array = stdlib::byte_array<Builder>;
    using packed_byte_array = stdlib::packed_byte_array<Builder>;

    using schnorr_signature = stdlib::schnorr::signature_bits<Builder>;
    using ecdsa_signature = stdlib::ecdsa::signature<Builder>;

    using AggregationObject = stdlib::recursion::aggregation_state<bn254>;
    using VK = stdlib::recursion::verification_key<bn254>;
    // Notice: no CircuitType for a Proof: we only ever handle native; the verify_proof() function swallows the
    // 'circuit-type-ness' of the proof.

    /// TODO: lots of these compress / commit functions aren't actually used: remove them.

    // Define the 'circuit' version of the function `compress`, with the name `compress`:
    static fr compress(std::vector<fr> const& inputs, const size_t hash_index = 0)
    {
        return plonk::stdlib::pedersen_commitment<Builder>::compress(inputs, hash_index);
    }

    static fr hash(std::vector<fr> const& inputs, const size_t hash_index = 0)
    {
        return plonk::stdlib::pedersen_plookup_commitment<Builder>::compress(inputs, hash_index);
    }

    template <size_t SIZE> static fr compress(std::array<fr, SIZE> const& inputs, const size_t hash_index = 0)
    {
        std::vector<fr> const inputs_vec(std::begin(inputs), std::end(inputs));
        return plonk::stdlib::pedersen_commitment<Builder>::compress(inputs_vec, hash_index);
    }

    template <size_t SIZE> static fr hash(std::array<fr, SIZE> const& inputs, const size_t hash_index = 0)
    {
        std::vector<fr> const inputs_vec(std::begin(inputs), std::end(inputs));
        return plonk::stdlib::pedersen_plookup_commitment<Builder>::compress(inputs_vec, hash_index);
    }

    static fr compress(std::vector<fr> const& inputs,
                       std::vector<size_t> const& hash_sub_indices,
                       const size_t hash_index = 0)
    {
        return plonk::stdlib::pedersen_commitment<Builder>::compress(inputs, hash_sub_indices, hash_index);
    }

    static fr compress(const std::vector<std::pair<fr, crypto::generators::generator_index_t>>& input_pairs)
    {
        return plonk::stdlib::pedersen_commitment<Builder>::compress(input_pairs);
    };

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
        return plonk::stdlib::pedersen_hash<Builder>::hash_multiple({ left, right }, 0);
    };

    static grumpkin_point commit(const std::vector<fr>& inputs, const size_t hash_index = 0)
    {
        return plonk::stdlib::pedersen_commitment<Builder>::commit(inputs, hash_index);
    };

    static grumpkin_point commit(const std::vector<std::pair<fr, crypto::generators::generator_index_t>>& input_pairs)
    {
        return plonk::stdlib::pedersen_commitment<Builder>::commit(input_pairs);
    };

    static byte_array blake2s(const byte_array& input) { return plonk::stdlib::blake2s(input); }

    static byte_array blake3s(const byte_array& input) { return plonk::stdlib::blake3s(input); }
};

}  // namespace aztec3::utils::types