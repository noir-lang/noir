#include <functional>

#include "avm_common.test.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/avm_trace/avm_kernel_trace.hpp"
#include "barretenberg/vm/avm_trace/constants.hpp"
#include "barretenberg/vm/tests/helpers.test.hpp"

namespace tests_avm {

using namespace bb;
using namespace bb::avm_trace;

auto const BAD_LOOKUP = "LOOKUP_INTO_KERNEL";

class AvmKernelTests : public ::testing::Test {
  protected:
    // TODO(640): The Standard Honk on Grumpkin test suite fails unless the SRS is initialised for every test.
    void SetUp() override { srs::init_crs_factory("../srs_db/ignition"); };
};

class AvmKernelPositiveTests : public AvmKernelTests {};
class AvmKernelNegativeTests : public AvmKernelTests {};

using KernelInputs = std::array<FF, KERNEL_INPUTS_LENGTH>;
const size_t INITIAL_GAS = 10000;

VmPublicInputs get_base_public_inputs()
{
    VmPublicInputs public_inputs = {};

    std::array<FF, KERNEL_INPUTS_LENGTH> kernel_inputs;
    for (size_t i = 0; i < KERNEL_INPUTS_LENGTH; i++) {
        kernel_inputs[i] = FF(i + 1);
    }

    // Set high initial gas
    kernel_inputs[L2_GAS_LEFT_CONTEXT_INPUTS_OFFSET] = INITIAL_GAS;
    kernel_inputs[DA_GAS_LEFT_CONTEXT_INPUTS_OFFSET] = INITIAL_GAS;

    // Copy the kernel inputs into the public inputs object
    std::get<KERNEL_INPUTS>(public_inputs) = kernel_inputs;

    return public_inputs;
}

VmPublicInputs get_public_inputs_with_output(uint32_t output_offset, FF value, FF side_effect_counter, FF metadata)
{
    VmPublicInputs public_inputs = get_base_public_inputs();

    std::get<KERNEL_OUTPUTS_VALUE>(public_inputs)[output_offset] = value;
    std::get<KERNEL_OUTPUTS_SIDE_EFFECT_COUNTER>(public_inputs)[output_offset] = side_effect_counter;
    std::get<KERNEL_OUTPUTS_METADATA>(public_inputs)[output_offset] = metadata;

    return public_inputs;
}

// Template helper function to apply boilerplate around the kernel lookup tests
using OpcodesFunc = std::function<void(AvmTraceBuilder&)>;
using CheckFunc = std::function<void(bool, const std::vector<Row>&)>;
void test_kernel_lookup(bool indirect,
                        OpcodesFunc apply_opcodes,
                        CheckFunc check_trace,
                        VmPublicInputs public_inputs = get_base_public_inputs(),
                        ExecutionHints execution_hints = {})
{
    AvmTraceBuilder trace_builder(public_inputs, std::move(execution_hints));

    apply_opcodes(trace_builder);

    trace_builder.halt();

    auto trace = trace_builder.finalize();

    check_trace(indirect, trace);

    validate_trace(std::move(trace), public_inputs);
}

/*
 * Helper function to assert row values for a kernel lookup opcode
 */
void expect_row(auto row, FF selector, FF ia, FF ind_a, FF mem_addr_a, AvmMemoryTag w_in_tag)
{
    // Checks dependent on the opcode
    EXPECT_EQ(row->kernel_kernel_in_offset, selector);
    EXPECT_EQ(row->main_ia, ia);
    EXPECT_EQ(row->main_mem_addr_a, mem_addr_a);

    // Checks that are fixed for kernel inputs
    EXPECT_EQ(row->main_rwa, FF(1));
    EXPECT_EQ(row->main_ind_addr_a, ind_a);
    EXPECT_EQ(row->main_sel_resolve_ind_addr_a, FF(ind_a != 0));
    EXPECT_EQ(row->main_sel_mem_op_a, FF(1));
    EXPECT_EQ(row->main_w_in_tag, static_cast<uint32_t>(w_in_tag));
    EXPECT_EQ(row->main_sel_q_kernel_lookup, FF(1));
}

void expect_output_table_row(auto row,
                             FF selector,
                             FF ia,
                             FF mem_addr_a,
                             FF ind_a,
                             AvmMemoryTag r_in_tag,
                             uint32_t side_effect_counter,
                             uint32_t rwa = 0)
{
    // Checks dependent on the opcode
    EXPECT_EQ(row->kernel_kernel_out_offset, selector);
    EXPECT_EQ(row->main_ia, ia);
    EXPECT_EQ(row->main_mem_addr_a, mem_addr_a);

    // Checks that are fixed for kernel inputs
    EXPECT_EQ(row->main_rwa, FF(rwa));
    EXPECT_EQ(row->main_ind_addr_a, ind_a);
    EXPECT_EQ(row->main_sel_resolve_ind_addr_a, FF(ind_a != 0));
    EXPECT_EQ(row->main_sel_mem_op_a, FF(1));
    EXPECT_EQ(row->main_r_in_tag, static_cast<uint32_t>(r_in_tag));
    EXPECT_EQ(row->main_sel_q_kernel_output_lookup, FF(1));

    EXPECT_EQ(row->kernel_side_effect_counter, FF(side_effect_counter));
}

void expect_output_table_row_with_metadata(auto row,
                                           FF selector,
                                           FF ia,
                                           FF mem_addr_a,
                                           FF ind_a,
                                           FF ib,
                                           FF mem_addr_b,
                                           FF ind_b,
                                           AvmMemoryTag r_in_tag,
                                           uint32_t side_effect_counter,
                                           uint32_t rwa = 0,
                                           bool no_b = false)
{
    expect_output_table_row(row, selector, ia, mem_addr_a, ind_a, r_in_tag, side_effect_counter, rwa);

    EXPECT_EQ(row->main_ib, ib);
    EXPECT_EQ(row->main_mem_addr_b, mem_addr_b);

    // Checks that are fixed for kernel inputs
    EXPECT_EQ(row->main_rwb, FF(0));

    if (!no_b) {
        EXPECT_EQ(row->main_ind_addr_b, ind_b);
        EXPECT_EQ(row->main_sel_resolve_ind_addr_b, FF(ind_b != 0));
        EXPECT_EQ(row->main_sel_mem_op_b, FF(1));
    }
}

void expect_output_table_row_with_exists_metadata(auto row,
                                                  FF selector,
                                                  FF ia,
                                                  FF mem_addr_a,
                                                  FF ind_a,
                                                  FF ib,
                                                  FF mem_addr_b,
                                                  FF ind_b,
                                                  AvmMemoryTag w_in_tag,
                                                  uint32_t side_effect_counter)
{
    expect_output_table_row(row, selector, ia, mem_addr_a, ind_a, w_in_tag, side_effect_counter);

    EXPECT_EQ(row->main_ib, ib);
    EXPECT_EQ(row->main_mem_addr_b, mem_addr_b);

    // Checks that are fixed for kernel inputs
    EXPECT_EQ(row->main_rwb, FF(1));
    EXPECT_EQ(row->main_ind_addr_b, ind_b);
    EXPECT_EQ(row->main_sel_resolve_ind_addr_b, FF(ind_b != 0));
    EXPECT_EQ(row->main_sel_mem_op_b, FF(1));
}

void check_kernel_outputs(const Row& row, FF value, FF side_effect_counter, FF metadata)
{
    EXPECT_EQ(row.kernel_kernel_value_out, value);
    EXPECT_EQ(row.kernel_kernel_side_effect_out, side_effect_counter);
    EXPECT_EQ(row.kernel_kernel_metadata_out, metadata);
}

TEST_F(AvmKernelPositiveTests, kernelSender)
{
    // Direct
    uint32_t dst_offset = 42;
    uint32_t indirect_dst_offset = 69;
    // We test that the sender opcode is included at index 0 in the public inputs
    auto direct_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_sender(/*indirect*/ false, dst_offset);
    };
    auto indirect_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_set(
            /*indirect*/ false,
            /*value*/ dst_offset,
            /*dst_offset*/ indirect_dst_offset,
            AvmMemoryTag::U32);
        trace_builder.op_sender(/*indirect*/ true, indirect_dst_offset);
    };

    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_sender == FF(1); });
        EXPECT_TRUE(row != trace.end());

        expect_row(row,
                   /*kernel_in_offset=*/SENDER_SELECTOR,
                   /*ia=*/SENDER_SELECTOR +
                       1, // Note the value generated above for public inputs is the same as the index read + 1
                   /*ind_a*/ indirect ? indirect_dst_offset : 0,
                   /*mem_addr_a=*/dst_offset,
                   /*w_in_tag=*/AvmMemoryTag::FF);
    };

    test_kernel_lookup(false, direct_apply_opcodes, checks);
    test_kernel_lookup(true, indirect_apply_opcodes, checks);
}

