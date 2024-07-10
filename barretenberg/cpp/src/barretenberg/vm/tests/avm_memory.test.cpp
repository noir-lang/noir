#include "avm_common.test.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"

namespace tests_avm {

using namespace bb;
using namespace bb::avm_trace;

class AvmMemoryTests : public ::testing::Test {
  public:
    AvmMemoryTests()
        : public_inputs(generate_base_public_inputs())
        , trace_builder(AvmTraceBuilder(public_inputs))
    {
        srs::init_crs_factory("../srs_db/ignition");
    }

    VmPublicInputs public_inputs;
    AvmTraceBuilder trace_builder;
};

/******************************************************************************
 *
 * MEMORY TESTS
 *
 ******************************************************************************
 * This test suite focuses on non-trivial memory-related tests which are
 * not implicitly covered by other tests such as AvmArithmeticTests.
 *
 * For instance, tests on checking error conditions related to memory or
 * violation of memory-related relations in malicious/malformed execution
 * trace is the focus.
 ******************************************************************************/

// Testing an addition operation with a mismatched memory tag.
// The proof must pass and we check that the AVM error is raised.
TEST_F(AvmMemoryTests, mismatchedTagAddOperation)
{
    std::vector<FF> const calldata = { 98, 12 };
    trace_builder = AvmTraceBuilder(public_inputs, {}, 0, calldata);
    trace_builder.op_calldata_copy(0, 0, 2, 0);

    trace_builder.op_add(0, 0, 1, 4, AvmMemoryTag::U8);
    trace_builder.halt();
    auto trace = trace_builder.finalize();

    // Find the first row enabling the addition selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_add == FF(1); });

    EXPECT_TRUE(row != trace.end());

    EXPECT_EQ(row->main_ia, FF(98));
    EXPECT_EQ(row->main_ib, FF(12));
    EXPECT_EQ(row->main_ic, FF(0));

    auto clk = row->main_clk;

    // Find the memory trace position corresponding to the load sub-operation of register ia.
    row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
        return r.mem_tsp == FF(AvmMemTraceBuilder::NUM_SUB_CLK) * clk + AvmMemTraceBuilder::SUB_CLK_LOAD_A;
    });

    EXPECT_TRUE(row != trace.end());

    EXPECT_EQ(row->mem_tag_err, FF(1)); // Error is raised
    EXPECT_EQ(row->mem_r_in_tag, FF(static_cast<uint32_t>(AvmMemoryTag::U8)));
    EXPECT_EQ(row->mem_tag, FF(static_cast<uint32_t>(AvmMemoryTag::FF)));

    // Find the memory trace position corresponding to the add sub-operation of register ib.
    row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
        return r.mem_tsp == FF(AvmMemTraceBuilder::NUM_SUB_CLK) * clk + AvmMemTraceBuilder::SUB_CLK_LOAD_B;
    });

    EXPECT_TRUE(row != trace.end());

    EXPECT_EQ(row->mem_tag_err, FF(1)); // Error is raised
    EXPECT_EQ(row->mem_r_in_tag, FF(static_cast<uint32_t>(AvmMemoryTag::U8)));
    EXPECT_EQ(row->mem_tag, FF(static_cast<uint32_t>(AvmMemoryTag::FF)));

    validate_trace(std::move(trace), public_inputs, calldata, {}, true);
}

// Testing an equality operation with a mismatched memory tag.
// The proof must pass and we check that the AVM error is raised.
TEST_F(AvmMemoryTests, mismatchedTagEqOperation)
{
    trace_builder.op_set(0, 3, 0, AvmMemoryTag::U32);
    trace_builder.op_set(0, 5, 1, AvmMemoryTag::U16);

    trace_builder.op_eq(0, 0, 1, 2, AvmMemoryTag::U32);
    trace_builder.halt();
    auto trace = trace_builder.finalize();

    // Find the first row enabling the equality selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_eq == FF(1); });

    EXPECT_TRUE(row != trace.end());

    auto clk = row->main_clk;

    // Find the memory trace position corresponding to the load sub-operation of register ia.
    row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
        return r.mem_tsp == FF(AvmMemTraceBuilder::NUM_SUB_CLK) * clk + AvmMemTraceBuilder::SUB_CLK_LOAD_A;
    });

    EXPECT_TRUE(row != trace.end());

    EXPECT_EQ(row->mem_tag_err, FF(0)); // Error is NOT raised
    EXPECT_EQ(row->mem_r_in_tag, FF(static_cast<uint32_t>(AvmMemoryTag::U32)));
    EXPECT_EQ(row->mem_tag, FF(static_cast<uint32_t>(AvmMemoryTag::U32)));

    // Find the memory trace position corresponding to the load sub-operation of register ib.
    row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
        return r.mem_tsp == FF(AvmMemTraceBuilder::NUM_SUB_CLK) * clk + AvmMemTraceBuilder::SUB_CLK_LOAD_B;
    });

    EXPECT_TRUE(row != trace.end());

    EXPECT_EQ(row->mem_tag_err, FF(1)); // Error is raised
    EXPECT_EQ(row->mem_r_in_tag, FF(static_cast<uint32_t>(AvmMemoryTag::U32)));
    EXPECT_EQ(row->mem_tag, FF(static_cast<uint32_t>(AvmMemoryTag::U16)));

    validate_trace(std::move(trace), public_inputs);
}

