#pragma once
#include "barretenberg/honk/proof_system/types/proof.hpp"
#include "barretenberg/srs/global_crs.hpp"
#include "barretenberg/stdlib_circuit_builders/goblin_ultra_flavor.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_flavor.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"

namespace bb {
template <typename Flavor> class UltraVerifier_ {
    using FF = typename Flavor::FF;
    using Commitment = typename Flavor::Commitment;
    using VerificationKey = typename Flavor::VerificationKey;
    using VerifierCommitmentKey = typename Flavor::VerifierCommitmentKey;
    using Transcript = typename Flavor::Transcript;

  public:
    explicit UltraVerifier_(const std::shared_ptr<Transcript>& transcript,
                            const std::shared_ptr<VerificationKey>& verifier_key = nullptr);

    explicit UltraVerifier_(const std::shared_ptr<VerificationKey>& verifier_key);
    UltraVerifier_(UltraVerifier_&& other);

    UltraVerifier_& operator=(const UltraVerifier_& other) = delete;
    UltraVerifier_& operator=(UltraVerifier_&& other);

    bool verify_proof(const HonkProof& proof);

    std::shared_ptr<VerificationKey> key;
    std::shared_ptr<Transcript> transcript;
};

using UltraVerifier = UltraVerifier_<UltraFlavor>;
using GoblinUltraVerifier = UltraVerifier_<GoblinUltraFlavor>;

} // namespace bb
