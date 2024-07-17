#include "avm_common.test.hpp"
#include "barretenberg/numeric/uint128/uint128.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/tests/helpers.test.hpp"
#include "gtest/gtest.h"
#include <algorithm>
#include <array>
#include <cstdint>
#include <iterator>
#include <ranges>
#include <tuple>
#include <vector>

namespace tests_avm {

using namespace bb;
using namespace bb::avm_trace;

namespace {

void common_validate_op_not(std::vector<Row> const& trace,
                            FF const& a,
                            FF const& c,
                            FF const& addr_a,
                            FF const& addr_c,
                            avm_trace::AvmMemoryTag const tag)
{

    // Find the first row enabling the not selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_not == FF(1); });

    // Use the row in the main trace to find the same operation in the alu trace.
    FF clk = row->main_clk;
    auto alu_row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) { return r.alu_clk == clk; });

    // Check that both rows were found
    EXPECT_TRUE(row != trace.end());
    EXPECT_TRUE(alu_row != trace.end());

    // Check that the correct result is stored at the expected memory location.
    EXPECT_EQ(row->main_ic, c);
    EXPECT_EQ(row->main_mem_addr_c, addr_c);
    EXPECT_EQ(row->main_sel_mem_op_c, FF(1));
    EXPECT_EQ(row->main_rwc, FF(1));

    // Check that ia register is correctly set with memory load operations.
    EXPECT_EQ(row->main_ia, a);
    EXPECT_EQ(row->main_mem_addr_a, addr_a);
    EXPECT_EQ(row->main_sel_mem_op_a, FF(1));
    EXPECT_EQ(row->main_rwa, FF(0));

    // Check the instruction tags
    EXPECT_EQ(row->main_r_in_tag, FF(static_cast<uint32_t>(tag)));
    EXPECT_EQ(row->main_w_in_tag, FF(static_cast<uint32_t>(tag)));

    // Check that intermediate registers are correctly copied in Alu trace
    EXPECT_EQ(alu_row->alu_ia, a);
    EXPECT_EQ(alu_row->alu_ib, FF(0));
    EXPECT_EQ(alu_row->alu_ic, c);

    // Check that not selector is set.
    EXPECT_EQ(row->main_sel_op_not, FF(1));
    EXPECT_EQ(alu_row->alu_op_not, FF(1));
    switch (tag) {
    // Handle the different mem_tags here since this is part of a
    // parameterised test
    case AvmMemoryTag::U0:
        FAIL() << "Unintialized Mem Tags Disallowed";
        break;
    case AvmMemoryTag::U8:
        EXPECT_EQ(alu_row->alu_u8_tag, FF(1));
        break;
    case AvmMemoryTag::U16:
        EXPECT_EQ(alu_row->alu_u16_tag, FF(1));
        break;
    case AvmMemoryTag::U32:
        EXPECT_EQ(alu_row->alu_u32_tag, FF(1));
        break;
    case AvmMemoryTag::U64:
        EXPECT_EQ(alu_row->alu_u64_tag, FF(1));
        break;
    case AvmMemoryTag::U128:
        EXPECT_EQ(alu_row->alu_u128_tag, FF(1));
        break;
    case AvmMemoryTag::FF:
        FAIL() << "FF Mem Tags Disallowed for bitwise";
        break;
    }
}

void common_validate_shift_op(std::vector<Row> const& trace,
                              FF const& a,
                              FF const& b,
                              FF const& c,
                              FF const& addr_a,
                              FF const& addr_b,
                              FF const& addr_c,
                              avm_trace::AvmMemoryTag const tag,
                              bool shr)
{
    auto row = shr ? std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_shr == FF(1); })
                   : std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_shl == FF(1); });
    ASSERT_TRUE(row != trace.end());
    FF clk = row->main_clk;
    auto alu_row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) { return r.alu_clk == clk; });
    ASSERT_TRUE(alu_row != trace.end());

    // Check that the correct result is stored at the expected memory location.
    EXPECT_EQ(row->main_ic, c);
    EXPECT_EQ(row->main_mem_addr_c, addr_c);
    EXPECT_EQ(row->main_sel_mem_op_c, FF(1));
    EXPECT_EQ(row->main_rwc, FF(1));

    // Check that ia register is correctly set with memory load operations.
    EXPECT_EQ(row->main_ia, a);
    EXPECT_EQ(row->main_mem_addr_a, addr_a);
    EXPECT_EQ(row->main_sel_mem_op_a, FF(1));
    EXPECT_EQ(row->main_rwa, FF(0));

    // Check that ib register is correctly set with memory load operations.
    EXPECT_EQ(row->main_ib, b);
    EXPECT_EQ(row->main_mem_addr_b, addr_b);
    EXPECT_EQ(row->main_sel_mem_op_b, FF(1));
    EXPECT_EQ(row->main_rwb, FF(0));

    // Check the instruction tags
    EXPECT_EQ(row->main_r_in_tag, FF(static_cast<uint32_t>(tag)));
    EXPECT_EQ(row->main_w_in_tag, FF(static_cast<uint32_t>(tag)));

    // Check that start row is the same as what is copied into the main trace
    EXPECT_EQ(alu_row->alu_ia, a);
    EXPECT_EQ(alu_row->alu_ib, b);
    EXPECT_EQ(alu_row->alu_ic, c);
}

