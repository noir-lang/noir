

#pragma once
#include "barretenberg/flavor/generated/AvmMini_flavor.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"

namespace bb::honk {
class AvmMiniVerifier {
    using Flavor = honk::flavor::AvmMiniFlavor;
    using FF = Flavor::FF;
    using Commitment = Flavor::Commitment;
    using VerificationKey = Flavor::VerificationKey;
    using VerifierCommitmentKey = Flavor::VerifierCommitmentKey;
    using Transcript = Flavor::Transcript;

  public:
    explicit AvmMiniVerifier(std::shared_ptr<VerificationKey> verifier_key = nullptr);
    AvmMiniVerifier(AvmMiniVerifier&& other) noexcept;
    AvmMiniVerifier(const AvmMiniVerifier& other) = delete;

    AvmMiniVerifier& operator=(const AvmMiniVerifier& other) = delete;
    AvmMiniVerifier& operator=(AvmMiniVerifier&& other) noexcept;

    bool verify_proof(const plonk::proof& proof);

    std::shared_ptr<VerificationKey> key;
    std::map<std::string, Commitment> commitments;
    std::shared_ptr<VerifierCommitmentKey> pcs_verification_key;
    std::shared_ptr<Transcript> transcript;
};

} // namespace bb::honk
