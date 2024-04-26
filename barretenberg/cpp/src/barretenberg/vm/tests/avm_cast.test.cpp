#include "avm_common.test.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/tests/helpers.test.hpp"
#include <gmock/gmock.h>
#include <gtest/gtest.h>

namespace tests_avm {
using namespace bb::avm_trace;
using namespace testing;

class AvmCastTests : public ::testing::Test {
  protected:
    AvmTraceBuilder trace_builder;
    std::vector<Row> trace;
    size_t main_idx;
    size_t alu_idx;
    size_t mem_idx_c;

    // TODO(640): The Standard Honk on Grumpkin test suite fails unless the SRS is initialised for every test.
    void SetUp() override { srs::init_crs_factory("../srs_db/ignition"); };

    void gen_trace(
        uint128_t const& a, uint32_t src_address, uint32_t dst_address, AvmMemoryTag src_tag, AvmMemoryTag dst_tag)
    {
        trace_builder.op_set(0, a, src_address, src_tag);
        trace_builder.op_cast(0, src_address, dst_address, dst_tag);
        trace_builder.return_op(0, 0, 0);
        trace = trace_builder.finalize();
        gen_indices();
    }

    void gen_indices()
    {
        auto row =
            std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_cast == FF(1); });
        ASSERT_TRUE(row != trace.end());
        main_idx = static_cast<size_t>(row - trace.begin());

        // Find the corresponding Alu trace row
        auto clk = row->avm_main_clk;
        auto alu_row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) { return r.avm_alu_clk == clk; });
        ASSERT_TRUE(alu_row != trace.end());
        alu_idx = static_cast<size_t>(alu_row - trace.begin());

        // Mem entry output ic write operation
        auto mem_row_c = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) {
            return r.avm_mem_tsp == FF(AvmMemTraceBuilder::NUM_SUB_CLK) * clk + AvmMemTraceBuilder::SUB_CLK_STORE_C;
        });
        ASSERT_TRUE(mem_row_c != trace.end());
        mem_idx_c = static_cast<size_t>(mem_row_c - trace.begin());
    }

    void validate_cast_trace(FF const& a,
                             FF const& cast_val,
                             uint32_t src_address,
                             uint32_t dst_address,
                             AvmMemoryTag src_tag,
                             AvmMemoryTag dst_tag

    )
    {
        auto const& row = trace.at(main_idx);
        EXPECT_THAT(row,
                    AllOf(Field("sel_op_cast", &Row::avm_main_sel_op_cast, 1),
                          Field("ia", &Row::avm_main_ia, a),
                          Field("ib", &Row::avm_main_ib, 0),
                          Field("ic", &Row::avm_main_ic, cast_val),
                          Field("r_in_tag", &Row::avm_main_r_in_tag, static_cast<uint32_t>(src_tag)),
                          Field("w_in_tag", &Row::avm_main_w_in_tag, static_cast<uint32_t>(dst_tag)),
                          Field("alu_in_tag", &Row::avm_main_alu_in_tag, static_cast<uint32_t>(dst_tag)),
                          Field("op_a", &Row::avm_main_mem_op_a, 1),
                          Field("op_c", &Row::avm_main_mem_op_c, 1),
                          Field("rwa", &Row::avm_main_rwa, 0),
                          Field("rwc", &Row::avm_main_rwc, 1),
                          Field("mem_idx_a", &Row::avm_main_mem_idx_a, src_address),
                          Field("mem_idx_c", &Row::avm_main_mem_idx_c, dst_address),
                          Field("tag_err", &Row::avm_main_tag_err, 0),
                          Field("alu_sel", &Row::avm_main_alu_sel, 1),
                          Field("sel_rng_8", &Row::avm_main_sel_rng_8, 1),
                          Field("sel_rng_16", &Row::avm_main_sel_rng_16, 1)));

        auto const& alu_row = trace.at(alu_idx);
        EXPECT_THAT(alu_row,
                    AllOf(Field("op_cast", &Row::avm_alu_op_cast, 1),
                          Field("alu_ia", &Row::avm_alu_ia, a),
                          Field("alu_ib", &Row::avm_alu_ib, 0),
                          Field("alu_ic", &Row::avm_alu_ic, cast_val),
                          Field("u8_tag", &Row::avm_alu_u8_tag, dst_tag == AvmMemoryTag::U8),
                          Field("u16_tag", &Row::avm_alu_u16_tag, dst_tag == AvmMemoryTag::U16),
                          Field("u32_tag", &Row::avm_alu_u32_tag, dst_tag == AvmMemoryTag::U32),
                          Field("u64_tag", &Row::avm_alu_u64_tag, dst_tag == AvmMemoryTag::U64),
                          Field("u128_tag", &Row::avm_alu_u128_tag, dst_tag == AvmMemoryTag::U128),
                          Field("ff_tag", &Row::avm_alu_ff_tag, dst_tag == AvmMemoryTag::FF),
                          Field("in_tag", &Row::avm_alu_in_tag, static_cast<uint32_t>(dst_tag)),
                          Field("op_cast_prev", &Row::avm_alu_op_cast_prev, 0),
                          Field("lookup_selector", &Row::avm_alu_rng_chk_lookup_selector, 1),
                          Field("alu_sel", &Row::avm_alu_alu_sel, 1)));

        // Check that there is a second ALU row
        auto alu_row_next = trace.at(alu_idx + 1);
        EXPECT_THAT(
            alu_row_next,
            AllOf(Field("op_cast", &Row::avm_alu_op_cast, 0), Field("op_cast_prev", &Row::avm_alu_op_cast_prev, 1)));

        validate_trace(std::move(trace));
    }
};

