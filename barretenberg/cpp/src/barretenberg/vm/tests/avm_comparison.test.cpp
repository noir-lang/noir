#include "avm_common.test.hpp"
#include "barretenberg/common/zip_view.hpp"
#include "barretenberg/numeric/uint128/uint128.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/tests/helpers.test.hpp"
#include "gtest/gtest.h"
#include <algorithm>
#include <array>
#include <cstdint>
#include <iterator>
#include <ranges>
#include <sys/types.h>
#include <tuple>
#include <vector>

namespace tests_avm {
using namespace bb::avm_trace;
namespace {

void common_validate_cmp(Row const& row,
                         Row const& alu_row,
                         FF const& a,
                         FF const& b,
                         FF const& c,
                         FF const& addr_a,
                         FF const& addr_b,
                         FF const& addr_c,
                         avm_trace::AvmMemoryTag const tag)
{

    // Use the row in the main trace to find the same operation in the alu trace.
    // Check that the correct result is stored at the expected memory location.
    EXPECT_EQ(row.avm_main_ic, c);
    EXPECT_EQ(row.avm_main_mem_idx_c, addr_c);
    EXPECT_EQ(row.avm_main_mem_op_c, FF(1));
    EXPECT_EQ(row.avm_main_rwc, FF(1));

    // Check that ia register is correctly set with memory load operations.
    EXPECT_EQ(row.avm_main_ia, a);
    EXPECT_EQ(row.avm_main_mem_idx_a, addr_a);
    EXPECT_EQ(row.avm_main_mem_op_a, FF(1));
    EXPECT_EQ(row.avm_main_rwa, FF(0));

    // Check that ib register is correctly set with memory load operations.
    EXPECT_EQ(row.avm_main_ib, b);
    EXPECT_EQ(row.avm_main_mem_idx_b, addr_b);
    EXPECT_EQ(row.avm_main_mem_op_b, FF(1));
    EXPECT_EQ(row.avm_main_rwb, FF(0));

    // Check the instruction tags
    EXPECT_EQ(row.avm_main_r_in_tag, FF(static_cast<uint32_t>(tag)));
    EXPECT_EQ(row.avm_main_w_in_tag, FF(static_cast<uint32_t>(AvmMemoryTag::U8)));

    // Check that intermediate registers are correctly copied in Alu trace
    EXPECT_EQ(alu_row.avm_alu_ia, a);
    EXPECT_EQ(alu_row.avm_alu_ib, b);
    EXPECT_EQ(alu_row.avm_alu_ic, c);
}
} // namespace
using ThreeOpParam = std::array<FF, 3>;
using ThreeOpParamRow = std::tuple<ThreeOpParam, AvmMemoryTag>;
std::vector<ThreeOpParam> positive_op_lt_test_values = { { { FF(1), FF(1), FF(0) },
                                                           { FF(5323), FF(321), FF(0) },
                                                           { FF(13793), FF(10590617LLU), FF(1) },
                                                           { FF(0x7bff744e3cdf79LLU), FF(0x14ccccccccb6LLU), FF(0) },
                                                           { FF(uint256_t{ 0xb900000000000001 }),
                                                             FF(uint256_t{ 0x1006021301080000 } << 64) +
                                                                 uint256_t{ 0x000000000000001080876844827 },
                                                             1 } } };
std::vector<ThreeOpParam> positive_op_lte_test_values = {
    { { FF(1), FF(1), FF(1) },
      { FF(5323), FF(321), FF(0) },
      { FF(13793), FF(10590617LLU), FF(1) },
      { FF(0x7bff744e3cdf79LLU), FF(0x14ccccccccb6LLU), FF(0) },
      { FF(uint256_t{ 0x1006021301080000 } << 64) + uint256_t{ 0x000000000000001080876844827 },
        FF(uint256_t{ 0x1006021301080000 } << 64) + uint256_t{ 0x000000000000001080876844827 },
        FF(1) } }
};

std::vector<ThreeOpParamRow> gen_three_op_params(std::vector<ThreeOpParam> operands,
                                                 std::vector<AvmMemoryTag> mem_tag_arr)
{
    std::vector<ThreeOpParamRow> params;
    for (size_t i = 0; i < 5; i++) {
        params.emplace_back(operands[i], mem_tag_arr[i]);
    }
    return params;
}
std::vector<AvmMemoryTag> mem_tag_arr{
    { AvmMemoryTag::U8, AvmMemoryTag::U16, AvmMemoryTag::U32, AvmMemoryTag::U64, AvmMemoryTag::U128 }
};
class AvmCmpTests : public ::testing::Test {
  public:
    AvmTraceBuilder trace_builder;

