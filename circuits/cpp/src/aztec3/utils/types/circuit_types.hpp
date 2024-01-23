#pragma once

// TODO(dbanks12) consider removing this include which is used by consumers of circuit_types.hpp

#include "aztec3/constants.hpp"

#include <barretenberg/barretenberg.hpp>

using namespace proof_system::plonk;

// Note: Inner proving system type for recursion is inflexibly set to UltraPlonk.
namespace aztec3::utils::types {

template <typename Builder> struct CircuitTypes {
    // Basic uint types
    using boolean = plonk::stdlib::bool_t<Builder>;
    using uint8 = plonk::stdlib::uint8<Builder>;
    using uint16 = plonk::stdlib::uint16<Builder>;
    using uint32 = plonk::stdlib::uint32<Builder>;
    using uint64 = plonk::stdlib::uint64<Builder>;

    // Types related to the bn254 curve
    using fr = plonk::stdlib::field_t<Builder>;
    using safe_fr = plonk::stdlib::safe_uint_t<Builder>;
    using address = plonk::stdlib::address_t<Builder>;
    using fq = plonk::stdlib::bigfield<Builder, barretenberg::Bn254FqParams>;

    using witness = plonk::stdlib::witness_t<Builder>;

    // typedef fq grumpkin_fr;
    // typedef fr grumpkin_fq;
    using grumpkin_point = plonk::stdlib::cycle_group<Builder>;  // affine

    using bn254 = plonk::stdlib::bn254<Builder>;
    // typedef bn254::g1_ct bn254_point;
    using bn254_point = plonk::stdlib::element<Builder, fq, fr, barretenberg::g1>;  // affine

    using bit_array = plonk::stdlib::bit_array<Builder>;
    using byte_array = plonk::stdlib::byte_array<Builder>;
    using packed_byte_array = plonk::stdlib::packed_byte_array<Builder>;

    using schnorr_signature = plonk::stdlib::schnorr::signature_bits<Builder>;
    using ecdsa_signature = plonk::stdlib::ecdsa::signature<Builder>;

    using AggregationObject = plonk::stdlib::recursion::aggregation_state<bn254>;
    using VK = plonk::stdlib::recursion::verification_key<bn254>;
    // Notice: no CircuitType for a Proof: we only ever handle native; the verify_proof() function swallows the
    // 'circuit-type-ness' of the proof.

    static crypto::GeneratorContext<curve::Grumpkin> get_generator_context(const size_t hash_index)
    {
        crypto::GeneratorContext<curve::Grumpkin> result;
        result.offset = hash_index;
        return result;
    }

    /// TODO: lots of these compress / commit functions aren't actually used: remove them.

    // Define the 'circuit' version of the function `hash`, with the name `hash`:
    static fr hash(std::vector<fr> const& inputs, const size_t hash_index = 0)
    {
        return plonk::stdlib::pedersen_hash<Builder>::hash(inputs, get_generator_context(hash_index));
    }


    static fr hash(std::vector<fr> const& inputs,
                   std::vector<size_t> const& hash_sub_indices,
                   const size_t hash_index = 0)
    {
        return plonk::stdlib::pedersen_hash<Builder>::hash(inputs, hash_sub_indices, get_generator_context(hash_index));
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
        return plonk::stdlib::pedersen_hash<Builder>::hash({ left, right }, 0);
    };

    static grumpkin_point commit(const std::vector<fr>& inputs, const size_t hash_index = 0)
    {
        return plonk::stdlib::pedersen_commitment<Builder>::commit(inputs, get_generator_context(hash_index));
    };

    static byte_array blake2s(const byte_array& input) { return plonk::stdlib::blake2s(input); }
};

}  // namespace aztec3::utils::types