class AvmCastNegativeTests : public AvmCastTests {};

TEST_F(AvmCastTests, basicU8ToU16)
{
    gen_trace(237, 0, 1, AvmMemoryTag::U8, AvmMemoryTag::U16);
    validate_cast_trace(237, 237, 0, 1, AvmMemoryTag::U8, AvmMemoryTag::U16);
}

TEST_F(AvmCastTests, truncationU32ToU8)
{
    gen_trace(876123, 0, 1, AvmMemoryTag::U32, AvmMemoryTag::U8);
    validate_cast_trace(876123, 91, 0, 1, AvmMemoryTag::U32, AvmMemoryTag::U8);
}

TEST_F(AvmCastTests, sameAddressU16ToU8)
{
    gen_trace(1049, 23, 23, AvmMemoryTag::U16, AvmMemoryTag::U8); // M[23] = 1049
    validate_cast_trace(1049, 25, 23, 23, AvmMemoryTag::U16, AvmMemoryTag::U8);
}

TEST_F(AvmCastTests, basicU64ToFF)
{
    gen_trace(987234987324233324UL, 0, 1, AvmMemoryTag::U64, AvmMemoryTag::FF);
    validate_cast_trace(987234987324233324UL, 987234987324233324UL, 0, 1, AvmMemoryTag::U64, AvmMemoryTag::FF);
}

TEST_F(AvmCastTests, sameTagU128)
{
    uint128_t a = 312;
    a = a << 99;
    gen_trace(a, 0, 1, AvmMemoryTag::U128, AvmMemoryTag::U128);
    validate_cast_trace(
        uint256_t::from_uint128(a), FF(uint256_t::from_uint128(a)), 0, 1, AvmMemoryTag::U128, AvmMemoryTag::U128);
}

TEST_F(AvmCastTests, noTruncationFFToU32)
{
    gen_trace(UINT32_MAX, 4, 9, AvmMemoryTag::FF, AvmMemoryTag::U32);
    validate_cast_trace(UINT32_MAX, UINT32_MAX, 4, 9, AvmMemoryTag::FF, AvmMemoryTag::U32);
}

