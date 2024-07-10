#include "avm_common.test.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/avm_trace/avm_mem_trace.hpp"
#include "barretenberg/vm/tests/helpers.test.hpp"
#include <cstddef>
#include <gtest/gtest.h>
#include <vector>

using namespace bb;

namespace tests_avm {
using namespace bb;
using namespace bb::avm_trace;

class AvmInterTableTests : public ::testing::Test {
  public:
    AvmInterTableTests()
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

/******************************************************************************
 *                         MAIN <-------> ALU
 ******************************************************************************/
class AvmPermMainAluNegativeTests : public AvmInterTableTests {
  protected:
    std::vector<Row> trace;
    size_t main_row_idx;
    size_t mem_row_idx;
    size_t alu_row_idx;

    void SetUp() override
    {
        AvmInterTableTests::SetUp();

        trace_builder.op_set(0, 19, 0, AvmMemoryTag::U64);
        trace_builder.op_set(0, 15, 1, AvmMemoryTag::U64);
        trace_builder.op_add(0, 0, 1, 1, AvmMemoryTag::U64); // 19 + 15 = 34
        trace_builder.op_add(0, 0, 1, 1, AvmMemoryTag::U64); // 19 + 34 = 53
        trace_builder.op_mul(0, 0, 1, 2, AvmMemoryTag::U64); // 19 * 53 = 1007
        trace_builder.op_return(0, 0, 0);

        trace = trace_builder.finalize();

        // Find the row with multiplication operation and retrieve clk.
        auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_mul == FF(1); });

        ASSERT_TRUE(row != trace.end());
        ASSERT_EQ(row->main_ic, 1007); // Sanity check
        auto clk = row->main_clk;

        // Find the corresponding Alu trace row
        auto alu_row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) { return r.alu_clk == clk; });
        ASSERT_TRUE(alu_row != trace.end());

        // Find memory trace entry related to storing output (intermediate register Ic) in memory.
        auto mem_row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
            return r.mem_clk == clk && r.mem_sel_op_c == FF(1) && r.mem_rw == FF(1);
        });
        ASSERT_TRUE(mem_row != trace.end());

        main_row_idx = static_cast<size_t>(row - trace.begin());
        alu_row_idx = static_cast<size_t>(alu_row - trace.begin());
        mem_row_idx = static_cast<size_t>(mem_row - trace.begin());
    }
};

TEST_F(AvmPermMainAluNegativeTests, wrongAluOutputCopyInMain)
{
    // Mutate the multiplication output. Note that the output alu counterpart is still valid
    // and pass the multiplication relation.
    trace.at(main_row_idx).main_ic = 1008;
    trace.at(mem_row_idx).mem_val = 1008;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "PERM_MAIN_ALU");
}

TEST_F(AvmPermMainAluNegativeTests, wrongCopyToAluIaInput)
{
    // Mutate the input of alu_ia and adapt the output ic accordingly.
    trace.at(alu_row_idx).alu_ia = 20;
    trace.at(alu_row_idx).alu_ic = 1060;  // 20 * 53; required to pass the alu mul relation
    trace.at(alu_row_idx).alu_u8_r0 = 36; // 1060 % 256 = 36
    trace.at(alu_row_idx).alu_u8_r1 = 4;  // 4 * 256 = 1024

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "PERM_MAIN_ALU");
}

TEST_F(AvmPermMainAluNegativeTests, wrongCopyToAluIbInput)
{
    // Mutate the input of alu_ia and adapt the output ic accordingly.
    trace.at(alu_row_idx).alu_ib = 10;
    trace.at(alu_row_idx).alu_ic = 190; // 19 * 10; required to pass the alu mul relation
    trace.at(alu_row_idx).alu_u8_r0 = 190;
    trace.at(alu_row_idx).alu_u8_r1 = 0;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "PERM_MAIN_ALU");
}

TEST_F(AvmPermMainAluNegativeTests, wrongCopyToAluOpSelector)
{
    trace.at(alu_row_idx).alu_op_mul = 0;
    trace.at(alu_row_idx).alu_op_add = 1;
    trace.at(alu_row_idx).alu_ic = 72; // 19 + 53
    trace.at(alu_row_idx).alu_u8_r0 = 72;
    trace.at(alu_row_idx).alu_u8_r1 = 0;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "PERM_MAIN_ALU");
}

