#include "AvmMini_common.test.hpp"

using namespace bb;

namespace tests_avm {
/**
 * @brief Helper routine proving and verifying a proof based on the supplied trace
 *
 * @param trace The execution trace
 */
void validate_trace_proof(std::vector<Row>&& trace)
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
        avm_trace::log_avmMini_trace(circuit_builder.rows, 0, 10);
    }
};

/**
 * @brief Helper routine for the negative tests. It mutates the output value of an operation
 *        located in the Ic intermediate register. The memory trace is adapted consistently.
 *
 * @param trace Execution trace
 * @param selectRow Lambda serving to select the row in trace
 * @param newValue The value that will be written in intermediate register Ic at the selected row.
 * @param alu A boolean telling whether we mutate the ic value in alu as well.
 */
void mutate_ic_in_trace(std::vector<Row>& trace, std::function<bool(Row)>&& selectRow, FF const& newValue, bool alu)
{
    // Find the first row matching the criteria defined by selectRow
    auto row = std::ranges::find_if(trace.begin(), trace.end(), selectRow);

    // Check that we found one
    EXPECT_TRUE(row != trace.end());

    // Mutate the correct result in the main trace
    row->avmMini_ic = newValue;

    // Optionally mutate the corresponding ic value in alu
    if (alu) {
        auto const clk = row->avmMini_clk;
        // Find the relevant alu trace entry.
        auto alu_row =
            std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) { return r.aluChip_alu_clk == clk; });

        EXPECT_TRUE(alu_row != trace.end());
        alu_row->aluChip_alu_ic = newValue;
    }

    // Adapt the memory trace to be consistent with the wrong result
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