#include <cstddef>
#include <cstdint>
#include <gtest/gtest.h>

#include "barretenberg/common/log.hpp"
#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/proof_system/circuit_builder/goblin_ultra_circuit_builder.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"
#include "barretenberg/proof_system/instance_inspector.hpp"
#include "barretenberg/ultra_honk/ultra_composer.hpp"
#include "barretenberg/ultra_honk/ultra_prover.hpp"
using namespace bb;
using namespace bb::honk;

namespace {
auto& engine = numeric::get_debug_randomness();
}

class DataBusComposerTests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }

    using Curve = curve::BN254;
    using FF = Curve::ScalarField;
    using Point = Curve::AffineElement;
    using CommitmentKey = pcs::CommitmentKey<Curve>;

    /**
     * @brief Generate a simple test circuit that includes arithmetic and goblin ecc op gates
     *
     * @param builder
     */
    void generate_test_circuit(auto& builder)
    {
        // Add some ecc op gates and arithmetic gates
        GoblinMockCircuits::construct_goblin_ecc_op_circuit(builder);
        GoblinMockCircuits::construct_arithmetic_circuit(builder);
    }
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

    // Add mock data to op queue to simulate interaction with a previous circuit
    op_queue->populate_with_mock_initital_data();

    auto builder = GoblinUltraCircuitBuilder{ op_queue };

    // Create a general test circuit
    generate_test_circuit(builder);

    // Add some values to calldata and store the corresponding witness index
    std::array<FF, 5> calldata_values = { 7, 10, 3, 12, 1 };
    for (auto& val : calldata_values) {
        builder.add_public_calldata(val);
    }

    // Define some indices at which to read calldata
    std::array<uint32_t, 2> read_index_values = { 1, 4 };

    // Create some calldata lookup gates
    for (uint32_t& read_index : read_index_values) {
        // Create a variable corresponding to the index at which we want to read into calldata
        uint32_t read_idx_witness_idx = builder.add_variable(read_index);

        // Create a variable corresponding to the result of the read. Note that we do not in general connect reads from
        // calldata via copy constraints (i.e. we create a unique variable for the result of each read)
        ASSERT_LT(read_index, builder.public_calldata.size());
        FF calldata_value = builder.get_variable(builder.public_calldata[read_index]);
        uint32_t value_witness_idx = builder.add_variable(calldata_value);

        builder.create_calldata_lookup_gate({ read_idx_witness_idx, value_witness_idx });
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/821): automate updating of read counts
        builder.calldata_read_counts[read_index]++;
    }

    auto composer = GoblinUltraComposer();

    // Construct and verify Honk proof
    auto instance = composer.create_instance(builder);
    // For debugging, use "instance_inspector::print_databus_info(instance)"
    auto prover = composer.create_prover(instance);
    auto verifier = composer.create_verifier(instance);
    auto proof = prover.construct_proof();
    bool verified = verifier.verify_proof(proof);
    EXPECT_TRUE(verified);
}
