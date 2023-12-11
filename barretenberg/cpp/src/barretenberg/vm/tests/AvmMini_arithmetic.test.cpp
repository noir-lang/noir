#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/flavor/generated/AvmMini_flavor.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/proof_system/circuit_builder/AvmMini_helper.hpp"
#include "barretenberg/proof_system/circuit_builder/AvmMini_trace.hpp"
#include "barretenberg/sumcheck/sumcheck_round.hpp"
#include "barretenberg/vm/generated/AvmMini_composer.hpp"
#include "barretenberg/vm/generated/AvmMini_prover.hpp"
#include "barretenberg/vm/generated/AvmMini_verifier.hpp"

#include <cstddef>
#include <cstdint>
#include <gtest/gtest.h>
#include <string>
#include <vector>

using namespace proof_system;

namespace tests_avm {

class AvmMiniArithmeticTests : public ::testing::Test {
  public:
    AvmMiniTraceBuilder trace_builder;

  protected:
    // TODO(640): The Standard Honk on Grumpkin test suite fails unless the SRS is initialised for every test.
    void SetUp() override
    {
        barretenberg::srs::init_crs_factory("../srs_db/ignition");
        trace_builder = AvmMiniTraceBuilder(); // Clean instance for every run.
    };
};

class AvmMiniArithmeticNegativeTests : public AvmMiniArithmeticTests {};

// We add some helper functions in the anonymous namespace.
namespace {

/**
 * @brief Helper routine proving and verifying a proof based on the supplied trace
 *
 * @param trace The execution trace
 */
void validateTraceProof(std::vector<Row>&& trace)
{
    auto circuit_builder = AvmMiniCircuitBuilder();
    circuit_builder.set_trace(std::move(trace));

    EXPECT_TRUE(circuit_builder.check_circuit());

    auto composer = honk::AvmMiniComposer();
    auto prover = composer.create_prover(circuit_builder);
    auto proof = prover.construct_proof();

    auto verifier = composer.create_verifier(circuit_builder);
    bool verified = verifier.verify_proof(proof);

    if (!verified) {
        log_avmMini_trace(circuit_builder.rows, 0, 10);
    }
};

/**
 * @brief Helper routine for the negative tests. It mutates the output value of an operation
 *        located in the Ic intermediate register. The memory trace is adapted consistently.
 *
 * @param trace Execution trace
 * @param selectRow Lambda serving to select the row in trace
 * @param newValue The value that will be written in intermediate register Ic at the selected row.
 */
void mutateIcInTrace(std::vector<Row>& trace, std::function<bool(Row)>&& selectRow, FF const& newValue)
{
    // Find the first row matching the criteria defined by selectRow
    auto row = std::ranges::find_if(trace.begin(), trace.end(), selectRow);

    // Check that we found one
    EXPECT_TRUE(row != trace.end());

    // Mutate the correct result in the main trace
    row->avmMini_ic = newValue;

    // Adapt the memory trace to be consistent with the wrongly computed addition
    auto const clk = row->avmMini_clk;
    auto const addr = row->avmMini_mem_idx_c;

    // Find the relevant memory trace entry.
    auto memRow = std::ranges::find_if(trace.begin(), trace.end(), [clk, addr](Row r) {
        return r.memTrace_m_clk == clk && r.memTrace_m_addr == addr;
    });

    EXPECT_TRUE(memRow != trace.end());
    memRow->memTrace_m_val = newValue;
};

} // anonymous namespace

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
    trace_builder.callDataCopy(0, 3, 0, std::vector<FF>{ 37, 4, 11 });

    //           Memory layout:    [37,4,11,0,0,0,....]
    trace_builder.add(0, 1, 4); // [37,4,11,0,41,0,....]
    trace_builder.returnOP(0, 5);
    auto trace = trace_builder.finalize();

    // Find the first row enabling the addition selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_add == FF(1); });

    // Check that the correct result is stored at the expected memory location.
    EXPECT_TRUE(row != trace.end());
    EXPECT_EQ(row->avmMini_ic, FF(41));
    EXPECT_EQ(row->avmMini_mem_idx_c, FF(4));
    EXPECT_EQ(row->avmMini_mem_op_c, FF(1));
    EXPECT_EQ(row->avmMini_rwc, FF(1));

    validateTraceProof(std::move(trace));
}

