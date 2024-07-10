#include "avm_common.test.hpp"

namespace tests_avm {

using namespace bb;
using namespace bb::avm_trace;

namespace {

void validate_internal_call(Row const& row, uint32_t current_pc, uint32_t target_pc, uint32_t stack_ptr)
{
    EXPECT_EQ(row.main_sel_op_internal_call, FF(1));
    EXPECT_EQ(row.main_pc, FF(current_pc));
    EXPECT_EQ(row.main_ia, FF(target_pc));
    EXPECT_EQ(row.main_internal_return_ptr, FF(stack_ptr));
    EXPECT_EQ(row.main_sel_mem_op_b, FF(1));
    EXPECT_EQ(row.main_rwb, FF(1));
    EXPECT_EQ(row.main_ib, FF(current_pc + 1));
    EXPECT_EQ(row.main_mem_addr_b, FF(stack_ptr));
    EXPECT_EQ(row.main_w_in_tag, FF(static_cast<uint32_t>(AvmMemoryTag::U32)));
    EXPECT_EQ(row.main_space_id, FF(INTERNAL_CALL_SPACE_ID));
};

void validate_internal_return(Row const& row, uint32_t current_pc, uint32_t return_pc, uint32_t stack_ptr)
{
    EXPECT_EQ(row.main_sel_op_internal_return, FF(1));
    EXPECT_EQ(row.main_pc, FF(current_pc));
    EXPECT_EQ(row.main_ia, FF(return_pc));
    EXPECT_EQ(row.main_internal_return_ptr, FF(stack_ptr));
    EXPECT_EQ(row.main_sel_mem_op_a, FF(1));
    EXPECT_EQ(row.main_rwa, FF(0));
    EXPECT_EQ(row.main_mem_addr_a, FF(stack_ptr - 1));
    EXPECT_EQ(row.main_r_in_tag, FF(static_cast<uint32_t>(AvmMemoryTag::U32)));
    EXPECT_EQ(row.main_space_id, FF(INTERNAL_CALL_SPACE_ID));
};

} // namespace

class AvmControlFlowTests : public ::testing::Test {
  public:
    AvmControlFlowTests()
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
 * POSITIVE TESTS - Control Flow
 *
 *****************************************************************************/

TEST_F(AvmControlFlowTests, simpleCall)
{
    uint32_t const CALL_PC = 4;

    // trace_builder for the following operation
    // pc   opcode
    // 0    INTERNAL_CALL(pc=4)
    // 4    HALT
    trace_builder.op_internal_call(CALL_PC);
    trace_builder.halt();

    auto trace = trace_builder.finalize();

    // Check call
    {
        auto call_row_iter = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_internal_call == FF(1); });
        EXPECT_TRUE(call_row_iter != trace.end());
        auto& call_row = trace.at(static_cast<size_t>(call_row_iter - trace.begin()));
        validate_internal_call(call_row, 0, CALL_PC, 0);
    }

    // Check halt
    {
        auto halt_row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_halt == FF(1); });

        // Check that the correct result is stored at the expected memory location.
        EXPECT_TRUE(halt_row != trace.end());
        EXPECT_EQ(halt_row->main_pc, FF(CALL_PC));
        EXPECT_EQ(halt_row->main_internal_return_ptr, FF(1));
    }
    validate_trace(std::move(trace), public_inputs, {}, {}, true);
}

TEST_F(AvmControlFlowTests, simpleJump)
{
    uint32_t const JUMP_PC = 4;

    // trace_builder for the following operation
    // pc   opcode
    // 0    JUMP(pc=4)
    // 4    HALT
    trace_builder.op_jump(JUMP_PC);
    trace_builder.halt();

    auto trace = trace_builder.finalize();

    // Check jump
    {
        auto call_row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_jump == FF(1); });
        EXPECT_TRUE(call_row != trace.end());
        EXPECT_EQ(call_row->main_pc, FF(0));
        EXPECT_EQ(call_row->main_ia, FF(JUMP_PC));
    }

    // Check halt
    {
        auto halt_row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_halt == FF(1); });

        EXPECT_TRUE(halt_row != trace.end());
        EXPECT_EQ(halt_row->main_pc, FF(JUMP_PC));
    }
    validate_trace(std::move(trace), public_inputs);
}

TEST_F(AvmControlFlowTests, simpleCallAndReturn)
{
    uint32_t const CALL_PC = 20;
    uint32_t const RETURN_PC = 1;
    // trace_builder for the following operation
    // pc   opcode
    // 0    INTERNAL_CALL(pc=20)
    // 20   INTERNAL_RETURN
    // 1    HALT
    trace_builder.op_internal_call(CALL_PC);
    trace_builder.op_internal_return();
    trace_builder.halt();

    auto trace = trace_builder.finalize();

    // Check call
    {
        auto call_row_iter = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_internal_call == FF(1); });
        EXPECT_TRUE(call_row_iter != trace.end());
        auto& call_row = trace.at(static_cast<size_t>(call_row_iter - trace.begin()));
        validate_internal_call(call_row, 0, CALL_PC, 0);
    }

    // Check return
    {
        auto return_row_iter = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_internal_return == FF(1); });

        // Check that the correct result is stored at the expected memory location.
        EXPECT_TRUE(return_row_iter != trace.end());
        auto& return_row = trace.at(static_cast<size_t>(return_row_iter - trace.begin()));
        validate_internal_return(return_row, CALL_PC, RETURN_PC, 1);
    }

    // Check halt
    {
        auto halt_row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_halt == FF(1); });

        EXPECT_TRUE(halt_row != trace.end());
        EXPECT_EQ(halt_row->main_pc, FF(RETURN_PC));
    }

    validate_trace(std::move(trace), public_inputs);
}

