#include "barretenberg/client_ivc/client_ivc.hpp"
#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/stdlib_circuit_builders/goblin_ultra_circuit_builder.hpp"

#include <gtest/gtest.h>

using namespace bb;

/**
 * @brief For benchmarking, we want to be sure that our mocking functions create circuits of a known size. We control
 * this, to the degree that matters for proof construction time, using these "pinning tests" that fix values.
 *
 */
class MockKernelTest : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { srs::init_crs_factory("../srs_db/ignition"); }
};

TEST_F(MockKernelTest, PinFoldingKernelSizes)
{
    ClientIVC ivc;
    ivc.precompute_folding_verification_keys();
    // Accumulate three circuits to generate two folding proofs for input to folding kernel
    GoblinUltraCircuitBuilder circuit_1{ ivc.goblin.op_queue };
    GoblinMockCircuits::construct_mock_function_circuit(circuit_1);
    ivc.initialize(circuit_1);
    auto kernel_acc = std::make_shared<ClientIVC::VerifierInstance>(ivc.vks.first_func_vk);
    kernel_acc->verification_key = ivc.vks.first_func_vk;

    GoblinUltraCircuitBuilder circuit_2{ ivc.goblin.op_queue };
    GoblinMockCircuits::construct_mock_function_circuit(circuit_2);
    auto func_fold_proof = ivc.accumulate(circuit_2);

    // Construct kernel circuit
    GoblinUltraCircuitBuilder kernel_circuit{ ivc.goblin.op_queue };
    kernel_acc = GoblinMockCircuits::construct_mock_folding_kernel(
        kernel_circuit, { func_fold_proof, ivc.vks.func_vk }, {}, kernel_acc);

    auto kernel_fold_proof = ivc.accumulate(kernel_circuit);
    EXPECT_EQ(ivc.prover_instance->proving_key.log_circuit_size, 17);

    GoblinUltraCircuitBuilder circuit_3{ ivc.goblin.op_queue };
    GoblinMockCircuits::construct_mock_function_circuit(circuit_3);
    func_fold_proof = ivc.accumulate(circuit_3);

    kernel_circuit = GoblinUltraCircuitBuilder{ ivc.goblin.op_queue };
    kernel_acc = GoblinMockCircuits::construct_mock_folding_kernel(kernel_circuit,
                                                                   { kernel_fold_proof, ivc.vks.first_kernel_vk },
                                                                   { func_fold_proof, ivc.vks.func_vk },
                                                                   kernel_acc);
    auto instance = std::make_shared<ClientIVC::ProverInstance>(kernel_circuit);
    EXPECT_EQ(instance->proving_key.log_circuit_size, 17);
}