#include "avm_common.test.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/avm_trace/avm_mem_trace.hpp"
#include "barretenberg/vm/tests/helpers.test.hpp"
#include <cstddef>
#include <gtest/gtest.h>
#include <vector>

using namespace bb;

namespace tests_avm {
using namespace bb::avm_trace;

class AvmInterTableTests : public ::testing::Test {
  public:
    AvmTraceBuilder trace_builder;

  protected:
    // TODO(640): The Standard Honk on Grumpkin test suite fails unless the SRS is initialised for every test.
    void SetUp() override { srs::init_crs_factory("../srs_db/ignition"); };
};

/******************************************************************************
 *
 *                          INTER-TABLE NEGATIVE TESTS
 *
 ******************************************************************************
 * These negative unit tests aim to catch violations related to inter-table
 * relations. Inter-table relations are implemented through permutation and
 * lookup relations. Each permutation and lookup relation defined in the AVM
 * has to be negatively tested in the current test suite.
 * The built trace in each test needs to be as correct as possible except the
 * relation being tested.
 ******************************************************************************/

// Error tag propagation from memory trace back to the main trace.
TEST_F(AvmInterTableTests, tagErrNotCopiedInMain)
{
    // Equality operation on U128 and second operand is of type U16.
    trace_builder.op_set(0, 32, 18, AvmMemoryTag::U128);
    trace_builder.op_set(0, 32, 76, AvmMemoryTag::U16);
    trace_builder.op_eq(0, 18, 76, 65, AvmMemoryTag::U128);
    trace_builder.halt();
    auto trace = trace_builder.finalize();

    // Find the row with equality operation and mutate the error tag.
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_eq == 1; });
    ASSERT_EQ(row->avm_main_tag_err, FF(1)); // Sanity check that the error tag is set.
    row->avm_main_tag_err = 0;
    row->avm_main_alu_sel = 1; // We have to activate ALU trace if no error tag is present.
    auto const clk = row->avm_main_clk;

    // Create a valid ALU entry for this equality operation.
    auto& alu_row = trace.at(1);
    alu_row.avm_alu_clk = clk;
    alu_row.avm_alu_alu_sel = 1;
    alu_row.avm_alu_ia = 32;
    alu_row.avm_alu_ib = 32;
    alu_row.avm_alu_ic = 1;
    alu_row.avm_alu_op_eq = 1;
    alu_row.avm_alu_in_tag = static_cast<uint32_t>(AvmMemoryTag::U128);
    alu_row.avm_alu_u128_tag = 1;

    // Adjust the output of the computation as it would have been performed without tag check.
    row->avm_main_ic = 1;
    // Find the memory row pertaining to write operation from Ic.
    auto mem_row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
        return r.avm_mem_clk == clk && r.avm_mem_sub_clk == AvmMemTraceBuilder::SUB_CLK_STORE_C;
    });

    // Adjust the output in the memory trace.
    mem_row->avm_mem_val = 1;
    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "INCL_MAIN_TAG_ERR");
}

/******************************************************************************
 *                         MAIN <-------> ALU
 ******************************************************************************/
class AvmPermMainAluNegativeTests : public AvmInterTableTests {
  protected:
    std::vector<Row> trace;
    size_t main_idx;
    size_t mem_idx;
    size_t alu_idx;

