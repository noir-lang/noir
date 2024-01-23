#include "AvmMini_helper.hpp"

namespace avm_trace {

/**
 * @brief Routine to log some slice of a trace of the AVM. Used to debug or in some unit tests.
 *
 * @param trace The whole trace for AVM as a vector of rows.
 * @param beg The index of the beginning of the slice. (included)
 * @param end The index of the end of the slice (not included).
 */
void log_avmMini_trace(std::vector<Row> const& trace, size_t beg, size_t end)
{
    info("Built circuit with ", trace.size(), " rows");

    for (size_t i = beg; i < end; i++) {
        info("=====================================================================================");
        info("==        ROW       ", i);
        info("=====================================================================================");

        info("=======MEMORY TRACE==================================================================");
        info("m_addr:             ", trace.at(i).memTrace_m_addr);
        info("m_clk:              ", trace.at(i).memTrace_m_clk);
        info("m_sub_clk:          ", trace.at(i).memTrace_m_sub_clk);
        info("m_val:              ", trace.at(i).memTrace_m_val);
        info("m_rw:               ", trace.at(i).memTrace_m_rw);
        info("m_tag:              ", trace.at(i).memTrace_m_tag);
        info("m_in_tag:           ", trace.at(i).memTrace_m_in_tag);
        info("m_tag_err:          ", trace.at(i).memTrace_m_tag_err);
        info("m_one_min_inv:      ", trace.at(i).memTrace_m_one_min_inv);

        info("m_lastAccess:       ", trace.at(i).memTrace_m_lastAccess);
        info("m_last:             ", trace.at(i).memTrace_m_last);
        info("m_val_shift:        ", trace.at(i).memTrace_m_val_shift);

        info("=======CONTROL_FLOW===================================================================");
        info("pc:                 ", trace.at(i).avmMini_pc);
        info("internal_call:      ", trace.at(i).avmMini_sel_internal_call);
        info("internal_return:    ", trace.at(i).avmMini_sel_internal_return);
        info("internal_return_ptr:", trace.at(i).avmMini_internal_return_ptr);

        info("=======ALU TRACE=====================================================================");
        info("alu_clk             ", trace.at(i).aluChip_alu_clk);
        info("alu_ia              ", trace.at(i).aluChip_alu_ia);
        info("alu_ib              ", trace.at(i).aluChip_alu_ib);
        info("alu_ic              ", trace.at(i).aluChip_alu_ic);

        info("=======MAIN TRACE====================================================================");
        info("ia:                 ", trace.at(i).avmMini_ia);
        info("ib:                 ", trace.at(i).avmMini_ib);
        info("ic:                 ", trace.at(i).avmMini_ic);
        info("first:              ", trace.at(i).avmMini_first);
        info("last:               ", trace.at(i).avmMini_last);

        info("=======MEM_OP_A======================================================================");
        info("clk:                ", trace.at(i).avmMini_clk);
        info("mem_op_a:           ", trace.at(i).avmMini_mem_op_a);
        info("mem_idx_a:          ", trace.at(i).avmMini_mem_idx_a);
        info("rwa:                ", trace.at(i).avmMini_rwa);

        info("=======MEM_OP_B======================================================================");
        info("mem_op_b:           ", trace.at(i).avmMini_mem_op_b);
        info("mem_idx_b:          ", trace.at(i).avmMini_mem_idx_b);
        info("rwb:                ", trace.at(i).avmMini_rwb);

        info("=======MEM_OP_C======================================================================");
        info("mem_op_c:           ", trace.at(i).avmMini_mem_op_c);
        info("mem_idx_c:          ", trace.at(i).avmMini_mem_idx_c);
        info("rwc:                ", trace.at(i).avmMini_rwc);
        info("\n");
    }
}

} // namespace avm_trace