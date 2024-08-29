#include "barretenberg/vm/avm/trace/common.hpp"
#include "barretenberg/vm/avm/trace/helper.hpp"
#include "barretenberg/vm/avm/trace/kernel_trace.hpp"
#include "barretenberg/vm/constants.hpp"
#include "common.test.hpp"

namespace tests_avm {
using namespace bb;
using namespace bb::avm_trace;

class AvmGasTests : public ::testing::Test {
  protected:
    // TODO(640): The Standard Honk on Grumpkin test suite fails unless the SRS is initialised for every test.
    void SetUp() override { srs::init_crs_factory("../srs_db/ignition"); };
};

class AvmGasPositiveTests : public AvmGasTests {};
class AvmGasNegativeTests : public AvmGasTests {
  protected:
    void SetUp() override { GTEST_SKIP(); }
};

// Helper to set the initial gas parameters for each test
struct StartGas {
    uint32_t l2_gas;
    uint32_t da_gas;
};

// TODO: migrate to helper
// Template helper function to apply boilerplate around gas tests
template <typename OpcodesFunc, typename CheckFunc>
void test_gas(StartGas startGas, OpcodesFunc apply_opcodes, CheckFunc check_trace)
{
    std::array<FF, KERNEL_INPUTS_LENGTH> kernel_inputs = {};

    kernel_inputs[L2_GAS_LEFT_CONTEXT_INPUTS_OFFSET] = FF(startGas.l2_gas);
    kernel_inputs[DA_GAS_LEFT_CONTEXT_INPUTS_OFFSET] = FF(startGas.da_gas);

    VmPublicInputs public_inputs;
    std::get<0>(public_inputs) = kernel_inputs;
    AvmTraceBuilder trace_builder(public_inputs);

    // We should return a value of 1 for the sender, as it exists at index 0
    apply_opcodes(trace_builder);

    trace_builder.op_return(0, 0, 0);

    auto trace = trace_builder.finalize();

    check_trace(trace);

    // log_avm_trace(trace, 0, 10);
    validate_trace(std::move(trace), public_inputs);
}

TEST_F(AvmGasPositiveTests, gasAdd)
{
    StartGas start_gas = {
        .l2_gas = 3000,
        .da_gas = 10,
    };

    // We test that the sender opcode is included at index 0 in the public inputs
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) { trace_builder.op_add(0, 1, 2, 3, AvmMemoryTag::FF); };

    auto checks = [=](const std::vector<Row>& trace) {
        auto sender_row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_add == FF(1); });
        EXPECT_TRUE(sender_row != trace.end());
    };

    test_gas(start_gas, apply_opcodes, checks);
}

} // namespace tests_avm
