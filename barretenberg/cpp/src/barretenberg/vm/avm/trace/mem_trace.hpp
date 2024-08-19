#pragma once

#include "barretenberg/vm/avm/trace/common.hpp"

#include <cstdint>

namespace bb::avm_trace {

class AvmMemTraceBuilder {

  public:
    static const uint32_t SUB_CLK_IND_LOAD_A = 0;
    static const uint32_t SUB_CLK_IND_LOAD_B = 1;
    static const uint32_t SUB_CLK_IND_LOAD_C = 2;
    static const uint32_t SUB_CLK_IND_LOAD_D = 3;
    static const uint32_t SUB_CLK_LOAD_A = 4;
    static const uint32_t SUB_CLK_LOAD_B = 5;
    static const uint32_t SUB_CLK_LOAD_C = 6;
    static const uint32_t SUB_CLK_LOAD_D = 7;
    static const uint32_t SUB_CLK_STORE_A = 8;
    static const uint32_t SUB_CLK_STORE_B = 9;
    static const uint32_t SUB_CLK_STORE_C = 10;
    static const uint32_t SUB_CLK_STORE_D = 11;
    static const uint32_t NUM_SUB_CLK = 12;

    // Keeps track of the number of times a mem tag err should appear in the trace
    // clk -> count
    std::map<uint32_t, uint32_t> m_tag_err_lookup_counts;

    struct MemoryTraceEntry {
        uint8_t m_space_id = 0;
        uint32_t m_clk = 0;
        uint32_t m_sub_clk = 0;
        uint32_t m_addr = 0;
        FF m_val{};
        AvmMemoryTag m_tag;
        AvmMemoryTag r_in_tag;
        AvmMemoryTag w_in_tag;
        bool m_rw = false;
        bool m_tag_err = false;
        FF m_one_min_inv{};
        bool m_sel_mov_ia_to_ic = false;
        bool m_sel_mov_ib_to_ic = false;
        bool m_sel_cmov = false;
        bool m_tag_err_count_relevant = false;
        bool m_sel_op_slice = false;

        bool poseidon_mem_op = false;

        /**
         * @brief A comparator on MemoryTraceEntry to be used by sorting algorithm. We sort first by
         *        ascending address (m_addr), then by clock (m_clk) and finally sub-clock (m_sub_clk).
         */
        bool operator<(MemoryTraceEntry const& other) const;
    };

    // Structure representing an entry for the memory used in the simulation (not the trace).
    struct MemEntry {
        FF val{};
        AvmMemoryTag tag = AvmMemoryTag::U0;
    };

    // Structure to return value and tag matching boolean after a memory read.
    struct MemRead {
        bool tag_match = false;
        FF val{};
    };

    // Enum to define which subtable triggered and manages (via perm) the memory operation.
    enum MemOpOwner {
        MAIN,
        SLICE,
        POSEIDON2,
    };

    AvmMemTraceBuilder() = default;

    void reset();

    std::vector<MemoryTraceEntry> finalize();

    MemEntry read_and_load_mov_opcode(uint8_t space_id, uint32_t clk, uint32_t addr);
    std::array<MemEntry, 3> read_and_load_cmov_opcode(
        uint8_t space_id, uint32_t clk, uint32_t a_addr, uint32_t b_addr, uint32_t cond_addr);
    MemEntry read_and_load_jumpi_opcode(uint8_t space_id, uint32_t clk, uint32_t cond_addr);
    MemEntry read_and_load_cast_opcode(uint8_t space_id, uint32_t clk, uint32_t addr, AvmMemoryTag w_in_tag);
    MemRead read_and_load_from_memory(uint8_t space_id,
                                      uint32_t clk,
                                      IntermRegister interm_reg,
                                      uint32_t addr,
                                      AvmMemoryTag r_in_tag,
                                      AvmMemoryTag w_in_tag,
                                      MemOpOwner mem_op_owner = MAIN);
    MemRead indirect_read_and_load_from_memory(uint8_t space_id, uint32_t clk, IndirectRegister ind_reg, uint32_t addr);
    void write_into_memory(uint8_t space_id,
                           uint32_t clk,
                           IntermRegister interm_reg,
                           uint32_t addr,
                           FF const& val,
                           AvmMemoryTag r_in_tag,
                           AvmMemoryTag w_in_tag,
                           MemOpOwner mem_op_owner = MAIN);
    void write_calldata_copy(std::vector<FF> const& calldata,
                             uint32_t clk,
                             uint8_t space_id,
                             uint32_t cd_offset,
                             uint32_t copy_size,
                             uint32_t direct_dst_offset);
    std::vector<FF> read_return_opcode(uint32_t clk, uint8_t space_id, uint32_t direct_ret_offset, uint32_t ret_size);

    // DO NOT USE FOR REAL OPERATIONS
    FF unconstrained_read(uint8_t space_id, uint32_t addr) { return memory[space_id][addr].val; }

  private:
    std::vector<MemoryTraceEntry> mem_trace; // Entries will be sorted by m_clk, m_sub_clk after finalize().

    // Global Memory table (used for simulation): (space_id, (address, mem_entry))
    std::array<std::unordered_map<uint32_t, MemEntry>, NUM_MEM_SPACES> memory;

    void insert_in_mem_trace(uint8_t space_id,
                             uint32_t m_clk,
                             uint32_t m_sub_clk,
                             uint32_t m_addr,
                             FF const& m_val,
                             AvmMemoryTag m_tag,
                             AvmMemoryTag r_in_tag,
                             AvmMemoryTag w_in_tag,
                             bool m_rw,
                             MemOpOwner mem_op_owner);

    void load_mismatch_tag_in_mem_trace(uint8_t space_id,
                                        uint32_t m_clk,
                                        uint32_t m_sub_clk,
                                        uint32_t m_addr,
                                        FF const& m_val,
                                        AvmMemoryTag r_in_tag,
                                        AvmMemoryTag w_in_tag,
                                        AvmMemoryTag m_tag);

    bool load_from_mem_trace(uint8_t space_id,
                             uint32_t clk,
                             uint32_t sub_clk,
                             uint32_t addr,
                             FF const& val,
                             AvmMemoryTag r_in_tag,
                             AvmMemoryTag w_in_tag,
                             MemOpOwner mem_op_owner = MAIN);
    void store_in_mem_trace(uint8_t space_id,
                            uint32_t clk,
                            IntermRegister interm_reg,
                            uint32_t addr,
                            FF const& val,
                            AvmMemoryTag r_in_tag,
                            AvmMemoryTag w_in_tag,
                            MemOpOwner mem_op_owner = MAIN);
    void write_in_simulated_mem_table(uint8_t space_id, uint32_t addr, FF const& val, AvmMemoryTag w_in_tag);
};
} // namespace bb::avm_trace
