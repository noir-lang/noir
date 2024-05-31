#pragma once

#include "barretenberg/eccvm/eccvm_circuit_builder.hpp"
#include "barretenberg/eccvm/eccvm_prover.hpp"
#include "barretenberg/eccvm/eccvm_trace_checker.hpp"
#include "barretenberg/eccvm/eccvm_verifier.hpp"
#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/goblin/types.hpp"
#include "barretenberg/plonk_honk_shared/instance_inspector.hpp"
#include "barretenberg/stdlib/honk_recursion/verifier/merge_recursive_verifier.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_circuit_builder.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_flavor.hpp"
#include "barretenberg/translator_vm/translator_circuit_builder.hpp"
#include "barretenberg/translator_vm/translator_prover.hpp"
#include "barretenberg/translator_vm/translator_verifier.hpp"
#include "barretenberg/ultra_honk/merge_prover.hpp"
#include "barretenberg/ultra_honk/merge_verifier.hpp"
#include "barretenberg/ultra_honk/ultra_prover.hpp"
#include "barretenberg/ultra_honk/ultra_verifier.hpp"

namespace bb {

class GoblinProver {
    using MegaCircuitBuilder = bb::MegaCircuitBuilder;
    using Commitment = MegaFlavor::Commitment;
    using FF = MegaFlavor::FF;

  public:
    using Builder = MegaCircuitBuilder;
    using Fr = bb::fr;
    using Transcript = NativeTranscript;
    using MegaProverInstance = ProverInstance_<MegaFlavor>;
    using OpQueue = bb::ECCOpQueue;
    using ECCVMFlavor = bb::ECCVMFlavor;
    using ECCVMBuilder = bb::ECCVMCircuitBuilder;
    using ECCVMProver = bb::ECCVMProver;
    using ECCVMProvingKey = ECCVMFlavor::ProvingKey;
    using TranslationEvaluations = ECCVMProver::TranslationEvaluations;
    using TranslatorBuilder = bb::TranslatorCircuitBuilder;
    using TranslatorProver = bb::TranslatorProver;
    using TranslatorProvingKey = bb::TranslatorFlavor::ProvingKey;
    using RecursiveMergeVerifier = bb::stdlib::recursion::goblin::MergeRecursiveVerifier_<MegaCircuitBuilder>;
    using MergeProver = bb::MergeProver_<MegaFlavor>;
    using VerificationKey = MegaFlavor::VerificationKey;
    /**
     * @brief Output of goblin::accumulate; an Ultra proof and the corresponding verification key
     *
     */

    std::shared_ptr<OpQueue> op_queue = std::make_shared<OpQueue>();

    HonkProof merge_proof;
    GoblinProof goblin_proof;

    // on the first call to accumulate there is no merge proof to verify
    bool merge_proof_exists{ false };

    std::shared_ptr<ECCVMProvingKey> get_eccvm_proving_key() const { return eccvm_prover->key; }
    std::shared_ptr<TranslatorProvingKey> get_translator_proving_key() const { return translator_prover->key; }

  private:
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/798) unique_ptr use is a hack
    std::unique_ptr<ECCVMBuilder> eccvm_builder;
    std::unique_ptr<TranslatorBuilder> translator_builder;
    std::unique_ptr<TranslatorProver> translator_prover;
    std::unique_ptr<ECCVMProver> eccvm_prover;

    GoblinAccumulationOutput accumulator; // Used only for ACIR methods for now

