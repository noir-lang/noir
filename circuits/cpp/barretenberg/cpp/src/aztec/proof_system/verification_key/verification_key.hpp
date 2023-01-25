#pragma once
#include <map>
#include <srs/reference_string/reference_string.hpp>
#include <polynomials/evaluation_domain.hpp>
#include <crypto/sha256/sha256.hpp>
#include "../../proof_system/types/polynomial_manifest.hpp"

namespace waffle {

struct verification_key_data {
    uint32_t composer_type;
    uint32_t n;
    uint32_t num_public_inputs;
    std::map<std::string, barretenberg::g1::affine_element> constraint_selectors;
    std::map<std::string, barretenberg::g1::affine_element> permutation_selectors;
    bool contains_recursive_proof = false;
    std::vector<uint32_t> recursive_proof_public_input_indices;
};

template <typename B> inline void read(B& buf, verification_key_data& key)
{
    using serialize::read;
    read(buf, key.composer_type);
    read(buf, key.n);
    read(buf, key.num_public_inputs);
    read(buf, key.constraint_selectors);
    read(buf, key.permutation_selectors);
    read(buf, key.contains_recursive_proof);
    read(buf, key.recursive_proof_public_input_indices);
}

template <typename B> inline void write(B& buf, verification_key_data const& key)
{
    using serialize::write;
    write(buf, key.composer_type);
    write(buf, key.n);
    write(buf, key.num_public_inputs);
    write(buf, key.constraint_selectors);
    write(buf, key.permutation_selectors);
    write(buf, key.contains_recursive_proof);
    write(buf, key.recursive_proof_public_input_indices);
}

inline bool operator==(verification_key_data const& lhs, verification_key_data const& rhs)
{
    return lhs.composer_type == rhs.composer_type && lhs.n == rhs.n && lhs.num_public_inputs == rhs.num_public_inputs &&
           lhs.constraint_selectors == rhs.constraint_selectors &&
           lhs.permutation_selectors == rhs.permutation_selectors;
}

struct verification_key {
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
    size_t n;
    size_t num_public_inputs;

    barretenberg::evaluation_domain domain;

    std::shared_ptr<VerifierReferenceString> reference_string;

    std::map<std::string, barretenberg::g1::affine_element> constraint_selectors;

    std::map<std::string, barretenberg::g1::affine_element> permutation_selectors;

    PolynomialManifest polynomial_manifest;

    // This is a member variable so as to avoid recomputing it in the different places of the verifier algorithm.
    // Note that recomputing would also have added constraints to the recursive verifier circuit.
    barretenberg::fr z_pow_n; // ʓ^n (ʓ being the 'evaluation challenge')

    bool contains_recursive_proof = false;
    std::vector<uint32_t> recursive_proof_public_input_indices;
    size_t program_width = 3;
};

template <typename B> inline void write(B& buf, verification_key const& key)
{
    using serialize::write;
    write(buf, key.composer_type);
    write(buf, static_cast<uint32_t>(key.n));
    write(buf, static_cast<uint32_t>(key.num_public_inputs));
    write(buf, key.constraint_selectors);
    write(buf, key.permutation_selectors);
    write(buf, key.contains_recursive_proof);
    write(buf, key.recursive_proof_public_input_indices);
}

} // namespace waffle