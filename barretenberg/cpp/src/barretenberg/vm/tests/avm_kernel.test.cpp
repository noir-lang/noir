
#include "avm_common.test.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/avm_trace/avm_kernel_trace.hpp"
#include "barretenberg/vm/avm_trace/constants.hpp"

namespace tests_avm {
using namespace bb::avm_trace;

class AvmKernelTests : public ::testing::Test {

  protected:
    // TODO(640): The Standard Honk on Grumpkin test suite fails unless the SRS is initialised for every test.
    void SetUp() override { srs::init_crs_factory("../srs_db/ignition"); };
};

class AvmKernelPositiveTests : public ::testing::Test {};
class AvmKernelNegativeTests : public ::testing::Test {};

using KernelInputs = std::array<FF, KERNEL_INPUTS_LENGTH>;
KernelInputs get_kernel_inputs()
{
    std::array<FF, KERNEL_INPUTS_LENGTH> kernel_inputs;
    for (size_t i = 0; i < KERNEL_INPUTS_LENGTH; i++) {
        kernel_inputs[i] = FF(i + 1);
    }
    return kernel_inputs;
}

// Template helper function to apply boilerplate around the kernel lookup tests
template <typename OpcodesFunc, typename CheckFunc>
void test_kernel_lookup(OpcodesFunc apply_opcodes, CheckFunc check_trace)
{
    KernelInputs kernel_inputs = get_kernel_inputs();
    AvmTraceBuilder trace_builder(kernel_inputs);

    // We should return a value of 1 for the sender, as it exists at index 0
    apply_opcodes(trace_builder);

    trace_builder.halt();

    auto trace = trace_builder.finalize();

    check_trace(trace);

    validate_trace(std::move(trace), kernel_inputs);
}

/*
 * Helper function to assert row values for a kernel lookup opcode
 */
void expect_row(std::vector<Row>::const_iterator row, FF selector, FF ia, FF mem_idx_a, AvmMemoryTag w_in_tag)
{
    // Checks dependent on the opcode
    EXPECT_EQ(row->avm_kernel_kernel_sel, selector);
    EXPECT_EQ(row->avm_main_ia, ia);
    EXPECT_EQ(row->avm_main_mem_idx_a, mem_idx_a);

    // Checks that are fixed for kernel inputs
    EXPECT_EQ(row->avm_main_rwa, FF(1));
    EXPECT_EQ(row->avm_main_ind_a, FF(0));
    EXPECT_EQ(row->avm_mem_op_a, FF(1));
    // TODO: below should really be a field element for each type
    EXPECT_EQ(row->avm_main_w_in_tag, static_cast<uint32_t>(w_in_tag));
    EXPECT_EQ(row->avm_main_q_kernel_lookup, FF(1));
}

TEST_F(AvmKernelPositiveTests, kernelSender)
{
    uint32_t dst_offset = 42;
    // We test that the sender opcode is included at index 0 in the public inputs
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) { trace_builder.op_sender(dst_offset); };
    auto checks = [=](const std::vector<Row>& trace) {
        std::vector<Row>::const_iterator sender_row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_sender == FF(1); });
        EXPECT_TRUE(sender_row != trace.end());

        expect_row(sender_row,
                   /*kernel_sel=*/SENDER_SELECTOR,
                   /*ia=*/SENDER_SELECTOR +
                       1, // Note the value generated above for public inputs is the same as the index read + 1
                   /*mem_idx_a=*/dst_offset,
                   /*w_in_tag=*/AvmMemoryTag::FF);
    };

    test_kernel_lookup(apply_opcodes, checks);
}

TEST_F(AvmKernelPositiveTests, kernelAddress)
{
    uint32_t dst_offset = 42;
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) { trace_builder.op_address(dst_offset); };
    auto checks = [=](const std::vector<Row>& trace) {
        std::vector<Row>::const_iterator address_row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_address == FF(1); });
        EXPECT_TRUE(address_row != trace.end());

        expect_row(address_row,
                   /*kernel_sel=*/ADDRESS_SELECTOR,
                   /*ia=*/ADDRESS_SELECTOR +
                       1, // Note the value generated above for public inputs is the same as the index read + 1
                   /*mem_idx_a=*/dst_offset,
                   /*w_in_tag=*/AvmMemoryTag::FF);
    };
    test_kernel_lookup(apply_opcodes, checks);
}

