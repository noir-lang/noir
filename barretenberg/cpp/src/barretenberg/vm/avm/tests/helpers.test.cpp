#include "barretenberg/vm/avm/tests/helpers.test.hpp"
#include "barretenberg/vm/avm/generated/flavor.hpp"
#include "barretenberg/vm/avm/trace/helper.hpp"
#include "barretenberg/vm/constants.hpp"
#include "common.test.hpp"

namespace tests_avm {

using namespace bb;

std::vector<ThreeOpParamRow> gen_three_op_params(std::vector<ThreeOpParam> operands,
                                                 std::vector<bb::avm_trace::AvmMemoryTag> mem_tags)
{
    std::vector<ThreeOpParamRow> params;
    for (size_t i = 0; i < 5; i++) {
        params.emplace_back(operands[i], mem_tags[i]);
    }
    return params;
}
/**
 * @brief Helper routine checking the circuit constraints without proving
 *
 * @param trace The execution trace
 */
void validate_trace_check_circuit(std::vector<Row>&& trace)
{
    validate_trace(std::move(trace), {}, {}, {}, false);
};

/**
 * @brief Helper routine which checks the circuit constraints and depending on
 *        the boolean with_proof value performs a proof generation and verification.
 *
 * @param trace The execution trace
 */
void validate_trace(std::vector<Row>&& trace,
                    VmPublicInputs const& public_inputs,
                    std::vector<FF> const& calldata,
                    std::vector<FF> const& returndata,
                    bool with_proof,
                    bool expect_proof_failure)
{
    const std::string avm_dump_trace_path =
        std::getenv("AVM_DUMP_TRACE_PATH") != nullptr ? std::getenv("AVM_DUMP_TRACE_PATH") : "";
    if (!avm_dump_trace_path.empty()) {
        info("Dumping trace as CSV to: " + avm_dump_trace_path);
        avm_trace::dump_trace_as_csv(trace, avm_dump_trace_path);
    }

    auto circuit_builder = AvmCircuitBuilder();
    circuit_builder.set_trace(std::move(trace));
    EXPECT_TRUE(circuit_builder.check_circuit());

    if (with_proof) {
        AvmComposer composer = AvmComposer();
        AvmProver prover = composer.create_prover(circuit_builder);
        HonkProof proof = prover.construct_proof();

        AvmVerifier verifier = composer.create_verifier(circuit_builder);

        std::vector<std::vector<FF>> public_inputs_as_vec =
            bb::avm_trace::copy_public_inputs_columns(public_inputs, calldata, returndata);

        bool verified = verifier.verify_proof(proof, { public_inputs_as_vec });

        if (expect_proof_failure) {
            EXPECT_FALSE(verified);
        } else {
            EXPECT_TRUE(verified);
        }
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
    row->main_ic = newValue;

    // Optionally mutate the corresponding ic value in alu
    if (alu) {
        auto const clk = row->main_clk;
        // Find the relevant alu trace entry.
        auto alu_row = std::ranges::find_if(trace.begin(), trace.end(), [clk](Row r) { return r.alu_clk == clk; });

        EXPECT_TRUE(alu_row != trace.end());
        alu_row->alu_ic = newValue;
    }

    // Adapt the memory trace to be consistent with the wrong result
    auto const clk = row->main_clk;
    auto const addr = row->main_mem_addr_c;

    // Find the relevant memory trace entry.
    auto mem_row = std::ranges::find_if(
        trace.begin(), trace.end(), [clk, addr](Row r) { return r.mem_clk == clk && r.mem_addr == addr; });

    EXPECT_TRUE(mem_row != trace.end());
    mem_row->mem_val = newValue;
};

VmPublicInputs generate_base_public_inputs()
{
    VmPublicInputs public_inputs;
    std::array<FF, KERNEL_INPUTS_LENGTH> kernel_inputs{};
    kernel_inputs.at(DA_GAS_LEFT_CONTEXT_INPUTS_OFFSET) = DEFAULT_INITIAL_DA_GAS;
    kernel_inputs.at(L2_GAS_LEFT_CONTEXT_INPUTS_OFFSET) = DEFAULT_INITIAL_L2_GAS;
    std::get<0>(public_inputs) = kernel_inputs;
    return public_inputs;
}

} // namespace tests_avm
