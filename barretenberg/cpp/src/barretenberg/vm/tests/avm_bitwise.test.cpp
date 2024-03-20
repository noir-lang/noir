#include "avm_common.test.hpp"
#include "barretenberg/numeric/uint128/uint128.hpp"
#include <algorithm>
#include <cstdint>
#include <vector>

namespace tests_avm {
using namespace bb::avm_trace;

namespace {

Row common_validate_op_not(std::vector<Row> const& trace,
                           FF const& a,
                           FF const& c,
                           FF const& addr_a,
                           FF const& addr_c,
                           avm_trace::AvmMemoryTag const tag)
{

    // Find the first row enabling the not selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_not == FF(1); });

    // Use the row in the main trace to find the same operation in the alu trace.
    FF clk = row->avm_main_clk;
    auto alu_row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) { return r.avm_alu_alu_clk == clk; });

    // Check that both rows were found
    EXPECT_TRUE(row != trace.end());
    EXPECT_TRUE(alu_row != trace.end());

    // Check that the correct result is stored at the expected memory location.
    EXPECT_EQ(row->avm_main_ic, c);
    EXPECT_EQ(row->avm_main_mem_idx_c, addr_c);
    EXPECT_EQ(row->avm_main_mem_op_c, FF(1));
    EXPECT_EQ(row->avm_main_rwc, FF(1));

    // Check that ia register is correctly set with memory load operations.
    EXPECT_EQ(row->avm_main_ia, a);
    EXPECT_EQ(row->avm_main_mem_idx_a, addr_a);
    EXPECT_EQ(row->avm_main_mem_op_a, FF(1));
    EXPECT_EQ(row->avm_main_rwa, FF(0));

    // Check the instruction tag
    EXPECT_EQ(row->avm_main_in_tag, FF(static_cast<uint32_t>(tag)));

    // Check that intermediate registers are correctly copied in Alu trace
    EXPECT_EQ(alu_row->avm_alu_alu_ia, a);
    EXPECT_EQ(alu_row->avm_alu_alu_ib, FF(0));
    EXPECT_EQ(alu_row->avm_alu_alu_ic, c);

    // Check that not selector is set.
    EXPECT_EQ(row->avm_main_sel_op_not, FF(1));
    EXPECT_EQ(alu_row->avm_alu_alu_op_not, FF(1));

    return *alu_row;
}

std::vector<Row> gen_mutated_trace_not(FF const& a, FF const& c_mutated, avm_trace::AvmMemoryTag tag)
{
    auto trace_builder = avm_trace::AvmTraceBuilder();
    trace_builder.set(uint128_t{ a }, 0, tag);
    trace_builder.op_not(0, 0, 1, tag);
    trace_builder.halt();
    auto trace = trace_builder.finalize();

    auto select_row = [](Row r) { return r.avm_main_sel_op_not == FF(1); };
    mutate_ic_in_trace(trace, select_row, c_mutated, true);

    return trace;
}
} // namespace

class AvmBitwiseTests : public ::testing::Test {
  public:
    AvmTraceBuilder trace_builder;

  protected:
    // TODO(640): The Standard Honk on Grumpkin test suite fails unless the SRS is initialised for every test.
    void SetUp() override { srs::init_crs_factory("../srs_db/ignition"); };
};

class AvmBitwiseTestsU8 : public AvmBitwiseTests {};
class AvmBitwiseTestsU16 : public AvmBitwiseTests {};
class AvmBitwiseTestsU32 : public AvmBitwiseTests {};
class AvmBitwiseTestsU64 : public AvmBitwiseTests {};
class AvmBitwiseTestsU128 : public AvmBitwiseTests {};

class AvmBitwiseNegativeTestsFF : public AvmBitwiseTests {};
class AvmBitwiseNegativeTestsU8 : public AvmBitwiseTests {};
class AvmBitwiseNegativeTestsU16 : public AvmBitwiseTests {};
class AvmBitwiseNegativeTestsU32 : public AvmBitwiseTests {};
class AvmBitwiseNegativeTestsU64 : public AvmBitwiseTests {};
class AvmBitwiseNegativeTestsU128 : public AvmBitwiseTests {};

/******************************************************************************
 *
 * POSITIVE TESTS
 *
 ******************************************************************************
 * See Avm_arithmetic.cpp for explanation of positive tests
 ******************************************************************************/

/******************************************************************************
 * Positive Tests - U8
 ******************************************************************************/

TEST_F(AvmBitwiseTestsU8, BitwiseNot)
{
    trace_builder.set(1, 0, AvmMemoryTag::U8);       // Memory Layout: [1,0,0,...]
    trace_builder.op_not(0, 0, 1, AvmMemoryTag::U8); // [1,254,0,0,....]
    trace_builder.return_op(0, 0, 0);
    auto trace = trace_builder.finalize();

    auto alu_row = common_validate_op_not(trace, FF(1), FF(254), FF(0), FF(1), AvmMemoryTag::U8);

    EXPECT_EQ(alu_row.avm_alu_alu_u8_tag, FF(1));
    validate_trace_proof(std::move(trace));
}

