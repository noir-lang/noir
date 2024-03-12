#pragma once

#include "barretenberg/goblin/goblin.hpp"
#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/protogalaxy/decider_prover.hpp"
#include "barretenberg/protogalaxy/decider_verifier.hpp"
#include "barretenberg/protogalaxy/protogalaxy_prover.hpp"
#include "barretenberg/protogalaxy/protogalaxy_verifier.hpp"
#include "barretenberg/sumcheck/instance/instances.hpp"

namespace bb {

/**
 * @brief The IVC interface to be used by the aztec client for private function execution
 * @details Combines Protogalaxy with Goblin to accumulate one circuit instance at a time with efficient EC group
 * operations
 *
 */
class ClientIVC {

  public:
    using Flavor = GoblinUltraFlavor;
    using VerificationKey = Flavor::VerificationKey;
    using FF = Flavor::FF;
    using FoldProof = std::vector<FF>;
    using ProverAccumulator = std::shared_ptr<ProverInstance_<Flavor>>;
    using VerifierAccumulator = std::shared_ptr<VerifierInstance_<Flavor>>;
    using ProverInstance = ProverInstance_<Flavor>;
    using VerifierInstance = VerifierInstance_<Flavor>;
    using ClientCircuit = GoblinUltraCircuitBuilder; // can only be GoblinUltra
    using DeciderProver = DeciderProver_<Flavor>;
    using DeciderVerifier = DeciderVerifier_<Flavor>;
    using ProverInstances = ProverInstances_<Flavor>;
    using FoldingProver = ProtoGalaxyProver_<ProverInstances>;
    using VerifierInstances = VerifierInstances_<Flavor>;
    using FoldingVerifier = ProtoGalaxyVerifier_<VerifierInstances>;

    // A full proof for the IVC scheme
    struct Proof {
        FoldProof fold_proof; // final fold proof
        HonkProof decider_proof;
        Goblin::Proof goblin_proof;
    };

    struct PrecomputedVerificationKeys {
        std::shared_ptr<VerificationKey> first_func_vk;
        std::shared_ptr<VerificationKey> func_vk;
        std::shared_ptr<VerificationKey> first_kernel_vk;
        std::shared_ptr<VerificationKey> kernel_vk;
    };

  private:
    using ProverFoldOutput = FoldingResult<Flavor>;
    // Note: We need to save the last instance that was folded in order to compute its verification key, this will not
    // be needed in the real IVC as they are provided as inputs

  public:
    Goblin goblin;
    ProverFoldOutput prover_fold_output;
    ProverAccumulator prover_accumulator;
    PrecomputedVerificationKeys vks;
    // Note: We need to save the last instance that was folded in order to compute its verification key, this will not
    // be needed in the real IVC as they are provided as inputs
    std::shared_ptr<ProverInstance> prover_instance;

    void initialize(ClientCircuit& circuit);

    FoldProof accumulate(ClientCircuit& circuit);

    Proof prove();

    bool verify(Proof& proof, const std::vector<VerifierAccumulator>& verifier_instances);

    HonkProof decider_prove() const;

    void decider_prove_and_verify(const VerifierAccumulator&) const;

    void precompute_folding_verification_keys();
};
} // namespace bb