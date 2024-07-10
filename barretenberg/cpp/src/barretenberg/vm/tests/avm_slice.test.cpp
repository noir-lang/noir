#include "avm_common.test.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/tests/helpers.test.hpp"
#include <gmock/gmock.h>
#include <gtest/gtest.h>

#include <ranges>

#define SLICE_ROW_FIELD_EQ(field_name, expression) Field(#field_name, &Row::slice_##field_name, expression)

namespace tests_avm {

using namespace bb;
using namespace bb::avm_trace;
using namespace testing;

class AvmSliceTests : public ::testing::Test {
  public:
    AvmSliceTests()
        : public_inputs(generate_base_public_inputs())
        , trace_builder(AvmTraceBuilder(public_inputs))
    {
        srs::init_crs_factory("../srs_db/ignition");
    }

    void gen_trace_builder(std::vector<FF> const& calldata)
    {
        trace_builder = AvmTraceBuilder(public_inputs, {}, 0, calldata);
        this->calldata = calldata;
    }

    void gen_single_calldata_copy(
        bool indirect, uint32_t cd_size, uint32_t col_offset, uint32_t copy_size, uint32_t dst_offset)
    {
        ASSERT_LE(col_offset + copy_size, cd_size);
        std::vector<FF> calldata;
        for (size_t i = 0; i < cd_size; i++) {
            calldata.emplace_back(i * i);
        }

        gen_trace_builder(calldata);
        trace_builder.op_calldata_copy(static_cast<uint8_t>(indirect), col_offset, copy_size, dst_offset);
        trace_builder.op_return(0, 0, 0);
        trace = trace_builder.finalize();
    }

    void validate_single_calldata_copy_trace(uint32_t col_offset,
                                             uint32_t copy_size,
                                             uint32_t dst_offset,
                                             bool proof_verif = false)
    {
        // Find the first row enabling the calldata_copy selector
        auto row = std::ranges::find_if(
            trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_calldata_copy == FF(1); });

        ASSERT_TRUE(row != trace.end());

        // Memory trace view pertaining to the calldata_copy operation.
        auto clk = row->main_clk;
        auto mem_view = std::views::filter(trace, [clk](Row r) {
            return r.mem_clk == clk && r.mem_rw == 1 && r.mem_sel_op_slice == 1 &&
                   r.mem_tag == static_cast<uint32_t>(AvmMemoryTag::FF);
        });

        // Check that the memory operations are as expected.
        size_t count = 0;
        for (auto const& mem_row : mem_view) {
            EXPECT_THAT(mem_row,
                        AllOf(MEM_ROW_FIELD_EQ(val, (col_offset + count) * (col_offset + count)),
                              MEM_ROW_FIELD_EQ(addr, dst_offset + count),
                              MEM_ROW_FIELD_EQ(tag, static_cast<uint32_t>(AvmMemoryTag::FF)),
                              MEM_ROW_FIELD_EQ(w_in_tag, static_cast<uint32_t>(AvmMemoryTag::FF)),
                              MEM_ROW_FIELD_EQ(r_in_tag, static_cast<uint32_t>(AvmMemoryTag::FF)),
                              MEM_ROW_FIELD_EQ(tag_err, 0)));
            count++;
        }

        EXPECT_EQ(count, copy_size);

        // Slice trace view pertaining to the calldata_copy operation.
        auto slice_view =
            std::views::filter(trace, [clk](Row r) { return r.slice_clk == clk && r.slice_sel_cd_cpy == 1; });

        FF last_row_idx = 0;

        // Check that the slice trace is as expected.
        count = 0;
        for (auto const& slice_row : slice_view) {
            EXPECT_THAT(slice_row,
                        AllOf(SLICE_ROW_FIELD_EQ(val, (col_offset + count) * (col_offset + count)),
                              SLICE_ROW_FIELD_EQ(addr, dst_offset + count),
                              SLICE_ROW_FIELD_EQ(col_offset, col_offset + count),
                              SLICE_ROW_FIELD_EQ(cnt, copy_size - count),
                              SLICE_ROW_FIELD_EQ(sel_start, static_cast<uint32_t>(count == 0))));
            count++;

            if (count == copy_size) {
                last_row_idx = slice_row.main_clk;
            }
        }

        // Check that the extra final row is well-formed.
        EXPECT_THAT(trace.at(static_cast<size_t>(last_row_idx + 1)),
                    AllOf(SLICE_ROW_FIELD_EQ(addr, FF(dst_offset) + FF(copy_size)),
                          SLICE_ROW_FIELD_EQ(col_offset, col_offset + copy_size),
                          SLICE_ROW_FIELD_EQ(cnt, 0),
                          SLICE_ROW_FIELD_EQ(clk, clk),
                          SLICE_ROW_FIELD_EQ(sel_cd_cpy, 0),
                          SLICE_ROW_FIELD_EQ(sel_start, 0)));

