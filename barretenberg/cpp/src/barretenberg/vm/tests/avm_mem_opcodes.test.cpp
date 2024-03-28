#include "avm_common.test.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/avm_trace/avm_mem_trace.hpp"
#include "barretenberg/vm/tests/helpers.test.hpp"
#include <cstddef>
#include <cstdint>
#include <gmock/gmock.h>
#include <gtest/gtest.h>

namespace tests_avm {
using namespace bb::avm_trace;
using namespace testing;

class AvmMemOpcodeTests : public ::testing::Test {
  public:
    AvmTraceBuilder trace_builder;

  protected:
    std::vector<Row> trace;
    size_t main_idx;
    size_t mem_a_idx;
    size_t mem_c_idx;
    size_t mem_ind_a_idx;
    size_t mem_ind_c_idx;

    // TODO(640): The Standard Honk on Grumpkin test suite fails unless the SRS is initialised for every test.
    void SetUp() override { srs::init_crs_factory("../srs_db/ignition"); };
    void buildTrace(bool indirect,
                    uint128_t const& val,
                    uint32_t src_offset,
                    uint32_t dst_offset,
                    AvmMemoryTag tag,
                    uint32_t dir_src_offset = 0,
                    uint32_t dir_dst_offset = 0)
    {
        if (indirect) {
            trace_builder.set(dir_src_offset, src_offset, AvmMemoryTag::U32);
            trace_builder.set(dir_dst_offset, dst_offset, AvmMemoryTag::U32);
            trace_builder.set(val, dir_src_offset, tag);
        } else {
            trace_builder.set(val, src_offset, tag);
        }

        trace_builder.op_mov(indirect ? 3 : 0, src_offset, dst_offset);
        trace_builder.return_op(0, 0, 0);
        trace = trace_builder.finalize();
    }

    void computeIndices(bool indirect)
    {
        // Find the first row enabling the MOV selector
        auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_mov == FF(1); });
        ASSERT_TRUE(row != trace.end());
        main_idx = static_cast<size_t>(row - trace.begin());

        auto clk = row->avm_main_clk;

        auto gen_matcher = [clk](uint32_t sub_clk) {
            return [clk, sub_clk](Row r) { return r.avm_mem_clk == clk && r.avm_mem_sub_clk == sub_clk; };
        };

        // Find the memory trace position corresponding to the load sub-operation of register ia.
        row = std::ranges::find_if(trace.begin(), trace.end(), gen_matcher(AvmMemTraceBuilder::SUB_CLK_LOAD_A));
        ASSERT_TRUE(row != trace.end());
        mem_a_idx = static_cast<size_t>(row - trace.begin());

        // Find the memory trace position corresponding to the write sub-operation of register ic.
        row = std::ranges::find_if(trace.begin(), trace.end(), gen_matcher(AvmMemTraceBuilder::SUB_CLK_STORE_C));
        ASSERT_TRUE(row != trace.end());
        mem_c_idx = static_cast<size_t>(row - trace.begin());