    void SetUp() override
    {
        AvmInterTableTests::SetUp();

        trace_builder.op_set(0, 19, 0, AvmMemoryTag::U64);
        trace_builder.op_set(0, 15, 1, AvmMemoryTag::U64);
        trace_builder.op_add(0, 0, 1, 1, AvmMemoryTag::U64); // 19 + 15 = 34
        trace_builder.op_add(0, 0, 1, 1, AvmMemoryTag::U64); // 19 + 34 = 53
        trace_builder.op_mul(0, 0, 1, 2, AvmMemoryTag::U64); // 19 * 53 = 1007
        trace_builder.return_op(0, 0, 0);

        trace = trace_builder.finalize();

        // Find the row with multiplication operation and retrieve clk.
        auto row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_mul == FF(1); });

        ASSERT_TRUE(row != trace.end());
        ASSERT_EQ(row->avm_main_ic, 1007); // Sanity check
        auto clk = row->avm_main_clk;

        // Find the corresponding Alu trace row
        auto alu_row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) { return r.avm_alu_clk == clk; });
        ASSERT_TRUE(alu_row != trace.end());

        // Find memory trace entry related to storing output (intermediate register Ic) in memory.
        auto mem_row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
            return r.avm_mem_clk == clk && r.avm_mem_op_c == FF(1) && r.avm_mem_rw == FF(1);
        });
        ASSERT_TRUE(mem_row != trace.end());

        main_idx = static_cast<size_t>(row - trace.begin());
        alu_idx = static_cast<size_t>(alu_row - trace.begin());
        mem_idx = static_cast<size_t>(mem_row - trace.begin());
    }
};

TEST_F(AvmPermMainAluNegativeTests, wrongAluOutputCopyInMain)
{
    // Mutate the multiplication output. Note that the output alu counterpart is still valid
    // and pass the multiplication relation.
    trace.at(main_idx).avm_main_ic = 1008;
    trace.at(mem_idx).avm_mem_val = 1008;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "PERM_MAIN_ALU");
}

TEST_F(AvmPermMainAluNegativeTests, wrongCopyToAluIaInput)
{
    // Mutate the input of alu_ia and adapt the output ic accordingly.
    trace.at(alu_idx).avm_alu_ia = 20;
    trace.at(alu_idx).avm_alu_ic = 1060;  // 20 * 53; required to pass the alu mul relation
    trace.at(alu_idx).avm_alu_u8_r0 = 36; // 1060 % 256 = 36
    trace.at(alu_idx).avm_alu_u8_r1 = 4;  // 4 * 256 = 1024

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "PERM_MAIN_ALU");
}

TEST_F(AvmPermMainAluNegativeTests, wrongCopyToAluIbInput)
{
    // Mutate the input of alu_ia and adapt the output ic accordingly.
    trace.at(alu_idx).avm_alu_ib = 10;
    trace.at(alu_idx).avm_alu_ic = 190; // 19 * 10; required to pass the alu mul relation
    trace.at(alu_idx).avm_alu_u8_r0 = 190;
    trace.at(alu_idx).avm_alu_u8_r1 = 0;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "PERM_MAIN_ALU");
}

TEST_F(AvmPermMainAluNegativeTests, wrongCopyToAluOpSelector)
{
    trace.at(alu_idx).avm_alu_op_mul = 0;
    trace.at(alu_idx).avm_alu_op_add = 1;
    trace.at(alu_idx).avm_alu_ic = 72; // 19 + 53
    trace.at(alu_idx).avm_alu_u8_r0 = 72;
    trace.at(alu_idx).avm_alu_u8_r1 = 0;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "PERM_MAIN_ALU");
}

TEST_F(AvmPermMainAluNegativeTests, removeAluSelector)
{
    trace.at(alu_idx).avm_alu_alu_sel = 0;
    trace.at(alu_idx).avm_alu_op_mul = 0;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "PERM_MAIN_ALU");
}

/******************************************************************************
 *                         MAIN <-------> ALU
 ******************************************************************************/
class AvmPermMainMemNegativeTests : public AvmInterTableTests {
  protected:
    std::vector<Row> trace;
    size_t main_idx;
    size_t mem_idx_a;
    size_t mem_idx_b;
    size_t mem_idx_c;
    size_t alu_idx;