TEST_F(AvmBitwiseTestsU16, BitwiseNot)
{
    trace_builder.set(512, 0, AvmMemoryTag::U16);     // Memory Layout: [512,0,0,...]
    trace_builder.op_not(0, 0, 1, AvmMemoryTag::U16); // [512,65023,0,0,0,....]
    trace_builder.return_op(0, 0, 0);
    auto trace = trace_builder.finalize();

    auto alu_row = common_validate_op_not(trace, FF(512), FF(65'023), FF(0), FF(1), AvmMemoryTag::U16);

    EXPECT_EQ(alu_row.avm_alu_alu_u16_tag, FF(1));
    validate_trace_proof(std::move(trace));
}

TEST_F(AvmBitwiseTestsU32, BitwiseNot)
{
    trace_builder.set(131'072, 0, AvmMemoryTag::U32); // Memory Layout: [131072,0,0,...]
    trace_builder.op_not(0, 0, 1, AvmMemoryTag::U32); // [131072,4294836223,,0,0,....]
    trace_builder.return_op(0, 0, 0);
    auto trace = trace_builder.finalize();

    auto alu_row = common_validate_op_not(trace, FF(131'072), FF(4'294'836'223LLU), FF(0), FF(1), AvmMemoryTag::U32);

    EXPECT_EQ(alu_row.avm_alu_alu_u32_tag, FF(1));
    validate_trace_proof(std::move(trace));
}

TEST_F(AvmBitwiseTestsU64, BitwiseNot)
{
    trace_builder.set(0x100000000LLU, 0, AvmMemoryTag::U64); // Memory Layout: [8589934592,0,0,...]
    trace_builder.op_not(0, 0, 1, AvmMemoryTag::U64);        // [8589934592,18446744069414584319,0,0,....]
    trace_builder.return_op(0, 0, 0);
    auto trace = trace_builder.finalize();

    auto alu_row =
        common_validate_op_not(trace, FF(0x100000000LLU), FF(0xfffffffeffffffffLLU), FF(0), FF(1), AvmMemoryTag::U64);

    EXPECT_EQ(alu_row.avm_alu_alu_u64_tag, FF(1));
    validate_trace_proof(std::move(trace));
}

TEST_F(AvmBitwiseTestsU128, BitwiseNot)
{

    uint128_t const a = uint128_t{ 0x4000000000000 } << 64;
    trace_builder.set(a, 0, AvmMemoryTag::U128);
    trace_builder.op_not(0, 0, 1, AvmMemoryTag::U128);
    trace_builder.return_op(0, 0, 0);
    auto trace = trace_builder.finalize();

    uint128_t const res = (uint128_t{ 0xfffbffffffffffff } << 64) + uint128_t{ 0xffffffffffffffff };
    auto alu_row = common_validate_op_not(
        trace, FF(uint256_t::from_uint128(a)), FF(uint256_t::from_uint128(res)), FF(0), FF(1), AvmMemoryTag::U128);

    EXPECT_EQ(alu_row.avm_alu_alu_u128_tag, FF(1));
    validate_trace_proof(std::move(trace));
}

/******************************************************************************
 *
 * NEGATIVE TESTS - Finite Field Type
 *
 ******************************************************************************
 * See Avm_arithmetic.cpp for explanation of negative tests
 ******************************************************************************/

/******************************************************************************
 * Negative Tests - FF
 ******************************************************************************/

TEST_F(AvmBitwiseNegativeTestsFF, UndefinedOverFF)
{
    auto trace_builder = avm_trace::AvmTraceBuilder();
    // Triggers a write row 1 of mem_trace and alu_trace
    trace_builder.set(10, 0, AvmMemoryTag::U8);
    // Triggers a write in row 2 of alu_trace
    trace_builder.op_not(0, 0, 1, AvmMemoryTag::U8);
    // Finally, we will have a write in row 3 of the mem_trace to copy the result
    // from the op_not operation.
    trace_builder.return_op(0, 0, 0);
    // Manually update the memory tags in the relevant trace;
    auto trace = trace_builder.finalize();
    // TODO(ilyas): When the SET opcodes applies relational constraints, this will fail
    // we will need to look at a new way of doing this test.
    for (size_t i = 1; i < 4; i++) {
        trace.at(i).avm_mem_m_tag = FF(6);
        trace.at(i).avm_mem_m_in_tag = FF(6);
        trace.at(i).avm_alu_alu_ff_tag = FF::one();
        trace.at(i).avm_alu_alu_u8_tag = FF::zero();
        trace.at(i).avm_main_in_tag = FF(6);
        trace.at(i).avm_alu_alu_in_tag = FF(6);
    }

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "ALU_FF_NOT_XOR");
}

TEST_F(AvmBitwiseNegativeTestsU8, BitwiseNot)
{
    std::vector<Row> trace = gen_mutated_trace_not(FF{ 1 }, FF{ 2 }, AvmMemoryTag::U8);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "ALU_OP_NOT");
}

TEST_F(AvmBitwiseNegativeTestsU16, BitwiseNot)
{
    std::vector<Row> trace = gen_mutated_trace_not(FF{ 32'768 }, FF{ 8'192 }, AvmMemoryTag::U16);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "ALU_OP_NOT");
}

TEST_F(AvmBitwiseNegativeTestsU32, BitwiseNot)
{
    std::vector<Row> trace = gen_mutated_trace_not(FF{ 0xdeadbeef }, FF{ 0x20020af }, AvmMemoryTag::U64);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "ALU_OP_NOT");
}

TEST_F(AvmBitwiseNegativeTestsU64, BitwiseNot)
{
    std::vector<Row> trace =
        gen_mutated_trace_not(FF{ 0x10000000000000LLU }, FF{ 0x10000fed0100000LLU }, AvmMemoryTag::U64);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "ALU_OP_NOT");
}

TEST_F(AvmBitwiseNegativeTestsU128, BitwiseNot)
{
    uint128_t const a = uint128_t{ 0x4000000000000 } << 64;
    uint128_t const b = uint128_t{ 0x300000ae921000 } << 64;
    std::vector<Row> trace =
        gen_mutated_trace_not(FF{ uint256_t::from_uint128(a) }, FF{ uint256_t::from_uint128(b) }, AvmMemoryTag::U128);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "ALU_OP_NOT");
}
} // namespace tests_avm