TEST_F(AvmKernelPositiveTests, kernelAddress)
{
    uint32_t dst_offset = 42;
    uint32_t indirect_dst_offset = 69;
    auto direct_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_address(/*indirect*/ false, dst_offset);
    };
    auto indirect_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_set(
            /*indirect*/ false,
            /*value*/ dst_offset,
            /*dst_offset*/ indirect_dst_offset,
            AvmMemoryTag::U32);
        trace_builder.op_address(/*indirect*/ true, indirect_dst_offset);
    };

    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto address_row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_address == FF(1); });
        EXPECT_TRUE(address_row != trace.end());

        expect_row(address_row,
                   /*kernel_in_offset=*/ADDRESS_SELECTOR,
                   /*ia=*/ADDRESS_SELECTOR +
                       1, // Note the value generated above for public inputs is the same as the index read + 1
                   /*ind_a*/ indirect ? indirect_dst_offset : 0,
                   /*mem_addr_a=*/dst_offset,
                   /*w_in_tag=*/AvmMemoryTag::FF);
    };
    test_kernel_lookup(false, direct_apply_opcodes, checks);
    test_kernel_lookup(true, indirect_apply_opcodes, checks);
}

TEST_F(AvmKernelPositiveTests, kernelStorageAddress)
{
    uint32_t dst_offset = 42;
    uint32_t indirect_dst_offset = 69;
    auto direct_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_storage_address(/*indirect*/ false, dst_offset);
    };
    auto indirect_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_set(
            /*indirect*/ false,
            /*value*/ dst_offset,
            /*dst_offset*/ indirect_dst_offset,
            AvmMemoryTag::U32);
        trace_builder.op_storage_address(/*indirect*/ true, indirect_dst_offset);
    };

    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto storage_address_row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_storage_address == FF(1); });
        EXPECT_TRUE(storage_address_row != trace.end());

        expect_row(storage_address_row,
                   /*kernel_in_offset=*/STORAGE_ADDRESS_SELECTOR,
                   /*ia=*/STORAGE_ADDRESS_SELECTOR +
                       1, // Note the value generated above for public inputs is the same as the index read + 1
                   /*ind_a*/ indirect ? indirect_dst_offset : 0,
                   /*mem_addr_a=*/dst_offset,
                   /*w_in_tag=*/AvmMemoryTag::FF);
    };

    test_kernel_lookup(false, direct_apply_opcodes, checks);
    test_kernel_lookup(true, indirect_apply_opcodes, checks);
}

TEST_F(AvmKernelPositiveTests, kernelFunctionSelector)
{
    // Direct
    uint32_t dst_offset = 42;
    uint32_t indirect_dst_offset = 69;
    // We test that the function selector opcode is included at index 0 in the public inputs
    auto direct_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_function_selector(/*indirect*/ false, dst_offset);
    };
    auto indirect_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_set(
            /*indirect*/ false,
            /*value*/ dst_offset,
            /*dst_offset*/ indirect_dst_offset,
            AvmMemoryTag::U32);
        trace_builder.op_function_selector(/*indirect*/ true, indirect_dst_offset);
    };

    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_function_selector == FF(1); });
        EXPECT_TRUE(row != trace.end());

        expect_row(row,
                   /*kernel_in_offset=*/FUNCTION_SELECTOR_SELECTOR,
                   /*ia=*/FUNCTION_SELECTOR_SELECTOR +
                       1, // Note the value generated above for public inputs is the same as the index read + 1
                   /*ind_a*/ indirect ? indirect_dst_offset : 0,
                   /*mem_addr_a=*/dst_offset,
                   /*w_in_tag=*/AvmMemoryTag::U32);
    };

    test_kernel_lookup(false, direct_apply_opcodes, checks);
    test_kernel_lookup(true, indirect_apply_opcodes, checks);
}

TEST_F(AvmKernelPositiveTests, kernelFeePerDa)
{
    uint32_t dst_offset = 42;
    uint32_t indirect_dst_offset = 69;
    auto direct_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_fee_per_da_gas(/*indirect*/ false, dst_offset);
    };
    auto indirect_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_set(
            /*indirect*/ false,
            /*value*/ dst_offset,
            /*dst_offset*/ indirect_dst_offset,
            AvmMemoryTag::U32);
        trace_builder.op_fee_per_da_gas(/*indirect*/ true, indirect_dst_offset);
    };

    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto fee_row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_fee_per_da_gas == FF(1); });
        EXPECT_TRUE(fee_row != trace.end());

        expect_row(fee_row,
                   /*kernel_in_offset=*/FEE_PER_DA_GAS_SELECTOR,
                   /*ia=*/FEE_PER_DA_GAS_SELECTOR +
                       1, // Note the value generated above for public inputs is the same as the index read + 1
                   /*ind_a*/ indirect ? indirect_dst_offset : 0,
                   /*mem_addr_a=*/dst_offset,
                   /*w_in_tag=*/AvmMemoryTag::FF);
    };

    test_kernel_lookup(false, direct_apply_opcodes, checks);
    test_kernel_lookup(true, indirect_apply_opcodes, checks);
}

