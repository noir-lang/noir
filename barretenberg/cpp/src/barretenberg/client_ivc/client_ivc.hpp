#pragma once

#include "barretenberg/goblin/goblin.hpp"
#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/protogalaxy/decider_verifier.hpp"
#include "barretenberg/protogalaxy/protogalaxy_prover.hpp"
#include "barretenberg/protogalaxy/protogalaxy_verifier.hpp"
#include "barretenberg/sumcheck/instance/instances.hpp"
#include "barretenberg/ultra_honk/decider_prover.hpp"
#include <algorithm>

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

        size_t size() const { return folding_proof.size() + decider_proof.size() + goblin_proof.size(); }

        MSGPACK_FIELDS(folding_proof, decider_proof, goblin_proof);
    };

    // A debugging utility for tracking the max size of each block over all circuits in the IVC
    struct MaxBlockSizes {
        size_t ecc_op{ 0 };
        size_t pub_inputs{ 0 };
        size_t arithmetic{ 0 };
        size_t delta_range{ 0 };
        size_t elliptic{ 0 };
        size_t aux{ 0 };
        size_t lookup{ 0 };
        size_t busread{ 0 };
        size_t poseidon_external{ 0 };
        size_t poseidon_internal{ 0 };

        void update(ClientCircuit& circuit)
        {
            ecc_op = std::max(circuit.blocks.ecc_op.size(), ecc_op);
            pub_inputs = std::max(circuit.public_inputs.size(), pub_inputs);
            arithmetic = std::max(circuit.blocks.arithmetic.size(), arithmetic);
            delta_range = std::max(circuit.blocks.delta_range.size(), delta_range);
            elliptic = std::max(circuit.blocks.elliptic.size(), elliptic);
            aux = std::max(circuit.blocks.aux.size(), aux);
            lookup = std::max(circuit.blocks.lookup.size(), lookup);
            busread = std::max(circuit.blocks.busread.size(), busread);
            poseidon_external = std::max(circuit.blocks.poseidon_external.size(), poseidon_external);
            poseidon_internal = std::max(circuit.blocks.poseidon_internal.size(), poseidon_internal);
        }

        void print()
        {
            info("Minimum required block sizes for structured trace: ");
            info("goblin ecc op :\t", ecc_op);
            info("pub inputs    :\t", pub_inputs);
            info("arithmetic    :\t", arithmetic);
            info("delta range   :\t", delta_range);
            info("elliptic      :\t", elliptic);
            info("auxiliary     :\t", aux);
            info("lookups       :\t", lookup);
            info("busread       :\t", busread);
            info("poseidon ext  :\t", poseidon_external);
            info("poseidon int  :\t", poseidon_internal);
            info("");
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
    TraceStructure trace_structure = TraceStructure::NONE;

    // A flag indicating whether the IVC has been initialized with an initial instance
    bool initialized = false;

    void accumulate(ClientCircuit& circuit, const std::shared_ptr<VerificationKey>& precomputed_vk = nullptr);

    Proof prove();

    static bool verify(const Proof& proof,
                       const std::shared_ptr<VerifierInstance>& accumulator,
                       const std::shared_ptr<VerifierInstance>& final_verifier_instance,
                       const std::shared_ptr<ClientIVC::ECCVMVerificationKey>& eccvm_vk,
                       const std::shared_ptr<ClientIVC::TranslatorVerificationKey>& translator_vk);

    bool verify(Proof& proof, const std::vector<std::shared_ptr<VerifierInstance>>& verifier_instances);

    bool prove_and_verify();

    HonkProof decider_prove() const;

    std::vector<std::shared_ptr<VerificationKey>> precompute_folding_verification_keys(std::vector<ClientCircuit>);

    MaxBlockSizes max_block_sizes; // for tracking minimum block size requirements across an IVC
};
} // namespace bb