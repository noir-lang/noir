#pragma once
#include <barretenberg/stdlib/primitives/address/address.hpp>
#include <barretenberg/stdlib/encryption/schnorr/schnorr.hpp>
#include <barretenberg/stdlib/encryption/ecdsa/ecdsa.hpp>
#include <barretenberg/stdlib/primitives/bigfield/bigfield.hpp>
#include <barretenberg/stdlib/primitives/biggroup/biggroup.hpp>
#include <barretenberg/stdlib/primitives/bit_array/bit_array.hpp>
#include <barretenberg/stdlib/primitives/bool/bool.hpp>
#include <barretenberg/stdlib/primitives/byte_array/byte_array.hpp>
#include <barretenberg/stdlib/primitives/packed_byte_array/packed_byte_array.hpp>
#include <barretenberg/stdlib/primitives/uint/uint.hpp>
#include <barretenberg/stdlib/primitives/point/point.hpp>
#include <barretenberg/stdlib/primitives/group/group.hpp>
#include <barretenberg/stdlib/primitives/curves/bn254.hpp>
#include <barretenberg/stdlib/recursion/verifier/verifier.hpp>
#include <barretenberg/stdlib/recursion/verification_key/verification_key.hpp>
#include <barretenberg/stdlib/commitment/pedersen/pedersen.hpp>
#include <barretenberg/stdlib/hash/blake2s/blake2s.hpp>
#include "native_types.hpp"

using namespace proof_system::plonk;

namespace aztec3::utils::types {

template <typename Composer> struct CircuitTypes {
    typedef stdlib::bool_t<Composer> boolean;

    typedef stdlib::uint8<Composer> uint8;
    typedef stdlib::uint16<Composer> uint16;
    typedef stdlib::uint32<Composer> uint32;
    typedef stdlib::uint64<Composer> uint64;

    typedef stdlib::field_t<Composer> fr; // of altbn
    typedef stdlib::safe_uint_t<Composer> safe_fr;
    typedef stdlib::address_t<Composer> address;

    typedef stdlib::bigfield<Composer, barretenberg::Bn254FqParams> fq; // of altbn

    // typedef fq grumpkin_fr;
    // typedef fr grumpkin_fq;
    typedef stdlib::point<Composer> grumpkin_point; // affine
    typedef stdlib::group<Composer> grumpkin_group;

    typedef stdlib::bn254<Composer> bn254;
    // typedef bn254::g1_ct bn254_point;
    typedef stdlib::element<Composer, fq, fr, barretenberg::g1> bn254_point; // affine

    typedef stdlib::bit_array<Composer> bit_array;
    typedef stdlib::byte_array<Composer> byte_array;
    typedef stdlib::packed_byte_array<Composer> packed_byte_array;

    typedef stdlib::schnorr::signature_bits<Composer> schnorr_signature;
    typedef stdlib::ecdsa::signature<Composer> ecdsa_signature;

    typedef stdlib::recursion::aggregation_state<bn254> AggregationObject;
    typedef stdlib::recursion::verification_key<bn254> VK;
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
        std::vector<fr> inputs_vec(std::begin(inputs), std::end(inputs));
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

} // namespace aztec3::utils::types