TEST_F(AvmKernelPositiveTests, kernelFeePerL2)
{
    uint32_t dst_offset = 42;
    uint32_t indirect_dst_offset = 69;
    auto direct_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_fee_per_l2_gas(/*indirect*/ false, dst_offset);
    };
    auto indirect_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_set(
            /*indirect*/ false,
            /*value*/ dst_offset,
            /*dst_offset*/ indirect_dst_offset,
            AvmMemoryTag::U32);
        trace_builder.op_fee_per_l2_gas(/*indirect*/ true, indirect_dst_offset);
    };

    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto fee_row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_fee_per_l2_gas == FF(1); });
        EXPECT_TRUE(fee_row != trace.end());

        expect_row(fee_row,
                   /*kernel_in_offset=*/FEE_PER_L2_GAS_SELECTOR,
                   /*ia=*/FEE_PER_L2_GAS_SELECTOR +
                       1, // Note the value generated above for public inputs is the same as the index read + 1
                   /*ind_a*/ indirect ? indirect_dst_offset : 0,
                   /*mem_addr_a=*/dst_offset,
                   /*w_in_tag=*/AvmMemoryTag::FF);
    };

    test_kernel_lookup(false, direct_apply_opcodes, checks);
    test_kernel_lookup(true, indirect_apply_opcodes, checks);
}

TEST_F(AvmKernelPositiveTests, kernelTransactionFee)
{
    uint32_t dst_offset = 42;
    uint32_t indirect_dst_offset = 69;
    auto direct_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_transaction_fee(/*indirect*/ false, dst_offset);
    };
    auto indirect_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_set(
            /*indirect*/ false,
            /*value*/ dst_offset,
            /*dst_offset*/ indirect_dst_offset,
            AvmMemoryTag::U32);
        trace_builder.op_transaction_fee(/*indirect*/ true, indirect_dst_offset);
    };

    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto fee_row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_transaction_fee == FF(1); });
        EXPECT_TRUE(fee_row != trace.end());

        expect_row(fee_row,
                   /*kernel_in_offset=*/TRANSACTION_FEE_SELECTOR,
                   /*ia=*/TRANSACTION_FEE_SELECTOR +
                       1, // Note the value generated above for public inputs is the same as the index read + 1
                   /*ind_a*/ indirect ? indirect_dst_offset : 0,
                   /*mem_addr_a=*/dst_offset,
                   /*w_in_tag=*/AvmMemoryTag::FF);
    };

    test_kernel_lookup(false, direct_apply_opcodes, checks);
    test_kernel_lookup(true, indirect_apply_opcodes, checks);
}

TEST_F(AvmKernelPositiveTests, kernelChainId)
{
    uint32_t dst_offset = 42;
    uint32_t indirect_dst_offset = 69;
    auto direct_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_chain_id(/*indirect*/ false, dst_offset);
    };
    auto indirect_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_set(
            /*indirect*/ false,
            /*value*/ dst_offset,
            /*dst_offset*/ indirect_dst_offset,
            AvmMemoryTag::U32);
        trace_builder.op_chain_id(/*indirect*/ true, indirect_dst_offset);
    };

    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto fee_row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_chain_id == FF(1); });
        EXPECT_TRUE(fee_row != trace.end());

        expect_row(fee_row,
                   /*kernel_in_offset=*/CHAIN_ID_SELECTOR,
                   /*ia=*/CHAIN_ID_SELECTOR +
                       1, // Note the value generated above for public inputs is the same as the index read + 1
                   /*ind_a*/ indirect ? indirect_dst_offset : 0,
                   /*mem_addr_a=*/dst_offset,
                   /*w_in_tag=*/AvmMemoryTag::FF);
    };

    test_kernel_lookup(false, direct_apply_opcodes, checks);
    test_kernel_lookup(true, indirect_apply_opcodes, checks);
}

TEST_F(AvmKernelPositiveTests, kernelVersion)
{
    uint32_t dst_offset = 42;
    uint32_t indirect_dst_offset = 69;
    auto direct_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_version(/*indirect*/ false, dst_offset);
    };
    auto indirect_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_set(
            /*indirect*/ false,
            /*value*/ dst_offset,
            /*dst_offset*/ indirect_dst_offset,
            AvmMemoryTag::U32);
        trace_builder.op_version(/*indirect*/ true, indirect_dst_offset);
    };

    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto fee_row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_version == FF(1); });
        EXPECT_TRUE(fee_row != trace.end());

        expect_row(fee_row,
                   /*kernel_in_offset=*/VERSION_SELECTOR,
                   /*ia=*/VERSION_SELECTOR +
                       1, // Note the value generated above for public inputs is the same as the index read + 1
                   /*ind_a*/ indirect ? indirect_dst_offset : 0,
                   /*mem_addr_a=*/dst_offset,
                   /*w_in_tag=*/AvmMemoryTag::FF);
    };

    test_kernel_lookup(false, direct_apply_opcodes, checks);
    test_kernel_lookup(true, indirect_apply_opcodes, checks);
}

TEST_F(AvmKernelPositiveTests, kernelBlockNumber)
{
    uint32_t dst_offset = 42;
    uint32_t indirect_dst_offset = 69;
    auto direct_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_block_number(/*indirect*/ false, dst_offset);
    };
    auto indirect_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_set(
            /*indirect*/ false,
            /*value*/ dst_offset,
            /*dst_offset*/ indirect_dst_offset,
            AvmMemoryTag::U32);
        trace_builder.op_block_number(/*indirect*/ true, indirect_dst_offset);
    };

    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto fee_row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_block_number == FF(1); });
        EXPECT_TRUE(fee_row != trace.end());

        expect_row(fee_row,
                   /*kernel_in_offset=*/BLOCK_NUMBER_SELECTOR,
                   /*ia=*/BLOCK_NUMBER_SELECTOR +
                       1, // Note the value generated above for public inputs is the same as the index read + 1
                   /*ind_a*/ indirect ? indirect_dst_offset : 0,
                   /*mem_addr_a=*/dst_offset,
                   /*w_in_tag=*/AvmMemoryTag::FF);
    };

    test_kernel_lookup(false, direct_apply_opcodes, checks);
    test_kernel_lookup(true, indirect_apply_opcodes, checks);
}

