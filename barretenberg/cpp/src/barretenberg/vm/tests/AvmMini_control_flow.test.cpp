#include "AvmMini_common.test.hpp"

namespace tests_avm {
using namespace avm_trace;

class AvmMiniControlFlowTests : public ::testing::Test {
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
 * POSITIVE TESTS - Control Flow
 *
 *****************************************************************************/

TEST_F(AvmMiniControlFlowTests, simpleCall)
{
    uint32_t const CALL_ADDRESS = 4;

    // trace_builder for the following operation
    // pc   opcode
    // 0    INTERNAL_CALL(pc=4)
    // 4    HALT
    trace_builder.internal_call(CALL_ADDRESS);
    trace_builder.halt();

    auto trace = trace_builder.finalize();

    // Check call
    {
        auto call_row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_internal_call == FF(1); });
        EXPECT_TRUE(call_row != trace.end());
        EXPECT_EQ(call_row->avmMini_pc, FF(0));
        EXPECT_EQ(call_row->avmMini_ia, FF(CALL_ADDRESS));
        EXPECT_EQ(call_row->avmMini_internal_return_ptr, FF(AvmMiniTraceBuilder::CALLSTACK_OFFSET));
        EXPECT_EQ(call_row->avmMini_ib, FF(1));
        EXPECT_EQ(call_row->avmMini_mem_idx_b,
                  FF(AvmMiniTraceBuilder::CALLSTACK_OFFSET)); // Store the return address (0) in memory
    }

    // Check halt
    {
        auto halt_row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_halt == FF(1); });

        // Check that the correct result is stored at the expected memory location.
        EXPECT_TRUE(halt_row != trace.end());
        EXPECT_EQ(halt_row->avmMini_pc, FF(CALL_ADDRESS));
        EXPECT_EQ(halt_row->avmMini_internal_return_ptr, FF(AvmMiniTraceBuilder::CALLSTACK_OFFSET + 1));
    }
    validate_trace_proof(std::move(trace));
}

TEST_F(AvmMiniControlFlowTests, simpleJump)
{
    uint32_t const JUMP_ADDRESS = 4;

    // trace_builder for the following operation
    // pc   opcode
    // 0    JUMP(pc=4)
    // 4    HALT
    trace_builder.jump(JUMP_ADDRESS);
    trace_builder.halt();

    auto trace = trace_builder.finalize();

    // Check jump
    {
        auto call_row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_jump == FF(1); });
        EXPECT_TRUE(call_row != trace.end());
        EXPECT_EQ(call_row->avmMini_pc, FF(0));
        EXPECT_EQ(call_row->avmMini_ia, FF(JUMP_ADDRESS));
    }

    // Check halt
    {
        auto halt_row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_halt == FF(1); });

        EXPECT_TRUE(halt_row != trace.end());
        EXPECT_EQ(halt_row->avmMini_pc, FF(JUMP_ADDRESS));
    }
    validate_trace_proof(std::move(trace));
}

TEST_F(AvmMiniControlFlowTests, simpleCallAndReturn)
{
    uint32_t const CALL_ADDRESS = 20;
    uint32_t const RETURN_ADDRESS = 1;
    // trace_builder for the following operation
    // pc   opcode
    // 0    INTERNAL_CALL(pc=20)
    // 20   INTERNAL_RETURN
    // 1    HALT
    trace_builder.internal_call(CALL_ADDRESS);
    trace_builder.internal_return();
    trace_builder.halt();

    auto trace = trace_builder.finalize();

    // Check call
    {
        auto call_row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_internal_call == FF(1); });
        EXPECT_TRUE(call_row != trace.end());
        EXPECT_EQ(call_row->avmMini_pc, FF(0));
        EXPECT_EQ(call_row->avmMini_internal_return_ptr, FF(AvmMiniTraceBuilder::CALLSTACK_OFFSET));
        EXPECT_EQ(call_row->avmMini_ib, FF(1));
        EXPECT_EQ(call_row->avmMini_mem_idx_b,
                  FF(AvmMiniTraceBuilder::CALLSTACK_OFFSET)); // Store the return address (0) in memory
    }

    // Check return
    {
        auto return_row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_internal_return == FF(1); });

        // Check that the correct result is stored at the expected memory location.
        EXPECT_TRUE(return_row != trace.end());
        EXPECT_EQ(return_row->avmMini_pc, FF(CALL_ADDRESS));
        EXPECT_EQ(return_row->avmMini_internal_return_ptr, FF(AvmMiniTraceBuilder::CALLSTACK_OFFSET + 1));
    }

    // Check halt
    {
        auto halt_row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_halt == FF(1); });

        EXPECT_TRUE(halt_row != trace.end());
        EXPECT_EQ(halt_row->avmMini_pc, FF(RETURN_ADDRESS));
    }

    validate_trace_proof(std::move(trace));
}