        if (proof_verif) {
            validate_trace(std::move(trace), public_inputs, calldata, {}, true);
        } else {
            validate_trace(std::move(trace), public_inputs, calldata);
        }
    }

    VmPublicInputs public_inputs;
    AvmTraceBuilder trace_builder;
    std::vector<FF> calldata;

    std::vector<Row> trace;
    size_t main_row_idx;
    size_t alu_row_idx;
    size_t mem_row_idx;
};

TEST_F(AvmSliceTests, simpleCopyAllCDValues)
{
    gen_single_calldata_copy(false, 12, 0, 12, 25);
    validate_single_calldata_copy_trace(0, 12, 25, true);
}

TEST_F(AvmSliceTests, singleCopyCDElement)
{
    gen_single_calldata_copy(false, 12, 5, 1, 25);
    validate_single_calldata_copy_trace(5, 1, 25);
}

TEST_F(AvmSliceTests, longCopyAllCDValues)
{
    gen_single_calldata_copy(false, 2000, 0, 2000, 873);
    validate_single_calldata_copy_trace(0, 2000, 873);
}

TEST_F(AvmSliceTests, copyFirstHalfCDValues)
{
    gen_single_calldata_copy(false, 12, 0, 6, 98127);
    validate_single_calldata_copy_trace(0, 6, 98127);
}

TEST_F(AvmSliceTests, copySecondHalfCDValues)
{
    gen_single_calldata_copy(false, 12, 6, 6, 0);
    validate_single_calldata_copy_trace(6, 6, 0);
}

TEST_F(AvmSliceTests, copyToHighestMemOffset)
{
    gen_single_calldata_copy(false, 8, 2, 6, UINT32_MAX - 5);
    validate_single_calldata_copy_trace(2, 6, UINT32_MAX - 5);
}

TEST_F(AvmSliceTests, twoCallsNoOverlap)
{
    calldata = { 2, 3, 4, 5, 6 };

    gen_trace_builder(calldata);
    trace_builder.op_calldata_copy(0, 0, 2, 34);
    trace_builder.op_calldata_copy(0, 3, 2, 2123);
    trace_builder.op_return(0, 0, 0);
    trace = trace_builder.finalize();

    // Main trace views of rows enabling the calldata_copy selector
    auto main_view = std::views::filter(trace, [](Row r) { return r.main_sel_op_calldata_copy == FF(1); });

    std::vector<Row> main_rows;
    for (auto const& row : main_view) {
        main_rows.push_back(row);
    }

    EXPECT_EQ(main_rows.size(), 2);

    EXPECT_THAT(main_rows.at(0),
                AllOf(MAIN_ROW_FIELD_EQ(ia, 0),
                      MAIN_ROW_FIELD_EQ(ib, 2),
                      MAIN_ROW_FIELD_EQ(mem_addr_c, 34),
                      MAIN_ROW_FIELD_EQ(clk, 1)));
    EXPECT_THAT(main_rows.at(1),
                AllOf(MAIN_ROW_FIELD_EQ(ia, 3),
                      MAIN_ROW_FIELD_EQ(ib, 2),
                      MAIN_ROW_FIELD_EQ(mem_addr_c, 2123),
                      MAIN_ROW_FIELD_EQ(clk, 2)));

    validate_trace(std::move(trace), public_inputs, calldata);
}

TEST_F(AvmSliceTests, indirectTwoCallsOverlap)
{
    calldata = { 2, 3, 4, 5, 6 };

    gen_trace_builder(calldata);
    trace_builder.op_set(0, 34, 100, AvmMemoryTag::U32);   // indirect address 100 resolves to 34
    trace_builder.op_set(0, 2123, 101, AvmMemoryTag::U32); // indirect address 101 resolves to 2123
    trace_builder.op_calldata_copy(1, 1, 3, 100);
    trace_builder.op_calldata_copy(1, 2, 3, 101);
    trace_builder.op_return(0, 0, 0);
    trace = trace_builder.finalize();

    // Main trace views of rows enabling the calldata_copy selector
    auto main_view = std::views::filter(trace, [](Row r) { return r.main_sel_op_calldata_copy == FF(1); });

    std::vector<Row> main_rows;
    for (auto const& row : main_view) {
        main_rows.push_back(row);
    }

    EXPECT_EQ(main_rows.size(), 2);

    EXPECT_THAT(main_rows.at(0),
                AllOf(MAIN_ROW_FIELD_EQ(ia, 1),
                      MAIN_ROW_FIELD_EQ(ib, 3),
                      MAIN_ROW_FIELD_EQ(sel_resolve_ind_addr_c, 1),
                      MAIN_ROW_FIELD_EQ(ind_addr_c, 100),
                      MAIN_ROW_FIELD_EQ(mem_addr_c, 34),
                      MAIN_ROW_FIELD_EQ(clk, 3)));
    EXPECT_THAT(main_rows.at(1),
                AllOf(MAIN_ROW_FIELD_EQ(ia, 2),
                      MAIN_ROW_FIELD_EQ(ib, 3),
                      MAIN_ROW_FIELD_EQ(sel_resolve_ind_addr_c, 1),
                      MAIN_ROW_FIELD_EQ(ind_addr_c, 101),
                      MAIN_ROW_FIELD_EQ(mem_addr_c, 2123),
                      MAIN_ROW_FIELD_EQ(clk, 4)));

    validate_trace(std::move(trace), public_inputs, calldata);
}