void common_validate_bit_op(std::vector<Row> const& trace,
                            uint8_t op_id,
                            FF const& a,
                            FF const& b,
                            FF const& c,
                            FF const& addr_a,
                            FF const& addr_b,
                            FF const& addr_c,
                            avm_trace::AvmMemoryTag const tag)
{

    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_xor == FF(1); });
    if (op_id == 0) {
        row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_and == FF(1); });
    } else if (op_id == 1) {
        row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_or == FF(1); });
    }

    // Use the row in the main trace to find the same operation in the alu trace.
    FF clk = row->main_clk;
    auto bin_row_start = std::ranges::find_if(
        trace.begin(), trace.end(), [clk](Row r) { return r.binary_clk == clk && r.binary_start == FF(1); });

    // Check that both rows were found
    ASSERT_TRUE(bin_row_start != trace.end());
    ASSERT_TRUE(row != trace.end());

    // Check that the correct result is stored at the expected memory location.
    EXPECT_EQ(row->main_ic, c);
    EXPECT_EQ(row->main_mem_addr_c, addr_c);
    EXPECT_EQ(row->main_sel_mem_op_c, FF(1));
    EXPECT_EQ(row->main_rwc, FF(1));

    // Check that ia register is correctly set with memory load operations.
    EXPECT_EQ(row->main_ia, a);
    EXPECT_EQ(row->main_mem_addr_a, addr_a);
    EXPECT_EQ(row->main_sel_mem_op_a, FF(1));
    EXPECT_EQ(row->main_rwa, FF(0));

    // Check that ia register is correctly set with memory load operations.
    EXPECT_EQ(row->main_ib, b);
    EXPECT_EQ(row->main_mem_addr_b, addr_b);
    EXPECT_EQ(row->main_sel_mem_op_b, FF(1));
    EXPECT_EQ(row->main_rwb, FF(0));

    // Check the instruction tags
    EXPECT_EQ(row->main_r_in_tag, FF(static_cast<uint32_t>(tag)));
    EXPECT_EQ(row->main_w_in_tag, FF(static_cast<uint32_t>(tag)));

    // Check that start row is the same as what is copied into the main trace
    EXPECT_EQ(bin_row_start->binary_acc_ia, a);
    EXPECT_EQ(bin_row_start->binary_acc_ib, b);
    EXPECT_EQ(bin_row_start->binary_acc_ic, c);

    EXPECT_EQ(bin_row_start->binary_op_id, op_id);
    EXPECT_EQ(bin_row_start->binary_sel_bin, FF(1));
    EXPECT_EQ(bin_row_start->binary_in_tag, static_cast<uint8_t>(tag));
}

// These are the potential failures we handle for the negative tests involving the binary trace.
enum BIT_FAILURES {
    BitDecomposition,
    MemTagCtr,
    IncorrectAcc,
    InconsistentOpId,
    ByteLookupError,
    ByteLengthError,
    IncorrectBinSelector,
};

enum SHIFT_FAILURES {
    IncorrectShiftPastBitLength, // Incorrect Setting shift_lt_bit_len
    IncorrectInputDecomposition,
    ShiftOutputIncorrect,
};

