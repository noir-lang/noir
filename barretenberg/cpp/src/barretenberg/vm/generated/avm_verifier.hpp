

#pragma once
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"
#include "barretenberg/vm/generated/avm_flavor.hpp"

namespace bb {
class AvmVerifier {
    using Flavor = AvmFlavor;
    using FF = Flavor::FF;
    using Commitment = Flavor::Commitment;
    using VerificationKey = Flavor::VerificationKey;
    using VerifierCommitmentKey = Flavor::VerifierCommitmentKey;
    using Transcript = Flavor::Transcript;

  public:
    explicit AvmVerifier(std::shared_ptr<VerificationKey> verifier_key = nullptr);
    AvmVerifier(AvmVerifier&& other) noexcept;
    AvmVerifier(const AvmVerifier& other) = delete;

    AvmVerifier& operator=(const AvmVerifier& other) = delete;
    AvmVerifier& operator=(AvmVerifier&& other) noexcept;

    bool verify_proof(const HonkProof& proof);

    std::shared_ptr<VerificationKey> key;
    std::map<std::string, Commitment> commitments;
    std::shared_ptr<VerifierCommitmentKey> pcs_verification_key;
    std::shared_ptr<Transcript> transcript;
};

} // namespace bb