TEST_F(AvmControlFlowTests, multipleCallsAndReturns)
{
    uint32_t const CALL_PC_1 = 420;
    uint32_t const CALL_PC_2 = 69;
    uint32_t const CALL_PC_3 = 1337;
    uint32_t const CALL_PC_4 = 4;

    uint32_t const JUMP_PC_1 = 22;

    // trace_builder for the following operation
    // pc    opcode
    // 0     INTERNAL_CALL(pc=420)
    // 420   INTERNAL_CALL(pc=69)
    // 69    INTERNAL_CALL(pc=1337)
    // 1337  INTERNAL_RETURN
    // 70    INTERNAL_CALL(pc=4)
    // 4     INTERNAL_RETURN
    // 71    JUMP(pc=22)
    // 22    INTERNAL_RETURN
    // 421   INTERNAL_RETURN
    // 1     HALT
    trace_builder.op_internal_call(CALL_PC_1);
    trace_builder.op_internal_call(CALL_PC_2);
    trace_builder.op_internal_call(CALL_PC_3);
    trace_builder.op_internal_return();
    trace_builder.op_internal_call(CALL_PC_4);
    trace_builder.op_internal_return();
    trace_builder.op_jump(JUMP_PC_1);
    trace_builder.op_internal_return();
    trace_builder.op_internal_return();
    trace_builder.halt();

    auto trace = trace_builder.finalize();

    // Check call 1
    {
        auto call_1 = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) {
            return r.main_sel_op_internal_call == FF(1) && r.main_ib == FF(1);
        });
        EXPECT_TRUE(call_1 != trace.end());
        auto& call_1_row = trace.at(static_cast<size_t>(call_1 - trace.begin()));
        validate_internal_call(call_1_row, 0, CALL_PC_1, 0);
    }

    // Call 2
    {
        auto call_2 = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) {
            return r.main_sel_op_internal_call == FF(1) && r.main_pc == FF(CALL_PC_1);
        });
        EXPECT_TRUE(call_2 != trace.end());
        auto& call_2_row = trace.at(static_cast<size_t>(call_2 - trace.begin()));
        validate_internal_call(call_2_row, CALL_PC_1, CALL_PC_2, 1);
    }

    // Call 3
    {
        auto call_3 = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) {
            return r.main_sel_op_internal_call == FF(1) && r.main_pc == FF(CALL_PC_2);
        });
        EXPECT_TRUE(call_3 != trace.end());
        auto& call_3_row = trace.at(static_cast<size_t>(call_3 - trace.begin()));
        validate_internal_call(call_3_row, CALL_PC_2, CALL_PC_3, 2);
    }

    // Return 1
    {
        auto return_1 = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_internal_return == FF(1); });
        EXPECT_TRUE(return_1 != trace.end());
        auto& return_1_row = trace.at(static_cast<size_t>(return_1 - trace.begin()));
        validate_internal_return(return_1_row, CALL_PC_3, CALL_PC_2 + 1, 3);
    }

    // Call 4
    {
        auto call_4 = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) {
            return r.main_sel_op_internal_call == FF(1) && r.main_pc == FF(CALL_PC_2 + 1);
        });
        EXPECT_TRUE(call_4 != trace.end());
        auto& call_4_row = trace.at(static_cast<size_t>(call_4 - trace.begin()));
        validate_internal_call(call_4_row, CALL_PC_2 + 1, CALL_PC_4, 2);
    }

    // Return 2
    {
        auto return_2 = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) {
            return r.main_sel_op_internal_return == FF(1) && r.main_pc == FF(CALL_PC_4);
        });
        EXPECT_TRUE(return_2 != trace.end());
        auto& return_2_row = trace.at(static_cast<size_t>(return_2 - trace.begin()));
        validate_internal_return(return_2_row, CALL_PC_4, CALL_PC_2 + 2, 3);
    }

    // Jump 1
    {
        auto jump_1 = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) {
            return r.main_sel_op_jump == FF(1) && r.main_pc == FF(CALL_PC_2 + 2);
        });
        EXPECT_TRUE(jump_1 != trace.end());
        EXPECT_EQ(jump_1->main_ia, FF(JUMP_PC_1));
        EXPECT_EQ(jump_1->main_internal_return_ptr, FF(2));
    }

    // Return 3
    {
        auto return_3 = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) {
            return r.main_sel_op_internal_return == FF(1) && r.main_pc == FF(JUMP_PC_1);
        });
        EXPECT_TRUE(return_3 != trace.end());
        auto& return_3_row = trace.at(static_cast<size_t>(return_3 - trace.begin()));
        validate_internal_return(return_3_row, JUMP_PC_1, CALL_PC_1 + 1, 2);
    }

    // Return 4
    {
        auto return_4 = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) {
            return r.main_sel_op_internal_return == FF(1) && r.main_pc == FF(CALL_PC_1 + 1);
        });
        EXPECT_TRUE(return_4 != trace.end());
        auto& return_4_row = trace.at(static_cast<size_t>(return_4 - trace.begin()));
        validate_internal_return(return_4_row, CALL_PC_1 + 1, 1, 1);
    }

    // Halt row
    auto halt_row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_halt == FF(1); });

    EXPECT_TRUE(halt_row != trace.end());
    EXPECT_EQ(halt_row->main_pc, FF(1));

    validate_trace(std::move(trace), public_inputs);
}

} // namespace tests_avm