std::tuple<std::vector<Row>, std::string> gen_mutated_trace_shift(std::vector<Row> trace,
                                                                  std::function<bool(Row)>&& select_row,
                                                                  FF const& c_mutated,
                                                                  SHIFT_FAILURES fail_mode,
                                                                  bool shr = true)
{
    auto main_trace_row = std::ranges::find_if(trace.begin(), trace.end(), select_row);
    auto main_clk = main_trace_row->main_clk;
    // The corresponding row in the alu trace as well as the row where start = 1
    auto alu_row =
        std::ranges::find_if(trace.begin(), trace.end(), [main_clk](Row r) { return r.alu_clk == main_clk; });

    std::string failure;
    switch (fail_mode) {
    case IncorrectShiftPastBitLength:
        alu_row->alu_shift_lt_bit_len = FF(0);
        update_slice_registers(*alu_row, uint256_t{ 0 });
        alu_row->alu_a_lo = FF(0);
        alu_row->alu_a_hi = FF(0);
        failure = "SHIFT_LT_BIT_LEN";
        return std::make_tuple(trace, failure);
    case IncorrectInputDecomposition: {
        // Subtrace one from b_lo and update b_lo
        uint256_t b_lo = alu_row->alu_b_lo - 1;
        uint256_t b_hi = alu_row->alu_b_hi;
        alu_row->alu_b_lo = b_lo;

        // Update the range checks involving b_lo and b_hi so we dont throw an error about the range checks
        if (shr) {
            uint256_t a_lo = (uint256_t(1) << alu_row->alu_ib) - b_lo - 1;
            uint256_t a_hi = (uint256_t(1) << (32 - uint8_t(alu_row->alu_ib))) - b_hi - 1;
            alu_row->alu_a_lo = a_lo & ((uint256_t(1) << 128) - 1);
            alu_row->alu_a_hi = a_hi;
            // Update slice registers
            update_slice_registers(*alu_row, a_lo + (a_hi << 128));
            failure = "SHR_INPUT_DECOMPOSITION";
            return std::make_tuple(trace, failure);
        }
        uint256_t a_lo = (uint256_t(1) << (32 - uint8_t(alu_row->alu_ib))) - b_lo - 1;
        uint256_t a_hi = (uint256_t(1) << alu_row->alu_ib) - b_hi - 1;
        alu_row->alu_a_lo = a_lo & ((uint256_t(1) << 128) - 1);
        alu_row->alu_a_hi = a_hi;
        // Update slice registers
        update_slice_registers(*alu_row, a_lo + (a_hi << 128));
        failure = "SHL_INPUT_DECOMPOSITION";
        return std::make_tuple(trace, failure);
    }
    case ShiftOutputIncorrect:
        alu_row->alu_ic = c_mutated;
        failure = shr ? "SHR_OUTPUT" : "SHL_OUTPUT";
        return std::make_tuple(trace, failure);
    }
    return std::make_tuple(trace, failure);
}
std::vector<Row> gen_mutated_trace_bit(std::vector<Row> trace,
                                       std::function<bool(Row)>&& select_row,
                                       FF const& c_mutated,
                                       BIT_FAILURES fail_mode)
{
    auto main_trace_row = std::ranges::find_if(trace.begin(), trace.end(), select_row);
    auto main_clk = main_trace_row->main_clk;
    // The corresponding row in the binary trace as well as the row where start = 1
    auto binary_row =
        std::ranges::find_if(trace.begin(), trace.end(), [main_clk](Row r) { return r.binary_clk == main_clk; });
    // The corresponding row in the binary trace where the computation ends.
    auto last_row = std::ranges::find_if(trace.begin(), trace.end(), [main_clk](Row r) {
        return r.binary_clk == main_clk && r.binary_mem_tag_ctr == FF(0);
    });
    switch (fail_mode) {
    case BitDecomposition: {
        // Incrementing the bytes should indicate an incorrect decomposition
        // The lookups are checked later so this will throw an error about decomposition
        binary_row->binary_ic_bytes++;
        break;
    }
    case MemTagCtr: {
        // Increment instead of decrementing
        binary_row->binary_mem_tag_ctr++;
        break;
    }
    case IncorrectAcc: {
        // The lookups are checked later so this will throw an error about accumulation
        binary_row->binary_acc_ic++;
        break;
    }
    case InconsistentOpId: {
        // We don't update the first index as that is checked by the permutation check.
        // So we update the next op_id to be incorrect.
        auto first_index = static_cast<size_t>(std::distance(trace.begin(), binary_row));
        trace.at(first_index + 1).binary_op_id++;
        break;
    }
    case ByteLookupError: {
        // Update the trace to be the mutated value, which also (conveniently)
        // fits into a byte so we can also update the ic_byte decomposition.
        // We intentionally select the mutated value to be 0-bytes everywhere else so we dont need to
        // update anything there or in the corresponding accumulators.
        mutate_ic_in_trace(trace, std::move(select_row), c_mutated, false);
        binary_row->binary_acc_ic = c_mutated;
        binary_row->binary_ic_bytes = static_cast<uint8_t>(uint128_t{ c_mutated });
        break;
    }
    case ByteLengthError: {
        // To trigger this error, we need to start the mem_tag ctr to be incorrect (one less than is should be)
        // However, to avoid the MEM_REL_TAG error from happening instead, we need to ensure we update the mem_tag
        // of all rows between the start = 1 and mem_tag = 0;
        auto last_index = static_cast<size_t>(std::distance(trace.begin(), last_row));
        auto first_index = static_cast<size_t>(std::distance(trace.begin(), binary_row));
        for (size_t i = first_index; i <= last_index; i++) {
            FF ctr = trace.at(i).binary_mem_tag_ctr;
            if (ctr == FF::one()) {
                // If the tag is currently 1, it will be set to 0 which means we need to set bin_sel to 0.
                trace.at(i).binary_sel_bin = FF(0);
                trace.at(i).binary_mem_tag_ctr = FF(0);
                trace.at(i).binary_mem_tag_ctr_inv = FF(0);
            } else if (ctr == FF::zero()) {
                // Leave as zero instead of underflowing
                trace.at(i).binary_mem_tag_ctr = FF(0);
            } else {
                // Replace the values with the next row's values
                trace.at(i).binary_mem_tag_ctr = trace.at(i + 1).binary_mem_tag_ctr;
                trace.at(i).binary_mem_tag_ctr_inv = trace.at(i + 1).binary_mem_tag_ctr_inv;
                trace.at(i).binary_sel_bin = trace.at(i + 1).binary_sel_bin;
            }
        }
        break;
    }
    case IncorrectBinSelector:
        binary_row->binary_sel_bin = FF(0);
        break;
    }
    return trace;
}
} // namespace

