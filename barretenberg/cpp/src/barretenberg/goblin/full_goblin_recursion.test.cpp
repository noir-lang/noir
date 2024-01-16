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
        bb::srs::init_crs_factory("../srs_db/ignition");
        bb::srs::init_grumpkin_crs_factory("../srs_db/grumpkin");
    }

    using Curve = curve::BN254;
    using FF = Curve::ScalarField;
    using GoblinUltraBuilder = proof_system::GoblinUltraCircuitBuilder;
    using KernelInput = Goblin::AccumulationOutput;
};

/** * @brief A full Goblin test that mimicks the basic aztec client architecture
 *
 */
TEST_F(GoblinRecursionTests, Pseudo)
{
    Goblin goblin;

    // Construct an initial circuit; its proof will be recursively verified by the first kernel
    GoblinUltraBuilder initial_circuit{ goblin.op_queue };
    GoblinMockCircuits::construct_simple_initial_circuit(initial_circuit);
    KernelInput kernel_input = goblin.accumulate(initial_circuit);

    // Construct a series of simple Goblin circuits; generate and verify their proofs
    size_t NUM_CIRCUITS = 2;
    for (size_t circuit_idx = 0; circuit_idx < NUM_CIRCUITS; ++circuit_idx) {
        // Construct a circuit with logic resembling that of the "kernel circuit"
        GoblinUltraBuilder circuit_builder{ goblin.op_queue };
        GoblinMockCircuits::construct_mock_kernel_circuit(circuit_builder, kernel_input);

        // Construct proof of the current kernel circuit to be recursively verified by the next one
        kernel_input = goblin.accumulate(circuit_builder);
    }

    Goblin::Proof proof = goblin.prove();
    // Verify the final ultra proof
    GoblinUltraVerifier ultra_verifier{ kernel_input.verification_key };
    bool ultra_verified = ultra_verifier.verify_proof(kernel_input.proof);
    // Verify the goblin proof (eccvm, translator, merge)
    bool verified = goblin.verify(proof);
    EXPECT_TRUE(ultra_verified && verified);
}

// TODO(https://github.com/AztecProtocol/barretenberg/issues/787) Expand these tests.
} // namespace goblin_recursion_tests