TEST_F(AvmPermMainAluNegativeTests, removeAluSelector)
{
    trace.at(alu_row_idx).alu_sel_alu = 0;
    trace.at(alu_row_idx).alu_op_mul = 0;
    trace.at(alu_row_idx).alu_sel_rng_chk_lookup = 0;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "PERM_MAIN_ALU");
}

/******************************************************************************
 *                   REGISTER RANGE CHECKS (MAIN <-------> ALU)
 ******************************************************************************/
class AvmRangeCheckNegativeTests : public AvmInterTableTests {
  protected:
    std::vector<Row> trace;
    size_t main_row_idx;
    size_t mem_row_idx;
    size_t alu_row_idx;

    void genTraceAdd(
        uint128_t const& a, uint128_t const& b, uint128_t const& c, AvmMemoryTag tag, uint32_t min_trace_size = 0)
    {
        trace_builder.op_set(0, a, 0, tag);
        trace_builder.op_set(0, b, 1, tag);
        trace_builder.op_add(0, 0, 1, 2, tag); // 7 + 8 = 15
        trace_builder.op_return(0, 0, 0);
        trace = trace_builder.finalize(min_trace_size);

        // Find the row with addition operation and retrieve clk.
        auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_add == FF(1); });

        ASSERT_TRUE(row != trace.end());
        ASSERT_EQ(row->main_ia, FF(uint256_t::from_uint128(a)));
        ASSERT_EQ(row->main_ib, FF(uint256_t::from_uint128(b)));
        ASSERT_EQ(row->main_ic, FF(uint256_t::from_uint128(c)));
        auto clk = row->main_clk;

        // Find the corresponding Alu trace row
        auto alu_row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) { return r.alu_clk == clk; });
        ASSERT_TRUE(alu_row != trace.end());

        // Find memory trace entry related to storing output (intermediate register Ic) in memory.
        auto mem_row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
            return r.mem_clk == clk && r.mem_sel_op_c == FF(1) && r.mem_rw == FF(1);
        });
        ASSERT_TRUE(mem_row != trace.end());

        main_row_idx = static_cast<size_t>(row - trace.begin());
        alu_row_idx = static_cast<size_t>(alu_row - trace.begin());
        mem_row_idx = static_cast<size_t>(mem_row - trace.begin());
    };
};

// Out-of-range value in register u8_r0
TEST_F(AvmRangeCheckNegativeTests, additionU8Reg0)
{
    genTraceAdd(7, 8, 15, AvmMemoryTag::U8);

    // We mutate the result 15 to 15 - 2^254 mod p
    // The value 15 - 2^254 mod p is set in register u8_r0 and 2^246 in u8_r1
    // Therefore, u8_r0 + 2^8 * u8_r1 mod p = 15
    // All constraints except range checks on u8_r0, u8_r1 are satisfied.

    FF const fake_c = FF(15).add(-FF(2).pow(254));
    auto& row = trace.at(main_row_idx);
    auto& mem_row = trace.at(mem_row_idx);
    auto& alu_row = trace.at(alu_row_idx);

    row.main_ic = fake_c;
    mem_row.mem_val = fake_c;
    alu_row.alu_ic = fake_c;

    ASSERT_EQ(alu_row.alu_u8_r0, 15);
    ASSERT_EQ(alu_row.alu_u8_r1, 0);

    alu_row.alu_u8_r0 = fake_c;
    alu_row.alu_u8_r1 = FF(2).pow(246);

    // We first try to validate without any range check counters adjustment.
    auto trace_same_cnt = trace;
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace_same_cnt)), "LOOKUP_U8_0");

    // Decrement the counter for former lookup values 15 resp. 0 for u8_r0 resp. u8_r1.
    trace.at(15 + 1).lookup_u8_0_counts -= FF(1);
    trace.at(1).lookup_u8_1_counts -= FF(1);

    // One cannot add the new values in counters as they are out of range.
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "LOOKUP_U8_0");
}

