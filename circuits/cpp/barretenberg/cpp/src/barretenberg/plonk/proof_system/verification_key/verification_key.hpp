#pragma once
#include <map>
#include "barretenberg/common/streams.hpp"
#include "barretenberg/srs/reference_string/reference_string.hpp"
#include "barretenberg/srs/reference_string/env_reference_string.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/polynomials/evaluation_domain.hpp"
#include "barretenberg/crypto/sha256/sha256.hpp"
#include "barretenberg/plonk/proof_system/types/polynomial_manifest.hpp"
#include "barretenberg/serialize/msgpack.hpp"

namespace proof_system::plonk {

struct verification_key_data {
    uint32_t composer_type;
    uint32_t circuit_size;
    uint32_t num_public_inputs;
    std::map<std::string, barretenberg::g1::affine_element> commitments;
    bool contains_recursive_proof = false;
    std::vector<uint32_t> recursive_proof_public_input_indices;

    // for serialization: update with any new fields
    MSGPACK_FIELDS(composer_type,
                   circuit_size,
                   num_public_inputs,
                   commitments,
                   contains_recursive_proof,
                   recursive_proof_public_input_indices);
    barretenberg::fr compress_native(size_t const hash_index = 0);
};

template <typename B> inline void read(B& buf, verification_key_data& key)
{
    using serialize::read;
    read(buf, key.composer_type);
    read(buf, key.circuit_size);
    read(buf, key.num_public_inputs);
    read(buf, key.commitments);
    read(buf, key.contains_recursive_proof);
    read(buf, key.recursive_proof_public_input_indices);
}

template <typename B> inline void write(B& buf, verification_key_data const& key)
{
    using serialize::write;
    write(buf, key.composer_type);
    write(buf, key.circuit_size);
    write(buf, key.num_public_inputs);
    write(buf, key.commitments);
    write(buf, key.contains_recursive_proof);
    write(buf, key.recursive_proof_public_input_indices);
}

inline bool operator==(verification_key_data const& lhs, verification_key_data const& rhs)
{
    return lhs.composer_type == rhs.composer_type && lhs.circuit_size == rhs.circuit_size &&
           lhs.num_public_inputs == rhs.num_public_inputs && lhs.commitments == rhs.commitments;
}

struct verification_key {
    // default constructor needed for msgpack unpack
    verification_key() = default;
    verification_key(verification_key_data&& data, std::shared_ptr<VerifierReferenceString> const& crs);
    verification_key(const size_t num_gates,
                     const size_t num_inputs,
                     std::shared_ptr<VerifierReferenceString> const& crs,
                     uint32_t composer_type);
    verification_key(const verification_key& other);
    verification_key(verification_key&& other);
    verification_key& operator=(verification_key&& other);

    ~verification_key() = default;

    sha256::hash sha256_hash();

    uint32_t composer_type;
    size_t circuit_size;
    size_t log_circuit_size;
    size_t num_public_inputs;

    barretenberg::evaluation_domain domain;

    std::shared_ptr<VerifierReferenceString> reference_string;

    std::map<std::string, barretenberg::g1::affine_element> commitments;

    PolynomialManifest polynomial_manifest;

    // This is a member variable so as to avoid recomputing it in the different places of the verifier algorithm.
    // Note that recomputing would also have added constraints to the recursive verifier circuit.
    barretenberg::fr z_pow_n; // ʓ^n (ʓ being the 'evaluation challenge')

    bool contains_recursive_proof = false;
    std::vector<uint32_t> recursive_proof_public_input_indices;
    size_t program_width = 3;

    // for serialization: update with new fields
    void msgpack_pack(auto& packer) const
    {
        verification_key_data data = { composer_type,
                                       static_cast<uint32_t>(circuit_size),
                                       static_cast<uint32_t>(num_public_inputs),
                                       commitments,
                                       contains_recursive_proof,
                                       recursive_proof_public_input_indices };
        packer.pack(data);
    }
    void msgpack_unpack(auto obj)
    {
        verification_key_data data = obj;
        auto env_crs = std::make_unique<proof_system::EnvReferenceStringFactory>();
        *this = verification_key{ std::move(data), env_crs->get_verifier_crs() };
    }
};

// specialize schema serialization
inline void msgpack_schema(auto& packer, proof_system::plonk::verification_key const&)
{
    packer.pack_schema(proof_system::plonk::verification_key_data{});
}

template <typename B> inline void read(B& buf, verification_key& key)
{
    auto env_crs = std::make_unique<proof_system::EnvReferenceStringFactory>();
    using serialize::read;
    verification_key_data vk_data;
    read(buf, vk_data);
    key = verification_key{ std::move(vk_data), env_crs->get_verifier_crs() };
}

template <typename B> inline void read(B& buf, std::shared_ptr<verification_key>& key)
{
    auto env_crs = std::make_unique<proof_system::EnvReferenceStringFactory>();
    using serialize::read;
    verification_key_data vk_data;
    read(buf, vk_data);
    key = std::make_shared<verification_key>(std::move(vk_data), env_crs->get_verifier_crs());
}

template <typename B> inline void write(B& buf, verification_key const& key)
{
    using serialize::write;
    write(buf, key.composer_type);
    write(buf, static_cast<uint32_t>(key.circuit_size));
    write(buf, static_cast<uint32_t>(key.num_public_inputs));
    write(buf, key.commitments);
    write(buf, key.contains_recursive_proof);
    write(buf, key.recursive_proof_public_input_indices);
}

inline std::ostream& operator<<(std::ostream& os, verification_key const& key)
{
    return os << "key.composer_type: " << key.composer_type << "\n"
              << "key.circuit_size: " << static_cast<uint32_t>(key.circuit_size) << "\n"
              << "key.num_public_inputs: " << static_cast<uint32_t>(key.num_public_inputs) << "\n"
              << "key.commitments: " << key.commitments << "\n"
              << "key.contains_recursive_proof: " << key.contains_recursive_proof << "\n"
              << "key.recursive_proof_public_input_indices: " << key.recursive_proof_public_input_indices << "\n";
};

} // namespace proof_system::plonk

// help our msgpack schema compiler with this struct
// Alias verification_key as verification_key_data
inline void msgpack_schema_pack(auto& packer, proof_system::plonk::verification_key const&)
{
    msgpack_schema_pack(packer, proof_system::plonk::verification_key_data{});
}