#include "avm_common.test.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/tests/helpers.test.hpp"
#include <cstddef>
#include <cstdint>

namespace tests_avm {
using namespace bb::avm_trace;

class AvmMemOpcodeTests : public ::testing::Test {
  public:
    AvmTraceBuilder trace_builder;

  protected:
    std::vector<Row> trace;
    size_t main_idx;
    size_t mem_a_idx;
    size_t mem_c_idx;

    // TODO(640): The Standard Honk on Grumpkin test suite fails unless the SRS is initialised for every test.
    void SetUp() override { srs::init_crs_factory("../srs_db/ignition"); };
    void buildTrace(uint128_t const val, uint32_t const src_offset, uint32_t const dst_offset, AvmMemoryTag const tag)
    {
        trace_builder.set(val, src_offset, tag);
        trace_builder.op_mov(src_offset, dst_offset);
        trace_builder.return_op(0, 0);
        trace = trace_builder.finalize();

        // Find the first row enabling the MOV selector
        auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_mov == FF(1); });
        ASSERT_TRUE(row != trace.end());
        main_idx = static_cast<size_t>(row - trace.begin());

        auto clk = row->avm_main_clk;

        // Find the memory trace position corresponding to the load sub-operation of register ia.
        row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
            return r.avm_mem_m_clk == clk && r.avm_mem_m_sub_clk == AvmMemTraceBuilder::SUB_CLK_LOAD_A;
        });
        ASSERT_TRUE(row != trace.end());
        mem_a_idx = static_cast<size_t>(row - trace.begin());

        // Find the memory trace position corresponding to the write sub-operation of register ic.
        row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
            return r.avm_mem_m_clk == clk && r.avm_mem_m_sub_clk == AvmMemTraceBuilder::SUB_CLK_STORE_C;
        });
        ASSERT_TRUE(row != trace.end());
        mem_c_idx = static_cast<size_t>(row - trace.begin());
    }

    void validate_trace(uint128_t const val,
                        uint32_t const src_offset,
                        uint32_t const dst_offset,
                        AvmMemoryTag const tag)
    {
        FF const val_ff = uint256_t::from_uint128(val);
        auto const& main_row = trace.at(main_idx);

        EXPECT_EQ(main_row.avm_main_ia, val_ff);
        EXPECT_EQ(main_row.avm_main_ib, FF(0));
        EXPECT_EQ(main_row.avm_main_ic, val_ff);
        EXPECT_EQ(main_row.avm_main_in_tag, FF(static_cast<uint32_t>(tag)));

        auto const& mem_a_row = trace.at(mem_a_idx);

        EXPECT_EQ(mem_a_row.avm_mem_m_tag_err, FF(0));
        EXPECT_EQ(mem_a_row.avm_mem_m_in_tag, FF(static_cast<uint32_t>(tag)));
        EXPECT_EQ(mem_a_row.avm_mem_m_tag, FF(static_cast<uint32_t>(tag)));
        EXPECT_EQ(mem_a_row.avm_mem_m_sel_mov, FF(1));
        EXPECT_EQ(mem_a_row.avm_mem_m_addr, FF(src_offset));
        EXPECT_EQ(mem_a_row.avm_mem_m_val, val_ff);
        EXPECT_EQ(mem_a_row.avm_mem_m_op_a, FF(1));

        auto const& mem_c_row = trace.at(mem_c_idx);

        EXPECT_EQ(mem_c_row.avm_mem_m_tag_err, FF(0));
        EXPECT_EQ(mem_c_row.avm_mem_m_in_tag, FF(static_cast<uint32_t>(tag)));
        EXPECT_EQ(mem_c_row.avm_mem_m_tag, FF(static_cast<uint32_t>(tag)));
        EXPECT_EQ(mem_c_row.avm_mem_m_addr, FF(dst_offset));
        EXPECT_EQ(mem_c_row.avm_mem_m_val, val_ff);
        EXPECT_EQ(mem_c_row.avm_mem_m_op_c, FF(1));

        validate_trace_proof(std::move(trace));
    }
};

class AvmMemOpcodeNegativeTests : public AvmMemOpcodeTests {};

/******************************************************************************
 *
 * MEMORY OPCODE TESTS
 *
 ******************************************************************************/

