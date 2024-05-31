#include "barretenberg/vm/avm_trace/avm_helper.hpp"
#include "barretenberg/vm/avm_trace/avm_mem_trace.hpp"

namespace bb::avm_trace {

/**
 * @brief Routine to log some slice of a trace of the AVM. Used to debug or in some unit tests.
 *
 * @param trace The whole trace for AVM as a vector of rows.
 * @param beg The index of the beginning of the slice. (included)
 * @param end The index of the end of the slice (not included).
 */
void log_avm_trace(std::vector<Row> const& trace, size_t beg, size_t end, bool enable_selectors)
{
    {
        info("Built circuit with ", trace.size(), " rows");

        for (size_t i = beg; i < end; i++) {
            info("=====================================================================================");
            info("==        ROW       ", i);
            info("=====================================================================================");

            info("=======MEMORY TRACE==================================================================");
            info("m_addr:             ", trace.at(i).avm_mem_addr);
            info("m_clk:              ", trace.at(i).avm_mem_clk);
            info("m_tsp:              ", trace.at(i).avm_mem_tsp);
            info("m_sub_clk:          ", uint32_t(trace.at(i).avm_mem_tsp) % AvmMemTraceBuilder::NUM_SUB_CLK);
            info("m_val:              ", trace.at(i).avm_mem_val);
            info("m_rw:               ", trace.at(i).avm_mem_rw);
            info("m_tag:              ", trace.at(i).avm_mem_tag);
            info("r_in_tag:           ", trace.at(i).avm_mem_r_in_tag);
            info("w_in_tag:           ", trace.at(i).avm_mem_w_in_tag);
            info("m_tag_err:          ", trace.at(i).avm_mem_tag_err);
            info("m_one_min_inv:      ", trace.at(i).avm_mem_one_min_inv);

            info("m_lastAccess:       ", trace.at(i).avm_mem_lastAccess);
            info("m_last:             ", trace.at(i).avm_mem_last);
            info("m_val_shift:        ", trace.at(i).avm_mem_val_shift);

            info("=======CONTROL_FLOW===================================================================");
            info("pc:                 ", trace.at(i).avm_main_pc);
            info("internal_call:      ", trace.at(i).avm_main_sel_internal_call);
            info("internal_return:    ", trace.at(i).avm_main_sel_internal_return);
            info("internal_return_ptr:", trace.at(i).avm_main_internal_return_ptr);

            info("=======ALU TRACE=====================================================================");
            info("alu_clk             ", trace.at(i).avm_alu_clk);
            info("alu_ia              ", trace.at(i).avm_alu_ia);
            info("alu_ib              ", trace.at(i).avm_alu_ib);
            info("alu_ic              ", trace.at(i).avm_alu_ic);

            info("=======MAIN TRACE====================================================================");
            info("clk:                ", trace.at(i).avm_main_clk);
            info("ia:                 ", trace.at(i).avm_main_ia);
            info("ib:                 ", trace.at(i).avm_main_ib);
            info("ic:                 ", trace.at(i).avm_main_ic);
            info("r_in_tag            ", trace.at(i).avm_main_r_in_tag);
            info("w_in_tag            ", trace.at(i).avm_main_w_in_tag);
            info("tag_err             ", trace.at(i).avm_main_tag_err);
            info("first:              ", trace.at(i).avm_main_first);
            info("last:               ", trace.at(i).avm_main_last);

            info("=======MEM_OP_A======================================================================");
            info("mem_op_a:           ", trace.at(i).avm_main_mem_op_a);
            info("mem_idx_a:          ", trace.at(i).avm_main_mem_idx_a);
            info("rwa:                ", trace.at(i).avm_main_rwa);

            info("=======MEM_OP_B======================================================================");
            info("mem_op_b:           ", trace.at(i).avm_main_mem_op_b);
            info("mem_idx_b:          ", trace.at(i).avm_main_mem_idx_b);
            info("rwb:                ", trace.at(i).avm_main_rwb);

            info("=======MEM_OP_C======================================================================");
            info("mem_op_c:           ", trace.at(i).avm_main_mem_op_c);
            info("mem_idx_c:          ", trace.at(i).avm_main_mem_idx_c);
            info("rwc:                ", trace.at(i).avm_main_rwc);
            info("diff_hi:            ", trace.at(i).avm_mem_diff_hi);
            info("diff_mid:           ", trace.at(i).avm_mem_diff_mid);
            info("diff_lo:            ", trace.at(i).avm_mem_diff_lo);

            info("=======GAS ACCOUNTING================================================================");
            info("opcode active:      ", trace.at(i).avm_main_gas_cost_active);
            info("l2_gas_remaining:   ", trace.at(i).avm_main_l2_gas_remaining);
            info("da_gas_remaining:   ", trace.at(i).avm_main_da_gas_remaining);
            info("l2_gas_op:          ", trace.at(i).avm_main_l2_gas_op);
            info("da_gas_op:          ", trace.at(i).avm_main_da_gas_op);

            if (enable_selectors) {
                info("=======SELECTORS======================================================================");
                info("sel_op_add:           ", trace.at(i).avm_main_sel_op_add);
                info("sel_op_sub:           ", trace.at(i).avm_main_sel_op_sub);
                info("sel_op_mul:           ", trace.at(i).avm_main_sel_op_mul);
                info("sel_op_eq:            ", trace.at(i).avm_main_sel_op_eq);
                info("sel_op_not:           ", trace.at(i).avm_main_sel_op_not);
                info("sel_op_sel_alu:       ", trace.at(i).avm_main_alu_sel);
            }
            info("\n");
        }
    }
}

bool is_operand_indirect(uint8_t ind_value, uint8_t operand_idx)
{
    if (operand_idx > 7) {
        return false;
    }

    return static_cast<bool>((ind_value & (1 << operand_idx)) >> operand_idx);
}

std::vector<std::vector<FF>> copy_public_inputs_columns(VmPublicInputs public_inputs)
{
    // We convert to a vector as the pil generated verifier is generic and unaware of the KERNEL_INPUTS_LENGTH
    // For each of the public input vectors
    std::vector<FF> public_inputs_kernel_inputs(KERNEL_INPUTS_LENGTH);
    std::vector<FF> public_inputs_kernel_value_outputs(KERNEL_OUTPUTS_LENGTH);
    std::vector<FF> public_inputs_kernel_side_effect_outputs(KERNEL_OUTPUTS_LENGTH);
    std::vector<FF> public_inputs_kernel_metadata_outputs(KERNEL_OUTPUTS_LENGTH);

    std::copy(std::get<0>(public_inputs).begin(), std::get<0>(public_inputs).end(), public_inputs_kernel_inputs.data());
    std::copy(std::get<1>(public_inputs).begin(),
              std::get<1>(public_inputs).end(),
              public_inputs_kernel_value_outputs.data());
    std::copy(std::get<2>(public_inputs).begin(),
              std::get<2>(public_inputs).end(),
              public_inputs_kernel_side_effect_outputs.data());
    std::copy(std::get<3>(public_inputs).begin(),
              std::get<3>(public_inputs).end(),
              public_inputs_kernel_metadata_outputs.data());

    std::vector<std::vector<FF>> public_inputs_as_vec(4);
    public_inputs_as_vec[0] = public_inputs_kernel_inputs;
    public_inputs_as_vec[1] = public_inputs_kernel_value_outputs;
    public_inputs_as_vec[2] = public_inputs_kernel_side_effect_outputs;
    public_inputs_as_vec[3] = public_inputs_kernel_metadata_outputs;

    return public_inputs_as_vec;
}

} // namespace bb::avm_trace
