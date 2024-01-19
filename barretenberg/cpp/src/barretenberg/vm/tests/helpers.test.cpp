#include "helpers.test.hpp"
#include "barretenberg/vm/avm_trace/AvmMini_helper.hpp"
#include "barretenberg/vm/generated/AvmMini_composer.hpp"
#include "barretenberg/vm/generated/AvmMini_prover.hpp"
#include "barretenberg/vm/generated/AvmMini_verifier.hpp"
#include <gtest/gtest.h>

namespace tests_avm {
using namespace avm_trace;

/**
 * @brief Helper routine proving and verifying a proof based on the supplied trace
 *
 * @param trace The execution trace
 */
void validate_trace_proof(std::vector<Row>&& trace)
{
    auto circuit_builder = bb::AvmMiniCircuitBuilder();
    circuit_builder.set_trace(std::move(trace));

    EXPECT_TRUE(circuit_builder.check_circuit());

    auto composer = bb::honk::AvmMiniComposer();
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
void mutate_ic_in_trace(std::vector<Row>& trace, std::function<bool(Row)>&& selectRow, FF const& newValue)
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
    auto mem_row = std::ranges::find_if(trace.begin(), trace.end(), [clk, addr](Row r) {
        return r.memTrace_m_clk == clk && r.memTrace_m_addr == addr;
    });

    EXPECT_TRUE(mem_row != trace.end());
    mem_row->memTrace_m_val = newValue;
};

} // namespace tests_avm