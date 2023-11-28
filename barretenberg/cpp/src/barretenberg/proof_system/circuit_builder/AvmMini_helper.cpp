#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/proof_system/circuit_builder/circuit_builder_base.hpp"

#include "barretenberg/flavor/generated/AvmMini_flavor.hpp"
#include "barretenberg/relations/generated/AvmMini.hpp"

#include "./AvmMini_helper.hpp"

namespace proof_system {

/**
 * @brief Routine to log some slice of a trace of the AVM. Used to debug or in some unit tests.
 *
 * @param trace The whole trace for AVM as a vector of rows.
 * @param beg The index of the beginning of the slice. (included)
 * @param end The index of the end of the slice (not included).
 */
void log_avmMini_trace(std::vector<Row> const& trace, size_t beg, size_t end)
{
    info("Built circuit with ", trace.size(), " trace");

    for (size_t i = beg; i < end; i++) {
        info("================================================================================");
        info("==        ROW ", i);
        info("================================================================================");

        info("m_addr:       ", trace.at(i).avmMini_m_addr);
        info("m_clk:        ", trace.at(i).avmMini_m_clk);
        info("m_sub_clk:    ", trace.at(i).avmMini_m_sub_clk);
        info("m_val:        ", trace.at(i).avmMini_m_val);
        info("m_lastAccess: ", trace.at(i).avmMini_m_lastAccess);
        info("m_rw:         ", trace.at(i).avmMini_m_rw);
        info("m_val_shift:  ", trace.at(i).avmMini_m_val_shift);
        info("first:        ", trace.at(i).avmMini_first);
        info("last:         ", trace.at(i).avmMini_last);

        info("=======MEM_OP_A=================================================================");
        info("clk:          ", trace.at(i).avmMini_clk);
        info("mem_op_a:     ", trace.at(i).avmMini_mem_op_a);
        info("mem_idx_a:    ", trace.at(i).avmMini_mem_idx_a);
        info("ia:           ", trace.at(i).avmMini_ia);
        info("rwa:          ", trace.at(i).avmMini_rwa);

        info("=======MEM_OP_B=================================================================");
        info("mem_op_b:     ", trace.at(i).avmMini_mem_op_b);
        info("mem_idx_b:    ", trace.at(i).avmMini_mem_idx_b);
        info("ib:           ", trace.at(i).avmMini_ib);
        info("rwb:          ", trace.at(i).avmMini_rwb);

        info("=======MEM_OP_C=================================================================");
        info("mem_op_c:     ", trace.at(i).avmMini_mem_op_c);
        info("mem_idx_c:    ", trace.at(i).avmMini_mem_idx_c);
        info("ic:           ", trace.at(i).avmMini_ic);
        info("rwc:          ", trace.at(i).avmMini_rwc);
        info("\n");
    }
}

} // namespace proof_system