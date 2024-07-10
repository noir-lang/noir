#include "barretenberg/vm/tests/helpers.test.hpp"
#include "avm_common.test.hpp"
#include "barretenberg/vm/avm_trace/avm_helper.hpp"
#include "barretenberg/vm/avm_trace/constants.hpp"
#include "barretenberg/vm/generated/avm_flavor.hpp"
#include <bits/utility.h>

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

// TODO: Should be a cleaner way to do this
void update_slice_registers(Row& row, uint256_t a)
{
    row.alu_u8_r0 = static_cast<uint8_t>(a);
    a >>= 8;
    row.alu_u8_r1 = static_cast<uint8_t>(a);
    a >>= 8;
    row.alu_u16_r0 = static_cast<uint16_t>(a);
    a >>= 16;
    row.alu_u16_r1 = static_cast<uint16_t>(a);
    a >>= 16;
    row.alu_u16_r2 = static_cast<uint16_t>(a);
    a >>= 16;
    row.alu_u16_r3 = static_cast<uint16_t>(a);
    a >>= 16;
    row.alu_u16_r4 = static_cast<uint16_t>(a);
    a >>= 16;
    row.alu_u16_r5 = static_cast<uint16_t>(a);
    a >>= 16;
    row.alu_u16_r6 = static_cast<uint16_t>(a);
    a >>= 16;
    row.alu_u16_r7 = static_cast<uint16_t>(a);
    a >>= 16;
    row.alu_u16_r8 = static_cast<uint16_t>(a);
    a >>= 16;
    row.alu_u16_r9 = static_cast<uint16_t>(a);
    a >>= 16;
    row.alu_u16_r10 = static_cast<uint16_t>(a);
    a >>= 16;
    row.alu_u16_r11 = static_cast<uint16_t>(a);
    a >>= 16;
    row.alu_u16_r12 = static_cast<uint16_t>(a);
    a >>= 16;
    row.alu_u16_r13 = static_cast<uint16_t>(a);
    a >>= 16;
    row.alu_u16_r14 = static_cast<uint16_t>(a);
}

// TODO: There has to be a better way to do.
// This is a helper function to clear the range check counters associated with the alu register decomposition of
// "previous_value" so we don't trigger a trivial range_check count error
void clear_range_check_counters(std::vector<Row>& trace, uint256_t previous_value)
{
    // Find the main row where the old u8 value in the first register is looked up
    size_t lookup_value = static_cast<uint8_t>(previous_value);
    // Decrement the counter
    trace.at(lookup_value).lookup_u8_0_counts = trace.at(lookup_value).lookup_u8_0_counts - 1;
    previous_value >>= 8;
    lookup_value = static_cast<uint8_t>(previous_value);
    // Decrement the counter
    trace.at(lookup_value).lookup_u8_1_counts = trace.at(lookup_value).lookup_u8_1_counts - 1;
    previous_value >>= 8;

    // U_16_0: Find the main row where the old u16 value in the first register is looked up
    lookup_value = static_cast<uint16_t>(previous_value);
    // Decrement the counter
    trace.at(lookup_value).lookup_u16_0_counts = trace.at(lookup_value).lookup_u16_0_counts - 1;
    previous_value >>= 16;

    // U_16_1: Find the main row where the old u16 value in the second register is looked up
    lookup_value = static_cast<uint16_t>(previous_value);
    // Decrement the counter
    trace.at(lookup_value).lookup_u16_1_counts = trace.at(lookup_value).lookup_u16_1_counts - 1;
    previous_value >>= 16;

    // U_16_2: Find the main row where the old u16 value in the second register is looked up
    lookup_value = static_cast<uint16_t>(previous_value);
    // Decrement the counter
    trace.at(lookup_value).lookup_u16_2_counts = trace.at(lookup_value).lookup_u16_2_counts - 1;
    previous_value >>= 16;

    // U_16_3: Find the main row where the old u16 value in the second register is looked up
    lookup_value = static_cast<uint16_t>(previous_value);
    // Decrement the counter
    trace.at(lookup_value).lookup_u16_3_counts = trace.at(lookup_value).lookup_u16_3_counts - 1;
    previous_value >>= 16;

    // U_16_4: Find the main row where the old u16 value in the second register is looked up
    lookup_value = static_cast<uint16_t>(previous_value);
    // Decrement the counter
    trace.at(lookup_value).lookup_u16_4_counts = trace.at(lookup_value).lookup_u16_4_counts - 1;
    previous_value >>= 16;

    // U_16_5: Find the main row where the old u16 value in the second register is looked up
    lookup_value = static_cast<uint16_t>(previous_value);
    // Decrement the counter
    trace.at(lookup_value).lookup_u16_5_counts = trace.at(lookup_value).lookup_u16_5_counts - 1;
    previous_value >>= 16;

    // U_16_6: Find the main row where the old u16 value in the second register is looked up
    lookup_value = static_cast<uint16_t>(previous_value);
    // Decrement the counter
    trace.at(lookup_value).lookup_u16_6_counts = trace.at(lookup_value).lookup_u16_6_counts - 1;
    previous_value >>= 16;

    // U_16_7: Find the main row where the old u16 value in the second register is looked up
    lookup_value = static_cast<uint16_t>(previous_value);
    // Decrement the counter
    trace.at(lookup_value).lookup_u16_7_counts = trace.at(lookup_value).lookup_u16_7_counts - 1;
    previous_value >>= 16;

    // U_16_8: Find the main row where the old u16 value in the second register is looked up
    lookup_value = static_cast<uint16_t>(previous_value);
    // Decrement the counter
    trace.at(lookup_value).lookup_u16_8_counts = trace.at(lookup_value).lookup_u16_8_counts - 1;
    previous_value >>= 16;

    // U_16_9: Find the main row where the old u16 value in the second register is looked up
    lookup_value = static_cast<uint16_t>(previous_value);
    // Decrement the counter
    trace.at(lookup_value).lookup_u16_9_counts = trace.at(lookup_value).lookup_u16_9_counts - 1;
    previous_value >>= 16;

    // U_16_10: Find the main row where the old u16 value in the second register is looked up
    lookup_value = static_cast<uint16_t>(previous_value);
    // Decrement the counter
    trace.at(lookup_value).lookup_u16_10_counts = trace.at(lookup_value).lookup_u16_10_counts - 1;
    previous_value >>= 16;

    // U_16_11: Find the main row where the old u16 value in the second register is looked up
    lookup_value = static_cast<uint16_t>(previous_value);
    // Decrement the counter
    trace.at(lookup_value).lookup_u16_11_counts = trace.at(lookup_value).lookup_u16_11_counts - 1;
    previous_value >>= 16;

    // U_16_12: Find the main row where the old u16 value in the second register is looked up
    lookup_value = static_cast<uint16_t>(previous_value);
    // Decrement the counter
    trace.at(lookup_value).lookup_u16_12_counts = trace.at(lookup_value).lookup_u16_12_counts - 1;
    previous_value >>= 16;

    // U_16_13: Find the main row where the old u16 value in the second register is looked up
    lookup_value = static_cast<uint16_t>(previous_value);
    // Decrement the counter
    trace.at(lookup_value).lookup_u16_13_counts = trace.at(lookup_value).lookup_u16_13_counts - 1;
    previous_value >>= 16;

    // U_16_14: Find the main row where the old u16 value in the second register is looked up
    lookup_value = static_cast<uint16_t>(previous_value);
    // Decrement the counter
    trace.at(lookup_value).lookup_u16_14_counts = trace.at(lookup_value).lookup_u16_14_counts - 1;
    previous_value >>= 16;
}

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
