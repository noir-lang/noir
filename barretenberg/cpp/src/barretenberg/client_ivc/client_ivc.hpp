#pragma once

#include "barretenberg/goblin/goblin.hpp"
#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/protogalaxy/decider_verifier.hpp"
#include "barretenberg/protogalaxy/protogalaxy_prover.hpp"
#include "barretenberg/protogalaxy/protogalaxy_verifier.hpp"
#include "barretenberg/sumcheck/instance/instances.hpp"
#include "barretenberg/ultra_honk/decider_prover.hpp"

namespace bb {

/**
 * @brief The IVC interface to be used by the aztec client for private function execution
 * @details Combines Protogalaxy with Goblin to accumulate one circuit instance at a time with efficient EC group
 * operations
 *
 */
class ClientIVC {

  public:
    using Flavor = MegaFlavor;
    using VerificationKey = Flavor::VerificationKey;
    using FF = Flavor::FF;
    using FoldProof = std::vector<FF>;
    using ProverInstance = ProverInstance_<Flavor>;
    using VerifierInstance = VerifierInstance_<Flavor>;
    using ClientCircuit = MegaCircuitBuilder; // can only be Mega
    using DeciderProver = DeciderProver_<Flavor>;
    using DeciderVerifier = DeciderVerifier_<Flavor>;
    using ProverInstances = ProverInstances_<Flavor>;
    using FoldingProver = ProtoGalaxyProver_<ProverInstances>;
    using VerifierInstances = VerifierInstances_<Flavor>;
    using FoldingVerifier = ProtoGalaxyVerifier_<VerifierInstances>;
    using ECCVMVerificationKey = bb::ECCVMFlavor::VerificationKey;
    using TranslatorVerificationKey = bb::TranslatorFlavor::VerificationKey;

    using GURecursiveFlavor = MegaRecursiveFlavor_<bb::MegaCircuitBuilder>;
    using RecursiveVerifierInstances = bb::stdlib::recursion::honk::RecursiveVerifierInstances_<GURecursiveFlavor, 2>;
    using FoldingRecursiveVerifier =
        bb::stdlib::recursion::honk::ProtoGalaxyRecursiveVerifier_<RecursiveVerifierInstances>;

    // A full proof for the IVC scheme
    struct Proof {
        FoldProof folding_proof; // final fold proof
        HonkProof decider_proof;
        GoblinProof goblin_proof;

        std::vector<FF> to_buffer() const
        {
            size_t proof_size = folding_proof.size() + decider_proof.size() + goblin_proof.size();

            std::vector<FF> result;
            result.reserve(proof_size);
            const auto insert = [&result](const std::vector<FF>& buf) {
                result.insert(result.end(), buf.begin(), buf.end());
            };
            insert(folding_proof);
            insert(decider_proof);
            insert(goblin_proof.to_buffer());
            return result;
        }
    };

  private:
    using ProverFoldOutput = FoldingResult<Flavor>;
    // Note: We need to save the last instance that was folded in order to compute its verification key, this will not
    // be needed in the real IVC as they are provided as inputs

  public:
    GoblinProver goblin;
    ProverFoldOutput fold_output;
    std::shared_ptr<ProverInstance> prover_accumulator;
    std::shared_ptr<VerifierInstance> verifier_accumulator;
    // Note: We need to save the last instance that was folded in order to compute its verification key, this will not
    // be needed in the real IVC as they are provided as inputs
    std::shared_ptr<ProverInstance> prover_instance;
    std::shared_ptr<VerificationKey> instance_vk;

    // A flag indicating whether or not to construct a structured trace in the ProverInstance
    bool structured_flag = false;

    // A flag indicating whether the IVC has been initialized with an initial instance
    bool initialized = false;

    void accumulate(ClientCircuit& circuit, const std::shared_ptr<VerificationKey>& precomputed_vk = nullptr);

    Proof prove();

    bool verify(Proof& proof, const std::vector<std::shared_ptr<VerifierInstance>>& verifier_instances);

    bool prove_and_verify();

    HonkProof decider_prove() const;

    std::vector<std::shared_ptr<VerificationKey>> precompute_folding_verification_keys(std::vector<ClientCircuit>);
};
} // namespace bb