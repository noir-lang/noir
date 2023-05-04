#pragma once
#include "barretenberg/honk/flavor/standard.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/honk/sumcheck/sumcheck.hpp"

namespace proof_system::honk {
template <typename Flavor> class StandardVerifier_ {
    using FF = typename Flavor::FF;
    using Commitment = typename Flavor::Commitment;
    using VerificationKey = typename Flavor::VerificationKey;
    using PCSVerificationKey = typename Flavor::PCSParams::VK;

  public:
    StandardVerifier_(std::shared_ptr<VerificationKey> verifier_key = nullptr);
    StandardVerifier_(StandardVerifier_&& other);
    StandardVerifier_(const StandardVerifier_& other) = delete;
    StandardVerifier_& operator=(const StandardVerifier_& other) = delete;
    StandardVerifier_& operator=(StandardVerifier_&& other);

    bool verify_proof(const plonk::proof& proof);

    std::shared_ptr<VerificationKey> key;
    std::map<std::string, Commitment> kate_g1_elements;
    std::map<std::string, FF> kate_fr_elements;
    std::shared_ptr<PCSVerificationKey> kate_verification_key;
    VerifierTranscript<FF> transcript;
};

extern template class StandardVerifier_<honk::flavor::Standard>;

using StandardVerifier = StandardVerifier_<honk::flavor::Standard>;

} // namespace proof_system::honk