  protected:
    // TODO(640): The Standard Honk on Grumpkin test suite fails unless the SRS is initialised for every test.
    void SetUp() override { srs::init_crs_factory("../srs_db/ignition"); };
};
class AvmCmpTestsLT : public AvmCmpTests, public testing::WithParamInterface<ThreeOpParamRow> {};
class AvmCmpTestsLTE : public AvmCmpTests, public testing::WithParamInterface<ThreeOpParamRow> {};

/******************************************************************************
 *
 * POSITIVE TESTS
 *
 ******************************************************************************/
TEST_P(AvmCmpTestsLT, ParamTest)
{
    const auto [params, mem_tag] = GetParam();
    const auto [a, b, c] = params;
    if (mem_tag == AvmMemoryTag::FF) {
        trace_builder.calldata_copy(0, 0, 2, 0, std::vector<FF>{ a, b });
    } else {
        trace_builder.op_set(0, uint128_t(a), 0, mem_tag);
        trace_builder.op_set(0, uint128_t(b), 1, mem_tag);
    }
    trace_builder.op_lt(0, 0, 1, 2, mem_tag);
    trace_builder.return_op(0, 0, 0);
    auto trace = trace_builder.finalize();

    // Get the row in the avm with the LT selector set
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_lt == FF(1); });

    // Use the row in the main trace to find the same operation in the alu trace.
    FF clk = row->avm_main_clk;
    auto alu_row = std::ranges::find_if(
        trace.begin(), trace.end(), [clk](Row r) { return r.avm_alu_clk == clk && r.avm_alu_op_lt == FF(1); });
    // Check that both rows were found
    ASSERT_TRUE(row != trace.end());
    ASSERT_TRUE(alu_row != trace.end());
    common_validate_cmp(*row, *alu_row, a, b, c, FF(0), FF(1), FF(2), mem_tag);

    validate_trace(std::move(trace));
}
INSTANTIATE_TEST_SUITE_P(AvmCmpTests,
                         AvmCmpTestsLT,
                         testing::ValuesIn(gen_three_op_params(positive_op_lt_test_values, mem_tag_arr)));

TEST_P(AvmCmpTestsLTE, ParamTest)
{
    const auto [params, mem_tag] = GetParam();
    const auto [a, b, c] = params;
    if (mem_tag == AvmMemoryTag::FF) {
        trace_builder.calldata_copy(0, 0, 2, 0, std::vector<FF>{ a, b });
    } else {
        trace_builder.op_set(0, uint128_t(a), 0, mem_tag);
        trace_builder.op_set(0, uint128_t(b), 1, mem_tag);
    }
    trace_builder.op_lte(0, 0, 1, 2, mem_tag);
    trace_builder.return_op(0, 0, 0);
    auto trace = trace_builder.finalize();
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_lte == FF(1); });

    // Use the row in the main trace to find the same operation in the alu trace.
    FF clk = row->avm_main_clk;
    auto alu_row = std::ranges::find_if(
        trace.begin(), trace.end(), [clk](Row r) { return r.avm_alu_clk == clk && r.avm_alu_op_lte; });
    // Check that both rows were found
    ASSERT_TRUE(row != trace.end());
    ASSERT_TRUE(alu_row != trace.end());
    common_validate_cmp(*row, *alu_row, a, b, c, FF(0), FF(1), FF(2), mem_tag);
    validate_trace(std::move(trace));
}
INSTANTIATE_TEST_SUITE_P(AvmCmpTests,
                         AvmCmpTestsLTE,
                         testing::ValuesIn(gen_three_op_params(positive_op_lte_test_values, mem_tag_arr)));

/******************************************************************************
 *
 * NEGATIVE TESTS
 *
 ******************************************************************************/
enum CMP_FAILURES {
    IncorrectInputDecomposition,
    SubLoCheckFailed,
    ResLoCheckFailed,
    ResHiCheckFailed,
    CounterRelationFailed,
    CounterNonZeroCheckFailed,
    ShiftRelationFailed,
    RangeCheckFailed,
};
std::vector<std::tuple<std::string, CMP_FAILURES>> cmp_failures = {
    { "INPUT_DECOMP_1", CMP_FAILURES::IncorrectInputDecomposition },
    { "SUB_LO_1", CMP_FAILURES::SubLoCheckFailed },
    { "RES_LO", CMP_FAILURES::ResLoCheckFailed },
    { "RES_HI", CMP_FAILURES::ResHiCheckFailed },
    { "CMP_CTR_REL_2", CMP_FAILURES::CounterRelationFailed },
    { "CTR_NON_ZERO_REL", CMP_FAILURES::CounterNonZeroCheckFailed },
    { "SHIFT_RELS_0", CMP_FAILURES::ShiftRelationFailed },
    { "LOOKUP_U16_0", CMP_FAILURES::RangeCheckFailed },

};
std::vector<ThreeOpParam> neg_test_lt = { { FF::modulus - 1, FF::modulus_minus_two, 0 } };
std::vector<ThreeOpParam> neg_test_lte = { { FF::modulus - 1, FF::modulus - 1, 0 } };