TEST_F(AvmCastTests, truncationFFToU16ModMinus1)
{
    trace_builder.calldata_copy(0, 0, 1, 0, { FF(FF::modulus - 1) });
    trace_builder.op_cast(0, 0, 1, AvmMemoryTag::U16);
    trace_builder.return_op(0, 0, 0);
    trace = trace_builder.finalize();
    gen_indices();

    validate_cast_trace(FF::modulus - 1, 0, 0, 1, AvmMemoryTag::FF, AvmMemoryTag::U16);
}

TEST_F(AvmCastTests, truncationFFToU16ModMinus2)
{
    trace_builder.calldata_copy(0, 0, 1, 0, { FF(FF::modulus_minus_two) });
    trace_builder.op_cast(0, 0, 1, AvmMemoryTag::U16);
    trace_builder.return_op(0, 0, 0);
    trace = trace_builder.finalize();
    gen_indices();

    validate_cast_trace(FF::modulus_minus_two, UINT16_MAX, 0, 1, AvmMemoryTag::FF, AvmMemoryTag::U16);
}

TEST_F(AvmCastTests, truncationU32ToU16)
{
    // 998877665 = OX3B89A9E1
    // Truncated to 16 bits: 0XA9E1 = 43489
    gen_trace(998877665UL, 0, 1, AvmMemoryTag::U32, AvmMemoryTag::U16);
    validate_cast_trace(998877665UL, 43489, 0, 1, AvmMemoryTag::U32, AvmMemoryTag::U16);
}

TEST_F(AvmCastTests, indirectAddrTruncationU64ToU8)
{
    // Indirect addresses. src:0  dst:1
    // Direct addresses.   src:10 dst:11
    // Source value: 256'000'000'203 --> truncated to 203
    trace_builder.op_set(0, 10, 0, AvmMemoryTag::U32);
    trace_builder.op_set(0, 11, 1, AvmMemoryTag::U32);
    trace_builder.op_set(0, 256'000'000'203UL, 10, AvmMemoryTag::U64);
    trace_builder.op_cast(3, 0, 1, AvmMemoryTag::U8);
    trace_builder.return_op(0, 0, 0);
    trace = trace_builder.finalize();
    gen_indices();

    validate_cast_trace(256'000'000'203UL, 203, 10, 11, AvmMemoryTag::U64, AvmMemoryTag::U8);
}

TEST_F(AvmCastTests, indirectAddrWrongResolutionU64ToU8)
{
    // Indirect addresses. src:5  dst:6
    // Direct addresses.   src:10 dst:11
    trace_builder.op_set(0, 10, 5, AvmMemoryTag::U8); // Not an address type
    trace_builder.op_set(0, 11, 6, AvmMemoryTag::U32);
    trace_builder.op_set(0, 4234, 10, AvmMemoryTag::U64);
    trace_builder.op_cast(3, 5, 6, AvmMemoryTag::U8);
    trace_builder.return_op(0, 0, 0);
    trace = trace_builder.finalize();

    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avm_main_sel_op_cast == FF(1); });
    ASSERT_TRUE(row != trace.end());

    EXPECT_THAT(*row,
                AllOf(Field("sel_op_cast", &Row::avm_main_sel_op_cast, 1),
                      Field("r_in_tag", &Row::avm_main_r_in_tag, static_cast<uint32_t>(AvmMemoryTag::U64)),
                      Field("w_in_tag", &Row::avm_main_w_in_tag, static_cast<uint32_t>(AvmMemoryTag::U8)),
                      Field("alu_in_tag", &Row::avm_main_alu_in_tag, static_cast<uint32_t>(AvmMemoryTag::U8)),
                      Field("op_a", &Row::avm_main_mem_op_a, 1),
                      Field("op_c", &Row::avm_main_mem_op_c, 1),
                      Field("ind_op_a", &Row::avm_main_ind_op_a, 1),
                      Field("ind_op_c", &Row::avm_main_ind_op_c, 1),
                      Field("ind_a", &Row::avm_main_ind_a, 5),
                      Field("ind_c", &Row::avm_main_ind_c, 6),
                      Field("rwa", &Row::avm_main_rwa, 0),
                      Field("rwc", &Row::avm_main_rwc, 1),
                      Field("alu_sel", &Row::avm_main_alu_sel, 0),   // ALU trace not activated
                      Field("tag_err", &Row::avm_main_tag_err, 1))); // Error activated

    validate_trace(std::move(trace));
}

