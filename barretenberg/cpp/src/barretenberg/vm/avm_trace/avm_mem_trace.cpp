#include "avm_mem_trace.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/avm_trace/avm_trace.hpp"
#include <cstdint>

namespace bb::avm_trace {

/**
 * @brief Constructor of a memory trace builder of AVM. Only serves to set the capacity of the
 *        underlying traces.
 */
AvmMemTraceBuilder::AvmMemTraceBuilder()
{
    mem_trace.reserve(AVM_TRACE_SIZE);
}

/**
 * @brief Resetting the internal state so that a new memory trace can be rebuilt using the same object.
 *
 */
void AvmMemTraceBuilder::reset()
{
    mem_trace.clear();
    memory.clear();
}

/**
 * @brief Prepare the memory trace to be incorporated into the main trace.
 *
 * @return The memory trace (which is moved).
 */
std::vector<AvmMemTraceBuilder::MemoryTraceEntry> AvmMemTraceBuilder::finalize()
{
    // Sort avm_mem
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
 * @param m_tag Memory tag associated with the value
 * @param r_in_tag Read memory tag pertaining to the instruction
 * @param w_in_tag Write memory tag pertaining to the instruction
 * @param m_rw Boolean telling whether it is a load (false) or store operation (true).
 */
void AvmMemTraceBuilder::insert_in_mem_trace(uint32_t const m_clk,
                                             uint32_t const m_sub_clk,
                                             uint32_t const m_addr,
                                             FF const& m_val,
                                             AvmMemoryTag const m_tag,
                                             AvmMemoryTag const r_in_tag,
                                             AvmMemoryTag const w_in_tag,
                                             bool const m_rw)
{
    mem_trace.emplace_back(MemoryTraceEntry{
        .m_clk = m_clk,
        .m_sub_clk = m_sub_clk,
        .m_addr = m_addr,
        .m_val = m_val,
        .m_tag = m_tag,
        .r_in_tag = r_in_tag,
        .w_in_tag = w_in_tag,
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
 * @param r_in_tag Memory read tag pertaining to the instruction
 * @param r_in_tag Memory write tag pertaining to the instruction
 * @param m_tag Memory tag pertaining to the address
 */
void AvmMemTraceBuilder::load_mismatch_tag_in_mem_trace(uint32_t const m_clk,
                                                        uint32_t const m_sub_clk,
                                                        uint32_t const m_addr,
                                                        FF const& m_val,
                                                        AvmMemoryTag const r_in_tag,
                                                        AvmMemoryTag const w_in_tag,
                                                        AvmMemoryTag const m_tag)
{
    FF one_min_inv = FF(1) - (FF(static_cast<uint32_t>(r_in_tag)) - FF(static_cast<uint32_t>(m_tag))).invert();

    // Relevant for inclusion (lookup) check #[INCL_MEM_TAG_ERR]. We need to
    // flag the first memory entry per clk key. The number of memory entries
    // with m_tag_err enabled can be higher than one for a given clk value.
    // The repetition of the same clk in the lookup table side (right hand
    // side, here, memory table) should be accounted for ONLY ONCE.
    bool tag_err_count_relevant = !m_tag_err_lookup_counts.contains(m_clk);

    // Lookup counter hint, used for #[INCL_MAIN_TAG_ERR] lookup (joined on clk)
    m_tag_err_lookup_counts[m_clk]++;

    mem_trace.emplace_back(MemoryTraceEntry{ .m_clk = m_clk,
                                             .m_sub_clk = m_sub_clk,
                                             .m_addr = m_addr,
                                             .m_val = m_val,
                                             .m_tag = m_tag,
                                             .r_in_tag = r_in_tag,
                                             .w_in_tag = w_in_tag,
                                             .m_tag_err = true,
                                             .m_one_min_inv = one_min_inv,
                                             .m_tag_err_count_relevant = tag_err_count_relevant });
}

/**
 * @brief Add a memory trace entry corresponding to a memory load.
 *
 * @param clk The main clock
 * @param sub_clk The sub-clock pertaining to the memory operation
 * @param addr The memory address
 * @param val The value to be loaded
 * @param r_in_tag The read memory tag of the instruction
 * @param w_in_tag The write memory tag of the instruction
 *
 * @return A boolean indicating that memory tag matches (resp. does not match) the
 *         instruction tag. Set to false in case of a mismatch.
 */
bool AvmMemTraceBuilder::load_from_mem_trace(
    uint32_t clk, uint32_t sub_clk, uint32_t addr, FF const& val, AvmMemoryTag r_in_tag, AvmMemoryTag w_in_tag)
{
    AvmMemoryTag m_tag = memory.contains(addr) ? memory.at(addr).tag : AvmMemoryTag::U0;

    if (m_tag == AvmMemoryTag::U0 || m_tag == r_in_tag) {
        insert_in_mem_trace(clk, sub_clk, addr, val, r_in_tag, r_in_tag, w_in_tag, false);
        return true;
    }

    // Handle memory tag inconsistency
    load_mismatch_tag_in_mem_trace(clk, sub_clk, addr, val, r_in_tag, w_in_tag, m_tag);
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
 * @param r_in_tag The write memory tag of the instruction
 * @param w_in_tag The write memory tag of the instruction
 */
void AvmMemTraceBuilder::store_in_mem_trace(
    uint32_t clk, IntermRegister interm_reg, uint32_t addr, FF const& val, AvmMemoryTag r_in_tag, AvmMemoryTag w_in_tag)
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
    case IntermRegister::ID:
        sub_clk = SUB_CLK_STORE_D;
        break;
    }

    insert_in_mem_trace(clk, sub_clk, addr, val, w_in_tag, r_in_tag, w_in_tag, true);
}

/**
 * @brief Handle a read memory operation specific to MOV opcode. Load the corresponding
 *        value to the intermediate register ia. A memory trace entry for the load
 *        operation is added. It is permissive in the sense that we do not enforce tag
 *        matching against any instruction tag. In addition, the specific selector
 *        for MOV opcode is enabled.
 *
 * @param clk Main clock
 * @param addr Memory address of the source offset
 *
 * @return Result of the read operation containing the value and the tag of the memory cell
 *         at the supplied address.
 */
AvmMemTraceBuilder::MemEntry AvmMemTraceBuilder::read_and_load_mov_opcode(uint32_t const clk, uint32_t const addr)
{
    MemEntry mem_entry = memory.contains(addr) ? memory.at(addr) : MemEntry{};

    mem_trace.emplace_back(MemoryTraceEntry{
        .m_clk = clk,
        .m_sub_clk = SUB_CLK_LOAD_A,
        .m_addr = addr,
        .m_val = mem_entry.val,
        .m_tag = mem_entry.tag,
        .r_in_tag = mem_entry.tag,
        .w_in_tag = mem_entry.tag,
        .m_sel_mov_a = true,
    });

    return mem_entry;
}

/**
 * @brief Handle a read memory operation specific to CMOV opcode. Load the corresponding
 *        values to the intermediate register ia, ib, id. Three memory trace entries for
 *        these load operations are added. They are permissive in the sense that we do not
 *        enforce tag matching against any instruction tag. In addition, the specific selector
 *        for CMOV opcode is enabled.
 *
 * @param clk Main clock
 * @param a_addr Memory address of the first value candidate a.
 * @param b_addr Memory address of the second value candidate b.
 * @param cond_addr Memory address of the conditional value.
 *
 * @return Result of the read operation containing the value and the tag of the memory cell
 *         at the supplied address.
 */
std::array<AvmMemTraceBuilder::MemEntry, 3> AvmMemTraceBuilder::read_and_load_cmov_opcode(uint32_t clk,
                                                                                          uint32_t a_addr,
                                                                                          uint32_t b_addr,
                                                                                          uint32_t cond_addr)
{
    MemEntry a_mem_entry = memory.contains(a_addr) ? memory.at(a_addr) : MemEntry{};
    MemEntry b_mem_entry = memory.contains(b_addr) ? memory.at(b_addr) : MemEntry{};
    MemEntry cond_mem_entry = memory.contains(cond_addr) ? memory.at(cond_addr) : MemEntry{};

    bool mov_b = cond_mem_entry.val == 0;

    AvmMemoryTag r_w_in_tag = mov_b ? b_mem_entry.tag : a_mem_entry.tag;

    mem_trace.emplace_back(MemoryTraceEntry{
        .m_clk = clk,
        .m_sub_clk = SUB_CLK_LOAD_A,
        .m_addr = a_addr,
        .m_val = a_mem_entry.val,
        .m_tag = a_mem_entry.tag,
        .r_in_tag = r_w_in_tag,
        .w_in_tag = r_w_in_tag,
        .m_sel_mov_a = !mov_b,
        .m_sel_cmov = true,
    });

    mem_trace.emplace_back(MemoryTraceEntry{
        .m_clk = clk,
        .m_sub_clk = SUB_CLK_LOAD_B,
        .m_addr = b_addr,
        .m_val = b_mem_entry.val,
        .m_tag = b_mem_entry.tag,
        .r_in_tag = r_w_in_tag,
        .w_in_tag = r_w_in_tag,
        .m_sel_mov_b = mov_b,
        .m_sel_cmov = true,
    });

    mem_trace.emplace_back(MemoryTraceEntry{
        .m_clk = clk,
        .m_sub_clk = SUB_CLK_LOAD_D,
        .m_addr = cond_addr,
        .m_val = cond_mem_entry.val,
        .m_tag = cond_mem_entry.tag,
        .r_in_tag = r_w_in_tag,
        .w_in_tag = r_w_in_tag,
        .m_sel_cmov = true,
    });

    return { a_mem_entry, b_mem_entry, cond_mem_entry };
}

/**
 * @brief Handle a read memory operation specific to CAST opcode. Load the corresponding
 *        value to the intermediate register ia. A memory trace entry for the load
 *        operation is added. It is permissive in the sense that we do not enforce tag
 *        matching against any instruction tag. The write instruction tag w_in_tag
 *        is passed and added in the memory trace entry.
 *
 * @param clk Main clock
 * @param addr Memory address of the source offset
 * @param w_in_tag Write instruction instruction tag (tag the value is casted to)
 *
 * @return Result of the read operation containing the value and the tag of the memory cell
 *         at the supplied address.
 */
AvmMemTraceBuilder::MemEntry AvmMemTraceBuilder::read_and_load_cast_opcode(uint32_t clk,
                                                                           uint32_t addr,
                                                                           AvmMemoryTag w_in_tag)
{
    MemEntry mem_entry = memory.contains(addr) ? memory.at(addr) : MemEntry{};

    mem_trace.emplace_back(MemoryTraceEntry{
        .m_clk = clk,
        .m_sub_clk = SUB_CLK_LOAD_A,
        .m_addr = addr,
        .m_val = mem_entry.val,
        .m_tag = mem_entry.tag,
        .r_in_tag = mem_entry.tag,
        .w_in_tag = w_in_tag,
    });

    return mem_entry;
}

/**
 * @brief Handle a read memory operation and load the corresponding value to the
 *        supplied intermediate register. A memory trace entry for the load operation
 *        is added.
 *
 * @param clk Main clock
 * @param interm_reg Intermediate register where we load the value
 * @param addr Memory address to be read and loaded
 * @param r_in_tag Read memory instruction tag
 * @param w_in_tag Write memory instruction tag
 *
 * @return Result of the read operation containing the value and a boolean telling
 *         potential mismatch between instruction tag and memory tag of the address.
 */
AvmMemTraceBuilder::MemRead AvmMemTraceBuilder::read_and_load_from_memory(uint32_t const clk,
                                                                          IntermRegister const interm_reg,
                                                                          uint32_t const addr,
                                                                          AvmMemoryTag const r_in_tag,
                                                                          AvmMemoryTag const w_in_tag)
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
    case IntermRegister::ID:
        sub_clk = SUB_CLK_LOAD_D;
        break;
    }

    FF val = memory.contains(addr) ? memory.at(addr).val : 0;
    bool tagMatch = load_from_mem_trace(clk, sub_clk, addr, val, r_in_tag, w_in_tag);

    return MemRead{
        .tag_match = tagMatch,
        .val = val,
    };
}

AvmMemTraceBuilder::MemRead AvmMemTraceBuilder::indirect_read_and_load_from_memory(uint32_t clk,
                                                                                   IndirectRegister ind_reg,
                                                                                   uint32_t addr)
{
    uint32_t sub_clk = 0;
    switch (ind_reg) {
    case IndirectRegister::IND_A:
        sub_clk = SUB_CLK_IND_LOAD_A;
        break;
    case IndirectRegister::IND_B:
        sub_clk = SUB_CLK_IND_LOAD_B;
        break;
    case IndirectRegister::IND_C:
        sub_clk = SUB_CLK_IND_LOAD_C;
        break;
    case IndirectRegister::IND_D:
        sub_clk = SUB_CLK_IND_LOAD_D;
        break;
    }

    FF val = memory.contains(addr) ? memory.at(addr).val : 0;
    bool tagMatch = load_from_mem_trace(clk, sub_clk, addr, val, AvmMemoryTag::U32, AvmMemoryTag::U0);

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
 * @param r_in_tag Read memory instruction tag
 * @param w_in_tag Write memory instruction tag
 */
void AvmMemTraceBuilder::write_into_memory(uint32_t const clk,
                                           IntermRegister interm_reg,
                                           uint32_t addr,
                                           FF const& val,
                                           AvmMemoryTag r_in_tag,
                                           AvmMemoryTag w_in_tag)
{
    MemEntry memEntry{ val, w_in_tag };
    auto it = memory.find(addr);
    if (it != memory.end()) {
        it->second = memEntry;
    } else {
        memory.emplace(addr, memEntry);
    }
    store_in_mem_trace(clk, interm_reg, addr, val, r_in_tag, w_in_tag);
}

} // namespace bb::avm_trace