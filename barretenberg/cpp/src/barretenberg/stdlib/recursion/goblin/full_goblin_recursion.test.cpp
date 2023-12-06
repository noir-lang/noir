#include "barretenberg/eccvm/eccvm_composer.hpp"
#include "barretenberg/goblin/goblin.hpp"
#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/goblin/translation_evaluations.hpp"
#include "barretenberg/proof_system/circuit_builder/eccvm/eccvm_circuit_builder.hpp"
#include "barretenberg/proof_system/circuit_builder/goblin_ultra_circuit_builder.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"
#include "barretenberg/stdlib/recursion/honk/verifier/ultra_recursive_verifier.hpp"
#include "barretenberg/translator_vm/goblin_translator_composer.hpp"
#include "barretenberg/ultra_honk/ultra_composer.hpp"

#include <gtest/gtest.h>

using namespace proof_system::honk;
namespace goblin_recursion_tests {

class GoblinRecursionTests : public ::testing::Test {
  protected:
    static void SetUpTestSuite()
    {
        barretenberg::srs::init_crs_factory("../srs_db/ignition");
        barretenberg::srs::init_grumpkin_crs_factory("../srs_db/grumpkin");
    }

    using Curve = curve::BN254;
    using FF = Curve::ScalarField;
    using Fbase = Curve::BaseField;
    using Point = Curve::AffineElement;
    using CommitmentKey = pcs::CommitmentKey<Curve>;
    using OpQueue = proof_system::ECCOpQueue;
    using GoblinUltraBuilder = proof_system::GoblinUltraCircuitBuilder;
    using ECCVMFlavor = flavor::ECCVM;
    using ECCVMBuilder = proof_system::ECCVMCircuitBuilder<ECCVMFlavor>;
    using ECCVMComposer = ECCVMComposer_<ECCVMFlavor>;
    using TranslatorFlavor = flavor::GoblinTranslator;
    using TranslatorBuilder = proof_system::GoblinTranslatorCircuitBuilder;
    using TranslatorComposer = GoblinTranslatorComposer;
    using TranslatorConsistencyData = barretenberg::TranslationEvaluations;
    using Proof = proof_system::plonk::proof;
    using NativeVerificationKey = flavor::GoblinUltra::VerificationKey;
    using RecursiveFlavor = flavor::GoblinUltraRecursive_<GoblinUltraBuilder>;
    using RecursiveVerifier = proof_system::plonk::stdlib::recursion::honk::UltraRecursiveVerifier_<RecursiveFlavor>;
    using KernelInput = Goblin::AccumulationOutput;

    /**
     * @brief Construct a mock kernel circuit
     * @details This circuit contains (1) some basic/arbitrary arithmetic gates, (2) a genuine recursive verification of
     * the proof provided as input. It does not contain any other real kernel logic.
     *
     * @param builder
     * @param kernel_input A proof to be recursively verified and the corresponding native verification key
     */
    static void construct_mock_kernel_circuit(GoblinUltraBuilder& builder, KernelInput& kernel_input)
    {
        // Generic operations e.g. state updates (just arith gates for now)
        GoblinTestingUtils::construct_arithmetic_circuit(builder);

        // Execute recursive aggregation of previous kernel proof
        RecursiveVerifier verifier{ &builder, kernel_input.verification_key };
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/801): Aggregation
        auto pairing_points = verifier.verify_proof(kernel_input.proof); // app function proof
        pairing_points = verifier.verify_proof(kernel_input.proof);      // previous kernel proof
    }
};

/**
 * @brief A full Goblin test that mimicks the basic aztec client architecture
 *
 */
TEST_F(GoblinRecursionTests, Pseudo)
{
    barretenberg::Goblin goblin;

    // Construct an initial circuit; its proof will be recursively verified by the first kernel
    GoblinUltraBuilder initial_circuit{ goblin.op_queue };
    GoblinTestingUtils::construct_simple_initial_circuit(initial_circuit);
    KernelInput kernel_input = goblin.accumulate(initial_circuit);

    // Construct a series of simple Goblin circuits; generate and verify their proofs
    size_t NUM_CIRCUITS = 2;
    for (size_t circuit_idx = 0; circuit_idx < NUM_CIRCUITS; ++circuit_idx) {
        // Construct a circuit with logic resembling that of the "kernel circuit"
        GoblinUltraBuilder circuit_builder{ goblin.op_queue };
        construct_mock_kernel_circuit(circuit_builder, kernel_input);

        // Construct proof of the current kernel circuit to be recursively verified by the next one
        kernel_input = goblin.accumulate(circuit_builder);
    }

    Goblin::Proof proof = goblin.prove();
    bool verified = goblin.verify(proof);
    EXPECT_TRUE(verified);
}

// TODO(https://github.com/AztecProtocol/barretenberg/issues/787) Expand these tests.
} // namespace goblin_recursion_tests