// Out-of-range value in register u8_r1
TEST_F(AvmRangeCheckNegativeTests, additionU8Reg1)
{
    genTraceAdd(19, 20, 39, AvmMemoryTag::U8);
    auto& row = trace.at(main_row_idx);
    auto& mem_row = trace.at(mem_row_idx);
    auto& alu_row = trace.at(alu_row_idx);

    // a + b = u8_r0 + 2^8 * u8_r1 (mod p)
    // We recall that p-1 is a multiple of a large power of two.
    // We select a maximal u8_r1 such that u8_r0 is still of type U8.
    // Namely, we pick (p-1)/2^8 so that we can replace c (i.e., u8_r0) with 40 as
    // 39 = 40 + p - 1 (mod p)
    uint256_t const r1 = (uint256_t(FF::modulus) - 1) / 256;
    FF const fake_c = FF(40);

    row.main_ic = fake_c;
    mem_row.mem_val = fake_c;
    alu_row.alu_ic = fake_c;

    ASSERT_EQ(alu_row.alu_u8_r0, 39);
    ASSERT_EQ(alu_row.alu_u8_r1, 0);

    alu_row.alu_u8_r0 = fake_c;
    alu_row.alu_u8_r1 = FF(r1);

    // We adjust counter to pass range check lookup for u8_r0
    trace.at(39).lookup_u8_0_counts -= FF(1);
    trace.at(40).lookup_u8_0_counts += FF(1);

    auto trace_same_cnt = trace;
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace_same_cnt)), "LOOKUP_U8_1");

    // Second attempt by decreasing counter for u8_r1 range check lookup
    trace.at(0).lookup_u8_1_counts -= FF(1);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "LOOKUP_U8_1");
}

// Out-of-range value in register u16_r0
TEST_F(AvmRangeCheckNegativeTests, additionU16Reg0)
{
    genTraceAdd(1200, 2000, 3200, AvmMemoryTag::U16, 130);
    auto& row = trace.at(main_row_idx);
    auto& mem_row = trace.at(mem_row_idx);
    auto& alu_row = trace.at(alu_row_idx);

    // a + b = u8_r0 + 2^8 * u8_r1 + 2^16 * u16_r0 (mod p)
    // We recall that p-1 is a multiple of a large power of two.
    // We select a maximal u16_r0 such that u8_r0 is still of type U16.
    // Namely, we pick (p-1)/2^16 so that we can replace c with 3201 as
    // 3201 = 3200 + p - 1 (mod p)
    uint256_t const u16_r0 = (uint256_t(FF::modulus) - 1) / 65536;
    FF const fake_c = FF(3201);

    row.main_ic = fake_c;
    mem_row.mem_val = fake_c;
    alu_row.alu_ic = fake_c;

    ASSERT_EQ(alu_row.alu_u8_r0, FF(128)); // 3200 % 256 = 128
    ASSERT_EQ(alu_row.alu_u8_r1, FF(12));  // 3200/256 = 12
    ASSERT_EQ(alu_row.alu_u16_r0, 0);

    alu_row.alu_u8_r0 = FF(129); // 3201 % 256 = 129
    // alu_row.alu_u8_r1 = FF(r1); // Does not change 3201/256 = 12
    alu_row.alu_u16_r0 = FF(u16_r0);

    // We adjust counter to pass range check lookup for u8_r0
    trace.at(128).lookup_u8_0_counts -= FF(1);
    trace.at(129).lookup_u8_0_counts += FF(1);

    auto trace_same_cnt = trace;
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace_same_cnt)), "LOOKUP_U16_0");

    // Second attempt by decreasing counter for u16_r0 range check lookup
    trace.at(0).lookup_u16_0_counts -= FF(1);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "LOOKUP_U16_0");
}

// Out-of-range value in registers u16_r7, .... u16_r14
// These registers are not involved for the arithmetic
// relations of the addition but the range checks are currently
// enabled.

