#include "barretenberg/vm/avm_trace/AvmMini_helper.hpp"
#include "barretenberg/vm/generated/AvmMini_composer.hpp"
#include "barretenberg/vm/generated/AvmMini_prover.hpp"
#include "barretenberg/vm/generated/AvmMini_verifier.hpp"
#include "helpers.test.hpp"

#include <cstddef>
#include <cstdint>
#include <gtest/gtest.h>
#include <string>
#include <vector>

namespace tests_avm {
using namespace avm_trace;

class AvmMiniArithmeticTests : public ::testing::Test {
  public:
    AvmMiniTraceBuilder trace_builder;

  protected:
    // TODO(640): The Standard Honk on Grumpkin test suite fails unless the SRS is initialised for every test.
    void SetUp() override
    {
        bb::srs::init_crs_factory("../srs_db/ignition");
        trace_builder = AvmMiniTraceBuilder(); // Clean instance for every run.
    };
};

class AvmMiniArithmeticNegativeTests : public AvmMiniArithmeticTests {};

/******************************************************************************
 *
 * POSITIVE TESTS - Finite Field Type
 *
 ******************************************************************************
 * The positive tests aim at testing that a genuinely generated execution trace
 * is correct, i.e., the evaluation is correct and the proof passes.
 * Positive refers to the proof system and not that the arithmetic operation has valid
 * operands. A division by zero needs to be handled by the AVM and needs to raise an error.
 * This will be positively tested, i.e., that the error is correctly raised.
 *
 * We isolate each operation addition, subtraction, multiplication and division
 * by having dedicated unit test for each of them.
 * In any positive test, we also verify that the main trace contains
 * a write memory operation for the intermediate register Ic at the
 * correct address. This operation belongs to the same row as the arithmetic
 * operation.
 *
 * Finding the row pertaining to the arithmetic operation is done through
 * a scan of all rows and stopping at the first one with the corresponding
 * operator selector. This mechanism is used with the hope that these unit tests
 * will still correctly work along the development of the AVM.
 ******************************************************************************/

// Test on basic addition over finite field type.
TEST_F(AvmMiniArithmeticTests, additionFF)
{
    // trace_builder
    trace_builder.call_data_copy(0, 3, 0, std::vector<FF>{ 37, 4, 11 });

    //                             Memory layout:    [37,4,11,0,0,0,....]
    trace_builder.add(0, 1, 4, AvmMemoryTag::ff); // [37,4,11,0,41,0,....]
    trace_builder.return_op(0, 5);
    auto trace = trace_builder.finalize();

    // Find the first row enabling the addition selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_add == FF(1); });

    // Check that the correct result is stored at the expected memory location.
    EXPECT_TRUE(row != trace.end());
    EXPECT_EQ(row->avmMini_ic, FF(41));
    EXPECT_EQ(row->avmMini_mem_idx_c, FF(4));
    EXPECT_EQ(row->avmMini_mem_op_c, FF(1));
    EXPECT_EQ(row->avmMini_rwc, FF(1));

    validate_trace_proof(std::move(trace));
}

// Test on basic subtraction over finite field type.
TEST_F(AvmMiniArithmeticTests, subtractionFF)
{
    trace_builder.call_data_copy(0, 3, 0, std::vector<FF>{ 8, 4, 17 });

    //                             Memory layout:    [8,4,17,0,0,0,....]
    trace_builder.sub(2, 0, 1, AvmMemoryTag::ff); // [8,9,17,0,0,0....]
    trace_builder.return_op(0, 3);
    auto trace = trace_builder.finalize();

    // Find the first row enabling the subtraction selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_sub == FF(1); });

    // Check that the correct result is stored at the expected memory location.
    EXPECT_TRUE(row != trace.end());
    EXPECT_EQ(row->avmMini_ic, FF(9));
    EXPECT_EQ(row->avmMini_mem_idx_c, FF(1));
    EXPECT_EQ(row->avmMini_mem_op_c, FF(1));
    EXPECT_EQ(row->avmMini_rwc, FF(1));

    validate_trace_proof(std::move(trace));
}

// Test on basic multiplication over finite field type.
TEST_F(AvmMiniArithmeticTests, multiplicationFF)
{
    trace_builder.call_data_copy(0, 3, 0, std::vector<FF>{ 5, 0, 20 });

    //                             Memory layout:    [5,0,20,0,0,0,....]
    trace_builder.mul(2, 0, 1, AvmMemoryTag::ff); // [5,100,20,0,0,0....]
    trace_builder.return_op(0, 3);
    auto trace = trace_builder.finalize();

    // Find the first row enabling the multiplication selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_mul == FF(1); });

    // Check that the correct result is stored at the expected memory location.
    EXPECT_TRUE(row != trace.end());
    EXPECT_EQ(row->avmMini_ic, FF(100));
    EXPECT_EQ(row->avmMini_mem_idx_c, FF(1));
    EXPECT_EQ(row->avmMini_mem_op_c, FF(1));
    EXPECT_EQ(row->avmMini_rwc, FF(1));

    validate_trace_proof(std::move(trace));
}

// Test on multiplication by zero over finite field type.
TEST_F(AvmMiniArithmeticTests, multiplicationByZeroFF)
{
    trace_builder.call_data_copy(0, 1, 0, std::vector<FF>{ 127 });

    //                             Memory layout:    [127,0,0,0,0,0,....]
    trace_builder.mul(0, 1, 2, AvmMemoryTag::ff); // [127,0,0,0,0,0....]
    trace_builder.return_op(0, 3);
    auto trace = trace_builder.finalize();

    // Find the first row enabling the multiplication selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_mul == FF(1); });

    // Check that the correct result is stored at the expected memory location.
    EXPECT_TRUE(row != trace.end());
    EXPECT_EQ(row->avmMini_ic, FF(0));
    EXPECT_EQ(row->avmMini_mem_idx_c, FF(2));
    EXPECT_EQ(row->avmMini_mem_op_c, FF(1));
    EXPECT_EQ(row->avmMini_rwc, FF(1));

    validate_trace_proof(std::move(trace));
}

// Test on basic division over finite field type.
TEST_F(AvmMiniArithmeticTests, divisionFF)
{
    trace_builder.call_data_copy(0, 2, 0, std::vector<FF>{ 15, 315 });

    //                             Memory layout:    [15,315,0,0,0,0,....]
    trace_builder.div(1, 0, 2, AvmMemoryTag::ff); // [15,315,21,0,0,0....]
    trace_builder.return_op(0, 3);
    auto trace = trace_builder.finalize();

    // Find the first row enabling the division selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_div == FF(1); });

    // Check that the correct result is stored at the expected memory location.
    EXPECT_TRUE(row != trace.end());
    EXPECT_EQ(row->avmMini_ic, FF(21));
    EXPECT_EQ(row->avmMini_mem_idx_c, FF(2));
    EXPECT_EQ(row->avmMini_mem_op_c, FF(1));
    EXPECT_EQ(row->avmMini_rwc, FF(1));

    validate_trace_proof(std::move(trace));
}

// Test on division with zero numerator over finite field type.
TEST_F(AvmMiniArithmeticTests, divisionNumeratorZeroFF)
{
    trace_builder.call_data_copy(0, 1, 0, std::vector<FF>{ 15 });

    //                             Memory layout:    [15,0,0,0,0,0,....]
    trace_builder.div(1, 0, 0, AvmMemoryTag::ff); // [0,0,0,0,0,0....]
    trace_builder.return_op(0, 3);
    auto trace = trace_builder.finalize();

    // Find the first row enabling the division selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_div == FF(1); });

    // Check that the correct result is stored at the expected memory location.
    EXPECT_TRUE(row != trace.end());
    EXPECT_EQ(row->avmMini_ic, FF(0));
    EXPECT_EQ(row->avmMini_mem_idx_c, FF(0));
    EXPECT_EQ(row->avmMini_mem_op_c, FF(1));
    EXPECT_EQ(row->avmMini_rwc, FF(1));

    validate_trace_proof(std::move(trace));
}

// Test on division by zero over finite field type.
// We check that the operator error flag is raised.
TEST_F(AvmMiniArithmeticTests, divisionByZeroErrorFF)
{
    trace_builder.call_data_copy(0, 1, 0, std::vector<FF>{ 15 });

    //                             Memory layout:    [15,0,0,0,0,0,....]
    trace_builder.div(0, 1, 2, AvmMemoryTag::ff); // [15,0,0,0,0,0....]
    trace_builder.halt();
    auto trace = trace_builder.finalize();

    // Find the first row enabling the division selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_div == FF(1); });

    // Check that the correct result is stored at the expected memory location.
    EXPECT_TRUE(row != trace.end());
    EXPECT_EQ(row->avmMini_ic, FF(0));
    EXPECT_EQ(row->avmMini_mem_idx_c, FF(2));
    EXPECT_EQ(row->avmMini_mem_op_c, FF(1));
    EXPECT_EQ(row->avmMini_rwc, FF(1));
    EXPECT_EQ(row->avmMini_op_err, FF(1));

    validate_trace_proof(std::move(trace));
}

// Test on division of zero by zero over finite field type.
// We check that the operator error flag is raised.
TEST_F(AvmMiniArithmeticTests, divisionZeroByZeroErrorFF)
{
    //                             Memory layout:    [0,0,0,0,0,0,....]
    trace_builder.div(0, 1, 2, AvmMemoryTag::ff); // [0,0,0,0,0,0....]
    trace_builder.halt();
    auto trace = trace_builder.finalize();

    // Find the first row enabling the division selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_div == FF(1); });

    // Check that the correct result is stored at the expected memory location.
    EXPECT_TRUE(row != trace.end());
    EXPECT_EQ(row->avmMini_ic, FF(0));
    EXPECT_EQ(row->avmMini_mem_idx_c, FF(2));
    EXPECT_EQ(row->avmMini_mem_op_c, FF(1));
    EXPECT_EQ(row->avmMini_rwc, FF(1));
    EXPECT_EQ(row->avmMini_op_err, FF(1));

    validate_trace_proof(std::move(trace));
}

// Testing an execution of the different arithmetic opcodes over finite field
// and finishing with a division by zero. The chosen combination is arbitrary.
// We only test that the proof can be correctly generated and verified.
// No check on the evaluation is performed here.
TEST_F(AvmMiniArithmeticTests, arithmeticFFWithError)
{
    trace_builder.call_data_copy(0, 3, 2, std::vector<FF>{ 45, 23, 12 });

    //                             Memory layout:    [0,0,45,23,12,0,0,0,....]
    trace_builder.add(2, 3, 4, AvmMemoryTag::ff); // [0,0,45,23,68,0,0,0,....]
    trace_builder.add(4, 5, 5, AvmMemoryTag::ff); // [0,0,45,23,68,68,0,0,....]
    trace_builder.add(5, 5, 5, AvmMemoryTag::ff); // [0,0,45,23,68,136,0,0,....]
    trace_builder.add(5, 6, 7, AvmMemoryTag::ff); // [0,0,45,23,68,136,0,136,0....]
    trace_builder.sub(7, 6, 8, AvmMemoryTag::ff); // [0,0,45,23,68,136,0,136,136,0....]
    trace_builder.mul(8, 8, 8, AvmMemoryTag::ff); // [0,0,45,23,68,136,0,136,136^2,0....]
    trace_builder.div(3, 5, 1, AvmMemoryTag::ff); // [0,23*136^(-1),45,23,68,136,0,136,136^2,0....]
    trace_builder.div(1, 1, 9, AvmMemoryTag::ff); // [0,23*136^(-1),45,23,68,136,0,136,136^2,1,0....]
    trace_builder.div(
        9, 0, 4, AvmMemoryTag::ff); // [0,23*136^(-1),45,23,1/0,136,0,136,136^2,1,0....] Error: division by 0
    trace_builder.halt();

    auto trace = trace_builder.finalize();
    validate_trace_proof(std::move(trace));
}

/******************************************************************************
 *
 * NEGATIVE TESTS - Finite Field Type
 *
 ******************************************************************************
 * The negative tests are the counterparts of the positive tests for which we want
 * to test that a deviation of the prescribed behaviour of the VM will lead to
 * an exception being raised while attempting to generate a proof.
 *
 * As for the positive tests, we isolate each operation addition, subtraction, multiplication
 * and division by having dedicated unit test for each of them.
 * A typical pattern is to wrongly mutate the result of the operation. The memory trace
 * is consistently adapted so that the negative test is applying to the relation
 * if the arithmetic operation and not the layout of the memory trace.
 *
 * Finding the row pertaining to the arithmetic operation is done through
 * a scan of all rows and stopping at the first one with the corresponding
 * operator selector. This mechanism is used with the hope that these unit tests
 * will still correctly work along the development of the AVM.
 ******************************************************************************/

// Test on basic incorrect addition over finite field type.
TEST_F(AvmMiniArithmeticNegativeTests, additionFF)
{
    trace_builder.call_data_copy(0, 3, 0, std::vector<FF>{ 37, 4, 11 });

    //                             Memory layout:    [37,4,11,0,0,0,....]
    trace_builder.add(0, 1, 4, AvmMemoryTag::ff); // [37,4,11,0,41,0,....]
    auto trace = trace_builder.finalize();

    auto select_row = [](Row r) { return r.avmMini_sel_op_add == FF(1); };
    mutate_ic_in_trace(trace, std::move(select_row), FF(40));

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "SUBOP_ADDITION_FF");
}

