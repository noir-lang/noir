#include "AvmMini_common.test.hpp"
namespace tests_avm {
using namespace avm_trace;

class AvmMiniMemoryTests : public ::testing::Test {
  public:
    AvmMiniTraceBuilder trace_builder;

  protected:
    // TODO(640): The Standard Honk on Grumpkin test suite fails unless the SRS is initialised for every test.
    void SetUp() override
    {
        srs::init_crs_factory("../srs_db/ignition");
        trace_builder = AvmMiniTraceBuilder(); // Clean instance for every run.
    };
};

/******************************************************************************
 *
 * MEMORY TESTS
 *
 ******************************************************************************
 * This test suite focuses on non-trivial memory-related tests which are
 * not implicitly covered by other tests such as AvmMiniArithmeticTests.
 *
 * For instance, tests on checking error conditions related to memory or
 * violation of memory-related relations in malicious/malformed execution
 * trace is the focus.
 ******************************************************************************/

// Testing an operation with a mismatched memory tag.
// The proof must pass and we check that the AVM error is raised.
TEST_F(AvmMiniMemoryTests, mismatchedTag)
{
    trace_builder.calldata_copy(0, 2, 0, std::vector<FF>{ 98, 12 });

    trace_builder.add(0, 1, 4, AvmMemoryTag::U8);
    trace_builder.halt();
    auto trace = trace_builder.finalize();

    // Find the first row enabling the addition selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_add == FF(1); });

    EXPECT_TRUE(row != trace.end());

    // All intermediate registers should be set to zero.
    EXPECT_EQ(row->avmMini_ia, FF(0));
    EXPECT_EQ(row->avmMini_ib, FF(0));
    EXPECT_EQ(row->avmMini_ic, FF(0));

    auto clk = row->avmMini_clk;

    // Find the memory trace position corresponding to the add sub-operation of register ia.
    row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
        return r.memTrace_m_clk == clk && r.memTrace_m_sub_clk == AvmMiniMemTraceBuilder::SUB_CLK_LOAD_A;
    });

    EXPECT_TRUE(row != trace.end());

    EXPECT_EQ(row->memTrace_m_tag_err, FF(1)); // Error is raised
    EXPECT_EQ(row->memTrace_m_in_tag, FF(static_cast<uint32_t>(AvmMemoryTag::U8)));
    EXPECT_EQ(row->memTrace_m_tag, FF(static_cast<uint32_t>(AvmMemoryTag::FF)));

    // Find the memory trace position corresponding to the add sub-operation of register ib.
    row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
        return r.memTrace_m_clk == clk && r.memTrace_m_sub_clk == AvmMiniMemTraceBuilder::SUB_CLK_LOAD_B;
    });

    EXPECT_TRUE(row != trace.end());

    EXPECT_EQ(row->memTrace_m_tag_err, FF(1)); // Error is raised
    EXPECT_EQ(row->memTrace_m_in_tag, FF(static_cast<uint32_t>(AvmMemoryTag::U8)));
    EXPECT_EQ(row->memTrace_m_tag, FF(static_cast<uint32_t>(AvmMemoryTag::FF)));

    validate_trace_proof(std::move(trace));
}

// Testing violation that m_lastAccess is a delimiter for two different addresses
// in the memory trace
TEST_F(AvmMiniMemoryTests, mLastAccessViolation)
{
    trace_builder.calldata_copy(0, 2, 0, std::vector<FF>{ 4, 9 });

    //                           Memory layout:     [4,9,0,0,0,0,....]
    trace_builder.sub(1, 0, 2, AvmMemoryTag::U8); // [4,9,5,0,0,0.....]
    trace_builder.halt();
    auto trace = trace_builder.finalize();

    // Find the row with subtraction operation
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_sub == FF(1); });

    EXPECT_TRUE(row != trace.end());
    auto clk = row->avmMini_clk;

    // Find the row for memory trace with last memory entry for address 1 (read for subtraction)
    row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
        return r.memTrace_m_clk == clk && r.memTrace_m_addr == FF(1) &&
               r.memTrace_m_sub_clk == AvmMiniMemTraceBuilder::SUB_CLK_LOAD_A;
    });

    EXPECT_TRUE(row != trace.end());

    row->memTrace_m_lastAccess = FF(0);

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "MEM_LAST_ACCESS_DELIMITER");
}

