#pragma once

#include "barretenberg/eccvm/eccvm_circuit_builder.hpp"
#include "barretenberg/eccvm/eccvm_composer.hpp"
#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/plonk_honk_shared/instance_inspector.hpp"
#include "barretenberg/stdlib/honk_recursion/verifier/merge_recursive_verifier.hpp"
#include "barretenberg/stdlib_circuit_builders/goblin_ultra_circuit_builder.hpp"
#include "barretenberg/stdlib_circuit_builders/goblin_ultra_flavor.hpp"
#include "barretenberg/translator_vm/goblin_translator_circuit_builder.hpp"
#include "barretenberg/translator_vm/goblin_translator_prover.hpp"
#include "barretenberg/translator_vm/goblin_translator_verifier.hpp"
#include "barretenberg/ultra_honk/merge_prover.hpp"
#include "barretenberg/ultra_honk/merge_verifier.hpp"
#include "barretenberg/ultra_honk/ultra_prover.hpp"
#include "barretenberg/ultra_honk/ultra_verifier.hpp"

namespace bb {

class Goblin {
    using GoblinUltraCircuitBuilder = bb::GoblinUltraCircuitBuilder;
    using Commitment = GoblinUltraFlavor::Commitment;
    using FF = GoblinUltraFlavor::FF;

  public:
    using Builder = GoblinUltraCircuitBuilder;
    using Fr = bb::fr;
    using Transcript = NativeTranscript;
    using GoblinUltraProverInstance = ProverInstance_<GoblinUltraFlavor>;
    using OpQueue = bb::ECCOpQueue;
    using ECCVMFlavor = bb::ECCVMFlavor;
    using ECCVMBuilder = bb::ECCVMCircuitBuilder;
    using ECCVMComposer = bb::ECCVMComposer;
    using ECCVMProver = bb::ECCVMProver_<ECCVMFlavor>;
    using TranslatorBuilder = bb::GoblinTranslatorCircuitBuilder;
    using TranslatorProver = bb::GoblinTranslatorProver;
    using RecursiveMergeVerifier = bb::stdlib::recursion::goblin::MergeRecursiveVerifier_<GoblinUltraCircuitBuilder>;
    using MergeProver = bb::MergeProver_<GoblinUltraFlavor>;
    using MergeVerifier = bb::MergeVerifier_<GoblinUltraFlavor>;
    using VerificationKey = GoblinUltraFlavor::VerificationKey;
    /**
     * @brief Output of goblin::accumulate; an Ultra proof and the corresponding verification key
     *
     */
    struct AccumulationOutput {
        HonkProof proof;
        std::shared_ptr<GoblinUltraFlavor::VerificationKey> verification_key;
    };

    struct Proof {
        HonkProof merge_proof;
        HonkProof eccvm_proof;
        HonkProof translator_proof;
        TranslationEvaluations translation_evaluations;
        std::vector<Fr> to_buffer()
        {
            // ACIRHACK: so much copying and duplication added here and elsewhere
            std::vector<Fr> translation_evaluations_buf; // = translation_evaluations.to_buffer();
            size_t proof_size =
                merge_proof.size() + eccvm_proof.size() + translator_proof.size() + translation_evaluations_buf.size();

            std::vector<Fr> result(proof_size);
            const auto insert = [&result](const std::vector<Fr>& buf) {
                result.insert(result.end(), buf.begin(), buf.end());
            };
            insert(merge_proof);
            insert(eccvm_proof);
            insert(translator_proof);
            insert(translation_evaluations_buf);
            return result;
        }
    };

    std::shared_ptr<OpQueue> op_queue = std::make_shared<OpQueue>();

    HonkProof merge_proof;
    Proof goblin_proof;

    // on the first call to accumulate there is no merge proof to verify
    bool merge_proof_exists{ false };

  private:
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/798) unique_ptr use is a hack
    std::unique_ptr<ECCVMBuilder> eccvm_builder;
    std::unique_ptr<TranslatorBuilder> translator_builder;
    std::unique_ptr<TranslatorProver> translator_prover;
    std::unique_ptr<ECCVMComposer> eccvm_composer;
    std::unique_ptr<ECCVMProver> eccvm_prover;

