#pragma once

#include "AvmMini_common.hpp"

namespace avm_trace {

class AvmMiniMemTraceBuilder {

  public:
    static const size_t MEM_SIZE = 1024;
    static const uint32_t SUB_CLK_LOAD_A = 0;
    static const uint32_t SUB_CLK_LOAD_B = 1;
    static const uint32_t SUB_CLK_LOAD_C = 2;
    static const uint32_t SUB_CLK_STORE_A = 3;
    static const uint32_t SUB_CLK_STORE_B = 4;
    static const uint32_t SUB_CLK_STORE_C = 5;

    struct MemoryTraceEntry {
        uint32_t m_clk{};
        uint32_t m_sub_clk{};
        uint32_t m_addr{};
        FF m_val{};
        AvmMemoryTag m_tag{};
        AvmMemoryTag m_in_tag{};
        bool m_rw = false;
        bool m_tag_err = false;
        FF m_one_min_inv{};

        /**
         * @brief A comparator on MemoryTraceEntry to be used by sorting algorithm. We sort first by
         *        ascending address (m_addr), then by clock (m_clk) and finally sub-clock (m_sub_clk).
         */
        bool operator<(const MemoryTraceEntry& other) const
        {
            if (m_addr < other.m_addr) {
                return true;
            }

            if (m_addr > other.m_addr) {
                return false;
            }

            if (m_clk < other.m_clk) {
                return true;
            }

            if (m_clk > other.m_clk) {
                return false;
            }

            // No safeguard in case they are equal. The caller should ensure this property.
            // Otherwise, relation will not be satisfied.
            return m_sub_clk < other.m_sub_clk;
        }
    };

    // Structure to return value and tag matching boolean after a memory read.
    struct MemRead {
        bool tag_match = false;
        FF val{};
    };

    AvmMiniMemTraceBuilder();

    void reset();

    std::vector<MemoryTraceEntry> finalize();

    MemRead read_and_load_from_memory(uint32_t clk, IntermRegister interm_reg, uint32_t addr, AvmMemoryTag m_in_tag);
    void write_into_memory(
        uint32_t clk, IntermRegister interm_reg, uint32_t addr, FF const& val, AvmMemoryTag m_in_tag);

  private:
    std::vector<MemoryTraceEntry> mem_trace;         // Entries will be sorted by m_clk, m_sub_clk after finalize().
    std::array<FF, MEM_SIZE> memory{};               // Memory table (used for simulation)
    std::array<AvmMemoryTag, MEM_SIZE> memory_tag{}; // The tag of the corresponding memory
                                                     // entry (aligned with the memory array).

    void insert_in_mem_trace(
        uint32_t m_clk, uint32_t m_sub_clk, uint32_t m_addr, FF const& m_val, AvmMemoryTag m_in_tag, bool m_rw);
    void load_mismatch_tag_in_mem_trace(uint32_t m_clk,
                                        uint32_t m_sub_clk,
                                        uint32_t m_addr,
                                        FF const& m_val,
                                        AvmMemoryTag m_in_tag,
                                        AvmMemoryTag m_tag);

    bool load_in_mem_trace(
        uint32_t clk, IntermRegister interm_reg, uint32_t addr, FF const& val, AvmMemoryTag m_in_tag);
    void store_in_mem_trace(
        uint32_t clk, IntermRegister interm_reg, uint32_t addr, FF const& val, AvmMemoryTag m_in_tag);
};
} // namespace avm_trace