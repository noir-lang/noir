#pragma once
#include "barretenberg/honk/flavor/standard.hpp"
#include "barretenberg/honk/flavor/standard_grumpkin.hpp"
#include "barretenberg/honk/sumcheck/sumcheck.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"

namespace proof_system::honk {
template <typename Flavor> class StandardVerifier_ {
    using FF = typename Flavor::FF;
    using Commitment = typename Flavor::Commitment;
    using VerificationKey = typename Flavor::VerificationKey;
    using PCSVerificationKey = typename Flavor::PCSParams::VerificationKey;

  public:
    StandardVerifier_(std::shared_ptr<VerificationKey> verifier_key = nullptr);
    StandardVerifier_(StandardVerifier_&& other);
    StandardVerifier_(const StandardVerifier_& other) = delete;
    StandardVerifier_& operator=(const StandardVerifier_& other) = delete;
    StandardVerifier_& operator=(StandardVerifier_&& other);

    bool verify_proof(const plonk::proof& proof);

    std::shared_ptr<VerificationKey> key;
    std::map<std::string, Commitment> commitments;
    std::map<std::string, FF> pcs_fr_elements;
    std::shared_ptr<PCSVerificationKey> pcs_verification_key;
    VerifierTranscript<FF> transcript;
};

extern template class StandardVerifier_<honk::flavor::Standard>;
extern template class StandardVerifier_<honk::flavor::StandardGrumpkin>;

using StandardVerifier = StandardVerifier_<honk::flavor::Standard>;

} // namespace proof_system::honk