class AvmBitwiseTests : public ::testing::Test {
  public:
    AvmBitwiseTests()
        : public_inputs(generate_base_public_inputs())
        , trace_builder(AvmTraceBuilder(public_inputs))
    {
        srs::init_crs_factory("../srs_db/ignition");
    }

    VmPublicInputs public_inputs;
    AvmTraceBuilder trace_builder;

    std::vector<Row> gen_mutated_trace_not(FF const& a, FF const& c_mutated, avm_trace::AvmMemoryTag tag)
    {
        trace_builder.op_set(0, uint128_t{ a }, 0, tag);
        trace_builder.op_not(0, 0, 1, tag);
        trace_builder.halt();
        auto trace = trace_builder.finalize();

        auto select_row = [](Row r) { return r.main_sel_op_not == FF(1); };
        mutate_ic_in_trace(trace, select_row, c_mutated, true);

        return trace;
    }
};

/******************************************************************************
 *
 * Helpers to set up Test Params
 *
 ******************************************************************************/

using TwoOpParamRow = std::tuple<std::array<uint128_t, 2>, AvmMemoryTag>;
std::vector<AvmMemoryTag> mem_tags{
    { AvmMemoryTag::U8, AvmMemoryTag::U16, AvmMemoryTag::U32, AvmMemoryTag::U64, AvmMemoryTag::U128 }
};