TEST_F(AvmSliceTests, indirectFailedResolution)
{
    calldata = { 2, 3, 4, 5, 6 };

    gen_trace_builder(calldata);
    trace_builder.op_set(0, 34, 100, AvmMemoryTag::U16); // indirect address 100 resolves to 34
    trace_builder.op_calldata_copy(1, 1, 3, 100);
    trace_builder.halt();
    trace = trace_builder.finalize();

    // Check that slice trace is empty
    auto slice_row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.slice_sel_cd_cpy == 1; });
    EXPECT_EQ(slice_row, trace.end());

    auto count = std::ranges::count_if(trace.begin(), trace.end(), [](Row r) { return r.mem_sel_op_slice == 1; });
    // Check that MEM trace does not contain any entry related to calldata_copy write.
    EXPECT_EQ(count, 0);

    // Find the first row enabling the calldata_copy selector
    auto row =
        std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.main_sel_op_calldata_copy == FF(1); });

    ASSERT_TRUE(row != trace.end());
    auto clk = row->main_clk;
    auto mem_row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) { return r.mem_clk == clk; });

    EXPECT_EQ(mem_row->mem_rw, 0);
    EXPECT_EQ(mem_row->mem_sel_resolve_ind_addr_c, 1);

    validate_trace(std::move(trace), public_inputs, calldata);
}

class AvmSliceNegativeTests : public AvmSliceTests {};

TEST_F(AvmSliceNegativeTests, wrongCDValueInSlice)
{
    gen_single_calldata_copy(false, 10, 0, 10, 0);

    trace.at(3).slice_val = 98;

    // Adapt corresponding MEM trace entry in a consistent way.
    auto clk = trace.at(3).slice_clk;
    auto addr = trace.at(3).slice_addr;
    auto mem_row = std::ranges::find_if(
        trace.begin(), trace.end(), [clk, addr](Row r) { return r.mem_clk == clk && r.mem_addr == addr; });
    mem_row->mem_val = 98;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "LOOKUP_CD_VALUE");
}

TEST_F(AvmSliceNegativeTests, wrongCDValueInMemory)
{
    gen_single_calldata_copy(false, 10, 0, 10, 0);

    auto clk = trace.at(5).slice_clk;
    auto addr = trace.at(5).slice_addr;
    auto mem_row = std::ranges::find_if(
        trace.begin(), trace.end(), [clk, addr](Row r) { return r.mem_clk == clk && r.mem_addr == addr; });
    mem_row->mem_val = 98;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "PERM_SLICE_MEM");
}

TEST_F(AvmSliceNegativeTests, wrongCDValueInCalldataColumn)
{
    gen_single_calldata_copy(false, 10, 0, 10, 0);

    trace.at(2).main_calldata = 12;
    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "LOOKUP_CD_VALUE");
}

TEST_F(AvmSliceNegativeTests, wrongCDValueInCalldataVerifier)
{
    calldata = { 2, 3, 4, 5, 6 };

    gen_trace_builder(calldata);
    trace_builder.op_calldata_copy(0, 1, 3, 100);
    trace_builder.op_return(0, 0, 0);
    trace = trace_builder.finalize();

    validate_trace(std::move(trace), public_inputs, { 2, 3, 4, 5, 7 }, {}, true, true);
}

TEST_F(AvmSliceNegativeTests, disableMemWriteEntry)
{
    gen_single_calldata_copy(false, 10, 0, 10, 0);

    // Multiple adjustements to get valid MEM trace.
    trace.at(10).mem_sel_op_slice = 0;
    trace.at(10).mem_skip_check_tag = 0;
    trace.at(10).mem_sel_mem = 0;
    trace.at(9).mem_last = 1;
    trace.at(10).mem_last = 0;
    trace.at(10).mem_tsp = 12;
    trace.at(9).mem_sel_rng_chk = 0;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_check_circuit(std::move(trace)), "PERM_SLICE_MEM");
}

} // namespace tests_avm