TEST_F(AvmKernelPositiveTests, kernelPortal)
{
    uint32_t dst_offset = 42;
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) { trace_builder.op_portal(dst_offset); };
    auto checks = [=](const std::vector<Row>& trace) {
        std::vector<Row>::const_iterator portal_row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_portal == FF(1); });
        EXPECT_TRUE(portal_row != trace.end());

        expect_row(portal_row,
                   /*kernel_sel=*/PORTAL_SELECTOR,
                   /*ia=*/PORTAL_SELECTOR +
                       1, // Note the value generated above for public inputs is the same as the index read + 1
                   /*mem_idx_a=*/dst_offset,
                   /*w_in_tag=*/AvmMemoryTag::FF);
    };
    test_kernel_lookup(apply_opcodes, checks);
}

TEST_F(AvmKernelPositiveTests, kernelFeePerDa)
{
    uint32_t dst_offset = 42;
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) { trace_builder.op_fee_per_da_gas(dst_offset); };
    auto checks = [=](const std::vector<Row>& trace) {
        std::vector<Row>::const_iterator fee_row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_fee_per_da_gas == FF(1); });
        EXPECT_TRUE(fee_row != trace.end());

        expect_row(fee_row,
                   /*kernel_sel=*/FEE_PER_DA_GAS_SELECTOR,
                   /*ia=*/FEE_PER_DA_GAS_SELECTOR +
                       1, // Note the value generated above for public inputs is the same as the index read + 1
                   /*mem_idx_a=*/dst_offset,
                   /*w_in_tag=*/AvmMemoryTag::FF);
    };
    test_kernel_lookup(apply_opcodes, checks);
}

TEST_F(AvmKernelPositiveTests, kernelFeePerL2)
{
    uint32_t dst_offset = 42;
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) { trace_builder.op_fee_per_l2_gas(dst_offset); };
    auto checks = [=](const std::vector<Row>& trace) {
        std::vector<Row>::const_iterator fee_row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_fee_per_l2_gas == FF(1); });
        EXPECT_TRUE(fee_row != trace.end());

        expect_row(fee_row,
                   /*kernel_sel=*/FEE_PER_L2_GAS_SELECTOR,
                   /*ia=*/FEE_PER_L2_GAS_SELECTOR +
                       1, // Note the value generated above for public inputs is the same as the index read + 1
                   /*mem_idx_a=*/dst_offset,
                   /*w_in_tag=*/AvmMemoryTag::FF);
    };
    test_kernel_lookup(apply_opcodes, checks);
}

TEST_F(AvmKernelPositiveTests, kernelTransactionFee)
{
    uint32_t dst_offset = 42;
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) { trace_builder.op_transaction_fee(dst_offset); };
    auto checks = [=](const std::vector<Row>& trace) {
        std::vector<Row>::const_iterator fee_row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_transaction_fee == FF(1); });
        EXPECT_TRUE(fee_row != trace.end());

        expect_row(fee_row,
                   /*kernel_sel=*/TRANSACTION_FEE_SELECTOR,
                   /*ia=*/TRANSACTION_FEE_SELECTOR +
                       1, // Note the value generated above for public inputs is the same as the index read + 1
                   /*mem_idx_a=*/dst_offset,
                   /*w_in_tag=*/AvmMemoryTag::FF);
    };
    test_kernel_lookup(apply_opcodes, checks);
}

TEST_F(AvmKernelPositiveTests, kernelChainId)
{
    uint32_t dst_offset = 42;
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) { trace_builder.op_chain_id(dst_offset); };
    auto checks = [=](const std::vector<Row>& trace) {
        std::vector<Row>::const_iterator fee_row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_chain_id == FF(1); });
        EXPECT_TRUE(fee_row != trace.end());

        expect_row(fee_row,
                   /*kernel_sel=*/CHAIN_ID_SELECTOR,
                   /*ia=*/CHAIN_ID_SELECTOR +
                       1, // Note the value generated above for public inputs is the same as the index read + 1
                   /*mem_idx_a=*/dst_offset,
                   /*w_in_tag=*/AvmMemoryTag::FF);
    };
    test_kernel_lookup(apply_opcodes, checks);
}

TEST_F(AvmKernelPositiveTests, kernelVersion)
{
    uint32_t dst_offset = 42;
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) { trace_builder.op_version(dst_offset); };
    auto checks = [=](const std::vector<Row>& trace) {
        std::vector<Row>::const_iterator fee_row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_version == FF(1); });
        EXPECT_TRUE(fee_row != trace.end());

        expect_row(fee_row,
                   /*kernel_sel=*/VERSION_SELECTOR,
                   /*ia=*/VERSION_SELECTOR +
                       1, // Note the value generated above for public inputs is the same as the index read + 1
                   /*mem_idx_a=*/dst_offset,
                   /*w_in_tag=*/AvmMemoryTag::FF);
    };
    test_kernel_lookup(apply_opcodes, checks);
}

