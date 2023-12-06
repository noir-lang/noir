#pragma once

#include "barretenberg/eccvm/eccvm_composer.hpp"
#include "barretenberg/proof_system/circuit_builder/eccvm/eccvm_circuit_builder.hpp"
#include "barretenberg/proof_system/circuit_builder/goblin_translator_circuit_builder.hpp"
#include "barretenberg/proof_system/circuit_builder/goblin_ultra_circuit_builder.hpp"
#include "barretenberg/translator_vm/goblin_translator_composer.hpp"
#include "barretenberg/ultra_honk/ultra_composer.hpp"

namespace barretenberg {

class Goblin {
    using HonkProof = proof_system::plonk::proof;

  public:
    /**
     * @brief Output of goblin::accumulate; an Ultra proof and the corresponding verification key
     *
     */
    struct AccumulationOutput {
        using NativeVerificationKey = proof_system::honk::flavor::GoblinUltra::VerificationKey;
        HonkProof proof;
        std::shared_ptr<NativeVerificationKey> verification_key;
    };

    struct Proof {
        HonkProof eccvm_proof;
        HonkProof translator_proof;
        TranslationEvaluations translation_evaluations;
    };

    using Fr = barretenberg::fr;
    using Fq = barretenberg::fq;

    using Transcript = proof_system::honk::BaseTranscript;
    using GoblinUltraComposer = proof_system::honk::GoblinUltraComposer;
    using GoblinUltraCircuitBuilder = proof_system::GoblinUltraCircuitBuilder;
    using OpQueue = proof_system::ECCOpQueue;
    using ECCVMFlavor = proof_system::honk::flavor::ECCVM;
    using ECCVMBuilder = proof_system::ECCVMCircuitBuilder<ECCVMFlavor>;
    using ECCVMComposer = proof_system::honk::ECCVMComposer;
    using TranslatorBuilder = proof_system::GoblinTranslatorCircuitBuilder;
    using TranslatorComposer = proof_system::honk::GoblinTranslatorComposer;

    std::shared_ptr<OpQueue> op_queue = std::make_shared<OpQueue>();

  private:
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/798) unique_ptr use is a hack
    std::unique_ptr<ECCVMBuilder> eccvm_builder;
    std::unique_ptr<TranslatorBuilder> translator_builder;
    std::unique_ptr<ECCVMComposer> eccvm_composer;
    std::unique_ptr<TranslatorComposer> translator_composer;

  public:
    /**
     * @brief
     *
     * @param circuit_builder
     */
    AccumulationOutput accumulate(GoblinUltraCircuitBuilder& circuit_builder)
    {
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/797) Complete the "kernel" logic by recursively
        // verifying previous merge proof

        GoblinUltraComposer composer;
        auto instance = composer.create_instance(circuit_builder);
        auto prover = composer.create_prover(instance);
        auto ultra_proof = prover.construct_proof();

        auto merge_prover = composer.create_merge_prover(op_queue);
        [[maybe_unused]] auto merge_proof = merge_prover.construct_proof();

        return { ultra_proof, instance->verification_key };
    };

    Proof prove()
    {
        Proof proof;
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
        return proof;
    };

    bool verify(const Proof& proof)
    {
        auto eccvm_verifier = eccvm_composer->create_verifier(*eccvm_builder);
        bool eccvm_verified = eccvm_verifier.verify_proof(proof.eccvm_proof);

        auto translator_verifier = translator_composer->create_verifier(*translator_builder, eccvm_verifier.transcript);
        bool accumulator_construction_verified = translator_verifier.verify_proof(proof.translator_proof);
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/799):
        //   Ensure translation_evaluations are passed correctly
        bool translation_verified = translator_verifier.verify_translation(proof.translation_evaluations);
        return eccvm_verified && accumulator_construction_verified && translation_verified;
    };
};
} // namespace barretenberg