        // Find the memory trace position of the indirect loads.
        if (indirect) {
            row = std::ranges::find_if(trace.begin(), trace.end(), gen_matcher(AvmMemTraceBuilder::SUB_CLK_IND_LOAD_A));
            ASSERT_TRUE(row != trace.end());
            mem_ind_a_idx = static_cast<size_t>(row - trace.begin());

            row = std::ranges::find_if(trace.begin(), trace.end(), gen_matcher(AvmMemTraceBuilder::SUB_CLK_IND_LOAD_C));
            ASSERT_TRUE(row != trace.end());
            mem_ind_c_idx = static_cast<size_t>(row - trace.begin());
        }
    }

    void validate_trace(bool indirect,
                        uint128_t const& val,
                        uint32_t src_offset,
                        uint32_t dst_offset,
                        AvmMemoryTag tag,
                        uint32_t dir_src_offset = 0,
                        uint32_t dir_dst_offset = 0)
    {
        computeIndices(indirect);
        FF const val_ff = uint256_t::from_uint128(val);
        auto const& main_row = trace.at(main_idx);

        if (indirect) {
            EXPECT_THAT(main_row,
                        AllOf(Field(&Row::avm_main_mem_idx_a, dir_src_offset),
                              Field(&Row::avm_main_mem_idx_c, dir_dst_offset)));
        }
        EXPECT_THAT(main_row,
                    AllOf(Field(&Row::avm_main_ia, val_ff),
                          Field(&Row::avm_main_ib, 0),
                          Field(&Row::avm_main_ic, val_ff),
                          Field(&Row::avm_main_r_in_tag, static_cast<uint32_t>(tag)),
                          Field(&Row::avm_main_w_in_tag, static_cast<uint32_t>(tag))));

        auto const& mem_a_row = trace.at(mem_a_idx);

        EXPECT_THAT(mem_a_row,
                    AllOf(Field(&Row::avm_mem_tag_err, 0),
                          Field(&Row::avm_mem_r_in_tag, static_cast<uint32_t>(tag)),
                          Field(&Row::avm_mem_tag, static_cast<uint32_t>(tag)),
                          Field(&Row::avm_mem_sel_mov, 1),
                          Field(&Row::avm_mem_addr, indirect ? dir_src_offset : src_offset),
                          Field(&Row::avm_mem_val, val_ff),
                          Field(&Row::avm_mem_rw, 0),
                          Field(&Row::avm_mem_op_a, 1)));

        auto const& mem_c_row = trace.at(mem_c_idx);

        EXPECT_THAT(mem_c_row,
                    AllOf(Field(&Row::avm_mem_tag_err, 0),
                          Field(&Row::avm_mem_w_in_tag, static_cast<uint32_t>(tag)),
                          Field(&Row::avm_mem_tag, static_cast<uint32_t>(tag)),
                          Field(&Row::avm_mem_addr, indirect ? dir_dst_offset : dst_offset),
                          Field(&Row::avm_mem_val, val_ff),
                          Field(&Row::avm_mem_op_c, 1)));

        if (indirect) {
            auto const& mem_ind_a_row = trace.at(mem_ind_a_idx);
            EXPECT_THAT(mem_ind_a_row,
                        AllOf(Field(&Row::avm_mem_tag_err, 0),
                              Field(&Row::avm_mem_r_in_tag, static_cast<uint32_t>(AvmMemoryTag::U32)),
                              Field(&Row::avm_mem_tag, static_cast<uint32_t>(AvmMemoryTag::U32)),
                              Field(&Row::avm_mem_addr, src_offset),
                              Field(&Row::avm_mem_val, dir_src_offset),
                              Field(&Row::avm_mem_ind_op_a, 1)));

            auto const& mem_ind_c_row = trace.at(mem_ind_c_idx);
            EXPECT_THAT(mem_ind_c_row,
                        AllOf(Field(&Row::avm_mem_tag_err, 0),
                              Field(&Row::avm_mem_r_in_tag, static_cast<uint32_t>(AvmMemoryTag::U32)),
                              Field(&Row::avm_mem_tag, static_cast<uint32_t>(AvmMemoryTag::U32)),
                              Field(&Row::avm_mem_addr, dst_offset),
                              Field(&Row::avm_mem_val, dir_dst_offset),
                              Field(&Row::avm_mem_ind_op_c, 1)));
        }

        validate_trace_proof(std::move(trace));
    }
};

class AvmMemOpcodeNegativeTests : public AvmMemOpcodeTests {};

/******************************************************************************
 *
 * MEMORY OPCODE POSITIVE TESTS
 *
 ******************************************************************************/

TEST_F(AvmMemOpcodeTests, basicMov)
{
    buildTrace(false, 42, 9, 13, AvmMemoryTag::U64);
    validate_trace(false, 42, 9, 13, AvmMemoryTag::U64);
}