// U16_R7
TEST_F(AvmRangeCheckNegativeTests, additionU16Reg7)
{
    genTraceAdd(4500, 45, 4545, AvmMemoryTag::U16);
    auto trace_original = trace;

    auto& alu_row = trace.at(alu_row_idx);
    alu_row.alu_u16_r7 = FF(235655);

    auto trace_same_cnt = trace;
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace_same_cnt)), "LOOKUP_U16_7");

    // Second attempt by decreasing counter for u16_r0 range check lookup
    trace.at(1).lookup_u16_7_counts -= FF(1);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "LOOKUP_U16_7");
}

// Subsequent range checks are attempted only after counter decrease.

// U16_R8
TEST_F(AvmRangeCheckNegativeTests, additionU16Reg8)
{
    genTraceAdd(4500, 45, 4545, AvmMemoryTag::U16);
    trace.at(alu_row_idx).alu_u16_r8 = FF(235655);
    trace.at(1).lookup_u16_8_counts -= FF(1);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "LOOKUP_U16_8");
}

// U16_R9
TEST_F(AvmRangeCheckNegativeTests, additionU16Reg9)
{
    genTraceAdd(4500, 45, 4545, AvmMemoryTag::U16);
    trace.at(alu_row_idx).alu_u16_r9 = FF(235655);
    trace.at(1).lookup_u16_9_counts -= FF(1);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "LOOKUP_U16_9");
}

// U16_R10
TEST_F(AvmRangeCheckNegativeTests, additionU16Reg10)
{
    genTraceAdd(4500, 45, 4545, AvmMemoryTag::U16);
    trace.at(alu_row_idx).alu_u16_r10 = FF(235655);
    trace.at(1).lookup_u16_10_counts -= FF(1);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "LOOKUP_U16_10");
}

// U16_R11
TEST_F(AvmRangeCheckNegativeTests, additionU16Reg11)
{
    genTraceAdd(4500, 45, 4545, AvmMemoryTag::U16);
    trace.at(alu_row_idx).alu_u16_r11 = FF(235655);
    trace.at(1).lookup_u16_11_counts -= FF(1);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "LOOKUP_U16_11");
}

// U16_R12
TEST_F(AvmRangeCheckNegativeTests, additionU16Reg12)
{
    genTraceAdd(4500, 45, 4545, AvmMemoryTag::U16);
    trace.at(alu_row_idx).alu_u16_r12 = FF(235655);
    trace.at(1).lookup_u16_12_counts -= FF(1);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "LOOKUP_U16_12");
}

// U16_R13
TEST_F(AvmRangeCheckNegativeTests, additionU16Reg13)
{
    genTraceAdd(4500, 45, 4545, AvmMemoryTag::U16);
    trace.at(alu_row_idx).alu_u16_r13 = FF(235655);
    trace.at(1).lookup_u16_13_counts -= FF(1);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "LOOKUP_U16_13");
}

// U16_R14
TEST_F(AvmRangeCheckNegativeTests, additionU16Reg14)
{
    genTraceAdd(4500, 45, 4545, AvmMemoryTag::U16);
    trace.at(alu_row_idx).alu_u16_r14 = FF(235655);
    trace.at(1).lookup_u16_14_counts -= FF(1);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "LOOKUP_U16_14");
}

/******************************************************************************
 *                         MAIN <-------> MEM
 ******************************************************************************/
class AvmPermMainMemNegativeTests : public AvmInterTableTests {
  protected:
    std::vector<Row> trace;
    size_t main_row_idx;
    size_t mem_a_row_idx;
    size_t mem_b_row_idx;
    size_t mem_c_row_idx;
    size_t alu_row_idx;