// Test on basic subtraction over finite field type.
TEST_F(AvmMiniArithmeticTests, subtractionFF)
{
    trace_builder.callDataCopy(0, 3, 0, std::vector<FF>{ 8, 4, 17 });

    //           Memory layout:    [8,4,17,0,0,0,....]
    trace_builder.sub(2, 0, 1); // [8,9,17,0,0,0....]
    trace_builder.returnOP(0, 3);
    auto trace = trace_builder.finalize();

    // Find the first row enabling the subtraction selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_sub == FF(1); });

    // Check that the correct result is stored at the expected memory location.
    EXPECT_TRUE(row != trace.end());
    EXPECT_EQ(row->avmMini_ic, FF(9));
    EXPECT_EQ(row->avmMini_mem_idx_c, FF(1));
    EXPECT_EQ(row->avmMini_mem_op_c, FF(1));
    EXPECT_EQ(row->avmMini_rwc, FF(1));

    validateTraceProof(std::move(trace));
}

// Test on basic multiplication over finite field type.
TEST_F(AvmMiniArithmeticTests, multiplicationFF)
{
    trace_builder.callDataCopy(0, 3, 0, std::vector<FF>{ 5, 0, 20 });

    //           Memory layout:    [5,0,20,0,0,0,....]
    trace_builder.mul(2, 0, 1); // [5,100,20,0,0,0....]
    trace_builder.returnOP(0, 3);
    auto trace = trace_builder.finalize();

    // Find the first row enabling the multiplication selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_mul == FF(1); });

    // Check that the correct result is stored at the expected memory location.
    EXPECT_TRUE(row != trace.end());
    EXPECT_EQ(row->avmMini_ic, FF(100));
    EXPECT_EQ(row->avmMini_mem_idx_c, FF(1));
    EXPECT_EQ(row->avmMini_mem_op_c, FF(1));
    EXPECT_EQ(row->avmMini_rwc, FF(1));

    validateTraceProof(std::move(trace));
}

// Test on multiplication by zero over finite field type.
TEST_F(AvmMiniArithmeticTests, multiplicationByZeroFF)
{
    trace_builder.callDataCopy(0, 1, 0, std::vector<FF>{ 127 });

    //           Memory layout:    [127,0,0,0,0,0,....]
    trace_builder.mul(0, 1, 2); // [127,0,0,0,0,0....]
    trace_builder.returnOP(0, 3);
    auto trace = trace_builder.finalize();

    // Find the first row enabling the multiplication selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_mul == FF(1); });

    // Check that the correct result is stored at the expected memory location.
    EXPECT_TRUE(row != trace.end());
    EXPECT_EQ(row->avmMini_ic, FF(0));
    EXPECT_EQ(row->avmMini_mem_idx_c, FF(2));
    EXPECT_EQ(row->avmMini_mem_op_c, FF(1));
    EXPECT_EQ(row->avmMini_rwc, FF(1));

    validateTraceProof(std::move(trace));
}

// Test on basic division over finite field type.
TEST_F(AvmMiniArithmeticTests, divisionFF)
{
    trace_builder.callDataCopy(0, 2, 0, std::vector<FF>{ 15, 315 });

    //           Memory layout:    [15,315,0,0,0,0,....]
    trace_builder.div(1, 0, 2); // [15,315,21,0,0,0....]
    trace_builder.returnOP(0, 3);
    auto trace = trace_builder.finalize();

    // Find the first row enabling the division selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_div == FF(1); });

    // Check that the correct result is stored at the expected memory location.
    EXPECT_TRUE(row != trace.end());
    EXPECT_EQ(row->avmMini_ic, FF(21));
    EXPECT_EQ(row->avmMini_mem_idx_c, FF(2));
    EXPECT_EQ(row->avmMini_mem_op_c, FF(1));
    EXPECT_EQ(row->avmMini_rwc, FF(1));

    validateTraceProof(std::move(trace));
}

// Test on division with zero numerator over finite field type.
TEST_F(AvmMiniArithmeticTests, divisionNumeratorZeroFF)
{
    trace_builder.callDataCopy(0, 1, 0, std::vector<FF>{ 15 });

    //           Memory layout:    [15,0,0,0,0,0,....]
    trace_builder.div(1, 0, 0); // [0,0,0,0,0,0....]
    trace_builder.returnOP(0, 3);
    auto trace = trace_builder.finalize();

    // Find the first row enabling the division selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_div == FF(1); });

    // Check that the correct result is stored at the expected memory location.
    EXPECT_TRUE(row != trace.end());
    EXPECT_EQ(row->avmMini_ic, FF(0));
    EXPECT_EQ(row->avmMini_mem_idx_c, FF(0));
    EXPECT_EQ(row->avmMini_mem_op_c, FF(1));
    EXPECT_EQ(row->avmMini_rwc, FF(1));

    validateTraceProof(std::move(trace));
}