std::vector<std::array<uint128_t, 2>> positive_op_not_test_values = { { { 1, 254 },
                                                                        { 512, 65'023 },
                                                                        { 131'072, 4'294'836'223LLU },
                                                                        { 0x100000000LLU, 0xfffffffeffffffffLLU },
                                                                        { uint128_t{ 0x4000000000000 } << 64,
                                                                          (uint128_t{ 0xfffbffffffffffff } << 64) +
                                                                              uint128_t{ 0xffffffffffffffff } } } };

// This is essentially a zip while we wait for C++23
std::vector<TwoOpParamRow> gen_two_op_params(std::vector<std::array<uint128_t, 2>> operands,
                                             std::vector<AvmMemoryTag> mem_tags)
{
    std::vector<TwoOpParamRow> params;
    for (size_t i = 0; i < 5; i++) {
        params.emplace_back(operands[i], mem_tags[i]);
    }
    return params;
}

std::vector<ThreeOpParam> positive_op_and_test_values = {
    { { FF(1), FF(1), FF(1) },
      { FF(5323), FF(321), FF(65) },
      { FF(13793), FF(10590617LLU), FF(4481) },
      { FF(0x7bff744e3cdf79LLU), FF(0x14ccccccccb6LLU), FF(0x14444c0ccc30LLU) },
      { uint256_t::from_uint128(uint128_t{ 0xb900000000000001 } << 64),
        uint256_t::from_uint128((uint128_t{ 0x1006021301080000 } << 64) + uint128_t{ 0x000000000000001080876844827 }),
        uint256_t::from_uint128(uint128_t{ 0x1000000000000000 } << 64) } }
};

std::vector<std::array<FF, 3>> positive_op_or_test_values = {
    { { FF(1), FF(1), FF(1) },
      { FF(5323), FF(321), FF(0x15cb) },
      { FF(13793), FF(10590617LLU), FF(0xa1bdf9) },
      { FF(0x7bff744e3cdf79LLU), FF(0x14ccccccccb6LLU), FF(0x7bfffccefcdfffLLU) },
      { uint256_t::from_uint128(uint128_t{ 0xb900000000000000 } << 64),
        uint256_t::from_uint128(uint128_t{ 0x1006021301080000 } << 64) + uint128_t{ 0x000000000000001080876844827 },
        uint256_t::from_uint128(uint128_t{ 0xb906021301080000 } << 64) + uint128_t{ 0x0001080876844827 } } }
};
std::vector<std::array<FF, 3>> positive_op_xor_test_values = {
    { { FF(1), FF(1), FF(0) },
      { FF(5323), FF(321), FF(0x158a) },
      { FF(13793), FF(10590617LLU), FF(0xa1ac78) },
      { FF(0x7bff744e3cdf79LLU), FF(0x14ccccccccb6LLU), uint256_t::from_uint128(0x7bebb882f013cf) },
      { uint256_t::from_uint128(uint128_t{ 0xb900000000000001 } << 64),
        uint256_t::from_uint128((uint128_t{ 0x1006021301080000 } << 64) + uint128_t{ 0x000000000000001080876844827 }),
        uint256_t::from_uint128((uint128_t{ 0xa906021301080001 } << 64) + uint128_t{ 0x0001080876844827 }) } }
};
std::vector<std::array<FF, 3>> positive_op_shr_test_values = {
    { { FF(20), FF(3), FF(2) },
      { FF(5323), FF(255), FF(0) },
      { FF(36148), FF(13), FF(4) },
      { FF(0x7bff744e3cdf79LLU), FF(64), FF(0) },
      { uint256_t::from_uint128((uint128_t{ 0x1006021301080000 } << 64) + uint128_t{ 0x000000000000001080876844827 }),
        FF(123),
        FF(2) } }
};
std::vector<std::array<FF, 3>> positive_op_shl_test_values = {
    { { FF(20), FF(8), FF(0) },
      { FF(5323), FF(10), FF(11264) },
      { FF(13793), FF(255), FF(0) },
      { FF(239), FF(50), uint256_t::from_uint128(269090077735387136) },
      { FF(9), FF(127), uint256_t::from_uint128(uint128_t{ 0x4800000000000000LLU } << 68) } }
};

class AvmBitwiseTestsNot : public AvmBitwiseTests, public testing::WithParamInterface<TwoOpParamRow> {};
class AvmBitwiseTestsAnd : public AvmBitwiseTests, public testing::WithParamInterface<ThreeOpParamRow> {};
class AvmBitwiseTestsOr : public AvmBitwiseTests, public testing::WithParamInterface<ThreeOpParamRow> {};
class AvmBitwiseTestsXor : public AvmBitwiseTests, public testing::WithParamInterface<ThreeOpParamRow> {};
class AvmBitwiseTestsShr : public AvmBitwiseTests, public testing::WithParamInterface<ThreeOpParamRow> {};
class AvmBitwiseTestsShl : public AvmBitwiseTests, public testing::WithParamInterface<ThreeOpParamRow> {};

/******************************************************************************
 *
 * POSITIVE TESTS
 *
 ******************************************************************************
 * See Avm_arithmetic.cpp for explanation of positive tests
 ******************************************************************************/

/******************************************************************************
 * Positive Tests
 ******************************************************************************/
TEST_P(AvmBitwiseTestsNot, ParamTest)
{
    const auto [operands, mem_tag] = GetParam();
    const auto [a, output] = operands;
    trace_builder.op_set(0, a, 0, mem_tag);
    trace_builder.op_not(0, 0, 1, mem_tag); // [1,254,0,0,....]
    trace_builder.op_return(0, 0, 0);
    auto trace = trace_builder.finalize();
    FF ff_a = FF(uint256_t::from_uint128(a));
    FF ff_output = FF(uint256_t::from_uint128(output));
    common_validate_op_not(trace, ff_a, ff_output, FF(0), FF(1), mem_tag);
    validate_trace(std::move(trace), public_inputs);
}

INSTANTIATE_TEST_SUITE_P(AvmBitwiseTests,
                         AvmBitwiseTestsNot,
                         testing::ValuesIn(gen_two_op_params(positive_op_not_test_values, mem_tags)));

TEST_P(AvmBitwiseTestsAnd, AllAndTest)
{
    const auto [operands, mem_tag] = GetParam();
    const auto [a, b, output] = operands;
    trace_builder.op_set(0, uint128_t(a), 0, mem_tag);
    trace_builder.op_set(0, uint128_t(b), 1, mem_tag);
    trace_builder.op_and(0, 0, 1, 2, mem_tag);
    trace_builder.op_return(0, 2, 1);

    auto trace = trace_builder.finalize();
    common_validate_bit_op(trace, 0, a, b, output, FF(0), FF(1), FF(2), mem_tag);
    validate_trace(std::move(trace), public_inputs, {}, { output }, true);
}
INSTANTIATE_TEST_SUITE_P(AvmBitwiseTests,
                         AvmBitwiseTestsAnd,
                         testing::ValuesIn(gen_three_op_params(positive_op_and_test_values, mem_tags)));

TEST_P(AvmBitwiseTestsOr, AllOrTest)
{
    const auto [operands, mem_tag] = GetParam();
    const auto [a, b, output] = operands;
    trace_builder.op_set(0, uint128_t(a), 0, mem_tag);
    trace_builder.op_set(0, uint128_t(b), 1, mem_tag);
    trace_builder.op_or(0, 0, 1, 2, mem_tag);
    trace_builder.op_return(0, 2, 1);
    auto trace = trace_builder.finalize();

    common_validate_bit_op(trace, 1, a, b, output, FF(0), FF(1), FF(2), mem_tag);
    validate_trace(std::move(trace), public_inputs, {}, { output });
}
INSTANTIATE_TEST_SUITE_P(AvmBitwiseTests,
                         AvmBitwiseTestsOr,
                         testing::ValuesIn(gen_three_op_params(positive_op_or_test_values, mem_tags)));

TEST_P(AvmBitwiseTestsXor, AllXorTest)
{
    const auto [operands, mem_tag] = GetParam();
    const auto [a, b, output] = operands;
    trace_builder.op_set(0, uint128_t(a), 0, mem_tag);
    trace_builder.op_set(0, uint128_t(b), 1, mem_tag);
    trace_builder.op_xor(0, 0, 1, 2, mem_tag);
    trace_builder.op_return(0, 2, 1);
    auto trace = trace_builder.finalize();

    common_validate_bit_op(trace, 2, a, b, output, FF(0), FF(1), FF(2), mem_tag);
    validate_trace(std::move(trace), public_inputs, {}, { output });
}

INSTANTIATE_TEST_SUITE_P(AvmBitwiseTests,
                         AvmBitwiseTestsXor,
                         testing::ValuesIn(gen_three_op_params(positive_op_xor_test_values, mem_tags)));

TEST_P(AvmBitwiseTestsShr, AllShrTest)
{
    const auto [operands, mem_tag] = GetParam();
    const auto [a, b, output] = operands;
    trace_builder.op_set(0, uint128_t(a), 0, mem_tag);
    trace_builder.op_set(0, uint128_t(b), 1, mem_tag);
    trace_builder.op_shr(0, 0, 1, 2, mem_tag);
    trace_builder.op_return(0, 2, 1);
    auto trace = trace_builder.finalize();
    common_validate_shift_op(trace, a, b, output, FF(0), FF(1), FF(2), mem_tag, true);
    validate_trace(std::move(trace), public_inputs, {}, { output });
}

INSTANTIATE_TEST_SUITE_P(AvmBitwiseTests,
                         AvmBitwiseTestsShr,
                         testing::ValuesIn(gen_three_op_params(positive_op_shr_test_values, mem_tags)));

TEST_P(AvmBitwiseTestsShl, AllShlTest)
{
    const auto [operands, mem_tag] = GetParam();
    const auto [a, b, output] = operands;
    trace_builder.op_set(0, uint128_t(a), 0, mem_tag);
    trace_builder.op_set(0, uint128_t(b), 1, mem_tag);
    trace_builder.op_shl(0, 0, 1, 2, mem_tag);
    trace_builder.op_return(0, 2, 1);
    auto trace = trace_builder.finalize();

    common_validate_shift_op(trace, a, b, output, FF(0), FF(1), FF(2), mem_tag, false);
    validate_trace(std::move(trace), public_inputs, {}, { output });
}

INSTANTIATE_TEST_SUITE_P(AvmBitwiseTests,
                         AvmBitwiseTestsShl,
                         testing::ValuesIn(gen_three_op_params(positive_op_shl_test_values, mem_tags)));
/******************************************************************************
 *
 * NEGATIVE TESTS - Finite Field Type
 *
 ******************************************************************************
 * See Avm_arithmetic.cpp for explanation of negative tests
 ******************************************************************************/
using EXPECTED_ERRORS = std::tuple<std::string, BIT_FAILURES>;

class AvmBitwiseNegativeTestsAnd : public AvmBitwiseTests,
                                   public testing::WithParamInterface<std::tuple<EXPECTED_ERRORS, ThreeOpParamRow>> {};
class AvmBitwiseNegativeTestsOr : public AvmBitwiseTests,
                                  public testing::WithParamInterface<std::tuple<EXPECTED_ERRORS, ThreeOpParamRow>> {};
class AvmBitwiseNegativeTestsXor : public AvmBitwiseTests,
                                   public testing::WithParamInterface<std::tuple<EXPECTED_ERRORS, ThreeOpParamRow>> {};
class AvmBitwiseNegativeTestsShr : public AvmBitwiseTests,
                                   public testing::WithParamInterface<std::tuple<SHIFT_FAILURES, ThreeOpParamRow>> {};
class AvmBitwiseNegativeTestsShl : public AvmBitwiseTests,
                                   public testing::WithParamInterface<std::tuple<SHIFT_FAILURES, ThreeOpParamRow>> {};
class AvmBitwiseNegativeTestsFF : public AvmBitwiseTests {};
class AvmBitwiseNegativeTestsU8 : public AvmBitwiseTests {};
class AvmBitwiseNegativeTestsU16 : public AvmBitwiseTests {};
class AvmBitwiseNegativeTestsU32 : public AvmBitwiseTests {};
class AvmBitwiseNegativeTestsU64 : public AvmBitwiseTests {};
class AvmBitwiseNegativeTestsU128 : public AvmBitwiseTests {};

std::vector<std::tuple<std::string, BIT_FAILURES>> bit_failures = {
    { "ACC_REL_C", BIT_FAILURES::IncorrectAcc },
    { "ACC_REL_C", BIT_FAILURES::BitDecomposition },
    { "MEM_TAG_REL", BIT_FAILURES::MemTagCtr },
    { "LOOKUP_BYTE_LENGTHS", BIT_FAILURES::ByteLengthError },
    { "LOOKUP_BYTE_OPERATIONS", BIT_FAILURES::ByteLookupError },
    { "OP_ID_REL", BIT_FAILURES::InconsistentOpId },
    { "SEL_BIN_CTR_REL", BIT_FAILURES::IncorrectBinSelector },
};
std::vector<SHIFT_FAILURES> shift_failures = { SHIFT_FAILURES::IncorrectShiftPastBitLength,
                                               SHIFT_FAILURES::IncorrectInputDecomposition,
                                               SHIFT_FAILURES::ShiftOutputIncorrect };
// For the negative test the output is set to be incorrect so that we can test the byte lookups.
// Picking "simple" inputs such as zero also makes it easier when check the byte length lookups as we dont
// need to worry about copying the accmulated a & b registers into the main trace.
std::vector<ThreeOpParamRow> neg_test_and = { { { 0, 0, 1 }, AvmMemoryTag::U32 } };
std::vector<ThreeOpParamRow> neg_test_or = { { { 0, 0, 1 }, AvmMemoryTag::U32 } };
std::vector<ThreeOpParamRow> neg_test_xor = { { { 0, 0, 1 }, AvmMemoryTag::U32 } };

std::vector<ThreeOpParamRow> neg_test_shr = { { { 7, 2, 0 }, AvmMemoryTag::U32 } };
/******************************************************************************
 * Negative Tests - FF
 ******************************************************************************/
TEST_P(AvmBitwiseNegativeTestsAnd, AllNegativeTests)
{
    const auto [failure, params] = GetParam();
    const auto [failure_string, failure_mode] = failure;
    const auto [operands, mem_tag] = params;
    const auto [a, b, output] = operands;
    trace_builder.op_set(0, uint128_t{ a }, 0, mem_tag);
    trace_builder.op_set(0, uint128_t{ b }, 1, mem_tag);
    trace_builder.op_and(0, 0, 1, 2, mem_tag);
    trace_builder.halt();
    auto trace = trace_builder.finalize();
    std::function<bool(Row)>&& select_row = [](Row r) { return r.main_sel_op_and == FF(1); };
    trace = gen_mutated_trace_bit(trace, std::move(select_row), output, failure_mode);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), failure_string);
}
INSTANTIATE_TEST_SUITE_P(AvmBitwiseNegativeTests,
                         AvmBitwiseNegativeTestsAnd,
                         testing::Combine(testing::ValuesIn(bit_failures), testing::ValuesIn(neg_test_and)));