TEST_F(AvmKernelPositiveTests, kernelCoinbase)
{
    uint32_t dst_offset = 42;
    uint32_t indirect_dst_offset = 69;
    auto direct_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_coinbase(/*indirect*/ false, dst_offset);
    };
    auto indirect_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_set(
            /*indirect*/ false,
            /*value*/ dst_offset,
            /*dst_offset*/ indirect_dst_offset,
            AvmMemoryTag::U32);
        trace_builder.op_coinbase(/*indirect*/ true, indirect_dst_offset);
    };

    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto fee_row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_coinbase == FF(1); });
        EXPECT_TRUE(fee_row != trace.end());

        expect_row(fee_row,
                   /*kernel_in_offset=*/COINBASE_SELECTOR,
                   /*ia=*/COINBASE_SELECTOR +
                       1, // Note the value generated above for public inputs is the same as the index read + 1
                   /*ind_a*/ indirect ? indirect_dst_offset : 0,
                   /*mem_addr_a*/ dst_offset,
                   /*w_in_tag=*/AvmMemoryTag::FF);
    };

    test_kernel_lookup(false, direct_apply_opcodes, checks);
    test_kernel_lookup(true, indirect_apply_opcodes, checks);
}

TEST_F(AvmKernelPositiveTests, kernelTimestamp)
{
    uint32_t dst_offset = 42;
    uint32_t indirect_dst_offset = 69;
    auto direct_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_timestamp(/*indirect*/ false, dst_offset);
    };
    auto indirect_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_set(
            /*indirect*/ false,
            /*value*/ dst_offset,
            /*dst_offset*/ indirect_dst_offset,
            AvmMemoryTag::U32);
        trace_builder.op_timestamp(/*indirect*/ true, indirect_dst_offset);
    };

    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto fee_row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_timestamp == FF(1); });
        EXPECT_TRUE(fee_row != trace.end());

        expect_row(fee_row,
                   /*kernel_in_offset=*/TIMESTAMP_SELECTOR,
                   /*ia=*/TIMESTAMP_SELECTOR +
                       1, // Note the value generated above for public inputs is the same as the index read + 1
                   /*ind_a*/ indirect ? indirect_dst_offset : 0,
                   /*mem_addr_a*/ dst_offset,
                   /*w_in_tag=*/AvmMemoryTag::U64);
    };

    test_kernel_lookup(false, direct_apply_opcodes, checks);
    test_kernel_lookup(true, indirect_apply_opcodes, checks);
}

/**
 * Negative Tests
 */

// Template helper function to apply boilerplate
template <typename OpcodesFunc, typename CheckFunc>
void negative_test_incorrect_ia_kernel_lookup(OpcodesFunc apply_opcodes,
                                              CheckFunc check_trace,
                                              FF incorrect_ia,
                                              auto expected_message)
{
    VmPublicInputs public_inputs = get_base_public_inputs();
    AvmTraceBuilder trace_builder(public_inputs);

    // We should return a value of 1 for the sender, as it exists at index 0
    apply_opcodes(trace_builder);

    trace_builder.halt();

    auto trace = trace_builder.finalize();

    // Change IA to be a value not in the lookup
    // Change the first row, as that will be where each of the opcodes are in the test
    auto& ta = trace.at(1);

    ta.main_ia = incorrect_ia;
    // memory trace should only have one row for these tests as well, so first row has looked-up val
    ta.mem_val = incorrect_ia;

    check_trace(/*indirect*/ false, trace);

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), expected_message);
}

TEST_F(AvmKernelNegativeTests, incorrectIaSender)
{
    uint32_t dst_offset = 42;
    FF incorrect_ia = FF(69);

    // We test that the sender opcode is inlcuded at index x in the public inputs
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_sender(/*indirect*/ false, dst_offset);
    };
    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_sender == FF(1); });
        EXPECT_TRUE(row != trace.end());

        expect_row(
            row,
            /*kernel_in_offset=*/SENDER_SELECTOR,
            /*ia=*/incorrect_ia, // Note the value generated above for public inputs is the same as the index read + 1
            /*ind_a*/ indirect,
            /*mem_addr_a=*/dst_offset,
            /*w_in_tag=*/AvmMemoryTag::FF);
    };

    negative_test_incorrect_ia_kernel_lookup(apply_opcodes, checks, incorrect_ia, BAD_LOOKUP);
}

TEST_F(AvmKernelNegativeTests, incorrectIaAddress)
{
    uint32_t dst_offset = 42;
    FF incorrect_ia = FF(69);

    // We test that the sender opcode is inlcuded at index x in the public inputs
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_address(/*indirect*/ false, dst_offset);
    };
    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_address == FF(1); });
        EXPECT_TRUE(row != trace.end());

        expect_row(
            row,
            /*kernel_in_offset=*/ADDRESS_SELECTOR,
            /*ia=*/incorrect_ia, // Note the value generated above for public inputs is the same as the index read + 1
            /*ind_a*/ indirect,
            /*mem_addr_a=*/dst_offset,
            /*w_in_tag=*/AvmMemoryTag::FF);
    };

    negative_test_incorrect_ia_kernel_lookup(apply_opcodes, checks, incorrect_ia, BAD_LOOKUP);
}

TEST_F(AvmKernelNegativeTests, incorrectIaStorageAddress)
{
    uint32_t dst_offset = 42;
    FF incorrect_ia = FF(69);

    // We test that the sender opcode is inlcuded at index x in the public inputs
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_storage_address(/*indirect*/ false, dst_offset);
    };
    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_storage_address == FF(1); });
        EXPECT_TRUE(row != trace.end());

        expect_row(
            row,
            /*kernel_in_offset=*/STORAGE_ADDRESS_SELECTOR,
            /*ia=*/incorrect_ia, // Note the value generated above for public inputs is the same as the index read + 1
            /*ind_a*/ indirect,
            /*mem_addr_a=*/dst_offset,
            /*w_in_tag=*/AvmMemoryTag::FF);
    };

    negative_test_incorrect_ia_kernel_lookup(apply_opcodes, checks, incorrect_ia, BAD_LOOKUP);
}

TEST_F(AvmKernelNegativeTests, incorrectIaFunctionSelector)
{
    uint32_t dst_offset = 42;
    FF incorrect_ia = FF(69);

    // We test that the sender opcode is inlcuded at index x in the public inputs
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_function_selector(/*indirect*/ false, dst_offset);
    };
    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_function_selector == FF(1); });
        EXPECT_TRUE(row != trace.end());

        expect_row(
            row,
            /*kernel_in_offset=*/FUNCTION_SELECTOR_SELECTOR,
            /*ia=*/incorrect_ia, // Note the value generated above for public inputs is the same as the index read + 1
            /*ind_a*/ indirect,
            /*mem_addr_a=*/dst_offset,
            /*w_in_tag=*/AvmMemoryTag::U32);
    };

    negative_test_incorrect_ia_kernel_lookup(apply_opcodes, checks, incorrect_ia, BAD_LOOKUP);
}

