#pragma once
#include <barretenberg/stdlib/primitives/address/address.hpp>
#include <barretenberg/crypto/pedersen_commitment/pedersen.hpp>
#include <barretenberg/crypto/generators/generator_data.hpp>
#include <barretenberg/crypto/schnorr/schnorr.hpp>
#include <barretenberg/crypto/ecdsa/ecdsa.hpp>
#include <barretenberg/ecc/curves/bn254/fq.hpp>
#include <barretenberg/ecc/curves/bn254/fr.hpp>
#include <barretenberg/ecc/curves/bn254/g1.hpp>
#include <barretenberg/ecc/curves/grumpkin/grumpkin.hpp>
#include <barretenberg/numeric/uint256/uint256.hpp>
#include <barretenberg/proof_system/verification_key/verification_key.hpp>
#include <barretenberg/plonk/proof_system/types/proof.hpp>
#include <barretenberg/stdlib/recursion/verifier/verifier.hpp>
#include <barretenberg/stdlib/recursion/aggregation_state/native_aggregation_state.hpp>
#include <barretenberg/crypto/blake3s/blake3s.hpp>

namespace aztec3::utils::types {

// using plonk::stdlib::address;

struct NativeTypes {
    typedef bool boolean;

    typedef uint8_t uint8;
    typedef uint16_t uint16;
    typedef uint32_t uint32;
    typedef uint64_t uint64;
    typedef uint256_t uint256;

    typedef barretenberg::fr fr;
    typedef barretenberg::fr safe_fr;
    typedef stdlib::address address;

    typedef barretenberg::fq fq;

    // typedef fq grumpkin_fr;
    // typedef fr grumpkin_fq;
    typedef grumpkin::g1::affine_element grumpkin_point;
    // typedef grumpkin::g1::element grumpkin_jac_point;
    typedef grumpkin::g1 grumpkin_group;

    typedef barretenberg::g1::affine_element bn254_point;
    // typedef barretenberg::g1::element bn254_jac_point;
    // typedef barretenberg::g1 bn254_group;

    typedef std::vector<bool> bit_array;
    typedef std::vector<uint8_t> byte_array;
    typedef std::string packed_byte_array;

    typedef crypto::schnorr::signature schnorr_signature;
    typedef crypto::ecdsa::signature ecdsa_signature;

    typedef stdlib::recursion::native_aggregation_state AggregationObject;
    typedef bonk::verification_key_data VKData;
    typedef bonk::verification_key VK;
    typedef plonk::proof Proof;

    /// TODO: lots of these compress / commit functions aren't actually used: remove them.

    // Define the 'native' version of the function `compress`, with the name `compress`:
    static fr compress(const std::vector<fr>& inputs, const size_t hash_index = 0)
    {
        return crypto::pedersen_commitment::compress_native(inputs, hash_index);
    }

    template <size_t SIZE> static fr compress(std::array<fr, SIZE> const& inputs, const size_t hash_index = 0)
    {
        std::vector<fr> inputs_vec(std::begin(inputs), std::end(inputs));
        return crypto::pedersen_commitment::compress_native(inputs_vec, hash_index);
    }

    static fr compress(const std::vector<std::pair<fr, crypto::generators::generator_index_t>>& input_pairs)
    {
        return crypto::pedersen_commitment::compress_native(input_pairs);
    }

    static grumpkin_point commit(const std::vector<fr>& inputs, const size_t hash_index = 0)
    {
        return crypto::pedersen_commitment::commit_native(inputs, hash_index);
    }

    static grumpkin_point commit(const std::vector<std::pair<fr, crypto::generators::generator_index_t>>& input_pairs)
    {
        return crypto::pedersen_commitment::commit_native(input_pairs);
    }

    static byte_array blake2s(const byte_array& input) { return blake2::blake2s(input); }

    static byte_array blake3s(const byte_array& input) { return blake3::blake3s(input); }
};

} // namespace aztec3::utils::types