  public:
    GoblinProver()
    { // Mocks the interaction of a first circuit with the op queue due to the inability to currently handle zero
      // commitments (https://github.com/AztecProtocol/barretenberg/issues/871) which would otherwise appear in the
      // first round of the merge protocol. To be removed once the issue has been resolved.
        GoblinMockCircuits::perform_op_queue_interactions_for_mock_first_circuit(op_queue);
    }
    /**
     * @brief Construct a MegaHonk proof and a merge proof for the present circuit.
     * @details If there is a previous merge proof, recursively verify it.
     *
     * @param circuit_builder
     */
    GoblinAccumulationOutput accumulate(MegaCircuitBuilder& circuit_builder)
    {
        // Complete the circuit logic by recursively verifying previous merge proof if it exists
        if (merge_proof_exists) {
            RecursiveMergeVerifier merge_verifier{ &circuit_builder };
            [[maybe_unused]] auto pairing_points = merge_verifier.verify_proof(merge_proof);
        }

        // Construct a Honk proof for the main circuit
        auto instance = std::make_shared<MegaProverInstance>(circuit_builder);
        MegaProver prover(instance);
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
    void merge(MegaCircuitBuilder& circuit_builder)
    {
        BB_OP_COUNT_TIME_NAME("Goblin::merge");
        // Complete the circuit logic by recursively verifying previous merge proof if it exists
        if (merge_proof_exists) {
            RecursiveMergeVerifier merge_verifier{ &circuit_builder };
            [[maybe_unused]] auto pairing_points = merge_verifier.verify_proof(merge_proof);
        }

        // TODO(https://github.com/AztecProtocol/barretenberg/issues/993): Some circuits (particularly on the first call
        // to accumulate) may not have any goblin ecc ops prior to the call to merge(), so the commitment to the new
        // contribution (C_t_shift) in the merge prover will be the point at infinity. (Note: Some dummy ops are added
        // in 'add_gates_to_ensure...' but not until instance construction which comes later). See issue for ideas about
        // how to resolve.
        if (circuit_builder.blocks.ecc_op.size() == 0) {
            MockCircuits::construct_goblin_ecc_op_circuit(circuit_builder); // Add some arbitrary goblin ECC ops
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
        eccvm_prover = std::make_unique<ECCVMProver>(*eccvm_builder);
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
        translator_prover = std::make_unique<TranslatorProver>(*translator_builder, eccvm_prover->transcript);
        goblin_proof.translator_proof = translator_prover->construct_proof();
    };

    /**
     * @brief Constuct a full Goblin proof (ECCVM, Translator, merge)
     * @details The merge proof is assumed to already have been constucted in the last accumulate step. It is simply
     * moved into the final proof here.
     *
     * @return Proof
     */
    GoblinProof prove()
    {
        goblin_proof.merge_proof = std::move(merge_proof);
        prove_eccvm();
        prove_translator();
        return goblin_proof;
    };
};

class GoblinVerifier {
  public:
    using ECCVMVerificationKey = ECCVMFlavor::VerificationKey;
    using TranslatorVerificationKey = bb::TranslatorFlavor::VerificationKey;
    using MergeVerifier = bb::MergeVerifier_<MegaFlavor>;

    struct VerifierInput {
        std::shared_ptr<ECCVMVerificationKey> eccvm_verification_key;
        std::shared_ptr<TranslatorVerificationKey> translator_verification_key;
    };

  private:
    std::shared_ptr<ECCVMVerificationKey> eccvm_verification_key;
    std::shared_ptr<TranslatorVerificationKey> translator_verification_key;

  public:
    GoblinVerifier(std::shared_ptr<ECCVMVerificationKey> eccvm_verification_key,
                   std::shared_ptr<TranslatorVerificationKey> translator_verification_key)
        : eccvm_verification_key(eccvm_verification_key)
        , translator_verification_key(translator_verification_key)
    {}

    GoblinVerifier(VerifierInput input)
        : eccvm_verification_key(input.eccvm_verification_key)
        , translator_verification_key(input.translator_verification_key)
    {}

    /**
     * @brief Verify a full Goblin proof (ECCVM, Translator, merge)
     *
     * @param proof
     * @return true
     * @return false
     */
    bool verify(const GoblinProof& proof)
    {
        MergeVerifier merge_verifier;
        bool merge_verified = merge_verifier.verify_proof(proof.merge_proof);

        ECCVMVerifier eccvm_verifier(eccvm_verification_key);
        bool eccvm_verified = eccvm_verifier.verify_proof(proof.eccvm_proof);

        TranslatorVerifier translator_verifier(translator_verification_key, eccvm_verifier.transcript);

        bool accumulator_construction_verified = translator_verifier.verify_proof(proof.translator_proof);
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/799): Ensure translation_evaluations are passed
        // correctly
        bool translation_verified = translator_verifier.verify_translation(proof.translation_evaluations);

        return merge_verified && eccvm_verified && accumulator_construction_verified && translation_verified;
    };
};
} // namespace bb