TEST_F(AvmKernelNegativeTests, incorrectIaDaGas)
{
    uint32_t dst_offset = 42;
    FF incorrect_ia = FF(69);

    // We test that the sender opcode is inlcuded at index x in the public inputs
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_fee_per_da_gas(/*indirect*/ false, dst_offset);
    };
    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_fee_per_da_gas == FF(1); });
        EXPECT_TRUE(row != trace.end());

        expect_row(
            row,
            /*kernel_in_offset=*/FEE_PER_DA_GAS_SELECTOR,
            /*ia=*/incorrect_ia, // Note the value generated above for public inputs is the same as the index read + 1
            /*ind_a*/ indirect,
            /*mem_addr_a=*/dst_offset,
            /*w_in_tag=*/AvmMemoryTag::FF);
    };

    negative_test_incorrect_ia_kernel_lookup(apply_opcodes, checks, incorrect_ia, BAD_LOOKUP);
}

TEST_F(AvmKernelNegativeTests, incorrectIal2Gas)
{
    uint32_t dst_offset = 42;
    FF incorrect_ia = FF(69);

    // We test that the sender opcode is inlcuded at index x in the public inputs
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_fee_per_l2_gas(/*indirect*/ false, dst_offset);
    };
    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_fee_per_l2_gas == FF(1); });
        EXPECT_TRUE(row != trace.end());

        expect_row(
            row,
            /*kernel_in_offset=*/FEE_PER_L2_GAS_SELECTOR,
            /*ia=*/incorrect_ia, // Note the value generated above for public inputs is the same as the index read + 1
            /*ind_a*/ indirect,
            /*mem_addr_a=*/dst_offset,
            /*w_in_tag=*/AvmMemoryTag::FF);
    };

    negative_test_incorrect_ia_kernel_lookup(apply_opcodes, checks, incorrect_ia, BAD_LOOKUP);
}

TEST_F(AvmKernelNegativeTests, incorrectIaTransactionFee)
{
    uint32_t dst_offset = 42;
    FF incorrect_ia = FF(69);

    // We test that the sender opcode is inlcuded at index x in the public inputs
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_transaction_fee(/*indirect*/ false, dst_offset);
    };
    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_transaction_fee == FF(1); });
        EXPECT_TRUE(row != trace.end());

        expect_row(
            row,
            /*kernel_in_offset=*/TRANSACTION_FEE_SELECTOR,
            /*ia=*/incorrect_ia, // Note the value generated above for public inputs is the same as the index read + 1
            /*ind_a*/ indirect,
            /*mem_addr_a=*/dst_offset,
            /*w_in_tag=*/AvmMemoryTag::FF);
    };

    negative_test_incorrect_ia_kernel_lookup(apply_opcodes, checks, incorrect_ia, BAD_LOOKUP);
}

TEST_F(AvmKernelNegativeTests, incorrectIaChainId)
{
    uint32_t dst_offset = 42;
    FF incorrect_ia = FF(69);

    // We test that the sender opcode is inlcuded at index x in the public inputs
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_chain_id(/*indirect*/ false, dst_offset);
    };
    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_chain_id == FF(1); });
        EXPECT_TRUE(row != trace.end());

        expect_row(
            row,
            /*kernel_in_offset=*/CHAIN_ID_SELECTOR,
            /*ia=*/incorrect_ia, // Note the value generated above for public inputs is the same as the index read + 1
            /*ind_a*/ indirect,
            /*mem_addr_a=*/dst_offset,
            /*w_in_tag=*/AvmMemoryTag::FF);
    };

    negative_test_incorrect_ia_kernel_lookup(apply_opcodes, checks, incorrect_ia, BAD_LOOKUP);
}

TEST_F(AvmKernelNegativeTests, incorrectIaVersion)
{
    uint32_t dst_offset = 42;
    FF incorrect_ia = FF(69);

    // We test that the sender opcode is inlcuded at index x in the public inputs
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_version(/*indirect*/ false, dst_offset);
    };
    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_version == FF(1); });
        EXPECT_TRUE(row != trace.end());

        expect_row(
            row,
            /*kernel_in_offset=*/VERSION_SELECTOR,
            /*ia=*/incorrect_ia, // Note the value generated above for public inputs is the same as the index read + 1
            /*ind_a*/ indirect,
            /*mem_addr_a=*/dst_offset,
            /*w_in_tag=*/AvmMemoryTag::FF);
    };

    negative_test_incorrect_ia_kernel_lookup(apply_opcodes, checks, incorrect_ia, BAD_LOOKUP);
}

TEST_F(AvmKernelNegativeTests, incorrectIaBlockNumber)
{
    uint32_t dst_offset = 42;
    FF incorrect_ia = FF(69);

    // We test that the sender opcode is inlcuded at index x in the public inputs
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_block_number(/*indirect*/ false, dst_offset);
    };
    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_block_number == FF(1); });
        EXPECT_TRUE(row != trace.end());

        expect_row(
            row,
            /*kernel_in_offset=*/BLOCK_NUMBER_SELECTOR,
            /*ia=*/incorrect_ia, // Note the value generated above for public inputs is the same as the index read + 1
            /*ind_a*/ indirect,
            /*mem_addr_a=*/dst_offset,
            /*w_in_tag=*/AvmMemoryTag::FF);
    };

    negative_test_incorrect_ia_kernel_lookup(apply_opcodes, checks, incorrect_ia, BAD_LOOKUP);
}

TEST_F(AvmKernelNegativeTests, incorrectIaTimestamp)
{
    uint32_t dst_offset = 42;
    FF incorrect_ia = FF(69);

    // We test that the sender opcode is inlcuded at index x in the public inputs
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_timestamp(/*indirect*/ false, dst_offset);
    };
    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_timestamp == FF(1); });
        EXPECT_TRUE(row != trace.end());

        expect_row(
            row,
            /*kernel_in_offset=*/TIMESTAMP_SELECTOR,
            /*ia=*/incorrect_ia, // Note the value generated above for public inputs is the same as the index read + 1
            /*ind_a*/ indirect,
            /*mem_addr_a*/ dst_offset,
            /*w_in_tag=*/AvmMemoryTag::U64);
    };

    negative_test_incorrect_ia_kernel_lookup(apply_opcodes, checks, incorrect_ia, BAD_LOOKUP);
}