TEST_P(AvmBitwiseNegativeTestsOr, AllNegativeTests)
{
    const auto [failure, params] = GetParam();
    const auto [failure_string, failure_mode] = failure;
    const auto [operands, mem_tag] = params;
    const auto [a, b, output] = operands;
    trace_builder.op_set(0, uint128_t{ a }, 0, mem_tag);
    trace_builder.op_set(0, uint128_t{ b }, 1, mem_tag);
    trace_builder.op_or(0, 0, 1, 2, mem_tag);
    trace_builder.halt();
    auto trace = trace_builder.finalize();
    std::function<bool(Row)>&& select_row = [](Row r) { return r.main_sel_op_or == FF(1); };
    trace = gen_mutated_trace_bit(trace, std::move(select_row), output, failure_mode);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), failure_string);
}
INSTANTIATE_TEST_SUITE_P(AvmBitwiseNegativeTests,
                         AvmBitwiseNegativeTestsOr,
                         testing::Combine(testing::ValuesIn(bit_failures), testing::ValuesIn(neg_test_or)));
TEST_P(AvmBitwiseNegativeTestsXor, AllNegativeTests)
{
    const auto [failure, params] = GetParam();
    const auto [failure_string, failure_mode] = failure;
    const auto [operands, mem_tag] = params;
    const auto [a, b, output] = operands;
    trace_builder.op_set(0, uint128_t{ a }, 0, mem_tag);
    trace_builder.op_set(0, uint128_t{ b }, 1, mem_tag);
    trace_builder.op_xor(0, 0, 1, 2, mem_tag);
    trace_builder.halt();
    auto trace = trace_builder.finalize();
    std::function<bool(Row)>&& select_row = [](Row r) { return r.main_sel_op_xor == FF(1); };
    trace = gen_mutated_trace_bit(trace, std::move(select_row), output, failure_mode);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), failure_string);
}
INSTANTIATE_TEST_SUITE_P(AvmBitwiseNegativeTests,
                         AvmBitwiseNegativeTestsXor,
                         testing::Combine(testing::ValuesIn(bit_failures), testing::ValuesIn(neg_test_xor)));