// Test on basic incorrect subtraction over finite field type.
TEST_F(AvmMiniArithmeticNegativeTests, subtractionFF)
{
    trace_builder.call_data_copy(0, 3, 0, std::vector<FF>{ 8, 4, 17 });

    //                             Memory layout:    [8,4,17,0,0,0,....]
    trace_builder.sub(2, 0, 1, AvmMemoryTag::ff); // [8,9,17,0,0,0....]
    auto trace = trace_builder.finalize();

    auto select_row = [](Row r) { return r.avmMini_sel_op_sub == FF(1); };
    mutate_ic_in_trace(trace, std::move(select_row), FF(-9));

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "SUBOP_SUBTRACTION_FF");
}

// Test on basic incorrect multiplication over finite field type.
TEST_F(AvmMiniArithmeticNegativeTests, multiplicationFF)
{
    trace_builder.call_data_copy(0, 3, 0, std::vector<FF>{ 5, 0, 20 });

    //                             Memory layout:    [5,0,20,0,0,0,....]
    trace_builder.mul(2, 0, 1, AvmMemoryTag::ff); // [5,100,20,0,0,0....]
    auto trace = trace_builder.finalize();

    auto select_row = [](Row r) { return r.avmMini_sel_op_mul == FF(1); };
    mutate_ic_in_trace(trace, std::move(select_row), FF(1000));

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "SUBOP_MULTIPLICATION_FF");
}

