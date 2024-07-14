#include <cstddef>
#include <cstdint>
#include <gtest/gtest.h>

#include "barretenberg/common/log.hpp"
#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/plonk_honk_shared/instance_inspector.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_circuit_builder.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_circuit_builder.hpp"

#include "barretenberg/ultra_honk/ultra_prover.hpp"
#include "barretenberg/ultra_honk/ultra_verifier.hpp"

using namespace bb;

namespace {
auto& engine = numeric::get_debug_randomness();
}

class DataBusTests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }

    using Curve = curve::BN254;
    using FF = Curve::ScalarField;

    // Construct and verify a MegaHonk proof for a given circuit
    static bool construct_and_verify_proof(MegaCircuitBuilder& builder)
    {
        MegaProver prover{ builder };
        auto verification_key = std::make_shared<MegaFlavor::VerificationKey>(prover.instance->proving_key);
        MegaVerifier verifier{ verification_key };
        auto proof = prover.construct_proof();
        return verifier.verify_proof(proof);
    }

    // Construct a Mega circuit with some arbitrary sample gates
    static MegaCircuitBuilder construct_test_builder()
    {
        auto op_queue = std::make_shared<bb::ECCOpQueue>();
        auto builder = MegaCircuitBuilder{ op_queue };
        GoblinMockCircuits::construct_simple_circuit(builder);
        return builder;
    }
};

/**
 * @brief Test proof construction/verification for a circuit with calldata lookup gates
 * gates
 *
 */
TEST_F(DataBusTests, CallDataRead)
{
    // Construct a circuit and add some ecc op gates and arithmetic gates
    auto builder = construct_test_builder();

    // Add some values to calldata
    std::vector<FF> calldata_values = { 7, 10, 3, 12, 1 };
    for (auto& val : calldata_values) {
        builder.add_public_calldata(builder.add_variable(val));
    }

    // Define some raw indices at which to read calldata
    std::vector<uint32_t> read_indices = { 1, 4 };

    // Create some calldata read gates and store the variable indices of the result for later
    std::vector<uint32_t> result_witness_indices;
    for (uint32_t& read_idx : read_indices) {
        // Create a variable corresponding to the index at which we want to read into calldata
        uint32_t read_idx_witness_idx = builder.add_variable(read_idx);

        auto value_witness_idx = builder.read_calldata(read_idx_witness_idx);
        result_witness_indices.emplace_back(value_witness_idx);
    }

    // Generally, we'll want to use the result of a read in some other operation. As an example, we construct a gate
    // that shows the sum of the two values just read is equal to the expected sum.
    FF expected_sum = 0;
    for (uint32_t& read_idx : read_indices) {
        expected_sum += calldata_values[read_idx];
    }
    builder.create_add_gate(
        { result_witness_indices[0], result_witness_indices[1], builder.zero_idx, 1, 1, 0, -expected_sum });

    // Construct and verify Honk proof
    bool result = construct_and_verify_proof(builder);
    EXPECT_TRUE(result);
}

/**
 * @brief Test proof construction/verification for a circuit with return data lookup gates
 * gates
 *
 */
TEST_F(DataBusTests, ReturnDataRead)
{
    // Construct a circuit and add some ecc op gates and arithmetic gates
    auto builder = construct_test_builder();

    // Add some values to return_data
    std::vector<FF> return_data_values = { 7, 10, 3, 12, 1 };
    for (auto& val : return_data_values) {
        builder.add_public_return_data(builder.add_variable(val));
    }

    // Define some raw indices at which to read return_data
    std::vector<uint32_t> read_indices = { 1, 4 };

    // Create some return_data read gates and store the variable indices of the result for later
    std::vector<uint32_t> result_witness_indices;
    for (uint32_t& read_idx : read_indices) {
        // Create a variable corresponding to the index at which we want to read into return_data
        uint32_t read_idx_witness_idx = builder.add_variable(read_idx);

        auto value_witness_idx = builder.read_return_data(read_idx_witness_idx);
        result_witness_indices.emplace_back(value_witness_idx);
    }

    // Generally, we'll want to use the result of a read in some other operation. As an example, we construct a gate
    // that shows the sum of the two values just read is equal to the expected sum.
    FF expected_sum = 0;
    for (uint32_t& read_idx : read_indices) {
        expected_sum += return_data_values[read_idx];
    }
    builder.create_add_gate(
        { result_witness_indices[0], result_witness_indices[1], builder.zero_idx, 1, 1, 0, -expected_sum });

    // Construct and verify Honk proof
    bool result = construct_and_verify_proof(builder);
    EXPECT_TRUE(result);
}

/**
 * @brief Test reads from calldata and return data in the same circuit
 *
 */
TEST_F(DataBusTests, CallDataAndReturnData)
{
    // Construct a circuit and add some ecc op gates and arithmetic gates
    auto builder = construct_test_builder();

    // Add some values to calldata
    std::vector<FF> calldata_values = { 5, 27, 11 };
    for (auto& val : calldata_values) {
        builder.add_public_calldata(builder.add_variable(val));
    }

    // Add some values to return_data
    std::vector<FF> return_data_values = { 7, 10 };
    for (auto& val : return_data_values) {
        builder.add_public_return_data(builder.add_variable(val));
    }

    // Make some aribitrary reads from calldata and return data
    uint32_t read_idx = 2;
    uint32_t read_idx_witness_idx = builder.add_variable(read_idx);
    builder.read_calldata(read_idx_witness_idx);

    read_idx = 0;
    read_idx_witness_idx = builder.add_variable(read_idx);
    builder.read_return_data(read_idx_witness_idx);

    // Construct and verify Honk proof
    bool result = construct_and_verify_proof(builder);
    EXPECT_TRUE(result);
}

/**
 * @brief Test proof construction/verification for a circuit with duplicate calldata reads
 *
 */
TEST_F(DataBusTests, CallDataDuplicateRead)
{
    // Construct a circuit and add some ecc op gates and arithmetic gates
    auto builder = construct_test_builder();

    // Add some values to calldata
    std::vector<FF> calldata_values = { 7, 10, 3, 12, 1 };
    for (auto& val : calldata_values) {
        builder.add_public_calldata(builder.add_variable(val));
    }

    // Define some read indices with a duplicate
    std::vector<uint32_t> read_indices = { 1, 4, 1 };

    // Create some calldata read gates and store the variable indices of the result for later
    std::vector<uint32_t> result_witness_indices;
    for (uint32_t& read_idx : read_indices) {
        // Create a variable corresponding to the index at which we want to read into calldata
        uint32_t read_idx_witness_idx = builder.add_variable(read_idx);

        auto value_witness_idx = builder.read_calldata(read_idx_witness_idx);
        result_witness_indices.emplace_back(value_witness_idx);
    }

    // Check that the read result is as expected and that the duplicate reads produce the same result
    auto expected_read_result_at_1 = calldata_values[1];
    auto expected_read_result_at_4 = calldata_values[4];
    auto duplicate_read_result_0 = builder.get_variable(result_witness_indices[0]);
    auto duplicate_read_result_1 = builder.get_variable(result_witness_indices[1]);
    auto duplicate_read_result_2 = builder.get_variable(result_witness_indices[2]);
    EXPECT_EQ(duplicate_read_result_0, expected_read_result_at_1);
    EXPECT_EQ(duplicate_read_result_1, expected_read_result_at_4);
    EXPECT_EQ(duplicate_read_result_2, expected_read_result_at_1);

    // Construct and verify Honk proof
    bool result = construct_and_verify_proof(builder);
    EXPECT_TRUE(result);
}