TEST_P(AvmBitwiseNegativeTestsShr, AllNegativeTests)
{
    const auto [failure, params] = GetParam();
    const auto [operands, mem_tag] = params;
    const auto [a, b, output] = operands;
    trace_builder.op_set(0, uint128_t{ a }, 0, mem_tag);
    trace_builder.op_set(0, uint128_t{ b }, 1, mem_tag);
    trace_builder.op_shr(0, 0, 1, 2, mem_tag);
    trace_builder.halt();
    auto trace = trace_builder.finalize();
    std::function<bool(Row)>&& select_row = [](Row r) { return r.main_sel_op_shr == FF(1); };

    auto [mutated_trace, str] = gen_mutated_trace_shift(std::move(trace), std::move(select_row), output, failure, true);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(mutated_trace)), str);
}
INSTANTIATE_TEST_SUITE_P(AvmBitwiseNegativeTests,
                         AvmBitwiseNegativeTestsShr,
                         testing::Combine(testing::ValuesIn(shift_failures), testing::ValuesIn(neg_test_shr)));

TEST_P(AvmBitwiseNegativeTestsShl, AllNegativeTests)
{
    const auto [failure, params] = GetParam();
    const auto [operands, mem_tag] = params;
    const auto [a, b, output] = operands;
    trace_builder.op_set(0, uint128_t{ a }, 0, mem_tag);
    trace_builder.op_set(0, uint128_t{ b }, 1, mem_tag);
    trace_builder.op_shl(0, 0, 1, 2, mem_tag);
    trace_builder.halt();
    auto trace = trace_builder.finalize();
    std::function<bool(Row)>&& select_row = [](Row r) { return r.main_sel_op_shl == FF(1); };

    auto [mutated_trace, str] =
        gen_mutated_trace_shift(std::move(trace), std::move(select_row), output, failure, false);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(mutated_trace)), str);
}
INSTANTIATE_TEST_SUITE_P(AvmBitwiseNegativeTests,
                         AvmBitwiseNegativeTestsShl,
                         testing::Combine(testing::ValuesIn(shift_failures), testing::ValuesIn(neg_test_shr)));