    // Helper function to generate a trace with a subtraction
    // for c = a - b at arbitray chosen addresses 52 (a), 11 (b), 55 (c).
    void executeSub(uint128_t const a, uint128_t const b)
    {
        trace_builder.op_set(0, a, 52, AvmMemoryTag::U8);
        trace_builder.op_set(0, b, 11, AvmMemoryTag::U8);
        trace_builder.op_sub(0, 52, 11, 55, AvmMemoryTag::U8);
        trace_builder.return_op(0, 0, 0);

        trace = trace_builder.finalize();

        // Find the row with subtraction operation and retrieve clk.
        auto row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_sub == FF(1); });

        ASSERT_TRUE(row != trace.end());
        auto clk = row->avm_main_clk;

        // Find the corresponding Alu trace row
        auto alu_row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) { return r.avm_alu_clk == clk; });
        ASSERT_TRUE(alu_row != trace.end());

        // Find memory trace entry related to storing output (intermediate register Ic) in memory.
        auto mem_row_c = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
            return r.avm_mem_clk == clk && r.avm_mem_op_c == FF(1) && r.avm_mem_rw == FF(1);
        });
        ASSERT_TRUE(mem_row_c != trace.end());

        // Find memory trace entry related to loading first input (intermediate register Ia) in memory.
        auto mem_row_a = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
            return r.avm_mem_clk == clk && r.avm_mem_op_a == FF(1) && r.avm_mem_rw == FF(0);
        });
        ASSERT_TRUE(mem_row_a != trace.end());

        // Find memory trace entry related to loading second input (intermediate register Ib) in memory.
        auto mem_row_b = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
            return r.avm_mem_clk == clk && r.avm_mem_op_b == FF(1) && r.avm_mem_rw == FF(0);
        });
        ASSERT_TRUE(mem_row_b != trace.end());

        main_idx = static_cast<size_t>(row - trace.begin());
        alu_idx = static_cast<size_t>(alu_row - trace.begin());
        mem_idx_a = static_cast<size_t>(mem_row_a - trace.begin());
        mem_idx_b = static_cast<size_t>(mem_row_b - trace.begin());
        mem_idx_c = static_cast<size_t>(mem_row_c - trace.begin());
    }
};

TEST_F(AvmPermMainMemNegativeTests, wrongValueIaInMem)
{
    executeSub(21, 3);
    trace.at(mem_idx_a).avm_mem_val = 26;     // Correct value: 21
    trace.at(mem_idx_a - 1).avm_mem_val = 26; // We need to adjust the write operation beforehand (set opcode).

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "PERM_MAIN_MEM_A");
}

TEST_F(AvmPermMainMemNegativeTests, wrongValueIbInMem)
{
    executeSub(21, 3);
    trace.at(mem_idx_b).avm_mem_val = 7;     // Correct value: 3
    trace.at(mem_idx_b - 1).avm_mem_val = 7; // We need to adjust the write operation beforehand (set opcode).

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "PERM_MAIN_MEM_B");
}

TEST_F(AvmPermMainMemNegativeTests, wrongValueIcInMem)
{
    executeSub(21, 3);
    trace.at(mem_idx_c).avm_mem_val = 17; // Correct value: 18 = 21 - 3

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "PERM_MAIN_MEM_C");
}

TEST_F(AvmPermMainMemNegativeTests, wrongAddressIaInMain)
{
    executeSub(21, 3);
    trace.at(main_idx).avm_main_mem_idx_a = 28; // Correct address: 52

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "PERM_MAIN_MEM_A");
}

TEST_F(AvmPermMainMemNegativeTests, wrongAddressIbInMain)
{
    executeSub(21, 3);
    trace.at(main_idx).avm_main_mem_idx_b = 2; // Correct address: 11

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "PERM_MAIN_MEM_B");
}

TEST_F(AvmPermMainMemNegativeTests, wrongAddressIcInMain)
{
    executeSub(21, 3);
    trace.at(main_idx).avm_main_mem_idx_c = 75; // Correct address: 55

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "PERM_MAIN_MEM_C");
}

