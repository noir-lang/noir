#pragma once
#include "barretenberg/common/streams.hpp"
#include "barretenberg/crypto/sha256/sha256.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/plonk/proof_system/types/polynomial_manifest.hpp"
#include "barretenberg/polynomials/evaluation_domain.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include "barretenberg/srs/global_crs.hpp"
#include <map>

namespace bb::plonk {

struct verification_key_data {
    uint32_t circuit_type;
    uint32_t circuit_size;
    uint32_t num_public_inputs;
    std::map<std::string, bb::g1::affine_element> commitments;
    bool contains_recursive_proof = false;
    std::vector<uint32_t> recursive_proof_public_input_indices;

    // for serialization: update with any new fields
    MSGPACK_FIELDS(circuit_type,
                   circuit_size,
                   num_public_inputs,
                   commitments,
                   contains_recursive_proof,
                   recursive_proof_public_input_indices);
    [[nodiscard]] bb::fr hash_native(size_t hash_index = 0) const;
};

inline std::ostream& operator<<(std::ostream& os, verification_key_data const& key)
{
    return os << "key.circuit_type: " << static_cast<uint32_t>(key.circuit_type) << "\n"
              << "key.circuit_size: " << static_cast<uint32_t>(key.circuit_size) << "\n"
              << "key.num_public_inputs: " << static_cast<uint32_t>(key.num_public_inputs) << "\n"
              << "key.commitments: " << key.commitments << "\n"
              << "key.contains_recursive_proof: " << key.contains_recursive_proof << "\n"
              << "key.recursive_proof_public_input_indices: " << key.recursive_proof_public_input_indices << "\n";
};

inline bool operator==(verification_key_data const& lhs, verification_key_data const& rhs)
{
    return lhs.circuit_type == rhs.circuit_type && lhs.circuit_size == rhs.circuit_size &&
           lhs.num_public_inputs == rhs.num_public_inputs && lhs.commitments == rhs.commitments;
}

struct verification_key {
    // default constructor needed for msgpack unpack
    verification_key() = default;
    verification_key(verification_key_data&& data,
                     std::shared_ptr<bb::srs::factories::VerifierCrs<curve::BN254>> const& crs);
    verification_key(size_t num_gates,
                     size_t num_inputs,
                     std::shared_ptr<bb::srs::factories::VerifierCrs<curve::BN254>> const& crs,
                     CircuitType circuit_type);

    verification_key(const verification_key& other);
    verification_key(verification_key&& other) noexcept;
    verification_key& operator=(verification_key&& other) noexcept;
    verification_key& operator=(const verification_key& other) = delete;
    ~verification_key() = default;

    sha256::hash sha256_hash();

    [[nodiscard]] verification_key_data as_data() const
    {
        return {
            .circuit_type = static_cast<uint32_t>(circuit_type),
            .circuit_size = static_cast<uint32_t>(circuit_size),
            .num_public_inputs = static_cast<uint32_t>(num_public_inputs),
            .commitments = commitments,
            .contains_recursive_proof = contains_recursive_proof,
            .recursive_proof_public_input_indices = recursive_proof_public_input_indices,
        };
    }

    CircuitType circuit_type;
    size_t circuit_size;
    size_t log_circuit_size;
    size_t num_public_inputs;

    bb::evaluation_domain domain;

    std::shared_ptr<bb::srs::factories::VerifierCrs<curve::BN254>> reference_string;

    std::map<std::string, bb::g1::affine_element> commitments;

    PolynomialManifest polynomial_manifest;

    // This is a member variable so as to avoid recomputing it in the different places of the verifier algorithm.
    // Note that recomputing would also have added constraints to the recursive verifier circuit.
    bb::fr z_pow_n; // ʓ^n (ʓ being the 'evaluation challenge')

    bool contains_recursive_proof = false;
    std::vector<uint32_t> recursive_proof_public_input_indices;
    size_t program_width = 3;

    // for serialization: update with new fields
    void msgpack_pack(auto& packer) const
    {
        verification_key_data data = { static_cast<uint32_t>(circuit_type),
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
        *this = verification_key{ std::move(data), bb::srs::get_crs_factory()->get_verifier_crs() };
    }
    // Alias verification_key as verification_key_data in the schema
    void msgpack_schema(auto& packer) const { packer.pack_schema(bb::plonk::verification_key_data{}); }
};

template <typename B> inline void read(B& buf, verification_key& key)
{
    using serialize::read;
    verification_key_data vk_data;
    read(buf, vk_data);
    key = verification_key{ std::move(vk_data), bb::srs::get_crs_factory()->get_verifier_crs() };
}

template <typename B> inline void read(B& buf, std::shared_ptr<verification_key>& key)
{
    using serialize::read;
    verification_key_data vk_data;
    read(buf, vk_data);
    key = std::make_shared<verification_key>(std::move(vk_data), bb::srs::get_crs_factory()->get_verifier_crs());
}

template <typename B> inline void write(B& buf, verification_key const& key)
{
    using serialize::write;
    write(buf, key.as_data());
}

inline std::ostream& operator<<(std::ostream& os, verification_key const& key)
{
    return os << key.as_data();
};

} // namespace bb::plonk