TEST_F(AvmCastNegativeTests, nonTruncatedOutputMainIc)
{
    gen_trace(300, 0, 1, AvmMemoryTag::U16, AvmMemoryTag::U8);
    ASSERT_EQ(trace.at(main_idx).avm_main_ic, 44);

    // Replace the output in main trace with the non-truncated value
    trace.at(main_idx).avm_main_ic = 300;

    // Adapt the memory trace entry
    trace.at(mem_idx_c).avm_mem_val = 300;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "PERM_MAIN_ALU");
}

TEST_F(AvmCastNegativeTests, wrongOutputMainIc)
{
    gen_trace(151515, 0, 1, AvmMemoryTag::U32, AvmMemoryTag::FF);
    ASSERT_EQ(trace.at(main_idx).avm_main_ic, 151515);

    // Replace the output in main trace with a wrong value
    trace.at(main_idx).avm_main_ic = 151516;

    // Adapt the memory trace entry
    trace.at(mem_idx_c).avm_mem_val = 151516;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "PERM_MAIN_ALU");
}

TEST_F(AvmCastNegativeTests, wrongOutputAluIc)
{
    gen_trace(6582736, 0, 1, AvmMemoryTag::U128, AvmMemoryTag::U16);
    ASSERT_EQ(trace.at(alu_idx).avm_alu_ic, 29136);

    // Replace output in ALU, MAIN, and MEM trace
    trace.at(alu_idx).avm_alu_ic = 33;
    trace.at(main_idx).avm_main_ic = 33;
    trace.at(mem_idx_c).avm_mem_val = 33;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "ALU_OP_CAST");
}

TEST_F(AvmCastNegativeTests, wrongLimbDecompositionInput)
{
    trace_builder.calldata_copy(0, 0, 1, 0, { FF(FF::modulus_minus_two) });
    trace_builder.op_cast(0, 0, 1, AvmMemoryTag::U16);
    trace_builder.return_op(0, 0, 0);
    trace = trace_builder.finalize();
    gen_indices();

    trace.at(alu_idx).avm_alu_a_lo -= 23;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "INPUT_DECOMP_1");
}

TEST_F(AvmCastNegativeTests, wrongPSubALo)
{
    gen_trace(12345, 0, 1, AvmMemoryTag::U32, AvmMemoryTag::U16);
    ASSERT_EQ(trace.at(alu_idx).avm_alu_ic, 12345);

    trace.at(alu_idx).avm_alu_p_sub_a_lo += 3;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "SUB_LO_1");
}

TEST_F(AvmCastNegativeTests, wrongPSubAHi)
{
    trace_builder.calldata_copy(0, 0, 1, 0, { FF(FF::modulus_minus_two - 987) });
    trace_builder.op_cast(0, 0, 1, AvmMemoryTag::U16);
    trace_builder.return_op(0, 0, 0);
    trace = trace_builder.finalize();
    gen_indices();

    trace.at(alu_idx).avm_alu_p_sub_a_hi += 3;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "SUB_HI_1");
}

TEST_F(AvmCastNegativeTests, disableRangecheck)
{
    gen_trace(123, 23, 43, AvmMemoryTag::U8, AvmMemoryTag::U8);

    trace.at(alu_idx).avm_alu_rng_chk_lookup_selector = 0;
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "RNG_CHK_LOOKUP_SELECTOR");
}