using EXPECTED_ERRORS = std::tuple<std::string, CMP_FAILURES>;

std::vector<Row> gen_mutated_trace_cmp(
    std::vector<Row> trace, std::function<bool(Row)> select_row, FF c_mutated, CMP_FAILURES fail_mode, bool is_lte)
{
    auto main_trace_row = std::ranges::find_if(trace.begin(), trace.end(), select_row);
    auto main_clk = main_trace_row->avm_main_clk;
    // The corresponding row in the alu trace as well as the row where start = 1
    auto alu_row =
        std::ranges::find_if(trace.begin(), trace.end(), [main_clk](Row r) { return r.avm_alu_clk == main_clk; });
    // The corresponding row in the alu trace where the computation ends.
    auto range_check_row =
        std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_alu_cmp_rng_ctr > FF(0); });
    switch (fail_mode) {
    case IncorrectInputDecomposition:
        alu_row->avm_alu_a_lo = alu_row->avm_alu_a_lo + FF(1);
        break;
    case SubLoCheckFailed:
        alu_row->avm_alu_p_a_borrow = FF::one() - alu_row->avm_alu_p_a_borrow;
        break;
    case ResLoCheckFailed:
        alu_row->avm_alu_res_lo = alu_row->avm_alu_res_lo - FF(1);
        break;
    case ResHiCheckFailed:
        alu_row->avm_alu_res_hi = FF(1);
        break;
    case CounterRelationFailed:
        range_check_row->avm_alu_cmp_rng_ctr = FF(0);
        break;
    case CounterNonZeroCheckFailed:
        range_check_row->avm_alu_rng_chk_sel = FF(0);
        range_check_row->avm_alu_rng_chk_lookup_selector = FF(0);
        break;
    case ShiftRelationFailed:
        range_check_row->avm_alu_a_lo = range_check_row->avm_alu_res_lo;
        range_check_row->avm_alu_a_hi = range_check_row->avm_alu_res_hi;
        break;
    case RangeCheckFailed: // Canonicalisation check failure
        // TODO: We can probably refactor this to another function later as it is a bit verbose
        // and we'll probably use it repeatedly for other range check test.

        // The range check fails in the context of the cmp operation if we set the boolean
        // result in ic to be incorrect.
        // Here we falsely claim LT(12,023, 439,321, 0). i.e. 12023 < 439321 is false.
        mutate_ic_in_trace(trace, std::move(select_row), c_mutated, true);

        // Now we have to also update the value of res_lo = (A_SUB_B_LO * IS_GT + B_SUB_A_LO * (1 - IS_GT))
        alu_row->avm_alu_borrow = FF(0);
        FF mutated_res_lo =
            alu_row->avm_alu_b_lo - alu_row->avm_alu_a_lo + alu_row->avm_alu_borrow * (uint256_t(1) << 128);
        FF mutated_res_hi = alu_row->avm_alu_b_hi - alu_row->avm_alu_a_hi - alu_row->avm_alu_borrow;

        if (is_lte) {
            mutated_res_lo = alu_row->avm_alu_a_lo - alu_row->avm_alu_b_lo - FF::one() +
                             alu_row->avm_alu_borrow * (uint256_t(1) << 128);
            mutated_res_hi = alu_row->avm_alu_a_hi - alu_row->avm_alu_b_hi - alu_row->avm_alu_borrow;
        }
        alu_row->avm_alu_res_lo = mutated_res_lo;
        alu_row->avm_alu_res_hi = mutated_res_hi;
        // For each subsequent row that involve the range check, we need to update the shifted values
        auto next_row = alu_row + 1;
        next_row->avm_alu_p_sub_b_lo = mutated_res_lo;
        next_row->avm_alu_p_sub_b_hi = mutated_res_hi;

        next_row = alu_row + 2;
        next_row->avm_alu_p_sub_a_lo = mutated_res_lo;
        next_row->avm_alu_p_sub_a_hi = mutated_res_hi;
        next_row = alu_row + 3;

        next_row->avm_alu_b_lo = mutated_res_lo;
        next_row->avm_alu_b_hi = mutated_res_hi;

        // The final row contains the mutated res_x values at the a_x slots that will be range check.
        auto final_row = alu_row + 4;
        // To prevent a trivial range check failure, we need to clear the lookup counters for the
        // current value of res_lo stored in a_lo
        clear_range_check_counters(trace, final_row->avm_alu_a_lo);
        final_row->avm_alu_a_lo = mutated_res_lo;
        final_row->avm_alu_a_hi = mutated_res_hi;

        uint256_t mutated_res_lo_u256 = mutated_res_lo;
        // We update range check lookup counters and the registers here

        // Assign the new u8 value that goes into the first slice register.
        final_row->avm_alu_u8_r0 = static_cast<uint8_t>(mutated_res_lo_u256);
        // Find the main row where the new u8 value in the first register WILL be looked up
        auto new_lookup_row = std::ranges::find_if(trace.begin(), trace.end(), [final_row](Row r) {
            return r.avm_main_clk == final_row->avm_alu_u8_r0 && r.avm_main_sel_rng_8 == FF(1);
        });
        // Increment the counter
        new_lookup_row->lookup_u8_0_counts = new_lookup_row->lookup_u8_0_counts + 1;
        mutated_res_lo_u256 >>= 8;

        // Assign the new u8 value that goes into the second slice register.
        final_row->avm_alu_u8_r1 = static_cast<uint8_t>(mutated_res_lo_u256);
        new_lookup_row = std::ranges::find_if(trace.begin(), trace.end(), [final_row](Row r) {
            return r.avm_main_clk == final_row->avm_alu_u8_r1 && r.avm_main_sel_rng_8 == FF(1);
        });
        new_lookup_row->lookup_u8_1_counts = new_lookup_row->lookup_u8_1_counts + 1;
        mutated_res_lo_u256 >>= 8;

        // Set the remaining bits (that are > 16) to the first u16 register to trigger the overflow
        final_row->avm_alu_u16_r0 = mutated_res_lo_u256;

        break;
    }
    return trace;
}
class AvmCmpNegativeTestsLT : public AvmCmpTests,
                              public testing::WithParamInterface<std::tuple<EXPECTED_ERRORS, ThreeOpParam>> {};
