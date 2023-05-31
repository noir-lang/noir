#pragma once
#include "native_types.hpp"

#include <barretenberg/stdlib/commitment/pedersen/pedersen.hpp>
#include <barretenberg/stdlib/encryption/ecdsa/ecdsa.hpp>
#include <barretenberg/stdlib/encryption/schnorr/schnorr.hpp>
#include <barretenberg/stdlib/hash/blake2s/blake2s.hpp>
#include <barretenberg/stdlib/hash/blake3s/blake3s.hpp>
#include <barretenberg/stdlib/hash/pedersen/pedersen.hpp>
#include <barretenberg/stdlib/primitives/address/address.hpp>
#include <barretenberg/stdlib/primitives/bigfield/bigfield.hpp>
#include <barretenberg/stdlib/primitives/biggroup/biggroup.hpp>
#include <barretenberg/stdlib/primitives/bit_array/bit_array.hpp>
#include <barretenberg/stdlib/primitives/bool/bool.hpp>
#include <barretenberg/stdlib/primitives/byte_array/byte_array.hpp>
#include <barretenberg/stdlib/primitives/curves/bn254.hpp>
#include <barretenberg/stdlib/primitives/group/group.hpp>
#include <barretenberg/stdlib/primitives/packed_byte_array/packed_byte_array.hpp>
#include <barretenberg/stdlib/primitives/point/point.hpp>
#include <barretenberg/stdlib/primitives/uint/uint.hpp>
#include <barretenberg/stdlib/recursion/aggregation_state/aggregation_state.hpp>
#include <barretenberg/stdlib/recursion/verification_key/verification_key.hpp>
#include <barretenberg/stdlib/recursion/verifier/program_settings.hpp>
#include <barretenberg/stdlib/recursion/verifier/verifier.hpp>

using namespace proof_system::plonk;

namespace aztec3::utils::types {

template <typename Composer> struct CircuitTypes {
    // Basic uint types
    using boolean = stdlib::bool_t<Composer>;
    using uint8 = stdlib::uint8<Composer>;
    using uint16 = stdlib::uint16<Composer>;
    using uint32 = stdlib::uint32<Composer>;
    using uint64 = stdlib::uint64<Composer>;

    // Types related to the bn254 curve
    using fr = stdlib::field_t<Composer>;
    using safe_fr = stdlib::safe_uint_t<Composer>;
    using address = stdlib::address_t<Composer>;
    using fq = stdlib::bigfield<Composer, barretenberg::Bn254FqParams>;

    using witness = stdlib::witness_t<Composer>;

    // typedef fq grumpkin_fr;
    // typedef fr grumpkin_fq;
    using grumpkin_point = stdlib::point<Composer>;  // affine
    using grumpkin_group = stdlib::group<Composer>;

    using bn254 = stdlib::bn254<Composer>;
    // typedef bn254::g1_ct bn254_point;
    using bn254_point = stdlib::element<Composer, fq, fr, barretenberg::g1>;  // affine

    using bit_array = stdlib::bit_array<Composer>;
    using byte_array = stdlib::byte_array<Composer>;
    using packed_byte_array = stdlib::packed_byte_array<Composer>;

    using schnorr_signature = stdlib::schnorr::signature_bits<Composer>;
    using ecdsa_signature = stdlib::ecdsa::signature<Composer>;

    using AggregationObject = stdlib::recursion::aggregation_state<bn254>;
    using recursive_inner_verifier_settings =
        std::conditional_t<std::same_as<Composer, plonk::TurboPlonkComposer>,
                           stdlib::recursion::recursive_turbo_verifier_settings<bn254>,
                           stdlib::recursion::recursive_ultra_verifier_settings<bn254>>;
    using VK = stdlib::recursion::verification_key<bn254>;
    // Notice: no CircuitType for a Proof: we only ever handle native; the verify_proof() function swallows the
    // 'circuit-type-ness' of the proof.

    /// TODO: lots of these compress / commit functions aren't actually used: remove them.

    // Define the 'circuit' version of the function `compress`, with the name `compress`:
    static fr compress(std::vector<fr> const& inputs, const size_t hash_index = 0)
    {
        return plonk::stdlib::pedersen_commitment<Composer>::compress(inputs, hash_index);
    }

    template <size_t SIZE> static fr compress(std::array<fr, SIZE> const& inputs, const size_t hash_index = 0)
    {
        std::vector<fr> const inputs_vec(std::begin(inputs), std::end(inputs));
        return plonk::stdlib::pedersen_commitment<Composer>::compress(inputs_vec, hash_index);
    }

    static fr compress(std::vector<fr> const& inputs,
                       std::vector<size_t> const& hash_sub_indices,
                       const size_t hash_index = 0)
    {
        return plonk::stdlib::pedersen_commitment<Composer>::compress(inputs, hash_sub_indices, hash_index);
    }

    static fr compress(const std::vector<std::pair<fr, crypto::generators::generator_index_t>>& input_pairs)
    {
        return plonk::stdlib::pedersen_commitment<Composer>::compress(input_pairs);
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
        return plonk::stdlib::pedersen_hash<Composer>::hash_multiple({ left, right }, 0);
    };

    static grumpkin_point commit(const std::vector<fr>& inputs, const size_t hash_index = 0)
    {
        return plonk::stdlib::pedersen_commitment<Composer>::commit(inputs, hash_index);
    };

    static grumpkin_point commit(const std::vector<std::pair<fr, crypto::generators::generator_index_t>>& input_pairs)
    {
        return plonk::stdlib::pedersen_commitment<Composer>::commit(input_pairs);
    };

    static byte_array blake2s(const byte_array& input) { return plonk::stdlib::blake2s(input); }

    static byte_array blake3s(const byte_array& input) { return plonk::stdlib::blake3s(input); }
};

}  // namespace aztec3::utils::types