TEST_F(AvmKernelNegativeTests, incorrectIaCoinbase)
{
    uint32_t dst_offset = 42;
    FF incorrect_ia = FF(69);

    // We test that the sender opcode is inlcuded at index x in the public inputs
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_coinbase(/*indirect*/ false, dst_offset);
    };
    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_coinbase == FF(1); });
        EXPECT_TRUE(row != trace.end());

        expect_row(
            row,
            /*kernel_in_offset=*/COINBASE_SELECTOR,
            /*ia=*/incorrect_ia, // Note the value generated above for public inputs is the same as the index read + 1
            /*ind_a*/ indirect,
            /*mem_addr_a=*/dst_offset,
            /*w_in_tag=*/AvmMemoryTag::FF);
    };

    negative_test_incorrect_ia_kernel_lookup(apply_opcodes, checks, incorrect_ia, BAD_LOOKUP);
}

// KERNEL OUTPUTS
class AvmKernelOutputPositiveTests : public AvmKernelTests {};
class AvmKernelOutputNegativeTests : public AvmKernelTests {};

TEST_F(AvmKernelOutputPositiveTests, kernelEmitNoteHash)
{
    uint32_t direct_offset = 42;
    uint32_t indirect_offset = 69;
    uint32_t value = 1234;

    uint32_t output_offset = START_EMIT_NOTE_HASH_WRITE_OFFSET;

    // We write the note hash into memory
    auto direct_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_set(0, 1234, direct_offset, AvmMemoryTag::FF);
        trace_builder.op_emit_note_hash(/*indirect=*/false, direct_offset);
    };
    auto indirect_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_set(0, 1234, direct_offset, AvmMemoryTag::FF);
        trace_builder.op_set(0, direct_offset, indirect_offset, AvmMemoryTag::U32);
        trace_builder.op_emit_note_hash(/*indirect=*/true, indirect_offset);
    };

    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_emit_note_hash == FF(1); });
        EXPECT_TRUE(row != trace.end());

        expect_output_table_row(
            row,
            /*kernel_in_offset=*/output_offset,
            /*ia=*/value, // Note the value generated above for public inputs is the same as the index read + 1
            /*mem_addr_a=*/direct_offset,
            /*ind_a*/ indirect ? indirect_offset : 0,
            /*w_in_tag=*/AvmMemoryTag::FF,
            /*side_effect_counter=*/0);

        check_kernel_outputs(trace.at(output_offset), value, /*side_effect_counter=*/0, /*metadata=*/0);
    };

    VmPublicInputs public_inputs =
        get_public_inputs_with_output(output_offset, value, /*side_effect_counter=*/0, /*metadata*/ 0);
    test_kernel_lookup(false, direct_apply_opcodes, checks, public_inputs);
    test_kernel_lookup(true, indirect_apply_opcodes, checks, public_inputs);
}

TEST_F(AvmKernelOutputPositiveTests, kernelEmitNullifier)
{
    uint32_t direct_offset = 42;
    uint32_t indirect_offset = 69;
    uint32_t value = 1234;

    uint32_t output_offset = START_EMIT_NULLIFIER_WRITE_OFFSET;

    // We write the note hash into memory
    auto direct_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_set(0, 1234, direct_offset, AvmMemoryTag::FF);
        trace_builder.op_emit_nullifier(/*indirect=*/false, direct_offset);
    };
    auto indirect_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_set(0, 1234, direct_offset, AvmMemoryTag::FF);
        trace_builder.op_set(0, direct_offset, indirect_offset, AvmMemoryTag::U32);
        trace_builder.op_emit_nullifier(/*indirect=*/true, indirect_offset);
    };

    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_emit_nullifier == FF(1); });
        EXPECT_TRUE(row != trace.end());

        expect_output_table_row(
            row,
            /*kernel_in_offset=*/output_offset,
            /*ia=*/value, // Note the value generated above for public inputs is the same as the index read + 1
            /*mem_addr_a=*/direct_offset,
            /*ind_a*/ indirect ? indirect_offset : 0,
            /*w_in_tag=*/AvmMemoryTag::FF,
            /*side_effect_counter=*/0);

        // Validate lookup and counts
        // Plus 1 as we have a padded empty first row
        check_kernel_outputs(trace.at(output_offset), value, /*side_effect_counter=*/0, /*metadata=*/0);
    };

    VmPublicInputs public_inputs =
        get_public_inputs_with_output(output_offset, value, /*side_effect_counter=*/0, /*metadata*/ 0);
    test_kernel_lookup(false, direct_apply_opcodes, checks, public_inputs);
    test_kernel_lookup(true, indirect_apply_opcodes, checks, public_inputs);
}

TEST_F(AvmKernelOutputPositiveTests, kernelEmitL2ToL1Msg)
{
    uint32_t msg_offset = 42;
    uint32_t indirect_msg_offset = 420;

    uint32_t recipient_offset = 69;
    uint32_t indirect_recipient_offset = 690;

    uint32_t value = 1234;
    uint32_t recipient = 420;
    uint32_t output_offset = START_EMIT_L2_TO_L1_MSG_WRITE_OFFSET;

    // auto direct_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
    //     trace_builder.op_set(0, 1234, msg_offset, AvmMemoryTag::FF);
    //     trace_builder.op_set(0, 420, recipient_offset, AvmMemoryTag::FF);
    //     trace_builder.op_emit_l2_to_l1_msg(false, recipient_offset, msg_offset);
    // };
    auto indirect_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_set(0, 1234, msg_offset, AvmMemoryTag::FF);
        trace_builder.op_set(0, msg_offset, indirect_msg_offset, AvmMemoryTag::U32);
        trace_builder.op_set(0, 420, recipient_offset, AvmMemoryTag::FF);
        trace_builder.op_set(0, recipient_offset, indirect_recipient_offset, AvmMemoryTag::U32);
        trace_builder.op_emit_l2_to_l1_msg(3, indirect_recipient_offset, indirect_msg_offset);
    };

    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_emit_l2_to_l1_msg == FF(1); });
        EXPECT_TRUE(row != trace.end());

        expect_output_table_row_with_metadata(
            row,
            /*kernel_in_offset=*/output_offset,
            /*ia=*/value, // Note the value generated above for public inputs is the same as the index read + 1
            /*mem_addr_a=*/msg_offset,
            /*ind_a*/ indirect ? indirect_msg_offset : 0,
            /*ib=*/recipient,
            /*mem_addr_b=*/recipient_offset,
            /*ind_a*/ indirect ? indirect_recipient_offset : 0,
            /*w_in_tag=*/AvmMemoryTag::FF,
            /*side_effect_counter=*/0);

        check_kernel_outputs(trace.at(output_offset), value, /*side_effect_counter=*/0, /*metadata=*/recipient);
    };

    // test_kernel_lookup(false, direct_apply_opcodes, checks);
    VmPublicInputs public_inputs =
        get_public_inputs_with_output(output_offset, value, /*side_effect_counter=*/0, recipient);
    test_kernel_lookup(true, indirect_apply_opcodes, checks, std::move(public_inputs));
}