    // Helper function to generate a trace with a subtraction
    // for c = a - b at arbitray chosen addresses 52 (a), 11 (b), 55 (c).
    void executeSub(uint128_t const a, uint128_t const b)
    {
        trace_builder.op_set(0, a, 52, AvmMemoryTag::U8);
        trace_builder.op_set(0, b, 11, AvmMemoryTag::U8);
        trace_builder.op_sub(0, 52, 11, 55, AvmMemoryTag::U8);
        trace_builder.op_return(0, 0, 0);

        trace = trace_builder.finalize();

        // Find the row with subtraction operation and retrieve clk.
        auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_sub == FF(1); });

        ASSERT_TRUE(row != trace.end());
        auto clk = row->main_clk;

        // Find the corresponding Alu trace row
        auto alu_row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) { return r.alu_clk == clk; });
        ASSERT_TRUE(alu_row != trace.end());

        // Find memory trace entry related to storing output (intermediate register Ic) in memory.
        auto mem_row_c = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
            return r.mem_clk == clk && r.mem_sel_op_c == FF(1) && r.mem_rw == FF(1);
        });
        ASSERT_TRUE(mem_row_c != trace.end());

        // Find memory trace entry related to loading first input (intermediate register Ia) in memory.
        auto mem_row_a = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
            return r.mem_clk == clk && r.mem_sel_op_a == FF(1) && r.mem_rw == FF(0);
        });
        ASSERT_TRUE(mem_row_a != trace.end());

        // Find memory trace entry related to loading second input (intermediate register Ib) in memory.
        auto mem_row_b = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
            return r.mem_clk == clk && r.mem_sel_op_b == FF(1) && r.mem_rw == FF(0);
        });
        ASSERT_TRUE(mem_row_b != trace.end());

        main_row_idx = static_cast<size_t>(row - trace.begin());
        alu_row_idx = static_cast<size_t>(alu_row - trace.begin());
        mem_a_row_idx = static_cast<size_t>(mem_row_a - trace.begin());
        mem_b_row_idx = static_cast<size_t>(mem_row_b - trace.begin());
        mem_c_row_idx = static_cast<size_t>(mem_row_c - trace.begin());
    }
};
// Error tag propagation from memory trace back to the main trace.
TEST_F(AvmPermMainMemNegativeTests, tagErrNotCopiedInMain)
{
    // Equality operation on U128 and second operand is of type U16.
    trace_builder.op_set(0, 32, 18, AvmMemoryTag::U128);
    trace_builder.op_set(0, 32, 76, AvmMemoryTag::U16);
    trace_builder.op_eq(0, 18, 76, 65, AvmMemoryTag::U128);
    trace_builder.halt();
    auto trace = trace_builder.finalize();

    // Find the row with equality operation and mutate the error tag.
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_eq == 1; });
    ASSERT_EQ(row->main_tag_err, FF(1)); // Sanity check that the error tag is set.
    row->main_tag_err = 0;
    row->main_sel_alu = 1; // We have to activate ALU trace if no error tag is present.
    auto const clk = row->main_clk;

    // Create a valid ALU entry for this equality operation.
    auto& alu_row = trace.at(1);
    alu_row.alu_clk = clk;
    alu_row.alu_sel_alu = 1;
    alu_row.alu_ia = 32;
    alu_row.alu_ib = 32;
    alu_row.alu_ic = 1;
    alu_row.alu_op_eq = 1;
    alu_row.alu_in_tag = static_cast<uint32_t>(AvmMemoryTag::U128);
    alu_row.alu_u128_tag = 1;

    // Adjust the output of the computation as it would have been performed without tag check.
    row->main_ic = 1;
    // Find the memory row pertaining to write operation from Ic.
    auto mem_row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
        return r.mem_tsp == FF(AvmMemTraceBuilder::NUM_SUB_CLK) * clk + AvmMemTraceBuilder::SUB_CLK_STORE_C;
    });

    // Adjust the output in the memory trace.
    mem_row->mem_val = 1;
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "INCL_MAIN_TAG_ERR");
}

TEST_F(AvmPermMainMemNegativeTests, wrongValueIaInMem)
{
    executeSub(21, 3);
    trace.at(mem_a_row_idx).mem_val = 26;     // Correct value: 21
    trace.at(mem_a_row_idx - 1).mem_val = 26; // We need to adjust the write operation beforehand (set opcode).

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "PERM_MAIN_MEM_A");
}

TEST_F(AvmPermMainMemNegativeTests, wrongValueIbInMem)
{
    executeSub(21, 3);
    trace.at(mem_b_row_idx).mem_val = 7;     // Correct value: 3
    trace.at(mem_b_row_idx - 1).mem_val = 7; // We need to adjust the write operation beforehand (set opcode).

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "PERM_MAIN_MEM_B");
}

