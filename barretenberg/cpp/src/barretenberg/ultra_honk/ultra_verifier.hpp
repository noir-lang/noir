#pragma once
#include "barretenberg/honk/proof_system/types/proof.hpp"
#include "barretenberg/srs/global_crs.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_flavor.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_flavor.hpp"
#include "barretenberg/sumcheck/instance//verifier_instance.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"
#include "barretenberg/ultra_honk/decider_verifier.hpp"

namespace bb {
template <typename Flavor> class UltraVerifier_ {
    using FF = typename Flavor::FF;
    using Commitment = typename Flavor::Commitment;
    using VerificationKey = typename Flavor::VerificationKey;
    using VerifierCommitmentKey = typename Flavor::VerifierCommitmentKey;
    using Transcript = typename Flavor::Transcript;
    using Instance = VerifierInstance_<Flavor>;
    using DeciderVerifier = DeciderVerifier_<Flavor>;

  public:
    explicit UltraVerifier_(const std::shared_ptr<VerificationKey>& verifier_key)
        : instance(std::make_shared<Instance>(verifier_key))
    {}

    bool verify_proof(const HonkProof& proof);

    std::shared_ptr<Transcript> transcript{ nullptr };
    std::shared_ptr<Instance> instance;
};

using UltraVerifier = UltraVerifier_<UltraFlavor>;
using UltraKeccakVerifier = UltraVerifier_<UltraKeccakFlavor>;
using MegaVerifier = UltraVerifier_<MegaFlavor>;

} // namespace bb
