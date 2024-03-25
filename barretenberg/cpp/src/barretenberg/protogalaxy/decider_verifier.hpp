#pragma once
#include "barretenberg/honk/proof_system/types/proof.hpp"
#include "barretenberg/srs/global_crs.hpp"
#include "barretenberg/stdlib_circuit_builders/goblin_ultra_flavor.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_flavor.hpp"
#include "barretenberg/sumcheck/instance/verifier_instance.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"

namespace bb {
template <typename Flavor> class DeciderVerifier_ {
    using FF = typename Flavor::FF;
    using Commitment = typename Flavor::Commitment;
    using VerificationKey = typename Flavor::VerificationKey;
    using VerifierCommitmentKey = typename Flavor::VerifierCommitmentKey;
    using Transcript = typename Flavor::Transcript;
    using VerifierInstance = VerifierInstance_<Flavor>;

  public:
    explicit DeciderVerifier_();
    explicit DeciderVerifier_(const std::shared_ptr<VerifierInstance>& accumulator,
                              const std::shared_ptr<Transcript>& transcript = std::make_shared<Transcript>());

    bool verify_proof(const HonkProof& proof);

    std::shared_ptr<VerificationKey> key;
    std::map<std::string, Commitment> commitments;
    std::shared_ptr<VerifierInstance> accumulator;
    std::shared_ptr<VerifierCommitmentKey> pcs_verification_key;
    std::shared_ptr<Transcript> transcript;
};

using UltraDeciderVerifier = DeciderVerifier_<UltraFlavor>;
using GoblinUltraDeciderVerifier = DeciderVerifier_<GoblinUltraFlavor>;

} // namespace bb