TEST_F(AvmMemOpcodeTests, sameAddressMov)
{
    buildTrace(false, 11, 356, 356, AvmMemoryTag::U16);
    validate_trace(false, 11, 356, 356, AvmMemoryTag::U16);
}

TEST_F(AvmMemOpcodeTests, uninitializedValueMov)
{
    auto trace_builder = AvmTraceBuilder();
    trace_builder.set(4, 1, AvmMemoryTag::U32);
    trace_builder.op_mov(0, 0, 1);
    trace_builder.return_op(0, 0, 0);
    trace = trace_builder.finalize();

    validate_trace(false, 0, 0, 1, AvmMemoryTag::U0);
}

TEST_F(AvmMemOpcodeTests, indUninitializedValueMov)
{
    auto trace_builder = AvmTraceBuilder();
    trace_builder.set(1, 3, AvmMemoryTag::U32);
    trace_builder.set(4, 1, AvmMemoryTag::U32);
    trace_builder.op_mov(3, 2, 3);
    trace_builder.return_op(0, 0, 0);
    trace = trace_builder.finalize();

    validate_trace(true, 0, 2, 3, AvmMemoryTag::U0, 0, 1);
}

TEST_F(AvmMemOpcodeTests, indirectMov)
{
    buildTrace(true, 23, 0, 1, AvmMemoryTag::U8, 2, 3);
    validate_trace(true, 23, 0, 1, AvmMemoryTag::U8, 2, 3);
}

TEST_F(AvmMemOpcodeTests, indirectMovInvalidAddressTag)
{
    trace_builder.set(15, 100, AvmMemoryTag::U32);
    trace_builder.set(16, 101, AvmMemoryTag::U128); // This will make the indirect load failing.
    trace_builder.set(5, 15, AvmMemoryTag::FF);
    trace_builder.op_mov(3, 100, 101);
    trace_builder.return_op(0, 0, 0);
    trace = trace_builder.finalize();

    computeIndices(true);

    EXPECT_EQ(trace.at(main_idx).avm_main_tag_err, 1);
    EXPECT_THAT(trace.at(mem_ind_c_idx),
                AllOf(Field(&Row::avm_mem_tag_err, 1),
                      Field(&Row::avm_mem_tag, static_cast<uint32_t>(AvmMemoryTag::U128)),
                      Field(&Row::avm_mem_r_in_tag, static_cast<uint32_t>(AvmMemoryTag::U32)),
                      Field(&Row::avm_mem_ind_op_c, 1)));

    validate_trace_proof(std::move(trace));
}

/******************************************************************************
 *
 * MEMORY OPCODE NEGATIVE TESTS
 *
 ******************************************************************************/

TEST_F(AvmMemOpcodeNegativeTests, movWrongOutputErrorTag)
{
    buildTrace(false, 234, 0, 1, AvmMemoryTag::U8);
    computeIndices(false);
    trace.at(main_idx).avm_main_tag_err = 1;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "INCL_MEM_TAG_ERR");
}

TEST_F(AvmMemOpcodeNegativeTests, movWrongOutputValue)
{
    buildTrace(false, 234, 0, 1, AvmMemoryTag::U8);
    computeIndices(false);
    trace.at(main_idx).avm_main_ic = 233;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "MOV_SAME_VALUE");
}

TEST_F(AvmMemOpcodeNegativeTests, indMovWrongOutputValue)
{
    buildTrace(true, 8732, 23, 24, AvmMemoryTag::U16, 432, 876);
    computeIndices(true);
    trace.at(main_idx).avm_main_ic = 8733;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "MOV_SAME_VALUE");
}

