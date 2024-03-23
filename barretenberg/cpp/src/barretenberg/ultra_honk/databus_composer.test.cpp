#include <cstddef>
#include <cstdint>
#include <gtest/gtest.h>

#include "barretenberg/common/log.hpp"
#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/proof_system/circuit_builder/goblin_ultra_circuit_builder.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"
#include "barretenberg/proof_system/instance_inspector.hpp"

#include "barretenberg/ultra_honk/ultra_prover.hpp"
#include "barretenberg/ultra_honk/ultra_verifier.hpp"

using namespace bb;

namespace {
auto& engine = numeric::get_debug_randomness();
}

class DataBusComposerTests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }

    using Curve = curve::BN254;
    using FF = Curve::ScalarField;
};

/**
 * @brief Test proof construction/verification for a circuit with calldata lookup gates
 * gates
 * @note We simulate op queue interactions with a previous circuit so the actual circuit under test utilizes an op queue
 * with non-empty 'previous' data. This avoid complications with zero-commitments etc.
 *
 */
TEST_F(DataBusComposerTests, CallDataRead)
{
    auto op_queue = std::make_shared<bb::ECCOpQueue>();

    auto builder = GoblinUltraCircuitBuilder{ op_queue };

    // Add some ecc op gates and arithmetic gates
    GoblinMockCircuits::construct_simple_circuit(builder);

    // Add some values to calldata
    std::vector<FF> calldata_values = { 7, 10, 3, 12, 1 };
    for (auto& val : calldata_values) {
        builder.add_public_calldata(val);
    }

    // Define some raw indices at which to read calldata (these will be ASSERTed to be valid)
    std::vector<uint32_t> read_indices = { 1, 4 };

    // Create some calldata read gates. (Normally we'd use the result of the read. Example of that is below)
    for (uint32_t& read_idx : read_indices) {
        // Create a variable corresponding to the index at which we want to read into calldata
        uint32_t read_idx_witness_idx = builder.add_variable(read_idx);

        builder.read_calldata(read_idx_witness_idx);
    }

    // In general we'll want to use the result of a calldata read in another operation. Here's an example using
    // an add gate to show that the result of the read is as expected:
    uint32_t read_idx = 2;
    FF expected_result = calldata_values[read_idx];
    uint32_t read_idx_witness_idx = builder.add_variable(read_idx);
    uint32_t result_witness_idx = builder.read_calldata(read_idx_witness_idx);
    builder.create_add_gate({ result_witness_idx, builder.zero_idx, builder.zero_idx, 1, 0, 0, -expected_result });

    // Construct and verify Honk proof
    auto instance = std::make_shared<ProverInstance_<GoblinUltraFlavor>>(builder);
    // For debugging, use "instance_inspector::print_databus_info(instance)"
    GoblinUltraProver prover(instance);
    auto verification_key = std::make_shared<GoblinUltraFlavor::VerificationKey>(instance->proving_key);
    GoblinUltraVerifier verifier(verification_key);
    auto proof = prover.construct_proof();
    bool verified = verifier.verify_proof(proof);
    EXPECT_TRUE(verified);
}
