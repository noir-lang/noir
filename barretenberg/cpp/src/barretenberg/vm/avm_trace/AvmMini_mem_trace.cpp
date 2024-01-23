#include "AvmMini_mem_trace.hpp"

namespace avm_trace {

/**
 * @brief Constructor of a memory trace builder of AVM. Only serves to set the capacity of the
 *        underlying traces.
 */
AvmMiniMemTraceBuilder::AvmMiniMemTraceBuilder()
{
    mem_trace.reserve(AVM_TRACE_SIZE);
}

/**
 * @brief Resetting the internal state so that a new memory trace can be rebuilt using the same object.
 *
 */
void AvmMiniMemTraceBuilder::reset()
{
    mem_trace.clear();
    memory.fill(FF(0));
}

/**
 * @brief Prepare the memory trace to be incorporated into the main trace.
 *
 * @return The memory trace (which is moved).
 */
std::vector<AvmMiniMemTraceBuilder::MemoryTraceEntry> AvmMiniMemTraceBuilder::finalize()
{
    // Sort memTrace
    std::sort(mem_trace.begin(), mem_trace.end());
    return std::move(mem_trace);
}

/**
 * @brief A method to insert a row/entry in the memory trace.
 *
 * @param m_clk Main clock
 * @param m_sub_clk Sub-clock used to order load/store sub operations
 * @param m_addr Address pertaining to the memory operation
 * @param m_val Value (FF) pertaining to the memory operation
 * @param m_in_tag Memory tag pertaining to the instruction
 * @param m_rw Boolean telling whether it is a load (false) or store operation (true).
 */
void AvmMiniMemTraceBuilder::insert_in_mem_trace(uint32_t const m_clk,
                                                 uint32_t const m_sub_clk,
                                                 uint32_t const m_addr,
                                                 FF const& m_val,
                                                 AvmMemoryTag const m_in_tag,
                                                 bool const m_rw)
{
    mem_trace.emplace_back(MemoryTraceEntry{
        .m_clk = m_clk,
        .m_sub_clk = m_sub_clk,
        .m_addr = m_addr,
        .m_val = m_val,
        .m_tag = m_in_tag,
        .m_in_tag = m_in_tag,
        .m_rw = m_rw,
    });
}

// Memory operations need to be performed before the addition of the corresponding row in
// MainTrace, otherwise the m_clk value will be wrong. This applies to loadInMemTrace and
// storeInMemTrace.

/**
 * @brief Add a memory trace entry for a load with a memory tag mismatching the instruction
 *        memory tag.
 *
 * @param m_clk Main clock
 * @param m_sub_clk Sub-clock used to order load/store sub operations
 * @param m_addr Address pertaining to the memory operation
 * @param m_val Value (FF) pertaining to the memory operation
 * @param m_in_tag Memory tag pertaining to the instruction
 * @param m_tag Memory tag pertaining to the address
 */
void AvmMiniMemTraceBuilder::load_mismatch_tag_in_mem_trace(uint32_t const m_clk,
                                                            uint32_t const m_sub_clk,
                                                            uint32_t const m_addr,
                                                            FF const& m_val,
                                                            AvmMemoryTag const m_in_tag,
                                                            AvmMemoryTag const m_tag)
{
    FF one_min_inv = FF(1) - (FF(static_cast<uint32_t>(m_in_tag)) - FF(static_cast<uint32_t>(m_tag))).invert();
    mem_trace.emplace_back(MemoryTraceEntry{ .m_clk = m_clk,
                                             .m_sub_clk = m_sub_clk,
                                             .m_addr = m_addr,
                                             .m_val = m_val,
                                             .m_tag = m_tag,
                                             .m_in_tag = m_in_tag,
                                             .m_tag_err = true,
                                             .m_one_min_inv = one_min_inv });
}

/**
 * @brief Add a memory trace entry corresponding to a memory load into the intermediate
 *        passed register.
 *
 * @param clk The main clock
 * @param interm_reg The intermediate register
 * @param addr The memory address
 * @param val The value to be loaded
 * @param m_in_tag The memory tag of the instruction
 *
 * @return A boolean indicating that memory tag matches (resp. does not match) the
 *         instruction tag. Set to false in case of a mismatch.
 */
bool AvmMiniMemTraceBuilder::load_in_mem_trace(
    uint32_t clk, IntermRegister interm_reg, uint32_t addr, FF const& val, AvmMemoryTag m_in_tag)
{
    uint32_t sub_clk = 0;
    switch (interm_reg) {
    case IntermRegister::IA:
        sub_clk = SUB_CLK_LOAD_A;
        break;
    case IntermRegister::IB:
        sub_clk = SUB_CLK_LOAD_B;
        break;
    case IntermRegister::IC:
        sub_clk = SUB_CLK_LOAD_C;
        break;
    }

    auto m_tag = memory_tag.at(addr);
    if (m_tag == AvmMemoryTag::U0 || m_tag == m_in_tag) {
        insert_in_mem_trace(clk, sub_clk, addr, val, m_in_tag, false);
        return true;
    }

    // Handle memory tag inconsistency
    load_mismatch_tag_in_mem_trace(clk, sub_clk, addr, val, m_in_tag, m_tag);
    return false;
}

/**
 * @brief Add a memory trace entry corresponding to a memory store from the intermediate
 *        register.
 *
 * @param clk The main clock
 * @param interm_reg The intermediate register
 * @param addr The memory address
 * @param val The value to be stored
 * @param m_in_tag The memory tag of the instruction
 */
void AvmMiniMemTraceBuilder::store_in_mem_trace(
    uint32_t clk, IntermRegister interm_reg, uint32_t addr, FF const& val, AvmMemoryTag m_in_tag)
{
    uint32_t sub_clk = 0;
    switch (interm_reg) {
    case IntermRegister::IA:
        sub_clk = SUB_CLK_STORE_A;
        break;
    case IntermRegister::IB:
        sub_clk = SUB_CLK_STORE_B;
        break;
    case IntermRegister::IC:
        sub_clk = SUB_CLK_STORE_C;
        break;
    }

    insert_in_mem_trace(clk, sub_clk, addr, val, m_in_tag, true);
}

/**
 * @brief Handle a read memory operation and load the corresponding value to the
 *        supplied intermediate register. A memory trace entry for the load operation
 *        is added.
 *
 * @param clk Main clock
 * @param interm_reg Intermediate register where we load the value
 * @param addr Memory address to be read and loaded
 * @param m_in_tag Memory instruction tag
 *
 * @return Result of the read operation containing the value and a boolean telling
 *         potential mismatch between instruction tag and memory tag of the address.
 */
AvmMiniMemTraceBuilder::MemRead AvmMiniMemTraceBuilder::read_and_load_from_memory(uint32_t const clk,
                                                                                  IntermRegister const interm_reg,
                                                                                  uint32_t const addr,
                                                                                  AvmMemoryTag const m_in_tag)
{
    FF val = memory.at(addr);
    bool tagMatch = load_in_mem_trace(clk, interm_reg, addr, val, m_in_tag);

    return MemRead{
        .tag_match = tagMatch,
        .val = val,
    };
}

/**
 * @brief Handle a write memory operation and store the supplied value into memory
 *        at the supplied address. A memory trace entry for the write operation
 *        is added.
 *
 * @param clk Main clock
 * @param interm_reg Intermediate register where we write the value
 * @param addr Memory address to be written to
 * @param val Value to be written into memory
 * @param m_in_tag Memory instruction tag
 */
void AvmMiniMemTraceBuilder::write_into_memory(
    uint32_t const clk, IntermRegister interm_reg, uint32_t addr, FF const& val, AvmMemoryTag m_in_tag)
{
    memory.at(addr) = val;
    memory_tag.at(addr) = m_in_tag;
    store_in_mem_trace(clk, interm_reg, addr, val, m_in_tag);
}

} // namespace avm_trace