TEST_F(AvmMiniControlFlowTests, multipleCallsAndReturns)
{
    uint32_t const CALL_ADDRESS_1 = 420;
    uint32_t const CALL_ADDRESS_2 = 69;
    uint32_t const CALL_ADDRESS_3 = 1337;
    uint32_t const CALL_ADDRESS_4 = 4;

    uint32_t const JUMP_ADDRESS_1 = 22;

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
    trace_builder.internal_call(CALL_ADDRESS_1);
    trace_builder.internal_call(CALL_ADDRESS_2);
    trace_builder.internal_call(CALL_ADDRESS_3);
    trace_builder.internal_return();
    trace_builder.internal_call(CALL_ADDRESS_4);
    trace_builder.internal_return();
    trace_builder.jump(JUMP_ADDRESS_1);
    trace_builder.internal_return();
    trace_builder.internal_return();
    trace_builder.halt();

    auto trace = trace_builder.finalize();

    // Check call 1
    {
        auto call_1 = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) {
            return r.avmMini_sel_internal_call == FF(1) && r.avmMini_ib == FF(1);
        });
        EXPECT_TRUE(call_1 != trace.end());
        EXPECT_EQ(call_1->avmMini_pc, FF(0));
        EXPECT_EQ(call_1->avmMini_ia, FF(CALL_ADDRESS_1));
        EXPECT_EQ(call_1->avmMini_internal_return_ptr, FF(AvmMiniTraceBuilder::CALLSTACK_OFFSET));
        EXPECT_EQ(call_1->avmMini_mem_idx_b,
                  FF(AvmMiniTraceBuilder::CALLSTACK_OFFSET)); // Store the return address (0) in memory
    }

    // Call 2
    {
        auto call_2 = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) {
            return r.avmMini_sel_internal_call == FF(1) && r.avmMini_pc == FF(CALL_ADDRESS_1);
        });
        EXPECT_TRUE(call_2 != trace.end());
        EXPECT_EQ(call_2->avmMini_ib, FF(CALL_ADDRESS_1 + 1));
        EXPECT_EQ(call_2->avmMini_ia, FF(CALL_ADDRESS_2));
        EXPECT_EQ(call_2->avmMini_internal_return_ptr, FF(AvmMiniTraceBuilder::CALLSTACK_OFFSET + 1));
        EXPECT_EQ(call_2->avmMini_mem_idx_b,
                  FF(AvmMiniTraceBuilder::CALLSTACK_OFFSET + 1)); // Store the return address (0) in memory
    }

    // Call 3
    {
        auto call_3 = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) {
            return r.avmMini_sel_internal_call == FF(1) && r.avmMini_pc == FF(CALL_ADDRESS_2);
        });
        EXPECT_TRUE(call_3 != trace.end());
        EXPECT_EQ(call_3->avmMini_ib, FF(CALL_ADDRESS_2 + 1));
        EXPECT_EQ(call_3->avmMini_internal_return_ptr, FF(AvmMiniTraceBuilder::CALLSTACK_OFFSET + 2));
        EXPECT_EQ(call_3->avmMini_mem_idx_b,
                  FF(AvmMiniTraceBuilder::CALLSTACK_OFFSET + 2)); // Store the return address (0) in memory
    }

    // Return 1
    {
        auto return_1 = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_internal_return == FF(1); });
        EXPECT_TRUE(return_1 != trace.end());
        EXPECT_EQ(return_1->avmMini_pc, FF(CALL_ADDRESS_3));
        EXPECT_EQ(return_1->avmMini_internal_return_ptr, FF(AvmMiniTraceBuilder::CALLSTACK_OFFSET + 3));
    }

    // Call 4
    {
        auto call_4 = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) {
            return r.avmMini_sel_internal_call == FF(1) && r.avmMini_pc == FF(CALL_ADDRESS_2 + 1);
        });
        EXPECT_TRUE(call_4 != trace.end());
        EXPECT_EQ(call_4->avmMini_ib, FF(CALL_ADDRESS_2 + 2));
        EXPECT_EQ(call_4->avmMini_internal_return_ptr, FF(AvmMiniTraceBuilder::CALLSTACK_OFFSET + 2));
        EXPECT_EQ(call_4->avmMini_mem_idx_b,
                  FF(AvmMiniTraceBuilder::CALLSTACK_OFFSET + 2)); // Store the return address (0) in memory
    }

    // Return 2
    {
        auto return_2 = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) {
            return r.avmMini_sel_internal_return == FF(1) && r.avmMini_pc == FF(CALL_ADDRESS_4);
        });
        EXPECT_TRUE(return_2 != trace.end());
        EXPECT_EQ(return_2->avmMini_ia, FF(CALL_ADDRESS_2 + 2));
        EXPECT_EQ(return_2->avmMini_internal_return_ptr, FF(AvmMiniTraceBuilder::CALLSTACK_OFFSET + 3));
    }

    // Jump 1
    {
        auto jump_1 = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) {
            return r.avmMini_sel_jump == FF(1) && r.avmMini_pc == FF(CALL_ADDRESS_2 + 2);
        });
        EXPECT_TRUE(jump_1 != trace.end());
        EXPECT_EQ(jump_1->avmMini_ia, FF(JUMP_ADDRESS_1));
        EXPECT_EQ(jump_1->avmMini_internal_return_ptr, FF(AvmMiniTraceBuilder::CALLSTACK_OFFSET + 2));
    }

    // Return 3
    {
        auto return_3 = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) {
            return r.avmMini_sel_internal_return == FF(1) && r.avmMini_pc == FF(JUMP_ADDRESS_1);
        });
        EXPECT_TRUE(return_3 != trace.end());
        EXPECT_EQ(return_3->avmMini_ia, FF(CALL_ADDRESS_1 + 1));
        EXPECT_EQ(return_3->avmMini_internal_return_ptr, FF(AvmMiniTraceBuilder::CALLSTACK_OFFSET + 2));
    }

    // Return 4
    {
        auto return_4 = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) {
            return r.avmMini_sel_internal_return == FF(1) && r.avmMini_pc == FF(CALL_ADDRESS_1 + 1);
        });
        EXPECT_TRUE(return_4 != trace.end());
        EXPECT_EQ(return_4->avmMini_ia, FF(1));
        EXPECT_EQ(return_4->avmMini_internal_return_ptr, FF(AvmMiniTraceBuilder::CALLSTACK_OFFSET + 1));
    }

    // Halt row
    auto halt_row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_halt == FF(1); });

    EXPECT_TRUE(halt_row != trace.end());
    EXPECT_EQ(halt_row->avmMini_pc, FF(1));

    validate_trace_proof(std::move(trace));
}
} // namespace tests_avm