TEST_F(AvmKernelOutputPositiveTests, kernelEmitUnencryptedLog)
{
    uint32_t direct_offset = 42;
    uint32_t indirect_offset = 69;
    uint32_t value = 1234;
    uint32_t slot = 0;
    uint32_t output_offset = START_EMIT_UNENCRYPTED_LOG_WRITE_OFFSET;

    // We write the note hash into memory
    auto direct_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_set(0, 1234, direct_offset, AvmMemoryTag::FF);
        trace_builder.op_emit_unencrypted_log(/*indirect=*/false, direct_offset, /*log_size_offset=*/0);
    };
    auto indirect_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_set(0, 1234, direct_offset, AvmMemoryTag::FF);
        trace_builder.op_set(0, direct_offset, indirect_offset, AvmMemoryTag::U32);
        trace_builder.op_emit_unencrypted_log(/*indirect=*/true, indirect_offset, /*log_size_offset=*/0);
    };

    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_emit_unencrypted_log == FF(1); });
        EXPECT_TRUE(row != trace.end());

        expect_output_table_row(
            row,
            /*kernel_in_offset=*/output_offset,
            /*ia=*/value, // Note the value generated above for public inputs is the same as the index read + 1
            /*mem_addr_a=*/direct_offset,
            /*ind_a*/ indirect ? indirect_offset : 0,
            /*w_in_tag=*/AvmMemoryTag::FF,
            /*side_effect_counter=*/0);

        check_kernel_outputs(trace.at(output_offset), value, 0, slot);
    };

    VmPublicInputs public_inputs = get_public_inputs_with_output(output_offset, value, /*side_effect_counter=*/0, slot);
    test_kernel_lookup(false, direct_apply_opcodes, checks, public_inputs);
    test_kernel_lookup(true, indirect_apply_opcodes, checks, public_inputs);
}

TEST_F(AvmKernelOutputPositiveTests, kernelSload)
{
    uint8_t indirect = 0;
    uint32_t dest_offset = 42;
    auto value = 1234;
    uint32_t size = 1;
    uint32_t slot_offset = 420;
    auto slot = 12345;
    uint32_t output_offset = START_SLOAD_WRITE_OFFSET;

    // Provide a hint for sload value slot
    auto execution_hints = ExecutionHints().with_storage_value_hints({ { 0, value } });

    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_set(0, static_cast<uint128_t>(slot), slot_offset, AvmMemoryTag::FF);
        trace_builder.op_sload(indirect, slot_offset, size, dest_offset);
    };
    auto checks = [=]([[maybe_unused]] bool indirect, const std::vector<Row>& trace) {
        auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_sload == FF(1); });
        ASSERT_TRUE(row != trace.end());

        // TODO: temporarily hardcoded to direct, resolved by dbanks12 / ilyas pr - use your changes
        expect_output_table_row_with_metadata(
            row,
            /*kernel_in_offset=*/output_offset,
            /*ia=*/value, // Note the value generated above for public inputs is the same as the index read + 1
            /*mem_addr_a=*/dest_offset,
            /*ind_a=*/false,
            /*ib=*/slot,
            /*mem_addr_b=*/0,
            /*ind_b=*/false,
            /*r_in_tag=*/AvmMemoryTag::U0, // Kernel Sload is writing to memory
            /*side_effect_counter=*/0,
            /*rwa=*/1,
            /*no_b=*/true);

        check_kernel_outputs(trace.at(output_offset), value, /*side_effect_counter=*/0, slot);
    };

    VmPublicInputs public_inputs = get_public_inputs_with_output(output_offset, value, /*side_effect_counter=*/0, slot);
    test_kernel_lookup(false, apply_opcodes, checks, std::move(public_inputs), execution_hints);
}

TEST_F(AvmKernelOutputPositiveTests, kernelSstore)
{
    uint32_t value_offset = 42;
    auto value = 1234;
    uint32_t metadata_offset = 420;
    auto slot = 12345;
    uint8_t indirect = 0;
    uint32_t size = 1;
    uint32_t output_offset = START_SSTORE_WRITE_OFFSET;

    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_set(0, static_cast<uint128_t>(value), value_offset, AvmMemoryTag::FF);
        trace_builder.op_set(0, static_cast<uint128_t>(slot), metadata_offset, AvmMemoryTag::FF);
        trace_builder.op_sstore(indirect, value_offset, size, metadata_offset);
    };
    auto checks = [=]([[maybe_unused]] bool indirect, const std::vector<Row>& trace) {
        auto row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_sstore == FF(1); });
        EXPECT_TRUE(row != trace.end());

        // TODO: temporarily hardcoded to direct, resolved by dbanks12 / ilyas pr - use your changes
        expect_output_table_row_with_metadata(
            row,
            /*kernel_in_offset=*/output_offset,
            /*ia=*/value, // Note the value generated above for public inputs is the same as the index read + 1
            /*mem_addr_a=*/value_offset,
            /*ind_a*/ false,
            /*ib=*/slot,
            /*mem_addr_b=*/0,
            /*ind_b*/ false,
            /*r_in_tag=*/AvmMemoryTag::FF,
            /*side_effect_counter=*/0,
            /*rwa=*/0,
            /*no_b=*/true);

        check_kernel_outputs(trace.at(output_offset), value, /*side_effect_counter=*/0, slot);
    };

    VmPublicInputs public_inputs = get_public_inputs_with_output(output_offset, value, /*side_effect_counter=*/0, slot);
    test_kernel_lookup(false, apply_opcodes, checks, std::move(public_inputs));
}