// Test on division by zero over finite field type.
// We check that the operator error flag is raised.
TEST_F(AvmMiniArithmeticTests, divisionByZeroErrorFF)
{
    trace_builder.callDataCopy(0, 1, 0, std::vector<FF>{ 15 });

    //           Memory layout:    [15,0,0,0,0,0,....]
    trace_builder.div(0, 1, 2); // [15,0,0,0,0,0....]
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

    validateTraceProof(std::move(trace));
}

// Testing an execution of the different arithmetic opcodes over finite field
// and finishing with a division by zero. The chosen combination is arbitrary.
// We only test that the proof can be correctly generated and verified.
// No check on the evaluation is performed here.
TEST_F(AvmMiniArithmeticTests, arithmeticFFWithError)
{
    trace_builder.callDataCopy(0, 3, 2, std::vector<FF>{ 45, 23, 12 });

    //           Memory layout:    [0,0,45,23,12,0,0,0,....]
    trace_builder.add(2, 3, 4); // [0,0,45,23,68,0,0,0,....]
    trace_builder.add(4, 5, 5); // [0,0,45,23,68,68,0,0,....]
    trace_builder.add(5, 5, 5); // [0,0,45,23,68,136,0,0,....]
    trace_builder.add(5, 6, 7); // [0,0,45,23,68,136,0,136,0....]
    trace_builder.sub(7, 6, 8); // [0,0,45,23,68,136,0,136,136,0....]
    trace_builder.mul(8, 8, 8); // [0,0,45,23,68,136,0,136,136^2,0....]
    trace_builder.div(3, 5, 1); // [0,23*136^(-1),45,23,68,136,0,136,136^2,0....]
    trace_builder.div(1, 1, 9); // [0,23*136^(-1),45,23,68,136,0,136,136^2,1,0....]
    trace_builder.div(9, 0, 4); // [0,23*136^(-1),45,23,1/0,136,0,136,136^2,1,0....] Error: division by 0

    auto trace = trace_builder.finalize();
    validateTraceProof(std::move(trace));
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
    trace_builder.callDataCopy(0, 3, 0, std::vector<FF>{ 37, 4, 11 });

    //           Memory layout:    [37,4,11,0,0,0,....]
    trace_builder.add(0, 1, 4); // [37,4,11,0,41,0,....]
    trace_builder.returnOP(0, 5);
    auto trace = trace_builder.finalize();

    auto selectRow = [](Row r) { return r.avmMini_sel_op_add == FF(1); };
    mutateIcInTrace(trace, std::move(selectRow), FF(40));

    // TODO: check that the expected sub-relation failed
    EXPECT_ANY_THROW(validateTraceProof(std::move(trace)));
}

// Test on basic incorrect subtraction over finite field type.
TEST_F(AvmMiniArithmeticNegativeTests, subtractionFF)
{
    trace_builder.callDataCopy(0, 3, 0, std::vector<FF>{ 8, 4, 17 });

    //           Memory layout:    [8,4,17,0,0,0,....]
    trace_builder.sub(2, 0, 1); // [8,9,17,0,0,0....]
    trace_builder.returnOP(0, 3);
    auto trace = trace_builder.finalize();

    auto selectRow = [](Row r) { return r.avmMini_sel_op_sub == FF(1); };
    mutateIcInTrace(trace, std::move(selectRow), FF(-9));

    // TODO: check that the expected sub-relation failed
    EXPECT_ANY_THROW(validateTraceProof(std::move(trace)));
}

// Test on basic incorrect multiplication over finite field type.
TEST_F(AvmMiniArithmeticNegativeTests, multiplicationFF)
{
    trace_builder.callDataCopy(0, 3, 0, std::vector<FF>{ 5, 0, 20 });

    //           Memory layout:    [5,0,20,0,0,0,....]
    trace_builder.mul(2, 0, 1); // [5,100,20,0,0,0....]
    trace_builder.returnOP(0, 3);
    auto trace = trace_builder.finalize();

    auto selectRow = [](Row r) { return r.avmMini_sel_op_mul == FF(1); };
    mutateIcInTrace(trace, std::move(selectRow), FF(1000));

    // TODO: check that the expected sub-relation failed
    EXPECT_ANY_THROW(validateTraceProof(std::move(trace)));
}

