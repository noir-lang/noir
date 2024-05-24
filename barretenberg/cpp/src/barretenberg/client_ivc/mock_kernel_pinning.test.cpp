#include "barretenberg/client_ivc/client_ivc.hpp"
#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_circuit_builder.hpp"

#include <gtest/gtest.h>

using namespace bb;

/**
 * @brief For benchmarking, we want to be sure that our mocking functions create circuits of a known size. We control
 * this, to the degree that matters for proof construction time, using these "pinning tests" that fix values.
 *
 */
class MockKernelTest : public ::testing::Test {
  public:
    using Builder = MegaCircuitBuilder;

  protected:
    static void SetUpTestSuite() { srs::init_crs_factory("../srs_db/ignition"); }
};

TEST_F(MockKernelTest, PinFoldingKernelSizes)
{
    ClientIVC ivc;

    // Construct two function circuits and a kernel circuit
    Builder circuit_1{ ivc.goblin.op_queue };
    Builder circuit_2{ ivc.goblin.op_queue };
    Builder kernel_circuit{ ivc.goblin.op_queue };

    GoblinMockCircuits::construct_mock_function_circuit(circuit_1);
    GoblinMockCircuits::construct_mock_function_circuit(circuit_2);
    GoblinMockCircuits::construct_mock_folding_kernel(kernel_circuit);

    // Accumulate all three; The kernel will contain a single recursive folding verifier
    ivc.accumulate(circuit_1);
    ivc.accumulate(circuit_2);
    ivc.accumulate(kernel_circuit);

    EXPECT_EQ(ivc.prover_instance->proving_key.log_circuit_size, 17);
}