// We want to test that the output tag for MOV cannot be altered.
// In this test, we modify the r_in_tag for load operation to Ia.
// Then, we propagate the error tag and the copy of r_in_tag to the
// main trace and the memory entry related to store operation from Ic.
TEST_F(AvmMemOpcodeNegativeTests, movWrongOutputTagLoadIa)
{
    FF const tag_u64 = FF(static_cast<uint32_t>(AvmMemoryTag::U64));
    FF const tag_u8 = FF(static_cast<uint32_t>(AvmMemoryTag::U8));
    FF const one_min_inverse_diff = FF(1) - (tag_u64 - tag_u8).invert();

    buildTrace(false, 234, 0, 1, AvmMemoryTag::U8);
    computeIndices(false);

    auto trace_tmp = trace;

    trace.at(mem_a_idx).avm_mem_r_in_tag = tag_u64;
    trace.at(mem_a_idx).avm_mem_tag_err = 1;
    trace.at(mem_a_idx).avm_mem_one_min_inv = one_min_inverse_diff;
    trace.at(mem_c_idx).avm_mem_tag = tag_u64;
    trace.at(mem_c_idx).avm_mem_w_in_tag = tag_u64;
    trace.at(main_idx).avm_main_r_in_tag = tag_u64;
    trace.at(main_idx).avm_main_w_in_tag = tag_u64;
    trace.at(main_idx).avm_main_tag_err = 1;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "MOV_SAME_TAG");
}

// Same as above but one tries to disable the selector of MOV opcode in
// the load operation.
TEST_F(AvmMemOpcodeNegativeTests, movWrongOutputTagDisabledSelector)
{
    FF const tag_u64 = FF(static_cast<uint32_t>(AvmMemoryTag::U64));
    FF const tag_u8 = FF(static_cast<uint32_t>(AvmMemoryTag::U8));
    FF const one_min_inverse_diff = FF(1) - (tag_u64 - tag_u8).invert();

    buildTrace(false, 234, 0, 1, AvmMemoryTag::U8);
    computeIndices(false);

    trace.at(mem_a_idx).avm_mem_r_in_tag = tag_u64;
    trace.at(mem_a_idx).avm_mem_w_in_tag = tag_u64;
    trace.at(mem_a_idx).avm_mem_tag_err = 1;
    trace.at(mem_a_idx).avm_mem_one_min_inv = one_min_inverse_diff;
    trace.at(mem_a_idx).avm_mem_sel_mov = 0;
    trace.at(mem_c_idx).avm_mem_tag = tag_u64;
    trace.at(mem_c_idx).avm_mem_r_in_tag = tag_u64;
    trace.at(mem_c_idx).avm_mem_w_in_tag = tag_u64;
    trace.at(main_idx).avm_main_r_in_tag = tag_u64;
    trace.at(main_idx).avm_main_w_in_tag = tag_u64;
    trace.at(main_idx).avm_main_tag_err = 1;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "PERM_MAIN_MEM_A");
}

// Same goal as above but we alter the w_in_tag in the main trace
// and propagate this to the store operation.
TEST_F(AvmMemOpcodeNegativeTests, movWrongOutputTagInMainTrace)
{
    FF const tag_u64 = FF(static_cast<uint32_t>(AvmMemoryTag::U64));

    buildTrace(false, 234, 0, 1, AvmMemoryTag::U8);
    computeIndices(false);

    trace.at(mem_c_idx).avm_mem_tag = tag_u64;
    trace.at(mem_c_idx).avm_mem_w_in_tag = tag_u64;
    trace.at(main_idx).avm_main_w_in_tag = tag_u64;
    trace.at(main_idx).avm_main_tag_err = 1;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "MOV_MAIN_SAME_TAG");
}

// The manipulation of the tag occurs in the store operation.
TEST_F(AvmMemOpcodeNegativeTests, movWrongOutputTagMainTraceRead)
{
    FF const tag_u64 = FF(static_cast<uint32_t>(AvmMemoryTag::U64));

    buildTrace(false, 234, 0, 1, AvmMemoryTag::U8);
    computeIndices(false);

    trace.at(mem_c_idx).avm_mem_tag = tag_u64;
    trace.at(mem_c_idx).avm_mem_w_in_tag = tag_u64;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "PERM_MAIN_MEM_C");
}

} // namespace tests_avm