// Test on basic incorrect division over finite field type.
TEST_F(AvmMiniArithmeticNegativeTests, divisionFF)
{
    trace_builder.callDataCopy(0, 2, 0, std::vector<FF>{ 15, 315 });

    //           Memory layout:    [15,315,0,0,0,0,....]
    trace_builder.div(1, 0, 2); // [15,315,21,0,0,0....]
    trace_builder.returnOP(0, 3);
    auto trace = trace_builder.finalize();

    auto selectRow = [](Row r) { return r.avmMini_sel_op_div == FF(1); };
    mutateIcInTrace(trace, std::move(selectRow), FF(0));

    // TODO: check that the expected sub-relation failed
    EXPECT_ANY_THROW(validateTraceProof(std::move(trace)));
}

// Test where division is not by zero but an operation error is wrongly raised
// in the trace.
TEST_F(AvmMiniArithmeticNegativeTests, divisionNoZeroButErrorFF)
{
    trace_builder.callDataCopy(0, 2, 0, std::vector<FF>{ 15, 315 });

    //           Memory layout:    [15,315,0,0,0,0,....]
    trace_builder.div(1, 0, 2); // [15,315,21,0,0,0....]
    trace_builder.returnOP(0, 3);
    auto trace = trace_builder.finalize();

    // Find the first row enabling the division selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_div == FF(1); });

    // Activate the operator error
    row->avmMini_op_err = FF(1);

    // TODO: check that the expected sub-relation failed
    EXPECT_ANY_THROW(validateTraceProof(std::move(trace)));
}

// Test with division by zero occurs and no error is raised (remove error flag)
TEST_F(AvmMiniArithmeticNegativeTests, divisionByZeroNoErrorFF)
{
    trace_builder.callDataCopy(0, 1, 0, std::vector<FF>{ 15 });

    //           Memory layout:    [15,0,0,0,0,0,....]
    trace_builder.div(0, 1, 2); // [15,0,0,0,0,0....]
    auto trace = trace_builder.finalize();

    // Find the first row enabling the division selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_div == FF(1); });

    // Remove the operator error flag
    row->avmMini_op_err = FF(0);

    // TODO: check that the expected sub-relation failed
    EXPECT_ANY_THROW(validateTraceProof(std::move(trace)));
}

// Test that error flag cannot be raised for a non-relevant operation such as
// the addition, subtraction, multiplication.
TEST_F(AvmMiniArithmeticNegativeTests, operationWithErrorFlagFF)
{
    trace_builder.callDataCopy(0, 3, 0, std::vector<FF>{ 37, 4, 11 });

    //           Memory layout:    [37,4,11,0,0,0,....]
    trace_builder.add(0, 1, 4); // [37,4,11,0,41,0,....]
    trace_builder.returnOP(0, 5);
    auto trace = trace_builder.finalize();

    // Find the first row enabling the addition selector
    auto row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_add == FF(1); });

    // Activate the operator error
    row->avmMini_op_err = FF(1);

    // TODO: check that the expected sub-relation failed
    EXPECT_ANY_THROW(validateTraceProof(std::move(trace)));

    trace_builder.reset();

    trace_builder.callDataCopy(0, 3, 0, std::vector<FF>{ 8, 4, 17 });

    //           Memory layout:    [8,4,17,0,0,0,....]
    trace_builder.sub(2, 0, 1); // [8,9,17,0,0,0....]
    trace_builder.returnOP(0, 3);
    trace = trace_builder.finalize();

    // Find the first row enabling the subtraction selector
    row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_sub == FF(1); });

    // Activate the operator error
    row->avmMini_op_err = FF(1);

    // TODO: check that the expected sub-relation failed
    EXPECT_ANY_THROW(validateTraceProof(std::move(trace)));

    trace_builder.reset();

    trace_builder.callDataCopy(0, 3, 0, std::vector<FF>{ 5, 0, 20 });

    //           Memory layout:    [5,0,20,0,0,0,....]
    trace_builder.mul(2, 0, 1); // [5,100,20,0,0,0....]
    trace_builder.returnOP(0, 3);
    trace = trace_builder.finalize();

    // Find the first row enabling the multiplication selector
    row = std::ranges::find_if(trace.begin(), trace.end(), [](Row r) { return r.avmMini_sel_op_mul == FF(1); });

    // Activate the operator error
    row->avmMini_op_err = FF(1);

    // TODO: check that the expected sub-relation failed
    EXPECT_ANY_THROW(validateTraceProof(std::move(trace)));
}

} // namespace tests_avm