TEST_F(AvmKernelPositiveTests, kernelBlockNumber)
{
    uint32_t dst_offset = 42;
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) { trace_builder.op_block_number(dst_offset); };
    auto checks = [=](const std::vector<Row>& trace) {
        std::vector<Row>::const_iterator fee_row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_block_number == FF(1); });
        EXPECT_TRUE(fee_row != trace.end());

        expect_row(fee_row,
                   /*kernel_sel=*/BLOCK_NUMBER_SELECTOR,
                   /*ia=*/BLOCK_NUMBER_SELECTOR +
                       1, // Note the value generated above for public inputs is the same as the index read + 1
                   /*mem_idx_a=*/dst_offset,
                   /*w_in_tag=*/AvmMemoryTag::FF);
    };
    test_kernel_lookup(apply_opcodes, checks);
}

TEST_F(AvmKernelPositiveTests, kernelCoinbase)
{
    uint32_t dst_offset = 42;
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) { trace_builder.op_coinbase(dst_offset); };
    auto checks = [=](const std::vector<Row>& trace) {
        std::vector<Row>::const_iterator fee_row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_coinbase == FF(1); });
        EXPECT_TRUE(fee_row != trace.end());

        expect_row(fee_row,
                   /*kernel_sel=*/COINBASE_SELECTOR,
                   /*ia=*/COINBASE_SELECTOR +
                       1, // Note the value generated above for public inputs is the same as the index read + 1
                   /*mem_idx_a*/ dst_offset,
                   /*w_in_tag=*/AvmMemoryTag::FF);
    };
    test_kernel_lookup(apply_opcodes, checks);
}

TEST_F(AvmKernelPositiveTests, kernelTimestamp)
{
    uint32_t dst_offset = 42;
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) { trace_builder.op_timestamp(dst_offset); };
    auto checks = [=](const std::vector<Row>& trace) {
        std::vector<Row>::const_iterator fee_row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_timestamp == FF(1); });
        EXPECT_TRUE(fee_row != trace.end());

        expect_row(fee_row,
                   /*kernel_sel=*/TIMESTAMP_SELECTOR,
                   /*ia=*/TIMESTAMP_SELECTOR +
                       1, // Note the value generated above for public inputs is the same as the index read + 1
                   /*mem_idx_a*/ dst_offset,
                   /*w_in_tag=*/AvmMemoryTag::U64);
    };
    test_kernel_lookup(apply_opcodes, checks);
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
    KernelInputs kernel_inputs = get_kernel_inputs();
    AvmTraceBuilder trace_builder(kernel_inputs);

    // We should return a value of 1 for the sender, as it exists at index 0
    apply_opcodes(trace_builder);

    trace_builder.halt();

    auto trace = trace_builder.finalize();

    // Change IA to be a value not in the lookup
    // Change the first row, as that will be where each of the opcodes are in the test
    auto& ta = trace.at(1);

    ta.avm_main_ia = incorrect_ia;

    check_trace(trace);

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace), kernel_inputs), expected_message);
}

TEST_F(AvmKernelNegativeTests, incorrectIaSender)
{
    uint32_t dst_offset = 42;
    FF incorrect_ia = FF(69);

    // We test that the sender opcode is inlcuded at index x in the public inputs
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) { trace_builder.op_sender(dst_offset); };
    auto checks = [=](const std::vector<Row>& trace) {
        std::vector<Row>::const_iterator sender_row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_sender == FF(1); });
        EXPECT_TRUE(sender_row != trace.end());

        expect_row(
            sender_row,
            /*kernel_sel=*/SENDER_SELECTOR,
            /*ia=*/incorrect_ia, // Note the value generated above for public inputs is the same as the index read + 1
            /*mem_idx_a=*/dst_offset,
            /*w_in_tag=*/AvmMemoryTag::FF);
    };

    negative_test_incorrect_ia_kernel_lookup(apply_opcodes, checks, incorrect_ia, "PERM_MAIN_MEM_A");
}

TEST_F(AvmKernelNegativeTests, incorrectIaAddress)
{
    uint32_t dst_offset = 42;
    FF incorrect_ia = FF(69);

    // We test that the sender opcode is inlcuded at index x in the public inputs
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) { trace_builder.op_address(dst_offset); };
    auto checks = [=](const std::vector<Row>& trace) {
        std::vector<Row>::const_iterator sender_row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_address == FF(1); });
        EXPECT_TRUE(sender_row != trace.end());

        expect_row(
            sender_row,
            /*kernel_sel=*/ADDRESS_SELECTOR,
            /*ia=*/incorrect_ia, // Note the value generated above for public inputs is the same as the index read + 1
            /*mem_idx_a=*/dst_offset,
            /*w_in_tag=*/AvmMemoryTag::FF);
    };

    negative_test_incorrect_ia_kernel_lookup(apply_opcodes, checks, incorrect_ia, "PERM_MAIN_MEM_A");
}