TEST_F(AvmPermMainMemNegativeTests, wrongValueIcInMem)
{
    executeSub(21, 3);
    trace.at(mem_c_row_idx).mem_val = 17; // Correct value: 18 = 21 - 3

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "PERM_MAIN_MEM_C");
}

TEST_F(AvmPermMainMemNegativeTests, wrongAddressIaInMain)
{
    executeSub(21, 3);
    trace.at(main_row_idx).main_mem_addr_a = 28; // Correct address: 52

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "PERM_MAIN_MEM_A");
}

TEST_F(AvmPermMainMemNegativeTests, wrongAddressIbInMain)
{
    executeSub(21, 3);
    trace.at(main_row_idx).main_mem_addr_b = 2; // Correct address: 11

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "PERM_MAIN_MEM_B");
}

TEST_F(AvmPermMainMemNegativeTests, wrongAddressIcInMain)
{
    executeSub(21, 3);
    trace.at(main_row_idx).main_mem_addr_c = 75; // Correct address: 55

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "PERM_MAIN_MEM_C");
}

TEST_F(AvmPermMainMemNegativeTests, wrongInTagIaInMem)
{
    executeSub(21, 3);
    auto wrong_in_tag = static_cast<uint32_t>(AvmMemoryTag::U32);
    trace.at(mem_a_row_idx).mem_r_in_tag = wrong_in_tag; // Correct value: AvmMemoryTag::U8
    trace.at(mem_a_row_idx).mem_tag = wrong_in_tag;

    // We need to adjust the write operation beforehand (set opcode).
    trace.at(mem_a_row_idx - 1).mem_r_in_tag = wrong_in_tag;
    trace.at(mem_a_row_idx - 1).mem_w_in_tag = wrong_in_tag;
    trace.at(mem_a_row_idx - 1).mem_tag = wrong_in_tag;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "PERM_MAIN_MEM_A");
}

TEST_F(AvmPermMainMemNegativeTests, wrongInTagIbInMem)
{
    executeSub(21, 3);
    auto wrong_in_tag = static_cast<uint32_t>(AvmMemoryTag::U16);
    trace.at(mem_b_row_idx).mem_r_in_tag = wrong_in_tag; // Correct value: AvmMemoryTag::U8
    trace.at(mem_b_row_idx).mem_tag = wrong_in_tag;

    // We need to adjust the write operation beforehand (set opcode).
    trace.at(mem_b_row_idx - 1).mem_r_in_tag = wrong_in_tag;
    trace.at(mem_b_row_idx - 1).mem_w_in_tag = wrong_in_tag;
    trace.at(mem_b_row_idx - 1).mem_tag = wrong_in_tag;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "PERM_MAIN_MEM_B");
}

TEST_F(AvmPermMainMemNegativeTests, wrongInTagIcInMem)
{
    executeSub(21, 3);
    auto wrong_in_tag = static_cast<uint32_t>(AvmMemoryTag::U128);
    trace.at(mem_c_row_idx).mem_w_in_tag = wrong_in_tag; // Correct value: AvmMemoryTag::U8
    trace.at(mem_c_row_idx).mem_tag = wrong_in_tag;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "PERM_MAIN_MEM_C");
}

TEST_F(AvmPermMainMemNegativeTests, wrongRwIaInMem)
{
    executeSub(21, 3);
    trace.at(mem_a_row_idx).mem_rw = 1; // Write instead of read.

    // Adjust timestamp value
    trace.at(mem_a_row_idx).mem_tsp += FF(AvmMemTraceBuilder::SUB_CLK_STORE_A - AvmMemTraceBuilder::SUB_CLK_LOAD_A);
    // Adjust diff value of previous row as well
    FF diff = trace.at(mem_a_row_idx - 1).mem_diff_lo + trace.at(mem_a_row_idx - 1).mem_diff_mid * FF(1 << 16) +
              FF(AvmMemTraceBuilder::SUB_CLK_STORE_A - AvmMemTraceBuilder::SUB_CLK_LOAD_A);
    trace.at(mem_a_row_idx - 1).mem_diff_mid = FF(uint32_t(diff) >> 16);
    trace.at(mem_a_row_idx - 1).mem_diff_lo = FF(uint32_t(diff) & UINT16_MAX);

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "PERM_MAIN_MEM_A");
}