// Test on basic incorrect division over finite field type.
TEST_F(AvmMiniArithmeticNegativeTests, divisionFF)
{
    trace_builder.call_data_copy(0, 2, 0, std::vector<FF>{ 15, 315 });

    //                             Memory layout:    [15,315,0,0,0,0,....]
    trace_builder.div(1, 0, 2, AvmMemoryTag::ff); // [15,315,21,0,0,0....]
    auto trace = trace_builder.finalize();

    auto select_row = [](Row r) { return r.avmMini_sel_op_div == FF(1); };
    mutate_ic_in_trace(trace, std::move(select_row), FF(0));

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "SUBOP_DIVISION_FF");
}

// Test where division is not by zero but an operation error is wrongly raised
// in the trace.
TEST_F(AvmMiniArithmeticNegativeTests, divisionNoZeroButErrorFF)
{
    trace_builder.call_data_copy(0, 2, 0, std::vector<FF>{ 15, 315 });

    //                             Memory layout:    [15,315,0,0,0,0,....]
    trace_builder.div(1, 0, 2, AvmMemoryTag::ff); // [15,315,21,0,0,0....]
    auto trace = trace_builder.finalize();

    // Find the first row enabling the division selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_div == FF(1); });

    size_t const index = static_cast<size_t>(row - trace.begin());

    // Activate the operator error
    trace[index].avmMini_op_err = FF(1);
    auto trace2 = trace;

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "SUBOP_DIVISION_ZERO_ERR1");

    // Even more malicious, one makes the first relation passes by setting the inverse to zero.
    trace2[index].avmMini_inv = FF(0);
    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace2)), "SUBOP_DIVISION_ZERO_ERR2");
}