TEST_F(AvmKernelOutputPositiveTests, kernelNoteHashExists)
{
    uint32_t value_offset = 42;
    uint32_t indirect_value_offset = 69;
    auto value = 1234;
    uint32_t metadata_offset = 420;
    uint32_t indirect_metadata_offset = 690;
    auto exists = 1;
    uint32_t output_offset = START_NOTE_HASH_EXISTS_WRITE_OFFSET;

    auto execution_hints = ExecutionHints().with_note_hash_exists_hints({ { 0, exists } });

    auto direct_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_set(0, static_cast<uint128_t>(value), value_offset, AvmMemoryTag::FF);
        trace_builder.op_note_hash_exists(/*indirect*/ false, value_offset, metadata_offset);
    };
    // TODO: fix
    auto indirect_apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_set(0, static_cast<uint128_t>(value), value_offset, AvmMemoryTag::FF);
        trace_builder.op_set(0, value_offset, indirect_value_offset, AvmMemoryTag::U32);
        trace_builder.op_set(0, metadata_offset, indirect_metadata_offset, AvmMemoryTag::U32);
        trace_builder.op_note_hash_exists(/*indirect*/ 3, indirect_value_offset, indirect_metadata_offset);
    };
    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_note_hash_exists == FF(1); });
        EXPECT_TRUE(row != trace.end());

        expect_output_table_row_with_exists_metadata(
            row,
            /*kernel_in_offset=*/output_offset,
            /*ia=*/value, // Note the value generated above for public inputs is the same as the index read + 1
            /*mem_addr_a=*/value_offset,
            /*ind_a*/ indirect ? FF(indirect_value_offset) : FF(0),
            /*ib=*/exists,
            /*mem_addr_b=*/metadata_offset,
            /*ind_b*/ indirect ? FF(indirect_metadata_offset) : FF(0),
            /*w_in_tag=*/AvmMemoryTag::FF,
            /*side_effect_counter=*/0);

        check_kernel_outputs(trace.at(output_offset), value, /*side_effect_counter=*/0, exists);
    };

    VmPublicInputs public_inputs =
        get_public_inputs_with_output(output_offset, value, /*side_effect_counter=*/0, exists);
    test_kernel_lookup(false, direct_apply_opcodes, checks, public_inputs, execution_hints);
    test_kernel_lookup(true, indirect_apply_opcodes, checks, public_inputs, execution_hints);
}

TEST_F(AvmKernelOutputPositiveTests, kernelNullifierExists)
{
    uint32_t value_offset = 42;
    auto value = 1234;
    uint32_t metadata_offset = 420;
    auto exists = 1;
    uint32_t output_offset = START_NULLIFIER_EXISTS_OFFSET;

    auto execution_hints = ExecutionHints().with_nullifier_exists_hints({ { 0, exists } });

    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_set(0, static_cast<uint128_t>(value), value_offset, AvmMemoryTag::FF);
        trace_builder.op_nullifier_exists(/*indirect=*/false, value_offset, metadata_offset);
    };
    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_nullifier_exists == FF(1); });
        EXPECT_TRUE(row != trace.end());

        expect_output_table_row_with_exists_metadata(
            row,
            /*kernel_in_offset=*/output_offset,
            /*ia=*/value, // Note the value generated above for public inputs is the same as the index read + 1
            /*mem_addr_a=*/value_offset,
            /*ind_a*/ indirect,
            /*ib=*/exists,
            /*mem_addr_b=*/metadata_offset,
            /*ind_b*/ indirect,
            /*w_in_tag=*/AvmMemoryTag::FF,
            /*side_effect_counter=*/0);

        check_kernel_outputs(trace.at(output_offset), value, /*side_effect_counter=*/0, exists);
    };

    VmPublicInputs public_inputs =
        get_public_inputs_with_output(output_offset, value, /*side_effect_counter=*/0, exists);
    test_kernel_lookup(false, apply_opcodes, checks, std::move(public_inputs), execution_hints);
}

TEST_F(AvmKernelOutputPositiveTests, kernelNullifierNonExists)
{
    uint32_t value_offset = 42;
    auto value = 1234;
    uint32_t metadata_offset = 420;
    auto exists = 0;
    uint32_t output_offset = START_NULLIFIER_NON_EXISTS_OFFSET;

    auto execution_hints = ExecutionHints().with_nullifier_exists_hints({ { 0, exists } });

    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_set(0, static_cast<uint128_t>(value), value_offset, AvmMemoryTag::FF);
        trace_builder.op_nullifier_exists(/*indirect=*/false, value_offset, metadata_offset);
    };
    auto checks = [=](bool indirect, const std::vector<Row>& trace) {
        auto row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_nullifier_exists == FF(1); });
        EXPECT_TRUE(row != trace.end());

        expect_output_table_row_with_exists_metadata(
            row,
            /*kernel_in_offset=*/output_offset,
            /*ia=*/value, // Note the value generated above for public inputs is the same as the index read + 1
            /*mem_addr_a=*/value_offset,
            /*ind_a*/ indirect,
            /*ib=*/exists,
            /*mem_addr_b=*/metadata_offset,
            /*ind_b*/ indirect,
            /*w_in_tag=*/AvmMemoryTag::FF,
            /*side_effect_counter=*/0);

        check_kernel_outputs(trace.at(output_offset), value, /*side_effect_counter=*/0, exists);
    };

    VmPublicInputs public_inputs =
        get_public_inputs_with_output(output_offset, value, /*side_effect_counter=*/0, exists);
    test_kernel_lookup(false, apply_opcodes, checks, std::move(public_inputs), execution_hints);
}

TEST_F(AvmKernelOutputPositiveTests, kernelL1ToL2MsgExists)
{
    uint32_t value_offset = 42;
    auto value = 1234;
    uint32_t metadata_offset = 420;
    auto exists = 1;
    uint32_t output_offset = START_L1_TO_L2_MSG_EXISTS_WRITE_OFFSET;

    // Create an execution hints object with the result of the operation
    auto execution_hints = ExecutionHints().with_l1_to_l2_message_exists_hints({ { 0, exists } });

    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) {
        trace_builder.op_set(0, static_cast<uint128_t>(value), value_offset, AvmMemoryTag::FF);
        trace_builder.op_l1_to_l2_msg_exists(/*indirect*/ false, value_offset, metadata_offset);
    };
    auto checks = [=]([[maybe_unused]] bool indirect, const std::vector<Row>& trace) {
        auto row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_l1_to_l2_msg_exists == FF(1); });
        EXPECT_TRUE(row != trace.end());

        expect_output_table_row_with_exists_metadata(
            row,
            /*kernel_in_offset=*/output_offset,
            /*ia=*/value, // Note the value generated above for public inputs is the same as the index read + 1
            /*mem_addr_a=*/value_offset,
            /*ind_a*/ indirect,
            /*ib=*/exists,
            /*mem_addr_b=*/metadata_offset,
            /*ind_b*/ indirect,
            /*w_in_tag=*/AvmMemoryTag::FF,
            /*side_effect_counter=*/0);

        check_kernel_outputs(trace.at(output_offset), value, /*side_effect_counter=*/0, exists);
    };

    VmPublicInputs public_inputs =
        get_public_inputs_with_output(output_offset, value, /*side_effect_counter=*/0, exists);
    test_kernel_lookup(false, apply_opcodes, checks, std::move(public_inputs), execution_hints);
}

} // namespace tests_avm