TEST_F(AvmKernelNegativeTests, incorrectIaPortal)
{
    uint32_t dst_offset = 42;
    FF incorrect_ia = FF(69);

    // We test that the sender opcode is inlcuded at index x in the public inputs
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) { trace_builder.op_portal(dst_offset); };
    auto checks = [=](const std::vector<Row>& trace) {
        std::vector<Row>::const_iterator sender_row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_portal == FF(1); });
        EXPECT_TRUE(sender_row != trace.end());

        expect_row(
            sender_row,
            /*kernel_sel=*/PORTAL_SELECTOR,
            /*ia=*/incorrect_ia, // Note the value generated above for public inputs is the same as the index read + 1
            /*mem_idx_a=*/dst_offset,
            /*w_in_tag=*/AvmMemoryTag::FF);
    };

    negative_test_incorrect_ia_kernel_lookup(apply_opcodes, checks, incorrect_ia, "PERM_MAIN_MEM_A");
}

TEST_F(AvmKernelNegativeTests, incorrectIaDaGas)
{
    uint32_t dst_offset = 42;
    FF incorrect_ia = FF(69);

    // We test that the sender opcode is inlcuded at index x in the public inputs
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) { trace_builder.op_fee_per_da_gas(dst_offset); };
    auto checks = [=](const std::vector<Row>& trace) {
        std::vector<Row>::const_iterator sender_row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_fee_per_da_gas == FF(1); });
        EXPECT_TRUE(sender_row != trace.end());

        expect_row(
            sender_row,
            /*kernel_sel=*/FEE_PER_DA_GAS_SELECTOR,
            /*ia=*/incorrect_ia, // Note the value generated above for public inputs is the same as the index read + 1
            /*mem_idx_a=*/dst_offset,
            /*w_in_tag=*/AvmMemoryTag::FF);
    };

    negative_test_incorrect_ia_kernel_lookup(apply_opcodes, checks, incorrect_ia, "PERM_MAIN_MEM_A");
}

TEST_F(AvmKernelNegativeTests, incorrectIal2Gas)
{
    uint32_t dst_offset = 42;
    FF incorrect_ia = FF(69);

    // We test that the sender opcode is inlcuded at index x in the public inputs
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) { trace_builder.op_fee_per_l2_gas(dst_offset); };
    auto checks = [=](const std::vector<Row>& trace) {
        std::vector<Row>::const_iterator sender_row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_fee_per_l2_gas == FF(1); });
        EXPECT_TRUE(sender_row != trace.end());

        expect_row(
            sender_row,
            /*kernel_sel=*/FEE_PER_L2_GAS_SELECTOR,
            /*ia=*/incorrect_ia, // Note the value generated above for public inputs is the same as the index read + 1
            /*mem_idx_a=*/dst_offset,
            /*w_in_tag=*/AvmMemoryTag::FF);
    };

    negative_test_incorrect_ia_kernel_lookup(apply_opcodes, checks, incorrect_ia, "PERM_MAIN_MEM_A");
}

TEST_F(AvmKernelNegativeTests, incorrectIaTransactionFee)
{
    uint32_t dst_offset = 42;
    FF incorrect_ia = FF(69);

    // We test that the sender opcode is inlcuded at index x in the public inputs
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) { trace_builder.op_transaction_fee(dst_offset); };
    auto checks = [=](const std::vector<Row>& trace) {
        std::vector<Row>::const_iterator sender_row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_transaction_fee == FF(1); });
        EXPECT_TRUE(sender_row != trace.end());

        expect_row(
            sender_row,
            /*kernel_sel=*/TRANSACTION_FEE_SELECTOR,
            /*ia=*/incorrect_ia, // Note the value generated above for public inputs is the same as the index read + 1
            /*mem_idx_a=*/dst_offset,
            /*w_in_tag=*/AvmMemoryTag::FF);
    };

    negative_test_incorrect_ia_kernel_lookup(apply_opcodes, checks, incorrect_ia, "PERM_MAIN_MEM_A");
}