// Test with division by zero occurs and no error is raised (remove error flag)
TEST_F(AvmMiniArithmeticNegativeTests, divisionByZeroNoErrorFF)
{
    trace_builder.call_data_copy(0, 1, 0, std::vector<FF>{ 15 });

    //                             Memory layout:    [15,0,0,0,0,0,....]
    trace_builder.div(0, 1, 2, AvmMemoryTag::ff); // [15,0,0,0,0,0....]
    trace_builder.halt();
    auto trace = trace_builder.finalize();

    // Find the first row enabling the division selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_div == FF(1); });

    // Remove the operator error flag
    row->avmMini_op_err = FF(0);

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "SUBOP_DIVISION_FF");
}

// Test with division of zero by zero occurs and no error is raised (remove error flag)
TEST_F(AvmMiniArithmeticNegativeTests, divisionZeroByZeroNoErrorFF)
{
    //                             Memory layout:    [0,0,0,0,0,0,....]
    trace_builder.div(0, 1, 2, AvmMemoryTag::ff); // [0,0,0,0,0,0....]
    auto trace = trace_builder.finalize();

    // Find the first row enabling the division selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_div == FF(1); });

    // Remove the operator error flag
    row->avmMini_op_err = FF(0);

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "SUBOP_DIVISION_ZERO_ERR1");
}

