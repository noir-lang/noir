#include "barretenberg/goblin/goblin.hpp"
#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/proof_system/circuit_builder/goblin_ultra_circuit_builder.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"
#include "barretenberg/ultra_honk/ultra_composer.hpp"

#include <gtest/gtest.h>

using namespace bb;

class GoblinRecursionTests : public ::testing::Test {
  protected:
    static void SetUpTestSuite()
    {
        srs::init_crs_factory("../srs_db/ignition");
        srs::init_grumpkin_crs_factory("../srs_db/grumpkin");
    }

    using Curve = curve::BN254;
    using FF = Curve::ScalarField;
    using GoblinUltraBuilder = GoblinUltraCircuitBuilder;
    using KernelInput = Goblin::AccumulationOutput;

    static Goblin::AccumulationOutput construct_accumulator(GoblinUltraBuilder& builder)
    {
        GoblinUltraComposer composer;
        auto instance = composer.create_instance(builder);
        auto prover = composer.create_prover(instance);
        auto ultra_proof = prover.construct_proof();
        return { ultra_proof, instance->verification_key };
    }
};

/**
 * @brief A full Goblin test that mimicks the basic aztec client architecture
 * @details
 */
TEST_F(GoblinRecursionTests, Vanilla)
{
    Goblin goblin;

    Goblin::AccumulationOutput kernel_accum;

    // TODO(https://github.com/AztecProtocol/barretenberg/issues/723):
    GoblinMockCircuits::perform_op_queue_interactions_for_mock_first_circuit(goblin.op_queue);

    size_t NUM_CIRCUITS = 2;
    for (size_t circuit_idx = 0; circuit_idx < NUM_CIRCUITS; ++circuit_idx) {

        // Construct and accumulate a mock function circuit
        GoblinUltraCircuitBuilder function_circuit{ goblin.op_queue };
        GoblinMockCircuits::construct_arithmetic_circuit(function_circuit, 1 << 8);
        GoblinMockCircuits::construct_goblin_ecc_op_circuit(function_circuit);
        info("function merge");
        goblin.merge(function_circuit);
        auto function_accum = construct_accumulator(function_circuit);

        // Construct and accumulate the mock kernel circuit (no kernel accum in first round)
        GoblinUltraCircuitBuilder kernel_circuit{ goblin.op_queue };
        GoblinMockCircuits::construct_mock_kernel_small(kernel_circuit, function_accum, kernel_accum);
        info("kernel accum");
        goblin.merge(kernel_circuit);
        kernel_accum = construct_accumulator(kernel_circuit);
    }

    Goblin::Proof proof = goblin.prove();
    // Verify the final ultra proof
    GoblinUltraVerifier ultra_verifier{ kernel_accum.verification_key };
    bool ultra_verified = ultra_verifier.verify_proof(kernel_accum.proof);
    // Verify the goblin proof (eccvm, translator, merge)
    bool verified = goblin.verify(proof);
    EXPECT_TRUE(ultra_verified && verified);
}

// TODO(https://github.com/AztecProtocol/barretenberg/issues/787) Expand these tests.