TEST_F(AvmKernelNegativeTests, incorrectIaChainId)
{
    uint32_t dst_offset = 42;
    FF incorrect_ia = FF(69);

    // We test that the sender opcode is inlcuded at index x in the public inputs
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) { trace_builder.op_chain_id(dst_offset); };
    auto checks = [=](const std::vector<Row>& trace) {
        std::vector<Row>::const_iterator sender_row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_chain_id == FF(1); });
        EXPECT_TRUE(sender_row != trace.end());

        expect_row(
            sender_row,
            /*kernel_sel=*/CHAIN_ID_SELECTOR,
            /*ia=*/incorrect_ia, // Note the value generated above for public inputs is the same as the index read + 1
            /*mem_idx_a=*/dst_offset,
            /*w_in_tag=*/AvmMemoryTag::FF);
    };

    negative_test_incorrect_ia_kernel_lookup(apply_opcodes, checks, incorrect_ia, "PERM_MAIN_MEM_A");
}

TEST_F(AvmKernelNegativeTests, incorrectIaVersion)
{
    uint32_t dst_offset = 42;
    FF incorrect_ia = FF(69);

    // We test that the sender opcode is inlcuded at index x in the public inputs
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) { trace_builder.op_version(dst_offset); };
    auto checks = [=](const std::vector<Row>& trace) {
        std::vector<Row>::const_iterator sender_row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_version == FF(1); });
        EXPECT_TRUE(sender_row != trace.end());

        expect_row(
            sender_row,
            /*kernel_sel=*/VERSION_SELECTOR,
            /*ia=*/incorrect_ia, // Note the value generated above for public inputs is the same as the index read + 1
            /*mem_idx_a=*/dst_offset,
            /*w_in_tag=*/AvmMemoryTag::FF);
    };

    negative_test_incorrect_ia_kernel_lookup(apply_opcodes, checks, incorrect_ia, "PERM_MAIN_MEM_A");
}

TEST_F(AvmKernelNegativeTests, incorrectIaBlockNumber)
{
    uint32_t dst_offset = 42;
    FF incorrect_ia = FF(69);

    // We test that the sender opcode is inlcuded at index x in the public inputs
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) { trace_builder.op_block_number(dst_offset); };
    auto checks = [=](const std::vector<Row>& trace) {
        std::vector<Row>::const_iterator sender_row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_block_number == FF(1); });
        EXPECT_TRUE(sender_row != trace.end());

        expect_row(
            sender_row,
            /*kernel_sel=*/BLOCK_NUMBER_SELECTOR,
            /*ia=*/incorrect_ia, // Note the value generated above for public inputs is the same as the index read + 1
            /*mem_idx_a=*/dst_offset,
            /*w_in_tag=*/AvmMemoryTag::FF);
    };

    negative_test_incorrect_ia_kernel_lookup(apply_opcodes, checks, incorrect_ia, "PERM_MAIN_MEM_A");
}

TEST_F(AvmKernelNegativeTests, incorrectIaTimestamp)
{
    uint32_t dst_offset = 42;
    FF incorrect_ia = FF(69);

    // We test that the sender opcode is inlcuded at index x in the public inputs
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) { trace_builder.op_timestamp(dst_offset); };
    auto checks = [=](const std::vector<Row>& trace) {
        std::vector<Row>::const_iterator sender_row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_timestamp == FF(1); });
        EXPECT_TRUE(sender_row != trace.end());

        expect_row(
            sender_row,
            /*kernel_sel=*/TIMESTAMP_SELECTOR,
            /*ia=*/incorrect_ia, // Note the value generated above for public inputs is the same as the index read + 1
            /*mem_idx_a*/ dst_offset,
            /*w_in_tag=*/AvmMemoryTag::U64);
    };

    negative_test_incorrect_ia_kernel_lookup(apply_opcodes, checks, incorrect_ia, "PERM_MAIN_MEM_A");
}

TEST_F(AvmKernelNegativeTests, incorrectIaCoinbase)
{
    uint32_t dst_offset = 42;
    FF incorrect_ia = FF(69);

    // We test that the sender opcode is inlcuded at index x in the public inputs
    auto apply_opcodes = [=](AvmTraceBuilder& trace_builder) { trace_builder.op_coinbase(dst_offset); };
    auto checks = [=](const std::vector<Row>& trace) {
        std::vector<Row>::const_iterator sender_row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_coinbase == FF(1); });
        EXPECT_TRUE(sender_row != trace.end());

        expect_row(
            sender_row,
            /*kernel_sel=*/COINBASE_SELECTOR,
            /*ia=*/incorrect_ia, // Note the value generated above for public inputs is the same as the index read + 1
            /*mem_idx_a=*/dst_offset,
            /*w_in_tag=*/AvmMemoryTag::FF);
    };

    negative_test_incorrect_ia_kernel_lookup(apply_opcodes, checks, incorrect_ia, "PERM_MAIN_MEM_A");
}

} // namespace tests_avm