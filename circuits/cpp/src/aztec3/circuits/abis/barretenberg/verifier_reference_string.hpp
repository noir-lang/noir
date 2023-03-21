#pragma once

#include <memory>
#include "barretenberg/srs/reference_string/reference_string.hpp"
#include "barretenberg/common/streams.hpp"
#include "barretenberg/proof_system/verification_key/verification_key.hpp"

namespace serialize {

// TODO(AD): After Milestone 1, rewrite this with better injection mechanism.
std::shared_ptr<bonk::VerifierReferenceString> get_global_verifier_reference_string();
// TODO(AD): After Milestone 1, rewrite this with better injection mechanism.
void set_global_verifier_reference_string(std::shared_ptr<bonk::VerifierReferenceString> const& vrs);

inline void read(uint8_t const*& it, std::shared_ptr<bonk::verification_key>& key)
{
    // Note: matches the structure of write verification_key
    bonk::verification_key_data vk_data;
    read(it, vk_data);
    // TODO(AD): After Milestone 1, rewrite this with better injection mechanism.
    key = std::make_shared<bonk::verification_key>(std::move(vk_data), get_global_verifier_reference_string());
}

inline void write(std::vector<uint8_t>& buf, std::shared_ptr<bonk::verification_key> const& key)
{
    // Note: matches the structure of write verification_key
    write(buf, *key.get());
}

} // namespace serialize

namespace std {

inline std::ostream& operator<<(std::ostream& os, std::shared_ptr<bonk::verification_key> const& key)
{
    return os << "composer_type: " << key->composer_type << '\n'
              << "circuit_size: " << key->circuit_size << '\n'
              << "num_public_inputs: " << key->num_public_inputs << '\n'
              << "commitments: " << key->commitments << '\n'
              << "contains_recursive_proof: " << key->contains_recursive_proof << '\n'
              << "recursive_proof_public_input_indices: " << key->recursive_proof_public_input_indices << '\n';
}

} // namespace std