class AvmCmpNegativeTestsLTE : public AvmCmpTests,
                               public testing::WithParamInterface<std::tuple<EXPECTED_ERRORS, ThreeOpParam>> {};

TEST_P(AvmCmpNegativeTestsLT, ParamTest)
{
    const auto [failure, params] = GetParam();
    const auto [failure_string, failure_mode] = failure;
    const auto [a, b, output] = params;
    auto trace_builder = avm_trace::AvmTraceBuilder();
    trace_builder.calldata_copy(0, 0, 3, 0, std::vector<FF>{ a, b, output });
    trace_builder.op_lt(0, 0, 1, 2, AvmMemoryTag::FF);
    trace_builder.return_op(0, 0, 0);
    auto trace = trace_builder.finalize();
    std::function<bool(Row)> select_row = [](Row r) { return r.avm_main_sel_op_lt == FF(1); };
    trace = gen_mutated_trace_cmp(trace, select_row, output, failure_mode, false);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), failure_string);
}

INSTANTIATE_TEST_SUITE_P(AvmCmpNegativeTests,
                         AvmCmpNegativeTestsLT,
                         testing::Combine(testing::ValuesIn(cmp_failures), testing::ValuesIn(neg_test_lt)));

TEST_P(AvmCmpNegativeTestsLTE, ParamTest)
{
    const auto [failure, params] = GetParam();
    const auto [failure_string, failure_mode] = failure;
    const auto [a, b, output] = params;
    auto trace_builder = avm_trace::AvmTraceBuilder();
    trace_builder.calldata_copy(0, 0, 3, 0, std::vector<FF>{ a, b, output });
    trace_builder.op_lte(0, 0, 1, 2, AvmMemoryTag::FF);
    trace_builder.return_op(0, 0, 0);
    auto trace = trace_builder.finalize();
    std::function<bool(Row)> select_row = [](Row r) { return r.avm_main_sel_op_lte == FF(1); };
    trace = gen_mutated_trace_cmp(trace, select_row, output, failure_mode, true);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), failure_string);
}
INSTANTIATE_TEST_SUITE_P(AvmCmpNegativeTests,
                         AvmCmpNegativeTestsLTE,
                         testing::Combine(testing::ValuesIn(cmp_failures), testing::ValuesIn(neg_test_lte)));
} // namespace tests_avm
