#include <crypto/sha256/sha256.hpp>
#include "verification_key.hpp"

namespace waffle {

verification_key::verification_key(const size_t num_gates,
                                   const size_t num_inputs,
                                   std::shared_ptr<VerifierReferenceString> const& crs)
    : n(num_gates)
    , num_public_inputs(num_inputs)
    , domain(n)
    , reference_string(crs)
{}

verification_key::verification_key(verification_key_data&& data, std::shared_ptr<VerifierReferenceString> const& crs)
    : composer_type(data.composer_type)
    , n(data.n)
    , num_public_inputs(data.num_public_inputs)
    , domain(n)
    , reference_string(crs)
    , constraint_selectors(std::move(data.constraint_selectors))
    , permutation_selectors(std::move(data.permutation_selectors))
    , contains_recursive_proof(data.contains_recursive_proof)
    , recursive_proof_public_input_indices(std::move(data.recursive_proof_public_input_indices))
{
    switch (composer_type) {
    case ComposerType::STANDARD: {
        std::copy(
            standard_polynomial_manifest, standard_polynomial_manifest + 12, std::back_inserter(polynomial_manifest));
        break;
    };
    case ComposerType::TURBO: {
        std::copy(turbo_polynomial_manifest, turbo_polynomial_manifest + 20, std::back_inserter(polynomial_manifest));
        break;
    };
    case ComposerType::PLOOKUP: {
        std::copy(
            plookup_polynomial_manifest, plookup_polynomial_manifest + 34, std::back_inserter(polynomial_manifest));
        break;
    };
    default: {
        throw_or_abort("Received invalid composer type");
    }
    }
}

verification_key::verification_key(const verification_key& other)
    : composer_type(other.composer_type)
    , n(other.n)
    , num_public_inputs(other.num_public_inputs)
    , domain(other.domain)
    , reference_string(other.reference_string)
    , constraint_selectors(other.constraint_selectors)
    , permutation_selectors(other.permutation_selectors)
    , polynomial_manifest(other.polynomial_manifest)
    , contains_recursive_proof(other.contains_recursive_proof)
    , recursive_proof_public_input_indices(other.recursive_proof_public_input_indices)
{}

verification_key::verification_key(verification_key&& other)
    : composer_type(other.composer_type)
    , n(other.n)
    , num_public_inputs(other.num_public_inputs)
    , domain(other.domain)
    , reference_string(other.reference_string)
    , constraint_selectors(other.constraint_selectors)
    , permutation_selectors(other.permutation_selectors)
    , polynomial_manifest(other.polynomial_manifest)
    , contains_recursive_proof(other.contains_recursive_proof)
    , recursive_proof_public_input_indices(other.recursive_proof_public_input_indices)
{}

verification_key& verification_key::operator=(verification_key&& other)
{
    composer_type = other.composer_type;
    n = other.n;
    num_public_inputs = other.num_public_inputs;
    reference_string = std::move(other.reference_string);
    constraint_selectors = std::move(other.constraint_selectors);
    permutation_selectors = std::move(other.permutation_selectors);
    polynomial_manifest = std::move(other.polynomial_manifest);
    domain = std::move(other.domain);
    contains_recursive_proof = (other.contains_recursive_proof);
    recursive_proof_public_input_indices = std::move(other.recursive_proof_public_input_indices);
    return *this;
}

sha256::hash verification_key::sha256_hash()
{
    std::vector<uint256_t> vk_data;
    vk_data.emplace_back(n);
    vk_data.emplace_back(num_public_inputs);
    for (auto& commitment_entry : constraint_selectors) {
        vk_data.emplace_back(commitment_entry.second.x);
        vk_data.emplace_back(commitment_entry.second.y);
    }
    for (auto& commitment_entry : permutation_selectors) {
        vk_data.emplace_back(commitment_entry.second.x);
        vk_data.emplace_back(commitment_entry.second.y);
    }
    vk_data.emplace_back(contains_recursive_proof);
    for (auto& index : recursive_proof_public_input_indices) {
        vk_data.emplace_back(index);
    }
    return sha256::sha256(to_buffer(vk_data));
}

} // namespace waffle