#include "barretenberg/vm/avm_trace/avm_helper.hpp"

#include <cassert>

#include "barretenberg/vm/avm_trace/avm_mem_trace.hpp"

namespace bb::avm_trace {

/**
 * @brief Routine to log some slice of a trace of the AVM. Used to debug or in some unit tests.
 *
 * @param trace The whole trace for AVM as a vector of rows.
 * @param beg The index of the beginning of the slice. (included)
 * @param end The index of the end of the slice (not included).
 */
void log_avm_trace([[maybe_unused]] std::vector<Row> const& trace,
                   [[maybe_unused]] size_t beg,
                   [[maybe_unused]] size_t end,
                   [[maybe_unused]] bool enable_selectors)
{
    info("Built circuit with ", trace.size(), " rows");

    for (size_t i = beg; i < end; i++) {
        info("=====================================================================================");
        info("==        ROW       ", i);
        info("=====================================================================================");

        info("=======MEMORY TRACE==================================================================");
        info("m_addr:             ", trace.at(i).mem_addr);
        info("m_clk:              ", trace.at(i).mem_clk);
        info("m_tsp:              ", trace.at(i).mem_tsp);
        info("m_sub_clk:          ", uint32_t(trace.at(i).mem_tsp) % AvmMemTraceBuilder::NUM_SUB_CLK);
        info("m_val:              ", trace.at(i).mem_val);
        info("m_rw:               ", trace.at(i).mem_rw);
        info("m_tag:              ", trace.at(i).mem_tag);
        info("r_in_tag:           ", trace.at(i).mem_r_in_tag);
        info("w_in_tag:           ", trace.at(i).mem_w_in_tag);
        info("m_tag_err:          ", trace.at(i).mem_tag_err);
        info("m_one_min_inv:      ", trace.at(i).mem_one_min_inv);

        info("m_lastAccess:       ", trace.at(i).mem_lastAccess);
        info("m_last:             ", trace.at(i).mem_last);

        info("=======CONTROL_FLOW===================================================================");
        info("pc:                 ", trace.at(i).main_pc);
        info("internal_call:      ", trace.at(i).main_sel_op_internal_call);
        info("internal_return:    ", trace.at(i).main_sel_op_internal_return);
        info("internal_return_ptr:", trace.at(i).main_internal_return_ptr);

        info("=======ALU TRACE=====================================================================");
        info("alu_clk             ", trace.at(i).alu_clk);
        info("alu_ia              ", trace.at(i).alu_ia);
        info("alu_ib              ", trace.at(i).alu_ib);
        info("alu_ic              ", trace.at(i).alu_ic);

        info("=======MAIN TRACE====================================================================");
        info("clk:                ", trace.at(i).main_clk);
        info("ia:                 ", trace.at(i).main_ia);
        info("ib:                 ", trace.at(i).main_ib);
        info("ic:                 ", trace.at(i).main_ic);
        info("r_in_tag            ", trace.at(i).main_r_in_tag);
        info("w_in_tag            ", trace.at(i).main_w_in_tag);
        info("tag_err             ", trace.at(i).main_tag_err);
        info("first:              ", trace.at(i).main_sel_first);
        info("last:               ", trace.at(i).main_sel_last);

        info("=======MEM_OP_A======================================================================");
        info("mem_op_a:           ", trace.at(i).main_sel_mem_op_a);
        info("mem_addr_a:         ", trace.at(i).main_mem_addr_a);
        info("rwa:                ", trace.at(i).main_rwa);

        info("=======MEM_OP_B======================================================================");
        info("mem_op_b:           ", trace.at(i).main_sel_mem_op_b);
        info("mem_addr_b:         ", trace.at(i).main_mem_addr_b);
        info("rwb:                ", trace.at(i).main_rwb);

        info("=======MEM_OP_C======================================================================");
        info("mem_op_c:           ", trace.at(i).main_sel_mem_op_c);
        info("mem_addr_c:         ", trace.at(i).main_mem_addr_c);
        info("rwc:                ", trace.at(i).main_rwc);

        info("=======MEM_DIFF======================================================================");
        info("diff_hi:            ", trace.at(i).mem_diff_hi);
        info("diff_mid:           ", trace.at(i).mem_diff_mid);
        info("diff_lo:            ", trace.at(i).mem_diff_lo);

        info("=======GAS ACCOUNTING================================================================");
        info("opcode active:      ", trace.at(i).main_sel_mem_op_activate_gas);
        info("l2_gas_remaining:   ", trace.at(i).main_l2_gas_remaining);
        info("da_gas_remaining:   ", trace.at(i).main_da_gas_remaining);
        info("l2_gas_op_cost:     ", trace.at(i).main_l2_gas_op_cost);
        info("da_gas_op_cost:     ", trace.at(i).main_da_gas_op_cost);
        info("l2_out_of_gas:      ", trace.at(i).main_l2_out_of_gas);
        info("da_out_of_gas:      ", trace.at(i).main_da_out_of_gas);
        info("abs_l2_hi_rem_gas:  ", trace.at(i).main_abs_l2_rem_gas_hi);
        info("abs_l2_lo_rem_gas:  ", trace.at(i).main_abs_l2_rem_gas_lo);
        info("abs_da_hi_rem_gas:  ", trace.at(i).main_abs_da_rem_gas_hi);
        info("abs_da_lo_rem_gas:  ", trace.at(i).main_abs_da_rem_gas_lo);

        if (enable_selectors) {
            info("=======SELECTORS======================================================================");
            info("sel_op_add:           ", trace.at(i).main_sel_op_add);
            info("sel_op_sub:           ", trace.at(i).main_sel_op_sub);
            info("sel_op_mul:           ", trace.at(i).main_sel_op_mul);
            info("sel_op_eq:            ", trace.at(i).main_sel_op_eq);
            info("sel_op_not:           ", trace.at(i).main_sel_op_not);
            info("sel_sel_alu:          ", trace.at(i).main_sel_alu);
        }
        info("\n");
    }
}

void dump_trace_as_csv(std::vector<Row> const& trace, std::filesystem::path const& filename)
{
    std::ofstream file;
    file.open(filename);

    for (const auto& row_name : Row::names()) {
        file << row_name << ",";
    }
    file << std::endl;

    for (const auto& row : trace) {
        file << row << std::endl;
    }
}

bool is_operand_indirect(uint8_t ind_value, uint8_t operand_idx)
{
    if (operand_idx > 7) {
        return false;
    }

    return static_cast<bool>((ind_value & (1 << operand_idx)) >> operand_idx);
}

std::vector<std::vector<FF>> copy_public_inputs_columns(VmPublicInputs const& public_inputs,
                                                        std::vector<FF> const& calldata,
                                                        std::vector<FF> const& returndata)
{
    // We convert to a vector as the pil generated verifier is generic and unaware of the KERNEL_INPUTS_LENGTH
    // For each of the public input vectors
    std::vector<FF> public_inputs_kernel_inputs(std::get<KERNEL_INPUTS>(public_inputs).begin(),
                                                std::get<KERNEL_INPUTS>(public_inputs).end());
    std::vector<FF> public_inputs_kernel_value_outputs(std::get<KERNEL_OUTPUTS_VALUE>(public_inputs).begin(),
                                                       std::get<KERNEL_OUTPUTS_VALUE>(public_inputs).end());
    std::vector<FF> public_inputs_kernel_side_effect_outputs(
        std::get<KERNEL_OUTPUTS_SIDE_EFFECT_COUNTER>(public_inputs).begin(),
        std::get<KERNEL_OUTPUTS_SIDE_EFFECT_COUNTER>(public_inputs).end());
    std::vector<FF> public_inputs_kernel_metadata_outputs(std::get<KERNEL_OUTPUTS_METADATA>(public_inputs).begin(),
                                                          std::get<KERNEL_OUTPUTS_METADATA>(public_inputs).end());

    assert(public_inputs_kernel_inputs.size() == KERNEL_INPUTS_LENGTH);
    assert(public_inputs_kernel_value_outputs.size() == KERNEL_OUTPUTS_LENGTH);
    assert(public_inputs_kernel_side_effect_outputs.size() == KERNEL_OUTPUTS_LENGTH);
    assert(public_inputs_kernel_metadata_outputs.size() == KERNEL_OUTPUTS_LENGTH);

    return {
        std::move(public_inputs_kernel_inputs),
        std::move(public_inputs_kernel_value_outputs),
        std::move(public_inputs_kernel_side_effect_outputs),
        std::move(public_inputs_kernel_metadata_outputs),
        calldata,
        returndata,
    };
}

} // namespace bb::avm_trace
