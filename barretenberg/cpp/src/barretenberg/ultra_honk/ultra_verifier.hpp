#pragma once
#include "barretenberg/flavor/goblin_ultra.hpp"
#include "barretenberg/flavor/ultra.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"

namespace proof_system::honk {
template <typename Flavor> class UltraVerifier_ {
    using FF = typename Flavor::FF;
    using Commitment = typename Flavor::Commitment;
    using VerificationKey = typename Flavor::VerificationKey;
    using VerifierCommitmentKey = typename Flavor::VerifierCommitmentKey;

  public:
    explicit UltraVerifier_(std::shared_ptr<VerificationKey> verifier_key = nullptr);
    UltraVerifier_(UltraVerifier_&& other);

    UltraVerifier_& operator=(const UltraVerifier_& other) = delete;
    UltraVerifier_& operator=(UltraVerifier_&& other);

    bool verify_proof(const plonk::proof& proof);

    std::shared_ptr<VerificationKey> key;
    std::map<std::string, Commitment> commitments;
    std::shared_ptr<VerifierCommitmentKey> pcs_verification_key;
    BaseTranscript transcript;
};

extern template class UltraVerifier_<honk::flavor::Ultra>;
extern template class UltraVerifier_<honk::flavor::GoblinUltra>;

using UltraVerifier = UltraVerifier_<honk::flavor::Ultra>;

} // namespace proof_system::honk
