#pragma once

#include "barretenberg/goblin/goblin.hpp"
#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/plonk_honk_shared/arithmetization/max_block_size_tracker.hpp"
#include "barretenberg/protogalaxy/decider_verifier.hpp"
#include "barretenberg/protogalaxy/protogalaxy_prover.hpp"
#include "barretenberg/protogalaxy/protogalaxy_verifier.hpp"
#include "barretenberg/sumcheck/instance/instances.hpp"
#include "barretenberg/ultra_honk/decider_prover.hpp"
#include <algorithm>

namespace bb {

/**
 * @brief The IVC scheme used by the aztec client for private function execution
 * @details Combines Protogalaxy with Goblin to accumulate one circuit instance at a time with efficient EC group
 * operations. It is assumed that the circuits being accumulated correspond alternatingly to an app and a kernel, as is
 * the case in Aztec. Two recursive folding verifiers are appended to each kernel (except the first one) to verify the
 * folding of a previous kernel and an app/function circuit. Due to this structure it is enforced that the total number
 * of circuits being accumulated is even.
 *
 */
class AztecIVC {

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

        size_t size() const { return folding_proof.size() + decider_proof.size() + goblin_proof.size(); }

        MSGPACK_FIELDS(folding_proof, decider_proof, goblin_proof);
    };

    struct FoldingVerifierInputs {
        FoldProof proof;
        std::shared_ptr<VerificationKey> instance_vk;
    };

    // Utility for tracking the max size of each block across the full IVC
    MaxBlockSizeTracker max_block_size_tracker;

  private:
    using ProverFoldOutput = FoldingResult<Flavor>;

  public:
    GoblinProver goblin;

    ProverFoldOutput fold_output; // prover accumulator instance and fold proof

    std::shared_ptr<VerifierInstance> verifier_accumulator; // verifier accumulator instance
    std::shared_ptr<VerificationKey> instance_vk;           // verification key for instance to be folded

    // Set of pairs of {fold_proof, verification_key} to be recursively verified
    std::vector<FoldingVerifierInputs> verification_queue;

    // A flag indicating whether or not to construct a structured trace in the ProverInstance
    TraceStructure trace_structure = TraceStructure::NONE;

    // The number of circuits processed into the IVC
    size_t circuit_count = 0;

    void accumulate(ClientCircuit& circuit, const std::shared_ptr<VerificationKey>& precomputed_vk = nullptr);

    Proof prove();

    static bool verify(const Proof& proof,
                       const std::shared_ptr<VerifierInstance>& accumulator,
                       const std::shared_ptr<VerifierInstance>& final_verifier_instance,
                       const std::shared_ptr<AztecIVC::ECCVMVerificationKey>& eccvm_vk,
                       const std::shared_ptr<AztecIVC::TranslatorVerificationKey>& translator_vk);

    bool verify(Proof& proof, const std::vector<std::shared_ptr<VerifierInstance>>& verifier_instances);

    bool prove_and_verify();

    HonkProof decider_prove() const;

    std::vector<std::shared_ptr<VerificationKey>> precompute_folding_verification_keys(std::vector<ClientCircuit>);
};
} // namespace bb