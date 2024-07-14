
#include <gtest/gtest.h>

#include "barretenberg/circuit_checker/circuit_checker.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_circuit_builder.hpp"
#include "databus.hpp"

using namespace bb;

using Builder = MegaCircuitBuilder;
using field_ct = stdlib::field_t<Builder>;
using witness_ct = stdlib::witness_t<Builder>;
using databus_ct = stdlib::databus<Builder>;

namespace {
auto& engine = bb::numeric::get_debug_randomness();
}

/**
 * @brief An expository test demonstrating the functionality of the databus in a small but representative use case
 *
 */
TEST(Databus, CallDataAndReturnData)
{
    Builder builder;
    databus_ct databus;

    // The databus is advantageous in situations where we want to pass large amounts of public inputs between circuits
    // in a chain (like private function execution in Aztec) but where we only need to use a small subset of those
    // values in any given circuit. As an example of this utility, consider the case where the output (return data) is
    // defined by simply taking the last two elements of the input (calldata) and summing them together. We can use the
    // databus mechanism to establish that the return data was indeed formed in this way.

    // Define some bus data that conform to the pattern described above
    std::array<fr, 4> raw_calldata_values = { 4, 5, 6, 7 };
    std::array<fr, 3> raw_return_data_values = { 4, 5, 13 }; // 13 = 6 + 7

    // Populate the calldata in the databus
    std::vector<field_ct> calldata_values;
    for (auto& value : raw_calldata_values) {
        calldata_values.emplace_back(witness_ct(&builder, value));
    }
    databus.calldata.set_values(calldata_values);

    // Populate the return data in the databus
    std::vector<field_ct> return_data_values;
    for (auto& value : raw_return_data_values) {
        return_data_values.emplace_back(witness_ct(&builder, value));
    }
    databus.return_data.set_values(return_data_values);

    // Establish that the first two outputs are simply copied over from the inputs. Each 'copy' requires two read gates.
    field_ct idx_0(witness_ct(&builder, 0));
    field_ct idx_1(witness_ct(&builder, 1));
    databus.calldata[idx_0].assert_equal(databus.return_data[idx_0]);
    databus.calldata[idx_1].assert_equal(databus.return_data[idx_1]);

    // Get the last two entries in calldata and compute their sum
    field_ct idx_2(witness_ct(&builder, 2));
    field_ct idx_3(witness_ct(&builder, 3));
    // This line creates an arithmetic gate and two calldata read gates (via operator[]).
    field_ct sum = databus.calldata[idx_2] + databus.calldata[idx_3];

    // Read the last index of the return data. (Creates a return data read gate via operator[]).
    field_ct idx(witness_ct(&builder, 2));
    field_ct read_result = databus.return_data[idx];

    // By construction, the last return data value is equal to the sum of the last two calldata values
    EXPECT_EQ(sum.get_value(), read_result.get_value());

    // Asserting that 'sum' is equal to the read result completes the process of establishing that the corresponding
    // return data entry was formed correctly; 'sum' is equal to the read result (enforced via copy constraint) and the
    // read result is connected to the value in the databus return data column via the read gate. 'sum' is connected to
    // the calldata values via an arithmetic gate and the two calldata read gates.
    sum.assert_equal(read_result);

    EXPECT_TRUE(CircuitChecker::check(builder));
}

/**
 * @brief A failure test demonstrating that trying to prove (via a databus read) that an erroneous value is present in
 * the databus will result in an invalid witness.
 *
 */
TEST(Databus, BadReadFailure)
{
    Builder builder;
    databus_ct databus;

    // Populate return data with a single arbitrary value
    fr actual_value = 13;
    databus.return_data.set_values({ witness_ct(&builder, actual_value) });

    // Read the value from the return data
    size_t raw_idx = 0; // read at 0th index
    field_ct idx(witness_ct(&builder, raw_idx));
    field_ct read_result = databus.return_data[idx];

    // The result of the read should be as expected
    EXPECT_EQ(actual_value, read_result.get_value());

    // Since the read gate implicitly created by using operator[] on return data is valid, the witness is valid
    EXPECT_TRUE(CircuitChecker::check(builder));

    // Now assert that the read result is equal to some erroneous value. This effectively updates the return data read
    // gate to attest to the erroneous value being present at index 0 in the return data.
    field_ct erroneous_value(witness_ct(&builder, actual_value - 1));
    erroneous_value.assert_equal(read_result);

    // Since the read gate is no longer valid, the circuit checker will fail
    EXPECT_FALSE(CircuitChecker::check(builder));
}

/**
 * @brief A failure test demonstrating that a bad input-output 'copy' will lead to an invalid witness
 *
 */
TEST(Databus, BadCopyFailure)
{
    Builder builder;
    databus_ct databus;

    // Populate calldata with a single input
    fr input = 13;
    databus.calldata.set_values({ witness_ct(&builder, input) });

    // Populate return data with an output different from the input
    fr output = input - 1;
    databus.return_data.set_values({ witness_ct(&builder, output) });

    // Attempt to attest that the calldata has been copied into the return data
    size_t raw_idx = 0; // read at 0th index
    field_ct idx(witness_ct(&builder, raw_idx));
    databus.calldata[idx].assert_equal(databus.return_data[idx]);

    // Since the output data is not a copy of the input, the checker should fail
    EXPECT_FALSE(CircuitChecker::check(builder));
}

/**
 * @brief Check that multiple reads from the same index results in a valid circuit
 *
 */
TEST(Databus, DuplicateRead)
{
    Builder builder;
    databus_ct databus;

    // Define some arbitrary bus data
    std::array<bb::fr, 3> raw_calldata_values = { 5, 1, 2 };
    std::array<bb::fr, 3> raw_return_data_values = { 25, 6, 3 };

    // Populate the calldata in the databus
    std::vector<field_ct> calldata_values;
    for (auto& value : raw_calldata_values) {
        calldata_values.emplace_back(witness_ct(&builder, value));
    }
    databus.calldata.set_values(calldata_values);

    // Populate the return data in the databus
    std::vector<field_ct> return_data_values;
    for (auto& value : raw_return_data_values) {
        return_data_values.emplace_back(witness_ct(&builder, value));
    }
    databus.return_data.set_values(return_data_values);

    // Perform some arbitrary reads from both calldata and return data with some repeated indices
    field_ct idx_1(witness_ct(&builder, 1));
    field_ct idx_2(witness_ct(&builder, 2));

    databus.calldata[idx_1];
    databus.calldata[idx_1];
    databus.calldata[idx_1];
    databus.calldata[idx_2];

    databus.return_data[idx_2];
    databus.return_data[idx_2];
    databus.return_data[idx_1];

    EXPECT_TRUE(CircuitChecker::check(builder));
}