// Testing violation that m_lastAccess is a delimiter for two different addresses
// in the memory trace
TEST_F(AvmMemoryTests, mLastAccessViolation)
{
    trace_builder.op_set(0, 4, 0, AvmMemoryTag::U8);
    trace_builder.op_set(0, 9, 1, AvmMemoryTag::U8);

    //                           Memory layout:     [4,9,0,0,0,0,....]
    trace_builder.op_sub(0, 1, 0, 2, AvmMemoryTag::U8); // [4,9,5,0,0,0.....]
    trace_builder.halt();
    auto trace = trace_builder.finalize();

    // Find the row with subtraction operation
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_sub == FF(1); });

    EXPECT_TRUE(row != trace.end());
    auto clk = row->main_clk;

    // Find the row for memory trace with last memory entry for address 1 (read for subtraction)
    row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
        return r.mem_addr == FF(1) &&
               r.mem_tsp == FF(AvmMemTraceBuilder::NUM_SUB_CLK) * clk + AvmMemTraceBuilder::SUB_CLK_LOAD_A;
    });

    EXPECT_TRUE(row != trace.end());

    row->mem_lastAccess = FF(0);

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "MEM_LAST_ACCESS_DELIMITER");
}

// Testing violation that a memory read operation must read the same value which was
// written into memory
TEST_F(AvmMemoryTests, readWriteConsistencyValViolation)
{
    trace_builder.op_set(0, 4, 0, AvmMemoryTag::U8);
    trace_builder.op_set(0, 9, 1, AvmMemoryTag::U8);

    //                           Memory layout:      [4,9,0,0,0,0,....]
    trace_builder.op_mul(0, 1, 0, 2, AvmMemoryTag::U8); // [4,9,36,0,0,0.....]
    trace_builder.op_return(0, 2, 1);                   // Return single memory word at position 2 (36)
    auto trace = trace_builder.finalize();

    // Find the row with multiplication operation
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_mul == FF(1); });

    EXPECT_TRUE(row != trace.end());
    auto clk = row->main_clk + 1; // return operation is just after the multiplication

    // Find the row for memory trace with last memory entry for address 2 (read for multiplication)
    row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
        return r.mem_addr == FF(2) &&
               r.mem_tsp == FF(AvmMemTraceBuilder::NUM_SUB_CLK) * clk + AvmMemTraceBuilder::SUB_CLK_LOAD_A;
    });

    EXPECT_TRUE(row != trace.end());

    row->mem_val = FF(35);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "MEM_READ_WRITE_VAL_CONSISTENCY");
}

// Testing violation that memory read operation must read the same tag which was
// written into memory
TEST_F(AvmMemoryTests, readWriteConsistencyTagViolation)
{
    trace_builder.op_set(0, 4, 0, AvmMemoryTag::U8);
    trace_builder.op_set(0, 9, 1, AvmMemoryTag::U8);

    //                           Memory layout:      [4,9,0,0,0,0,....]
    trace_builder.op_mul(0, 1, 0, 2, AvmMemoryTag::U8); // [4,9,36,0,0,0.....]
    trace_builder.op_return(0, 2, 1);                   // Return single memory word at position 2 (36)
    auto trace = trace_builder.finalize();

    // Find the row with multiplication operation
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_mul == FF(1); });

    EXPECT_TRUE(row != trace.end());
    auto clk = row->main_clk + 1; // return operation is just after the multiplication

    // Find the row for memory trace with last memory entry for address 2 (read for multiplication)
    row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
        return r.mem_addr == FF(2) &&
               r.mem_tsp == FF(AvmMemTraceBuilder::NUM_SUB_CLK) * clk + AvmMemTraceBuilder::SUB_CLK_LOAD_A;
    });

    EXPECT_TRUE(row != trace.end());

    row->mem_tag = static_cast<uint32_t>(AvmMemoryTag::U16);

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "MEM_READ_WRITE_TAG_CONSISTENCY");
}