    AccumulationOutput accumulator; // Used only for ACIR methods for now

  public:
    Goblin()
    { // Mocks the interaction of a first circuit with the op queue due to the inability to currently handle zero
      // commitments (https://github.com/AztecProtocol/barretenberg/issues/871) which would otherwise appear in the
      // first round of the merge protocol. To be removed once the issue has been resolved.
        GoblinMockCircuits::perform_op_queue_interactions_for_mock_first_circuit(op_queue);
    }
    /**
     * @brief Construct a GUH proof and a merge proof for the present circuit.
     * @details If there is a previous merge proof, recursively verify it.
     *
     * @param circuit_builder
     */
    AccumulationOutput accumulate(GoblinUltraCircuitBuilder& circuit_builder)
    {
        // Complete the circuit logic by recursively verifying previous merge proof if it exists
        if (merge_proof_exists) {
            RecursiveMergeVerifier merge_verifier{ &circuit_builder };
            [[maybe_unused]] auto pairing_points = merge_verifier.verify_proof(merge_proof);
        }

        // Construct a Honk proof for the main circuit
        auto instance = std::make_shared<GoblinUltraProverInstance>(circuit_builder);
        GoblinUltraProver prover(instance);
        auto ultra_proof = prover.construct_proof();
        auto verification_key = std::make_shared<VerificationKey>(instance->proving_key);

        // Construct and store the merge proof to be recursively verified on the next call to accumulate
        MergeProver merge_prover{ circuit_builder.op_queue };
        merge_proof = merge_prover.construct_proof();

        if (!merge_proof_exists) {
            merge_proof_exists = true;
        }

        return { ultra_proof, verification_key };
    };

    /**
     * @brief Add a recursive merge verifier to input circuit and construct a merge proof for the updated op queue
     * @details When this method is used, the "prover" functionality of the IVC scheme must be performed explicitly, but
     * this method has to be called first so that the recursive merge verifier can be "appended" to the circuit being
     * accumulated
     *
     * @param circuit_builder
     */
    void merge(GoblinUltraCircuitBuilder& circuit_builder)
    {
        BB_OP_COUNT_TIME_NAME("Goblin::merge");
        // Complete the circuit logic by recursively verifying previous merge proof if it exists
        if (merge_proof_exists) {
            RecursiveMergeVerifier merge_verifier{ &circuit_builder };
            [[maybe_unused]] auto pairing_points = merge_verifier.verify_proof(merge_proof);
        }

        // Construct and store the merge proof to be recursively verified on the next call to accumulate
        MergeProver merge_prover{ circuit_builder.op_queue };
        merge_proof = merge_prover.construct_proof();

        if (!merge_proof_exists) {
            merge_proof_exists = true;
        }
    };

    /**
     * @brief Construct an ECCVM proof and the translation polynomial evaluations
     *
     */
    void prove_eccvm()
    {
        eccvm_builder = std::make_unique<ECCVMBuilder>(op_queue);
        eccvm_composer = std::make_unique<ECCVMComposer>();
        eccvm_prover = std::make_unique<ECCVMProver>(eccvm_composer->create_prover(*eccvm_builder));
        goblin_proof.eccvm_proof = eccvm_prover->construct_proof();
        goblin_proof.translation_evaluations = eccvm_prover->translation_evaluations;
    };

    /**
     * @brief Construct a translator proof
     *
     */
    void prove_translator()
    {
        translator_builder = std::make_unique<TranslatorBuilder>(
            eccvm_prover->translation_batching_challenge_v, eccvm_prover->evaluation_challenge_x, op_queue);
        translator_prover = std::make_unique<GoblinTranslatorProver>(*translator_builder, eccvm_prover->transcript);
        goblin_proof.translator_proof = translator_prover->construct_proof();
    };

    /**
     * @brief Constuct a full Goblin proof (ECCVM, Translator, merge)
     * @details The merge proof is assumed to already have been constucted in the last accumulate step. It is simply
     * moved into the final proof here.
     *
     * @return Proof
     */
    Proof prove()
    {
        goblin_proof.merge_proof = std::move(merge_proof);
        prove_eccvm();
        prove_translator();
        return goblin_proof;
    };

