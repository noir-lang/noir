#pragma once

#include "barretenberg/eccvm/eccvm_composer.hpp"
#include "barretenberg/proof_system/circuit_builder/eccvm/eccvm_circuit_builder.hpp"
#include "barretenberg/proof_system/circuit_builder/goblin_translator_circuit_builder.hpp"
#include "barretenberg/proof_system/circuit_builder/goblin_ultra_circuit_builder.hpp"
#include "barretenberg/proof_system/instance_inspector.hpp"
#include "barretenberg/stdlib/recursion/honk/verifier/merge_recursive_verifier.hpp"
#include "barretenberg/translator_vm/goblin_translator_composer.hpp"
#include "barretenberg/ultra_honk/ultra_composer.hpp"

namespace bb {

class Goblin {
    using HonkProof = bb::plonk::proof;

    using GUHFlavor = bb::honk::flavor::GoblinUltra;
    using GoblinUltraCircuitBuilder = bb::GoblinUltraCircuitBuilder;

    using GUHVerificationKey = GUHFlavor::VerificationKey;
    using Commitment = GUHFlavor::Commitment;
    using FF = GUHFlavor::FF;

  public:
    /**
     * @brief Output of goblin::accumulate; an Ultra proof and the corresponding verification key
     *
     */
    struct AccumulationOutput {
        HonkProof proof;
        std::shared_ptr<GUHVerificationKey> verification_key;
    };

    struct Proof {
        HonkProof merge_proof;
        HonkProof eccvm_proof;
        HonkProof translator_proof;
        TranslationEvaluations translation_evaluations;
        std::vector<uint8_t> to_buffer()
        {
            // ACIRHACK: so much copying and duplication added here and elsewhere
            std::vector<uint8_t> translation_evaluations_buf = translation_evaluations.to_buffer();
            size_t proof_size = merge_proof.proof_data.size() + eccvm_proof.proof_data.size() +
                                translator_proof.proof_data.size() + translation_evaluations_buf.size();

            std::vector<uint8_t> result(proof_size);
            const auto insert = [&result](const std::vector<uint8_t>& buf) {
                result.insert(result.end(), buf.begin(), buf.end());
            };
            insert(merge_proof.proof_data);
            insert(eccvm_proof.proof_data);
            insert(translator_proof.proof_data);
            insert(translation_evaluations_buf);
            return result;
        }
    };

    using GoblinUltraComposer = bb::honk::UltraComposer_<GUHFlavor>;
    using GoblinUltraVerifier = bb::honk::UltraVerifier_<GUHFlavor>;
    using Builder = GoblinUltraCircuitBuilder;
    using OpQueue = bb::ECCOpQueue;
    using ECCVMFlavor = bb::honk::flavor::ECCVM;
    using ECCVMBuilder = bb::ECCVMCircuitBuilder<ECCVMFlavor>;
    using ECCVMComposer = bb::honk::ECCVMComposer;
    using ECCVMProver = bb::honk::ECCVMProver_<ECCVMFlavor>;
    using TranslatorBuilder = bb::GoblinTranslatorCircuitBuilder;
    using TranslatorComposer = bb::honk::GoblinTranslatorComposer;
    using RecursiveMergeVerifier =
        bb::plonk::stdlib::recursion::goblin::MergeRecursiveVerifier_<GoblinUltraCircuitBuilder>;
    using MergeVerifier = bb::honk::MergeVerifier_<GUHFlavor>;

    std::shared_ptr<OpQueue> op_queue = std::make_shared<OpQueue>();

    HonkProof merge_proof;
    Proof goblin_proof;

    // on the first call to accumulate there is no merge proof to verify
    bool merge_proof_exists{ false };

  private:
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/798) unique_ptr use is a hack
    std::unique_ptr<ECCVMBuilder> eccvm_builder;
    std::unique_ptr<TranslatorBuilder> translator_builder;
    std::unique_ptr<ECCVMComposer> eccvm_composer;
    std::unique_ptr<ECCVMProver> eccvm_prover;
    std::unique_ptr<TranslatorComposer> translator_composer;

    AccumulationOutput accumulator; // ACIRHACK
    Proof proof_;                   // ACIRHACK

