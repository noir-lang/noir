

#pragma once
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"
#include "barretenberg/vm/generated/spike_flavor.hpp"

namespace bb {
class SpikeVerifier {
    using Flavor = SpikeFlavor;
    using FF = Flavor::FF;
    using Commitment = Flavor::Commitment;
    using VerificationKey = Flavor::VerificationKey;
    using VerifierCommitmentKey = Flavor::VerifierCommitmentKey;
    using Transcript = Flavor::Transcript;

  public:
    explicit SpikeVerifier(std::shared_ptr<VerificationKey> verifier_key = nullptr);
    SpikeVerifier(SpikeVerifier&& other) noexcept;
    SpikeVerifier(const SpikeVerifier& other) = delete;

    SpikeVerifier& operator=(const SpikeVerifier& other) = delete;
    SpikeVerifier& operator=(SpikeVerifier&& other) noexcept;

    bool verify_proof(const HonkProof& proof, const std::vector<FF>& public_inputs);

    std::shared_ptr<VerificationKey> key;
    std::map<std::string, Commitment> commitments;
    std::shared_ptr<VerifierCommitmentKey> pcs_verification_key;
    std::shared_ptr<Transcript> transcript;
};

} // namespace bb