TEST_F(AvmCastNegativeTests, disableRangecheckSub)
{
    gen_trace(123, 23, 43, AvmMemoryTag::U8, AvmMemoryTag::U8);

    trace.at(alu_idx + 1).avm_alu_rng_chk_lookup_selector = 0;
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "RNG_CHK_LOOKUP_SELECTOR");
}

TEST_F(AvmCastNegativeTests, wrongRangeCheckDecompositionLo)
{
    gen_trace(987344323, 23, 43, AvmMemoryTag::FF, AvmMemoryTag::U128);
    ASSERT_EQ(trace.at(alu_idx).avm_alu_ic, 987344323);

    trace.at(alu_idx).avm_alu_u16_r0 = 5555;
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "LOWER_CMP_RNG_CHK");
}

TEST_F(AvmCastNegativeTests, wrongRangeCheckDecompositionHi)
{
    trace_builder.calldata_copy(0, 0, 1, 0, { FF(FF::modulus_minus_two - 987) });
    trace_builder.op_cast(0, 0, 1, AvmMemoryTag::U16);
    trace_builder.return_op(0, 0, 0);
    trace = trace_builder.finalize();
    gen_indices();

    trace.at(alu_idx).avm_alu_u16_r9 = 5555;
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "UPPER_CMP_RNG_CHK");
}

TEST_F(AvmCastNegativeTests, outOfRangeU8Registers)
{
    gen_trace(987344323, 23, 43, AvmMemoryTag::FF, AvmMemoryTag::U128);
    ASSERT_EQ(trace.at(alu_idx).avm_alu_ic, 987344323);

    trace.at(alu_idx).avm_alu_u8_r0 += 256;
    trace.at(alu_idx).avm_alu_u8_r1 -= 1; // Adjust so that the decomposition is correct.

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "Lookup LOOKUP_U8_0");
}

TEST_F(AvmCastNegativeTests, outOfRangeU16Registers)
{
    gen_trace(987344323, 23, 43, AvmMemoryTag::FF, AvmMemoryTag::U128);
    ASSERT_EQ(trace.at(alu_idx).avm_alu_ic, 987344323);

    trace.at(alu_idx).avm_alu_u16_r0 += 65536;
    trace.at(alu_idx).avm_alu_u16_r1 -= 1; // Adjust so that the decomposition is correct.

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "Lookup LOOKUP_U16_0");
}

TEST_F(AvmCastNegativeTests, wrongCopySubLoForRangeCheck)
{
    gen_trace(987344323, 23, 43, AvmMemoryTag::U64, AvmMemoryTag::U128);
    ASSERT_EQ(trace.at(alu_idx).avm_alu_ic, 987344323);

    ASSERT_EQ(trace.at(alu_idx + 1).avm_alu_a_lo, trace.at(alu_idx).avm_alu_p_sub_a_lo);
    trace.at(alu_idx + 1).avm_alu_a_lo -= 1;
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "OP_CAST_RNG_CHECK_P_SUB_A_LOW");
}

TEST_F(AvmCastNegativeTests, wrongCopySubHiForRangeCheck)
{
    trace_builder.calldata_copy(0, 0, 1, 0, { FF(FF::modulus_minus_two - 972836) });
    trace_builder.op_cast(0, 0, 1, AvmMemoryTag::U128);
    trace_builder.return_op(0, 0, 0);
    trace = trace_builder.finalize();
    gen_indices();

    ASSERT_EQ(trace.at(alu_idx + 1).avm_alu_a_hi, trace.at(alu_idx).avm_alu_p_sub_a_hi);
    trace.at(alu_idx + 1).avm_alu_a_hi += 2;
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "OP_CAST_RNG_CHECK_P_SUB_A_HIGH");
}

} // namespace tests_avm
