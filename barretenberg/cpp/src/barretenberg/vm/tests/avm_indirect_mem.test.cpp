#include "avm_common.test.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"

namespace tests_avm {
using namespace bb;
using namespace bb::avm_trace;

class AvmIndirectMemTests : public ::testing::Test {
  public:
    AvmIndirectMemTests()
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
 * INDIRECT MEMORY - POSITIVE TESTS
 *
 ******************************************************************************/

// Testing an addition operation with all indirect operands.
// Indirect addresses are located at indices 0,1,2
// Direct addresses are located at indices 10,11,12
// Input values are respectively: a=100, b=101
TEST_F(AvmIndirectMemTests, allIndirectAdd)
{
    // Set direct addresses
    trace_builder.op_set(0, 10, 0, AvmMemoryTag::U32);
    trace_builder.op_set(0, 11, 1, AvmMemoryTag::U32);
    trace_builder.op_set(0, 12, 2, AvmMemoryTag::U32);

    // Set input values
    trace_builder.op_set(0, 100, 10, AvmMemoryTag::U16);
    trace_builder.op_set(0, 101, 11, AvmMemoryTag::U16);

    // All indirect flags are encoded as 7 = 1 + 2 + 4
    trace_builder.op_add(7, 0, 1, 2, AvmMemoryTag::U16);
    trace_builder.op_return(0, 0, 0);
    auto trace = trace_builder.finalize();

    // Find the first row enabling the addition selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_add == FF(1); });

    EXPECT_TRUE(row != trace.end());

    // Check all addresses and values
    EXPECT_EQ(row->main_ia, FF(100));
    EXPECT_EQ(row->main_ib, FF(101));
    EXPECT_EQ(row->main_ic, FF(201));
    EXPECT_EQ(row->main_ind_addr_a, FF(0));
    EXPECT_EQ(row->main_ind_addr_b, FF(1));
    EXPECT_EQ(row->main_ind_addr_c, FF(2));
    EXPECT_EQ(row->main_mem_addr_a, FF(10));
    EXPECT_EQ(row->main_mem_addr_b, FF(11));
    EXPECT_EQ(row->main_mem_addr_c, FF(12));

    // Check memory operation tags
    EXPECT_EQ(row->main_sel_resolve_ind_addr_a, FF(1));
    EXPECT_EQ(row->main_sel_resolve_ind_addr_b, FF(1));
    EXPECT_EQ(row->main_sel_resolve_ind_addr_c, FF(1));
    EXPECT_EQ(row->main_sel_mem_op_a, FF(1));
    EXPECT_EQ(row->main_sel_mem_op_b, FF(1));
    EXPECT_EQ(row->main_sel_mem_op_c, FF(1));

    validate_trace(std::move(trace), public_inputs, {}, {}, true);
}

// Testing a subtraction operation with direct input operands a, b, and an indirect
// output operand c.
// Indirect address is located at index 5
// Direct addresses are located at indices 50,51,52
// Input values are respectively: a=600, b=500
TEST_F(AvmIndirectMemTests, indirectOutputSub)
{
    // Set direct output address
    trace_builder.op_set(0, 52, 5, AvmMemoryTag::U32);

    // Set input values
    trace_builder.op_set(0, 600, 50, AvmMemoryTag::U128);
    trace_builder.op_set(0, 500, 51, AvmMemoryTag::U128);

    // The indirect flag is encoded as 4
    trace_builder.op_sub(4, 50, 51, 5, AvmMemoryTag::U128);
    trace_builder.op_return(0, 0, 0);
    auto trace = trace_builder.finalize();

    // Find the first row enabling the subtraction selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_sub == FF(1); });

    EXPECT_TRUE(row != trace.end());

    // Check all addresses and values
    EXPECT_EQ(row->main_ia, FF(600));
    EXPECT_EQ(row->main_ib, FF(500));
    EXPECT_EQ(row->main_ic, FF(100));
    EXPECT_EQ(row->main_ind_addr_a, FF(0));
    EXPECT_EQ(row->main_ind_addr_b, FF(0));
    EXPECT_EQ(row->main_ind_addr_c, FF(5));
    EXPECT_EQ(row->main_mem_addr_a, FF(50));
    EXPECT_EQ(row->main_mem_addr_b, FF(51));
    EXPECT_EQ(row->main_mem_addr_c, FF(52));

    // Check memory operation tags
    EXPECT_EQ(row->main_sel_resolve_ind_addr_a, FF(0));
    EXPECT_EQ(row->main_sel_resolve_ind_addr_b, FF(0));
    EXPECT_EQ(row->main_sel_resolve_ind_addr_c, FF(1));
    EXPECT_EQ(row->main_sel_mem_op_a, FF(1));
    EXPECT_EQ(row->main_sel_mem_op_b, FF(1));
    EXPECT_EQ(row->main_sel_mem_op_c, FF(1));

    validate_trace(std::move(trace), public_inputs);
}

// Testing a multiplication operation with indirect input operand a,
// and indirect input operand b and output operand c.
// Indirect address is located at index 1000
// Direct addresses are located at indices 100,101,102
// Input values are respectively: a=4, b=7
TEST_F(AvmIndirectMemTests, indirectInputAMul)
{
    // Set direct input address for a
    trace_builder.op_set(0, 100, 1000, AvmMemoryTag::U32);

    // Set input values
    trace_builder.op_set(0, 4, 100, AvmMemoryTag::U64);
    trace_builder.op_set(0, 7, 101, AvmMemoryTag::U64);

    // The indirect flag is encoded as 1
    trace_builder.op_mul(1, 1000, 101, 102, AvmMemoryTag::U64);
    trace_builder.op_return(0, 0, 0);
    auto trace = trace_builder.finalize();

    // Find the first row enabling the multiplication selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_mul == FF(1); });

    EXPECT_TRUE(row != trace.end());

    // Check all addresses and values
    EXPECT_EQ(row->main_ia, FF(4));
    EXPECT_EQ(row->main_ib, FF(7));
    EXPECT_EQ(row->main_ic, FF(28));
    EXPECT_EQ(row->main_ind_addr_a, FF(1000));
    EXPECT_EQ(row->main_ind_addr_b, FF(0));
    EXPECT_EQ(row->main_ind_addr_c, FF(0));
    EXPECT_EQ(row->main_mem_addr_a, FF(100));
    EXPECT_EQ(row->main_mem_addr_b, FF(101));
    EXPECT_EQ(row->main_mem_addr_c, FF(102));

    // Check memory operation tags
    EXPECT_EQ(row->main_sel_resolve_ind_addr_a, FF(1));
    EXPECT_EQ(row->main_sel_resolve_ind_addr_b, FF(0));
    EXPECT_EQ(row->main_sel_resolve_ind_addr_c, FF(0));
    EXPECT_EQ(row->main_sel_mem_op_a, FF(1));
    EXPECT_EQ(row->main_sel_mem_op_b, FF(1));
    EXPECT_EQ(row->main_sel_mem_op_c, FF(1));

    validate_trace(std::move(trace), public_inputs);
}

} // namespace tests_avm