// Testing violation that a memory read at uninitialized location must have value 0.
TEST_F(AvmMemoryTests, readUninitializedMemoryViolation)
{
    trace_builder.op_return(0, 1, 1); // Return single memory word at position 1
    auto trace = trace_builder.finalize();

    trace[1].mem_val = 9;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "MEM_ZERO_INIT");
}

// Testing violation that an operation with a mismatched memory tag
// must raise a VM error.
TEST_F(AvmMemoryTests, mismatchedTagErrorViolation)
{
    trace_builder = AvmTraceBuilder(public_inputs, {}, 0, { 98, 12 });
    trace_builder.op_calldata_copy(0, 0, 2, 0);

    trace_builder.op_sub(0, 0, 1, 4, AvmMemoryTag::U8);
    trace_builder.halt();
    auto trace = trace_builder.finalize();

    // Find the first row enabling the subtraction selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_sub == FF(1); });

    EXPECT_TRUE(row != trace.end());

    auto clk = row->main_clk;

    // Find the memory trace position corresponding to the subtraction sub-operation of register ia.
    row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
        return r.mem_tsp == FF(AvmMemTraceBuilder::NUM_SUB_CLK) * clk + AvmMemTraceBuilder::SUB_CLK_LOAD_A;
    });

    row->mem_tag_err = FF(0);
    auto index = static_cast<uint32_t>(row - trace.begin());
    auto trace2 = trace;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "MEM_IN_TAG_CONSISTENCY_1");

    // More sophisticated attempt by adapting witness "on_min_inv" to make pass the above constraint
    trace2[index].mem_one_min_inv = FF(1);

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace2)), "MEM_IN_TAG_CONSISTENCY_2");
}

// Testing violation that an operation with a consistent memory tag
// must not set a VM error.
TEST_F(AvmMemoryTests, consistentTagNoErrorViolation)
{
    trace_builder = AvmTraceBuilder(public_inputs, {}, 0, std::vector<FF>{ 84, 7 });
    trace_builder.op_calldata_copy(0, 0, 2, 0);
    trace_builder.op_fdiv(0, 0, 1, 4);
    trace_builder.halt();
    auto trace = trace_builder.finalize();

    // Find the first row enabling the fdiv selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_fdiv == FF(1); });

    EXPECT_TRUE(row != trace.end());

    auto clk = row->main_clk;

    // Find the memory trace position corresponding to the fdiv sub-operation of register ia.
    row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
        return r.mem_tsp == FF(AvmMemTraceBuilder::NUM_SUB_CLK) * clk + AvmMemTraceBuilder::SUB_CLK_LOAD_A;
    });

    row->mem_tag_err = FF(1);

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "MEM_IN_TAG_CONSISTENCY_1");
}

// Testing violation that a write operation must not set a VM error.
TEST_F(AvmMemoryTests, noErrorTagWriteViolation)
{
    trace_builder = AvmTraceBuilder(public_inputs, {}, 0, { 84, 7 });
    trace_builder.op_calldata_copy(0, 0, 2, 0);
    trace_builder.op_fdiv(0, 0, 1, 4);
    trace_builder.halt();
    auto trace = trace_builder.finalize();

    // Find the first row enabling the fdiv selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_fdiv == FF(1); });

    ASSERT_TRUE(row != trace.end());

    auto clk = row->main_clk;

    // Find the memory trace position corresponding to the fdiv sub-operation of register ic.
    row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
        return r.mem_tsp == FF(AvmMemTraceBuilder::NUM_SUB_CLK) * clk + AvmMemTraceBuilder::SUB_CLK_STORE_C;
    });

    ASSERT_TRUE(row != trace.end());
    row->mem_tag_err = FF(1);

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "NO_TAG_ERR_WRITE");
}

} // namespace tests_avm