TEST_F(AvmMemOpcodeTests, basicMov)
{
    buildTrace(42, 9, 13, AvmMemoryTag::U64);
    validate_trace(42, 9, 13, AvmMemoryTag::U64);
}

TEST_F(AvmMemOpcodeTests, sameAddressMov)
{

    buildTrace(11, 356, 356, AvmMemoryTag::U16);

    validate_trace(11, 356, 356, AvmMemoryTag::U16);
}

TEST_F(AvmMemOpcodeNegativeTests, wrongOutputErrorTag)
{
    buildTrace(234, 0, 1, AvmMemoryTag::U8);
    trace.at(main_idx).avm_main_tag_err = 1;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "INCL_MEM_TAG_ERR");
}

TEST_F(AvmMemOpcodeNegativeTests, wrongOutputValue)
{
    buildTrace(234, 0, 1, AvmMemoryTag::U8);
    trace.at(main_idx).avm_main_ic = 233;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "MOV_SAME_VALUE");
}

// We want to test that the output tag cannot be changed.
// In this test, we modify the m_in_tag for load operation to Ia.
// Then, we propagate the error tag and the copy of m_in_tag to the
// main trace and the memory entry related to stor operation from Ic.
TEST_F(AvmMemOpcodeNegativeTests, wrongOutputTagLoadIa)
{
    FF const tag_u64 = FF(static_cast<uint32_t>(AvmMemoryTag::U64));
    FF const tag_u8 = FF(static_cast<uint32_t>(AvmMemoryTag::U8));
    FF const one_min_inverse_diff = FF(1) - (tag_u64 - tag_u8).invert();

    buildTrace(234, 0, 1, AvmMemoryTag::U8);

    trace.at(mem_a_idx).avm_mem_m_in_tag = tag_u64;
    trace.at(mem_a_idx).avm_mem_m_tag_err = 1;
    trace.at(mem_a_idx).avm_mem_m_one_min_inv = one_min_inverse_diff;
    trace.at(mem_c_idx).avm_mem_m_tag = tag_u64;
    trace.at(mem_c_idx).avm_mem_m_in_tag = tag_u64;
    trace.at(main_idx).avm_main_in_tag = tag_u64;
    trace.at(main_idx).avm_main_tag_err = 1;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "MOV_SAME_TAG");
}

// Same as above but one tries to disable the selector of MOV opcode in
// the load operation.
TEST_F(AvmMemOpcodeNegativeTests, wrongOutputTagDisabledSelector)
{
    FF const tag_u64 = FF(static_cast<uint32_t>(AvmMemoryTag::U64));
    FF const tag_u8 = FF(static_cast<uint32_t>(AvmMemoryTag::U8));
    FF const one_min_inverse_diff = FF(1) - (tag_u64 - tag_u8).invert();

    buildTrace(234, 0, 1, AvmMemoryTag::U8);

    trace.at(mem_a_idx).avm_mem_m_in_tag = tag_u64;
    trace.at(mem_a_idx).avm_mem_m_tag_err = 1;
    trace.at(mem_a_idx).avm_mem_m_one_min_inv = one_min_inverse_diff;
    trace.at(mem_a_idx).avm_mem_m_sel_mov = 0;
    trace.at(mem_c_idx).avm_mem_m_tag = tag_u64;
    trace.at(mem_c_idx).avm_mem_m_in_tag = tag_u64;
    trace.at(main_idx).avm_main_in_tag = tag_u64;
    trace.at(main_idx).avm_main_tag_err = 1;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "PERM_MAIN_MEM_A");
}

// The manipulation of the tag occurs in the main trace and then we
// propagate this change to the store memory operation of Ic.
TEST_F(AvmMemOpcodeNegativeTests, wrongOutputTagMainTrace)
{
    FF const tag_u64 = FF(static_cast<uint32_t>(AvmMemoryTag::U64));

    buildTrace(234, 0, 1, AvmMemoryTag::U8);
    trace.at(main_idx).avm_main_in_tag = tag_u64;

    trace.at(mem_c_idx).avm_mem_m_tag = tag_u64;
    trace.at(mem_c_idx).avm_mem_m_in_tag = tag_u64;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "PERM_MAIN_MEM_A");
}

} // namespace tests_avm