TEST_F(AvmPermMainMemNegativeTests, wrongRwIbInMem)
{
    executeSub(21, 3);
    trace.at(mem_b_row_idx).mem_rw = 1; // Write instead of read.

    // Adjust timestamp value
    trace.at(mem_b_row_idx).mem_tsp += FF(AvmMemTraceBuilder::SUB_CLK_STORE_B - AvmMemTraceBuilder::SUB_CLK_LOAD_B);
    // Adjust diff value of previous row as well
    FF diff = trace.at(mem_b_row_idx - 1).mem_diff_lo + trace.at(mem_b_row_idx - 1).mem_diff_mid * FF(1 << 16) +
              FF(AvmMemTraceBuilder::SUB_CLK_STORE_B - AvmMemTraceBuilder::SUB_CLK_LOAD_B);
    trace.at(mem_b_row_idx - 1).mem_diff_mid = FF(uint32_t(diff) >> 16);
    trace.at(mem_b_row_idx - 1).mem_diff_lo = FF(uint32_t(diff) & UINT16_MAX);

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "PERM_MAIN_MEM_B");
}

TEST_F(AvmPermMainMemNegativeTests, wrongRwIcInMem)
{
    // For this test, we need the result to be zero. Otherwise, swapping
    // a write for a read of Ic below leads to a violation that the memory
    // is initialized with zero values.
    executeSub(11, 11);
    trace.at(mem_c_row_idx).mem_rw = 0; // Read instead of write.

    // Adjust timestamp value.
    trace.at(mem_c_row_idx).mem_tsp -= FF(AvmMemTraceBuilder::SUB_CLK_STORE_C - AvmMemTraceBuilder::SUB_CLK_LOAD_C);

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "PERM_MAIN_MEM_C");
}

TEST_F(AvmPermMainMemNegativeTests, wrongClkIaInMem)
{
    executeSub(87, 23);
    trace.at(mem_a_row_idx).mem_clk += 3;
    trace.at(mem_a_row_idx).mem_tsp += AvmMemTraceBuilder::NUM_SUB_CLK * 3;
    // Adjust diff value of previous row as well
    FF diff = trace.at(mem_a_row_idx - 1).mem_diff_lo + trace.at(mem_a_row_idx - 1).mem_diff_mid * FF(1 << 16) +
              FF(AvmMemTraceBuilder::NUM_SUB_CLK * 3);
    trace.at(mem_a_row_idx - 1).mem_diff_mid = FF(uint32_t(diff) >> 16);
    trace.at(mem_a_row_idx - 1).mem_diff_lo = FF(uint32_t(diff) & UINT16_MAX);

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "PERM_MAIN_MEM_A");
}

TEST_F(AvmPermMainMemNegativeTests, wrongClkIbInMem)
{
    executeSub(87, 23);
    trace.at(mem_b_row_idx).mem_clk += 5;
    trace.at(mem_b_row_idx).mem_tsp += AvmMemTraceBuilder::NUM_SUB_CLK * 5;
    FF diff = trace.at(mem_b_row_idx - 1).mem_diff_lo + trace.at(mem_b_row_idx - 1).mem_diff_mid * FF(1 << 16) +
              FF(AvmMemTraceBuilder::NUM_SUB_CLK * 5);
    trace.at(mem_b_row_idx - 1).mem_diff_mid = FF(uint32_t(diff) >> 16);
    trace.at(mem_b_row_idx - 1).mem_diff_lo = FF(uint32_t(diff) & UINT16_MAX);

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "PERM_MAIN_MEM_B");
}

TEST_F(AvmPermMainMemNegativeTests, wrongClkIcInMem)
{
    executeSub(87, 23);
    trace.at(mem_c_row_idx).mem_clk += 7;
    trace.at(mem_c_row_idx).mem_tsp += AvmMemTraceBuilder::NUM_SUB_CLK * 7;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "PERM_MAIN_MEM_C");
}

} // namespace tests_avm