TEST_F(AvmPermMainMemNegativeTests, wrongInTagIaInMem)
{
    executeSub(21, 3);
    auto wrong_in_tag = static_cast<uint32_t>(AvmMemoryTag::U32);
    trace.at(mem_idx_a).avm_mem_r_in_tag = wrong_in_tag; // Correct value: AvmMemoryTag::U8
    trace.at(mem_idx_a).avm_mem_tag = wrong_in_tag;

    // We need to adjust the write operation beforehand (set opcode).
    trace.at(mem_idx_a - 1).avm_mem_r_in_tag = wrong_in_tag;
    trace.at(mem_idx_a - 1).avm_mem_w_in_tag = wrong_in_tag;
    trace.at(mem_idx_a - 1).avm_mem_tag = wrong_in_tag;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "PERM_MAIN_MEM_A");
}

TEST_F(AvmPermMainMemNegativeTests, wrongInTagIbInMem)
{
    executeSub(21, 3);
    auto wrong_in_tag = static_cast<uint32_t>(AvmMemoryTag::U16);
    trace.at(mem_idx_b).avm_mem_r_in_tag = wrong_in_tag; // Correct value: AvmMemoryTag::U8
    trace.at(mem_idx_b).avm_mem_tag = wrong_in_tag;

    // We need to adjust the write operation beforehand (set opcode).
    trace.at(mem_idx_b - 1).avm_mem_r_in_tag = wrong_in_tag;
    trace.at(mem_idx_b - 1).avm_mem_w_in_tag = wrong_in_tag;
    trace.at(mem_idx_b - 1).avm_mem_tag = wrong_in_tag;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "PERM_MAIN_MEM_B");
}

TEST_F(AvmPermMainMemNegativeTests, wrongInTagIcInMem)
{
    executeSub(21, 3);
    auto wrong_in_tag = static_cast<uint32_t>(AvmMemoryTag::U128);
    trace.at(mem_idx_c).avm_mem_w_in_tag = wrong_in_tag; // Correct value: AvmMemoryTag::U8
    trace.at(mem_idx_c).avm_mem_tag = wrong_in_tag;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "PERM_MAIN_MEM_C");
}

TEST_F(AvmPermMainMemNegativeTests, wrongRwIaInMem)
{
    executeSub(21, 3);
    trace.at(mem_idx_a).avm_mem_rw = 1; // Write instead of read.

    // Adjust sub_clk value
    trace.at(mem_idx_a).avm_mem_sub_clk = AvmMemTraceBuilder::SUB_CLK_STORE_A;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "PERM_MAIN_MEM_A");
}

TEST_F(AvmPermMainMemNegativeTests, wrongRwIbInMem)
{
    executeSub(21, 3);
    trace.at(mem_idx_b).avm_mem_rw = 1; // Write instead of read.

    // Adjust sub_clk value
    trace.at(mem_idx_b).avm_mem_sub_clk = AvmMemTraceBuilder::SUB_CLK_STORE_B;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "PERM_MAIN_MEM_B");
}

TEST_F(AvmPermMainMemNegativeTests, wrongRwIcInMem)
{
    // For this test, we need the result to be zero. Otherwise, swapping
    // a write for a read of Ic below leads to a violation that the memory
    // is initialized with zero values.
    executeSub(11, 11);
    trace.at(mem_idx_c).avm_mem_rw = 0; // Read instead of write.

    // Adjust sub_clk value
    trace.at(mem_idx_c).avm_mem_sub_clk = AvmMemTraceBuilder::SUB_CLK_LOAD_C;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "PERM_MAIN_MEM_C");
}

TEST_F(AvmPermMainMemNegativeTests, wrongClkIaInMem)
{
    executeSub(87, 23);
    trace.at(mem_idx_a).avm_mem_clk = 11;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "PERM_MAIN_MEM_A");
}

TEST_F(AvmPermMainMemNegativeTests, wrongClkIbInMem)
{
    executeSub(87, 23);
    trace.at(mem_idx_b).avm_mem_clk = 21;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "PERM_MAIN_MEM_B");
}

TEST_F(AvmPermMainMemNegativeTests, wrongClkIcInMem)
{
    executeSub(87, 23);
    trace.at(mem_idx_c).avm_mem_clk = 7;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "PERM_MAIN_MEM_C");
}

} // namespace tests_avm