TEST_F(AvmBitwiseNegativeTestsFF, UndefinedOverFF)
{
    // Triggers a write row 1 of mem_trace and alu_trace
    trace_builder.op_set(0, 10, 0, AvmMemoryTag::U8);
    // Triggers a write in row 2 of alu_trace
    trace_builder.op_not(0, 0, 1, AvmMemoryTag::U8);
    // Finally, we will have a write in row 3 of the mem_trace to copy the result
    // from the op_not operation.
    trace_builder.op_return(0, 0, 0);
    // Manually update the memory tags in the relevant trace;
    auto trace = trace_builder.finalize();
    // TODO(ilyas): When the SET opcodes applies relational constraints, this will fail
    // we will need to look at a new way of doing this test.
    for (size_t i = 1; i < 4; i++) {
        trace.at(i).mem_tag = FF(6);
        trace.at(i).mem_r_in_tag = FF(6);
        trace.at(i).mem_w_in_tag = FF(6);
        trace.at(i).alu_ff_tag = FF::one();
        trace.at(i).alu_u8_tag = FF::zero();
        trace.at(i).main_r_in_tag = FF(6);
        trace.at(i).main_w_in_tag = FF(6);
        trace.at(i).alu_in_tag = FF(6);
    }

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "ALU_FF_NOT_XOR");
}

TEST_F(AvmBitwiseNegativeTestsU8, BitwiseNot)
{
    std::vector<Row> trace = gen_mutated_trace_not(FF{ 1 }, FF{ 2 }, AvmMemoryTag::U8);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "ALU_OP_NOT");
}

TEST_F(AvmBitwiseNegativeTestsU16, BitwiseNot)
{
    std::vector<Row> trace = gen_mutated_trace_not(FF{ 32'768 }, FF{ 8'192 }, AvmMemoryTag::U16);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "ALU_OP_NOT");
}

TEST_F(AvmBitwiseNegativeTestsU32, BitwiseNot)
{
    std::vector<Row> trace = gen_mutated_trace_not(FF{ 0xdeadbeef }, FF{ 0x20020af }, AvmMemoryTag::U64);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "ALU_OP_NOT");
}

TEST_F(AvmBitwiseNegativeTestsU64, BitwiseNot)
{
    std::vector<Row> trace =
        gen_mutated_trace_not(FF{ 0x10000000000000LLU }, FF{ 0x10000fed0100000LLU }, AvmMemoryTag::U64);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "ALU_OP_NOT");
}

TEST_F(AvmBitwiseNegativeTestsU128, BitwiseNot)
{
    uint128_t const a = uint128_t{ 0x4000000000000 } << 64;
    uint128_t const b = uint128_t{ 0x300000ae921000 } << 64;
    std::vector<Row> trace =
        gen_mutated_trace_not(FF{ uint256_t::from_uint128(a) }, FF{ uint256_t::from_uint128(b) }, AvmMemoryTag::U128);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "ALU_OP_NOT");
}
} // namespace tests_avm