    /**
     * @brief Verify a full Goblin proof (ECCVM, Translator, merge)
     *
     * @param proof
     * @return true
     * @return false
     */
    bool verify(const Proof& proof)
    {
        MergeVerifier merge_verifier;
        bool merge_verified = merge_verifier.verify_proof(proof.merge_proof);

        auto eccvm_verifier = eccvm_composer->create_verifier(*eccvm_builder);
        bool eccvm_verified = eccvm_verifier.verify_proof(proof.eccvm_proof);

        GoblinTranslatorVerifier translator_verifier(translator_prover->key, eccvm_verifier.transcript);

        bool accumulator_construction_verified = translator_verifier.verify_proof(proof.translator_proof);
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/799): Ensure translation_evaluations are passed
        // correctly
        bool translation_verified = translator_verifier.verify_translation(proof.translation_evaluations);

        return merge_verified && eccvm_verified && accumulator_construction_verified && translation_verified;
    };

    // The methods below this point are to be used only for ACIR. They exist while the interface is in flux. Eventually
    // there will be agreement and no acir-specific methods should be needed.

    /**
     * @brief Construct a GUH proof for the given circuit. (No merge proof for now)
     *
     * @param circuit_builder
     * @return std::vector<bb::fr>
     */
    std::vector<bb::fr> accumulate_for_acir(GoblinUltraCircuitBuilder& circuit_builder)
    {
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/811): no merge prover for now
        // // Complete the circuit logic by recursively verifying previous merge proof if it exists
        // if (merge_proof_exists) {
        //     RecursiveMergeVerifier merge_verifier{ &circuit_builder };
        //     [[maybe_unused]] auto pairing_points = merge_verifier.verify_proof(merge_proof);
        // }

        // Construct a Honk proof for the main circuit
        auto instance = std::make_shared<GoblinUltraProverInstance>(circuit_builder);
        GoblinUltraProver prover(instance);
        auto ultra_proof = prover.construct_proof();
        auto verification_key = std::make_shared<VerificationKey>(instance->proving_key);

        accumulator = { ultra_proof, verification_key };

        // TODO(https://github.com/AztecProtocol/barretenberg/issues/811): no merge prover for now since we're not
        // mocking the first set of ecc ops
        // // Construct and store the merge proof to be recursively verified on the next call to accumulate
        // MergeProver merge_prover{ op_queue };
        // merge_proof = merge_prover.construct_proof();

        // if (!merge_proof_exists) {
        //     merge_proof_exists = true;
        // }

        return ultra_proof;
    };

    /**
     * @brief Verify a GUH proof
     *
     * @param proof_buf
     * @return true
     * @return false
     */
    bool verify_accumulator_for_acir(const std::vector<bb::fr>& proof_buf) const
    {
        GoblinUltraVerifier verifier{ accumulator.verification_key };
        HonkProof proof{ proof_buf };
        bool verified = verifier.verify_proof(proof);

        return verified;
    }

    /**
     * @brief Construct a Goblin proof
     *
     * @return Proof
     */
    Proof prove_for_acir() { return prove(); };

    /**
     * @brief Verify a Goblin proof (excluding the merge proof for now)
     *
     * @return true
     * @return false
     */
    bool verify_for_acir() const
    {
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/811): No merge proof for now
        // MergeVerifier merge_verifier;
        // bool merge_verified = merge_verifier.verify_proof(goblin_proof.merge_proof);

        auto eccvm_verifier = eccvm_composer->create_verifier(*eccvm_builder);
        bool eccvm_verified = eccvm_verifier.verify_proof(goblin_proof.eccvm_proof);

        GoblinTranslatorVerifier translator_verifier(translator_prover->key, eccvm_verifier.transcript);

        bool translation_accumulator_construction_verified =
            translator_verifier.verify_proof(goblin_proof.translator_proof);
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/799): Ensure translation_evaluations are passed
        // correctly
        bool translation_verified = translator_verifier.verify_translation(goblin_proof.translation_evaluations);

        return /* merge_verified && */ eccvm_verified && translation_accumulator_construction_verified &&
               translation_verified;
    };
};
} // namespace bb