// Test that error flag cannot be raised for a non-relevant operation such as
// the addition, subtraction, multiplication.
TEST_F(AvmMiniArithmeticNegativeTests, operationWithErrorFlagFF)
{
    trace_builder.call_data_copy(0, 3, 0, std::vector<FF>{ 37, 4, 11 });

    //                             Memory layout:    [37,4,11,0,0,0,....]
    trace_builder.add(0, 1, 4, AvmMemoryTag::ff); // [37,4,11,0,41,0,....]
    trace_builder.return_op(0, 5);
    auto trace = trace_builder.finalize();

    // Find the first row enabling the addition selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_add == FF(1); });

    // Activate the operator error
    row->avmMini_op_err = FF(1);

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "SUBOP_ERROR_RELEVANT_OP");

    trace_builder.reset();

    trace_builder.call_data_copy(0, 3, 0, std::vector<FF>{ 8, 4, 17 });

    //                             Memory layout:    [8,4,17,0,0,0,....]
    trace_builder.sub(2, 0, 1, AvmMemoryTag::ff); // [8,9,17,0,0,0....]
    trace_builder.return_op(0, 3);
    trace = trace_builder.finalize();

    // Find the first row enabling the subtraction selector
    row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_sub == FF(1); });

    // Activate the operator error
    row->avmMini_op_err = FF(1);

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "SUBOP_ERROR_RELEVANT_OP");

    trace_builder.reset();

    trace_builder.call_data_copy(0, 3, 0, std::vector<FF>{ 5, 0, 20 });

    //                             Memory layout:    [5,0,20,0,0,0,....]
    trace_builder.mul(2, 0, 1, AvmMemoryTag::ff); // [5,100,20,0,0,0....]
    trace_builder.return_op(0, 3);
    trace = trace_builder.finalize();

    // Find the first row enabling the multiplication selector
    row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_mul == FF(1); });

    // Activate the operator error
    row->avmMini_op_err = FF(1);

    EXPECT_THROW_WITH_MESSAGE(validate_trace_proof(std::move(trace)), "SUBOP_ERROR_RELEVANT_OP");
}

} // namespace tests_avm