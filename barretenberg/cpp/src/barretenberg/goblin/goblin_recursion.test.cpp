#include "barretenberg/goblin/goblin.hpp"
#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/stdlib_circuit_builders/goblin_ultra_circuit_builder.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_circuit_builder.hpp"

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
    using KernelInput = Goblin::AccumulationOutput;
    using ProverInstance = ProverInstance_<GoblinUltraFlavor>;
    using VerifierInstance = VerifierInstance_<GoblinUltraFlavor>;

    static Goblin::AccumulationOutput construct_accumulator(GoblinUltraCircuitBuilder& builder)
    {
        auto prover_instance = std::make_shared<ProverInstance>(builder);
        auto verification_key = std::make_shared<GoblinUltraFlavor::VerificationKey>(prover_instance->proving_key);
        auto verifier_instance = std::make_shared<VerifierInstance>(verification_key);
        GoblinUltraProver prover(prover_instance);
        auto ultra_proof = prover.construct_proof();
        return { ultra_proof, verifier_instance->verification_key };
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

    size_t NUM_CIRCUITS = 2;
    for (size_t circuit_idx = 0; circuit_idx < NUM_CIRCUITS; ++circuit_idx) {

        // Construct and accumulate a mock function circuit containing both arbitrary arithmetic gates and goblin
        // ecc op gates to make it a meaningful test
        GoblinUltraCircuitBuilder function_circuit{ goblin.op_queue };
        MockCircuits::construct_arithmetic_circuit(function_circuit, /*target_log2_dyadic_size=*/8);
        MockCircuits::construct_goblin_ecc_op_circuit(function_circuit);
        goblin.merge(function_circuit);
        auto function_accum = construct_accumulator(function_circuit);

        // Construct and accumulate the mock kernel circuit (no kernel accum in first round)
        GoblinUltraCircuitBuilder kernel_circuit{ goblin.op_queue };
        GoblinMockCircuits::construct_mock_kernel_small(kernel_circuit,
                                                        { function_accum.proof, function_accum.verification_key },
                                                        { kernel_accum.proof, kernel_accum.verification_key });
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
