#pragma once
#include "barretenberg/plonk/proof_system/proving_key/proving_key.hpp"
#include "barretenberg/plonk/proof_system/verification_key/verification_key.hpp"

namespace proof_system::honk {

/**
 * @brief Computes the verification key.
 * @details Does the following
 * (1) commitments to the selector, permutation, and lagrange (first/last) polynomials,
 * (2) sets the polynomial manifest using the data from proving key.
 *
 * @tparam Flavor
 * @param proving_key A completely construted proving key.
 * @param vrs The reference string used by the verifier
 * @return std::shared_ptr<typename Flavor::VerificationKey>
 */
// TODO(Cody): This function is not in use?
template <typename Flavor>
std::shared_ptr<typename Flavor::VerificationKey> compute_verification_key_common(
    std::shared_ptr<typename Flavor::ProvingKey> const& proving_key,
    std::shared_ptr<barretenberg::srs::factories::VerifierCrs<typename Flavor::Curve>> const& vrs)
{
    auto verification_key = std::make_shared<typename Flavor::VerificationKey>(
        proving_key->circuit_size, proving_key->num_public_inputs, vrs);

    auto commitment_key = typename Flavor::PCSParams::CommitmentKey(proving_key->circuit_size, proving_key->crs);

    size_t poly_idx = 0; // TODO(#391) zip
    for (auto& polynomial : proving_key) {
        verification_key[poly_idx] = commitment_key.commit(polynomial);
        ++polynomial_idx;
    }

    return verification_key;
}

} // namespace proof_system::honk