  public:
    /**
     * @brief If there is a previous merge proof, recursively verify it. Generate next accmulated proof and merge proof.
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
        GoblinUltraComposer composer;
        auto instance = composer.create_instance(circuit_builder);
        auto prover = composer.create_prover(instance);
        auto ultra_proof = prover.construct_proof();

        // Construct and store the merge proof to be recursively verified on the next call to accumulate
        auto merge_prover = composer.create_merge_prover(op_queue);
        merge_proof = merge_prover.construct_proof();

        if (!merge_proof_exists) {
            merge_proof_exists = true;
        }

        return { ultra_proof, instance->verification_key };
    };

    void prove_eccvm()
    {
        goblin_proof.merge_proof = std::move(merge_proof);

        eccvm_builder = std::make_unique<ECCVMBuilder>(op_queue);
        eccvm_composer = std::make_unique<ECCVMComposer>();
        eccvm_prover = std::make_unique<ECCVMProver>(eccvm_composer->create_prover(*eccvm_builder));
        goblin_proof.eccvm_proof = eccvm_prover->construct_proof();
        goblin_proof.translation_evaluations = eccvm_prover->translation_evaluations;
    };

    void prove_translator()
    {
        translator_builder = std::make_unique<TranslatorBuilder>(
            eccvm_prover->translation_batching_challenge_v, eccvm_prover->evaluation_challenge_x, op_queue);
        translator_composer = std::make_unique<TranslatorComposer>();
        auto translator_prover = translator_composer->create_prover(*translator_builder, eccvm_prover->transcript);
        goblin_proof.translator_proof = translator_prover.construct_proof();
    };

    Proof prove()
    {
        prove_eccvm();
        prove_translator();
        return goblin_proof;
    };

    bool verify(const Proof& proof)
    {
        MergeVerifier merge_verifier;
        bool merge_verified = merge_verifier.verify_proof(proof.merge_proof);

        auto eccvm_verifier = eccvm_composer->create_verifier(*eccvm_builder);
        bool eccvm_verified = eccvm_verifier.verify_proof(proof.eccvm_proof);

        auto translator_verifier = translator_composer->create_verifier(*translator_builder, eccvm_verifier.transcript);
        bool accumulator_construction_verified = translator_verifier.verify_proof(proof.translator_proof);
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/799): Ensure translation_evaluations are passed
        // correctly
        bool translation_verified = translator_verifier.verify_translation(proof.translation_evaluations);

        return merge_verified && eccvm_verified && accumulator_construction_verified && translation_verified;
    };

    // ACIRHACK
    AccumulationOutput accumulate_for_acir(GoblinUltraCircuitBuilder& circuit_builder)
    {
        // Complete the circuit logic by recursively verifying previous merge proof if it exists
        if (merge_proof_exists) {
            RecursiveMergeVerifier merge_verifier{ &circuit_builder };
            [[maybe_unused]] auto pairing_points = merge_verifier.verify_proof(merge_proof);
        }

        // Construct a Honk proof for the main circuit
        GoblinUltraComposer composer;
        auto instance = composer.create_instance(circuit_builder);
        auto prover = composer.create_prover(instance);
        auto ultra_proof = prover.construct_proof();

        // TODO(https://github.com/AztecProtocol/barretenberg/issues/811): no merge prover for now since we're not
        // mocking the first set of ecc ops
        // // Construct and store the merge proof to be recursively verified on the next call to accumulate
        // auto merge_prover = composer.create_merge_prover(op_queue);
        // merge_proof = merge_prover.construct_proof();

        // if (!merge_proof_exists) {
        //     merge_proof_exists = true;
        // }

        accumulator = { ultra_proof, instance->verification_key };
        return accumulator;
    };

    // ACIRHACK
    Proof prove_for_acir()
    {
        Proof proof;

        proof.merge_proof = std::move(merge_proof);

        eccvm_builder = std::make_unique<ECCVMBuilder>(op_queue);
        eccvm_composer = std::make_unique<ECCVMComposer>();
        auto eccvm_prover = eccvm_composer->create_prover(*eccvm_builder);
        proof.eccvm_proof = eccvm_prover.construct_proof();
        proof.translation_evaluations = eccvm_prover.translation_evaluations;

        translator_builder = std::make_unique<TranslatorBuilder>(
            eccvm_prover.translation_batching_challenge_v, eccvm_prover.evaluation_challenge_x, op_queue);
        translator_composer = std::make_unique<TranslatorComposer>();
        auto translator_prover = translator_composer->create_prover(*translator_builder, eccvm_prover.transcript);
        proof.translator_proof = translator_prover.construct_proof();

        proof_ = proof; // ACIRHACK
        return proof;
    };

    // ACIRHACK
    bool verify_for_acir(const Proof& proof) const
    {
        // ACIRHACK
        // MergeVerifier merge_verifier;
        // bool merge_verified = merge_verifier.verify_proof(proof.merge_proof);

        auto eccvm_verifier = eccvm_composer->create_verifier(*eccvm_builder);
        bool eccvm_verified = eccvm_verifier.verify_proof(proof.eccvm_proof);

        auto translator_verifier = translator_composer->create_verifier(*translator_builder, eccvm_verifier.transcript);
        bool accumulator_construction_verified = translator_verifier.verify_proof(proof.translator_proof);
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/799): Ensure translation_evaluations are passed
        // correctly
        bool translation_verified = translator_verifier.verify_translation(proof.translation_evaluations);

        return /* merge_verified && */ eccvm_verified && accumulator_construction_verified && translation_verified;
    };

    // ACIRHACK
    std::vector<uint8_t> construct_proof(GoblinUltraCircuitBuilder& builder)
    {
        // Construct a GUH proof
        accumulate_for_acir(builder);

        std::vector<uint8_t> result(accumulator.proof.proof_data.size());

        const auto insert = [&result](const std::vector<uint8_t>& buf) {
            result.insert(result.end(), buf.begin(), buf.end());
        };

        insert(accumulator.proof.proof_data);

        // TODO(https://github.com/AztecProtocol/barretenberg/issues/819): Skip ECCVM/Translator proof for now
        // std::vector<uint8_t> goblin_proof = prove_for_acir().to_buffer();
        // insert(goblin_proof);

        return result;
    }

    // ACIRHACK
    bool verify_proof([[maybe_unused]] const bb::plonk::proof& proof) const
    {
        // ACIRHACK: to do this properly, extract the proof correctly or maybe share transcripts.
        const auto extract_final_kernel_proof = [&]([[maybe_unused]] auto& input_proof) { return accumulator.proof; };

        GoblinUltraVerifier verifier{ accumulator.verification_key };
        bool verified = verifier.verify_proof(extract_final_kernel_proof(proof));

        // TODO(https://github.com/AztecProtocol/barretenberg/issues/819): Skip ECCVM/Translator verification for now
        // const auto extract_goblin_proof = [&]([[maybe_unused]] auto& input_proof) { return proof_; };
        // auto goblin_proof = extract_goblin_proof(proof);
        // verified = verified && verify_for_acir(goblin_proof);

        return verified;
    }
};
} // namespace bb