// Testing violation that a memory read operation must read the same value which was
// written into memory
TEST_F(AvmMiniMemoryTests, readWriteConsistencyValViolation)
{
    trace_builder.calldata_copy(0, 2, 0, std::vector<FF>{ 4, 9 });

    //                           Memory layout:      [4,9,0,0,0,0,....]
    trace_builder.mul(1, 0, 2, AvmMemoryTag::U8); // [4,9,36,0,0,0.....]
    trace_builder.return_op(2, 1);                // Return single memory word at position 2 (36)
    auto trace = trace_builder.finalize();

    // Find the row with multiplication operation
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_mul == FF(1); });

    EXPECT_TRUE(row != trace.end());
    auto clk = row->avmMini_clk + 1; // return operation is just after the multiplication

    // Find the row for memory trace with last memory entry for address 2 (read for multiplication)
    row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
        return r.memTrace_m_clk == clk && r.memTrace_m_addr == FF(2) &&
               r.memTrace_m_sub_clk == AvmMiniMemTraceBuilder::SUB_CLK_LOAD_A;
    });

    EXPECT_TRUE(row != trace.end());

    row->memTrace_m_val = FF(35);

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "MEM_READ_WRITE_VAL_CONSISTENCY");
}

// Testing violation that memory read operation must read the same tag which was
// written into memory
TEST_F(AvmMiniMemoryTests, readWriteConsistencyTagViolation)
{
    trace_builder.calldata_copy(0, 2, 0, std::vector<FF>{ 4, 9 });

    //                           Memory layout:      [4,9,0,0,0,0,....]
    trace_builder.mul(1, 0, 2, AvmMemoryTag::U8); // [4,9,36,0,0,0.....]
    trace_builder.return_op(2, 1);                // Return single memory word at position 2 (36)
    auto trace = trace_builder.finalize();

    // Find the row with multiplication operation
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_mul == FF(1); });

    EXPECT_TRUE(row != trace.end());
    auto clk = row->avmMini_clk + 1; // return operation is just after the multiplication

    // Find the row for memory trace with last memory entry for address 2 (read for multiplication)
    row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
        return r.memTrace_m_clk == clk && r.memTrace_m_addr == FF(2) &&
               r.memTrace_m_sub_clk == AvmMiniMemTraceBuilder::SUB_CLK_LOAD_A;
    });

    EXPECT_TRUE(row != trace.end());

    row->memTrace_m_tag = static_cast<uint32_t>(AvmMemoryTag::U16);

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "MEM_READ_WRITE_TAG_CONSISTENCY");
}

// Testing violation that a memory read at uninitialized location must have value 0.
TEST_F(AvmMiniMemoryTests, readUninitializedMemoryViolation)
{
    trace_builder.return_op(1, 1); // Return single memory word at position 1
    auto trace = trace_builder.finalize();

    trace[1].memTrace_m_val = 9;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "MEM_ZERO_INIT");
}

// Testing violation that an operation with a mismatched memory tag
// must raise a VM error.
TEST_F(AvmMiniMemoryTests, mismatchedTagErrorViolation)
{
    trace_builder.calldata_copy(0, 2, 0, std::vector<FF>{ 98, 12 });

    trace_builder.sub(0, 1, 4, AvmMemoryTag::U8);
    trace_builder.halt();
    auto trace = trace_builder.finalize();

    // Find the first row enabling the subtraction selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_sub == FF(1); });

    EXPECT_TRUE(row != trace.end());

    auto clk = row->avmMini_clk;

    // Find the memory trace position corresponding to the subtraction sub-operation of register ia.
    row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
        return r.memTrace_m_clk == clk && r.memTrace_m_sub_clk == AvmMiniMemTraceBuilder::SUB_CLK_LOAD_A;
    });

    row->memTrace_m_tag_err = FF(0);
    auto index = static_cast<uint32_t>(row - trace.begin());
    auto trace2 = trace;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "MEM_IN_TAG_CONSISTENCY_1");

    // More sophisticated attempt by adapting witness "on_min_inv" to make pass the above constraint
    trace2[index].memTrace_m_one_min_inv = FF(1);

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace2)), "MEM_IN_TAG_CONSISTENCY_2");
}

// Testing violation that an operation with a consistent memory tag
// must not set a VM error.
TEST_F(AvmMiniMemoryTests, consistentTagNoErrorViolation)
{
    trace_builder.calldata_copy(0, 2, 0, std::vector<FF>{ 84, 7 });

    trace_builder.div(0, 1, 4, AvmMemoryTag::FF);
    trace_builder.halt();
    auto trace = trace_builder.finalize();

    // Find the first row enabling the division selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_div == FF(1); });

    EXPECT_TRUE(row != trace.end());

    auto clk = row->avmMini_clk;

    // Find the memory trace position corresponding to the div sub-operation of register ia.
    row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
        return r.memTrace_m_clk == clk && r.memTrace_m_sub_clk == AvmMiniMemTraceBuilder::SUB_CLK_LOAD_A;
    });

    row->memTrace_m_tag_err = FF(1);

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "MEM_IN_TAG_CONSISTENCY_1");
}
} // namespace tests_avm