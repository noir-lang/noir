#include <array>
#include <cassert>
#include <cstddef>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <string>
#include <sys/types.h>
#include <vector>

#include "./AvmMini_trace.hpp"
#include "./generated/AvmMini_circuit_builder.hpp"

namespace proof_system {

/**
 * @brief Constructor of a trace builder of AVM. Only serves to set the capacity of the
 *        underlying traces.
 */
AvmMiniTraceBuilder::AvmMiniTraceBuilder()
{
    mainTrace.reserve(N);
    memTrace.reserve(N);
}

/**
 * @brief Resetting the internal state so that a new trace can be rebuilt using the same object.
 *
 */
void AvmMiniTraceBuilder::reset()
{
    mainTrace.clear();
    memTrace.clear();
    memory.fill(FF(0));
}

/**
 * @brief A comparator on MemoryTraceEntry to be used by sorting algorithm.
 *
 */
bool AvmMiniTraceBuilder::compareMemEntries(const MemoryTraceEntry& left, const MemoryTraceEntry& right)
{
    if (left.m_addr < right.m_addr) {
        return true;
    }

    if (left.m_addr > right.m_addr) {
        return false;
    }

    if (left.m_clk < right.m_clk) {
        return true;
    }

    if (left.m_clk > right.m_clk) {
        return false;
    }

    // No safeguard in case they are equal. The caller should ensure this property.
    // Otherwise, relation will not be satisfied.
    return left.m_sub_clk < right.m_sub_clk;
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
void AvmMiniTraceBuilder::insertInMemTrace(
    uint32_t m_clk, uint32_t m_sub_clk, uint32_t m_addr, FF m_val, AvmMemoryTag m_in_tag, bool m_rw)
{
    memTrace.emplace_back(MemoryTraceEntry{
        .m_clk = m_clk,
        .m_sub_clk = m_sub_clk,
        .m_addr = m_addr,
        .m_val = m_val,
        .m_tag = m_in_tag,
        .m_in_tag = m_in_tag,
        .m_rw = m_rw,
    });
}

void AvmMiniTraceBuilder::loadMismatchTagInMemTrace(
    uint32_t m_clk, uint32_t m_sub_clk, uint32_t m_addr, FF m_val, AvmMemoryTag m_in_tag, AvmMemoryTag m_tag)
{
    FF one_min_inv = FF(1) - (FF(static_cast<uint32_t>(m_in_tag)) - FF(static_cast<uint32_t>(m_tag))).invert();
    memTrace.emplace_back(MemoryTraceEntry{ .m_clk = m_clk,
                                            .m_sub_clk = m_sub_clk,
                                            .m_addr = m_addr,
                                            .m_val = m_val,
                                            .m_tag = m_tag,
                                            .m_in_tag = m_in_tag,
                                            .m_tag_err = true,
                                            .m_one_min_inv = one_min_inv });
}

// Memory operations need to be performed before the addition of the corresponding row in
// MainTrace, otherwise the m_clk value will be wrong. This applies to loadInMemTrace and
// storeInMemTrace.

/**
 * @brief Add a memory trace entry corresponding to a memory load into the intermediate
 *        passed register.
 *
 * @param intermReg The intermediate register
 * @param addr The memory address
 * @param val The value to be loaded
 * @param m_in_tag The memory tag of the instruction
 */
bool AvmMiniTraceBuilder::loadInMemTrace(IntermRegister intermReg, uint32_t addr, FF val, AvmMemoryTag m_in_tag)
{
    uint32_t sub_clk = 0;
    switch (intermReg) {
    case IntermRegister::ia:
        sub_clk = SUB_CLK_LOAD_A;
        break;
    case IntermRegister::ib:
        sub_clk = SUB_CLK_LOAD_B;
        break;
    case IntermRegister::ic:
        sub_clk = SUB_CLK_LOAD_C;
        break;
    }

    auto m_tag = memoryTag.at(addr);
    if (m_tag == AvmMemoryTag::u0 || m_tag == m_in_tag) {
        insertInMemTrace(static_cast<uint32_t>(mainTrace.size()), sub_clk, addr, val, m_in_tag, false);
        return true;
    }

    // Handle memory tag inconsistency
    loadMismatchTagInMemTrace(static_cast<uint32_t>(mainTrace.size()), sub_clk, addr, val, m_in_tag, m_tag);
    return false;
}

/**
 * @brief Add a memory trace entry corresponding to a memory store from the intermediate
 *        register.
 *
 * @param intermReg The intermediate register
 * @param addr The memory address
 * @param val The value to be stored
 * @param m_in_tag The memory tag of the instruction
 */
void AvmMiniTraceBuilder::storeInMemTrace(IntermRegister intermReg, uint32_t addr, FF val, AvmMemoryTag m_in_tag)
{
    uint32_t sub_clk = 0;
    switch (intermReg) {
    case IntermRegister::ia:
        sub_clk = SUB_CLK_STORE_A;
        break;
    case IntermRegister::ib:
        sub_clk = SUB_CLK_STORE_B;
        break;
    case IntermRegister::ic:
        sub_clk = SUB_CLK_STORE_C;
        break;
    }

    insertInMemTrace(static_cast<uint32_t>(mainTrace.size()), sub_clk, addr, val, m_in_tag, true);
}

/** TODO: Implement for non finite field types
 * @brief Addition with direct memory access.
 *
 * @param aOffset An index in memory pointing to the first operand of the addition.
 * @param bOffset An index in memory pointing to the second operand of the addition.
 * @param dstOffset An index in memory pointing to the output of the addition.
 * @param inTag The instruction memory tag of the operands.
 */
void AvmMiniTraceBuilder::add(uint32_t aOffset, uint32_t bOffset, uint32_t dstOffset, AvmMemoryTag inTag)
{
    // a + b = c
    FF a = memory.at(aOffset);
    FF b = memory.at(bOffset);
    FF c = a + b;
    memory.at(dstOffset) = c;
    memoryTag.at(dstOffset) = inTag;

    // Loading into Ia, Ib and storing into Ic
    bool tagMatch = loadInMemTrace(IntermRegister::ia, aOffset, a, inTag);
    tagMatch = loadInMemTrace(IntermRegister::ib, bOffset, b, inTag) && tagMatch;
    storeInMemTrace(IntermRegister::ic, dstOffset, c, inTag);

    auto clk = mainTrace.size();

    mainTrace.push_back(Row{
        .avmMini_clk = clk,
        .avmMini_pc = FF(pc++),
        .avmMini_internal_return_ptr = FF(internal_return_ptr),
        .avmMini_sel_op_add = FF(1),
        .avmMini_in_tag = FF(static_cast<uint32_t>(inTag)),
        .avmMini_tag_err = FF(static_cast<uint32_t>(!tagMatch)),
        .avmMini_ia = tagMatch ? a : FF(0),
        .avmMini_ib = tagMatch ? b : FF(0),
        .avmMini_ic = tagMatch ? c : FF(0),
        .avmMini_mem_op_a = FF(1),
        .avmMini_mem_op_b = FF(1),
        .avmMini_mem_op_c = FF(1),
        .avmMini_rwc = FF(1),
        .avmMini_mem_idx_a = FF(aOffset),
        .avmMini_mem_idx_b = FF(bOffset),
        .avmMini_mem_idx_c = FF(dstOffset),
    });
};

/** TODO: Implement for non finite field types
 * @brief Subtraction with direct memory access.
 *
 * @param aOffset An index in memory pointing to the first operand of the subtraction.
 * @param bOffset An index in memory pointing to the second operand of the subtraction.
 * @param dstOffset An index in memory pointing to the output of the subtraction.
 * @param inTag The instruction memory tag of the operands.
 */
void AvmMiniTraceBuilder::sub(uint32_t aOffset, uint32_t bOffset, uint32_t dstOffset, AvmMemoryTag inTag)
{
    // a - b = c
    FF a = memory.at(aOffset);
    FF b = memory.at(bOffset);
    FF c = a - b;
    memory.at(dstOffset) = c;
    memoryTag.at(dstOffset) = inTag;

    // Loading into Ia, Ib and storing into Ic
    bool tagMatch = loadInMemTrace(IntermRegister::ia, aOffset, a, inTag);
    tagMatch = loadInMemTrace(IntermRegister::ib, bOffset, b, inTag) && tagMatch;
    storeInMemTrace(IntermRegister::ic, dstOffset, c, inTag);

    auto clk = mainTrace.size();

    mainTrace.push_back(Row{
        .avmMini_clk = clk,
        .avmMini_pc = FF(pc++),
        .avmMini_internal_return_ptr = FF(internal_return_ptr),
        .avmMini_sel_op_sub = FF(1),
        .avmMini_in_tag = FF(static_cast<uint32_t>(inTag)),
        .avmMini_tag_err = FF(static_cast<uint32_t>(!tagMatch)),
        .avmMini_ia = tagMatch ? a : FF(0),
        .avmMini_ib = tagMatch ? b : FF(0),
        .avmMini_ic = tagMatch ? c : FF(0),
        .avmMini_mem_op_a = FF(1),
        .avmMini_mem_op_b = FF(1),
        .avmMini_mem_op_c = FF(1),
        .avmMini_rwc = FF(1),
        .avmMini_mem_idx_a = FF(aOffset),
        .avmMini_mem_idx_b = FF(bOffset),
        .avmMini_mem_idx_c = FF(dstOffset),
    });
};

/** TODO: Implement for non finite field types
 * @brief Multiplication with direct memory access.
 *
 * @param aOffset An index in memory pointing to the first operand of the multiplication.
 * @param bOffset An index in memory pointing to the second operand of the multiplication.
 * @param dstOffset An index in memory pointing to the output of the multiplication.
 * @param inTag The instruction memory tag of the operands.
 */
void AvmMiniTraceBuilder::mul(uint32_t aOffset, uint32_t bOffset, uint32_t dstOffset, AvmMemoryTag inTag)
{
    // a * b = c
    FF a = memory.at(aOffset);
    FF b = memory.at(bOffset);
    FF c = a * b;
    memory.at(dstOffset) = c;
    memoryTag.at(dstOffset) = inTag;

    // Loading into Ia, Ib and storing into Ic
    bool tagMatch = loadInMemTrace(IntermRegister::ia, aOffset, a, inTag);
    tagMatch = loadInMemTrace(IntermRegister::ib, bOffset, b, inTag) && tagMatch;
    storeInMemTrace(IntermRegister::ic, dstOffset, c, inTag);

    auto clk = mainTrace.size();

    mainTrace.push_back(Row{
        .avmMini_clk = clk,
        .avmMini_pc = FF(pc++),
        .avmMini_internal_return_ptr = FF(internal_return_ptr),
        .avmMini_sel_op_mul = FF(1),
        .avmMini_in_tag = FF(static_cast<uint32_t>(inTag)),
        .avmMini_tag_err = FF(static_cast<uint32_t>(!tagMatch)),
        .avmMini_ia = tagMatch ? a : FF(0),
        .avmMini_ib = tagMatch ? b : FF(0),
        .avmMini_ic = tagMatch ? c : FF(0),
        .avmMini_mem_op_a = FF(1),
        .avmMini_mem_op_b = FF(1),
        .avmMini_mem_op_c = FF(1),
        .avmMini_rwc = FF(1),
        .avmMini_mem_idx_a = FF(aOffset),
        .avmMini_mem_idx_b = FF(bOffset),
        .avmMini_mem_idx_c = FF(dstOffset),
    });
}

/** TODO: Implement for non finite field types
 * @brief Division with direct memory access.
 *
 * @param aOffset An index in memory pointing to the first operand of the division.
 * @param bOffset An index in memory pointing to the second operand of the division.
 * @param dstOffset An index in memory pointing to the output of the division.
 * @param inTag The instruction memory tag of the operands.
 */
void AvmMiniTraceBuilder::div(uint32_t aOffset, uint32_t bOffset, uint32_t dstOffset, AvmMemoryTag inTag)
{
    // a * b^(-1) = c
    FF a = memory.at(aOffset);
    FF b = memory.at(bOffset);
    FF c;
    FF inv;
    FF error;

    if (!b.is_zero()) {

        inv = b.invert();
        c = a * inv;
        error = 0;
    } else {
        inv = 1;
        c = 0;
        error = 1;
    }

    memory.at(dstOffset) = c;
    memoryTag.at(dstOffset) = inTag;

    // Loading into Ia, Ib and storing into Ic
    bool tagMatch = loadInMemTrace(IntermRegister::ia, aOffset, a, inTag);
    tagMatch = loadInMemTrace(IntermRegister::ib, bOffset, b, inTag) && tagMatch;
    storeInMemTrace(IntermRegister::ic, dstOffset, c, inTag);

    auto clk = mainTrace.size();

    mainTrace.push_back(Row{
        .avmMini_clk = clk,
        .avmMini_pc = FF(pc++),
        .avmMini_internal_return_ptr = FF(internal_return_ptr),
        .avmMini_sel_op_div = FF(1),
        .avmMini_in_tag = FF(static_cast<uint32_t>(inTag)),
        .avmMini_op_err = tagMatch ? error : FF(1),
        .avmMini_tag_err = FF(static_cast<uint32_t>(!tagMatch)),
        .avmMini_inv = tagMatch ? inv : FF(1),
        .avmMini_ia = tagMatch ? a : FF(0),
        .avmMini_ib = tagMatch ? b : FF(0),
        .avmMini_ic = tagMatch ? c : FF(0),
        .avmMini_mem_op_a = FF(1),
        .avmMini_mem_op_b = FF(1),
        .avmMini_mem_op_c = FF(1),
        .avmMini_rwc = FF(1),
        .avmMini_mem_idx_a = FF(aOffset),
        .avmMini_mem_idx_b = FF(bOffset),
        .avmMini_mem_idx_c = FF(dstOffset),
    });
}

/**
 * @brief CALLDATACOPY opcode with direct memory access, i.e.,
 *        M[dstOffset:dstOffset+copySize] = calldata[cdOffset:cdOffset+copySize]
 *        Simplified version with exclusively memory store operations and
 *        values from M_calldata passed by an array and loaded into
 *        intermediate registers.
 *        Assume that caller passes callDataMem which is large enough so that
 *        no out-of-bound memory issues occur.
 *        TODO: Implement the indirect memory version (maybe not required)
 *        TODO: taking care of intermediate register values consistency and propagating their
 *        values to the next row when not overwritten.
 *
 * @param cdOffset The starting index of the region in calldata to be copied.
 * @param copySize The number of finite field elements to be copied into memory.
 * @param dstOffset The starting index of memory where calldata will be copied to.
 * @param callDataMem The vector containing calldata.
 */
void AvmMiniTraceBuilder::callDataCopy(uint32_t cdOffset,
                                       uint32_t copySize,
                                       uint32_t dstOffset,
                                       std::vector<FF> const& callDataMem)
{
    // We parallelize storing memory operations in chunk of 3, i.e., 1 per intermediate register.
    // The variable pos is an index pointing to the first storing operation (pertaining to intermediate
    // register Ia) relative to cdOffset:
    // cdOffset + pos:       Ia memory store operation
    // cdOffset + pos + 1:   Ib memory store operation
    // cdOffset + pos + 2:   Ic memory store operation

    uint32_t pos = 0;

    while (pos < copySize) {
        FF ib(0);
        FF ic(0);
        uint32_t mem_op_b(0);
        uint32_t mem_op_c(0);
        uint32_t mem_idx_b(0);
        uint32_t mem_idx_c(0);
        uint32_t rwb(0);
        uint32_t rwc(0);
        auto clk = mainTrace.size();

        FF ia = callDataMem.at(cdOffset + pos);
        uint32_t mem_op_a(1);
        uint32_t mem_idx_a = dstOffset + pos;
        uint32_t rwa = 1;

        // Storing from Ia
        memory.at(mem_idx_a) = ia;
        memoryTag.at(mem_idx_a) = AvmMemoryTag::ff;
        storeInMemTrace(IntermRegister::ia, mem_idx_a, ia, AvmMemoryTag::ff);

        if (copySize - pos > 1) {
            ib = callDataMem.at(cdOffset + pos + 1);
            mem_op_b = 1;
            mem_idx_b = dstOffset + pos + 1;
            rwb = 1;

            // Storing from Ib
            memory.at(mem_idx_b) = ib;
            memoryTag.at(mem_idx_b) = AvmMemoryTag::ff;
            storeInMemTrace(IntermRegister::ib, mem_idx_b, ib, AvmMemoryTag::ff);
        }

        if (copySize - pos > 2) {
            ic = callDataMem.at(cdOffset + pos + 2);
            mem_op_c = 1;
            mem_idx_c = dstOffset + pos + 2;
            rwc = 1;

            // Storing from Ic
            memory.at(mem_idx_c) = ic;
            memoryTag.at(mem_idx_c) = AvmMemoryTag::ff;
            storeInMemTrace(IntermRegister::ic, mem_idx_c, ic, AvmMemoryTag::ff);
        }

        mainTrace.push_back(Row{
            .avmMini_clk = clk,
            .avmMini_pc = FF(pc++),
            .avmMini_internal_return_ptr = FF(internal_return_ptr),
            .avmMini_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::ff)),
            .avmMini_ia = ia,
            .avmMini_ib = ib,
            .avmMini_ic = ic,
            .avmMini_mem_op_a = FF(mem_op_a),
            .avmMini_mem_op_b = FF(mem_op_b),
            .avmMini_mem_op_c = FF(mem_op_c),
            .avmMini_rwa = FF(rwa),
            .avmMini_rwb = FF(rwb),
            .avmMini_rwc = FF(rwc),
            .avmMini_mem_idx_a = FF(mem_idx_a),
            .avmMini_mem_idx_b = FF(mem_idx_b),
            .avmMini_mem_idx_c = FF(mem_idx_c),
        });

        if (copySize - pos > 2) { // Guard to prevent overflow if copySize is close to uint32_t maximum value.
            pos += 3;
        } else {
            pos = copySize;
        }
    }
}

/**
 * @brief RETURN opcode with direct memory access, i.e.,
 *        return(M[retOffset:retOffset+retSize])
 *        Simplified version with exclusively memory load operations into
 *        intermediate registers and then values are copied to the returned vector.
 *        TODO: Implement the indirect memory version (maybe not required)
 *        TODO: taking care of flagging this row as the last one? Special STOP flag?
 *
 * @param retOffset The starting index of the memory region to be returned.
 * @param retSize The number of elements to be returned.
 * @return The returned memory region as a std::vector.
 */

std::vector<FF> AvmMiniTraceBuilder::returnOP(uint32_t retOffset, uint32_t retSize)
{
    // We parallelize loading memory operations in chunk of 3, i.e., 1 per intermediate register.
    // The variable pos is an index pointing to the first storing operation (pertaining to intermediate
    // register Ia) relative to retOffset:
    // retOffset + pos:       Ia memory load operation
    // retOffset + pos + 1:   Ib memory load operation
    // retOffset + pos + 2:   Ic memory load operation

    uint32_t pos = 0;
    std::vector<FF> returnMem;

    while (pos < retSize) {
        FF ib(0);
        FF ic(0);
        uint32_t mem_op_b(0);
        uint32_t mem_op_c(0);
        uint32_t mem_idx_b(0);
        uint32_t mem_idx_c(0);
        auto clk = mainTrace.size();

        uint32_t mem_op_a(1);
        uint32_t mem_idx_a = retOffset + pos;
        FF ia = memory.at(mem_idx_a);

        // Loading from Ia
        returnMem.push_back(ia);
        loadInMemTrace(IntermRegister::ia, mem_idx_a, ia, AvmMemoryTag::ff);

        if (retSize - pos > 1) {
            mem_op_b = 1;
            mem_idx_b = retOffset + pos + 1;
            ib = memory.at(mem_idx_b);

            // Loading from Ib
            returnMem.push_back(ib);
            loadInMemTrace(IntermRegister::ib, mem_idx_b, ib, AvmMemoryTag::ff);
        }

        if (retSize - pos > 2) {
            mem_op_c = 1;
            mem_idx_c = retOffset + pos + 2;
            ic = memory.at(mem_idx_c);

            // Loading from Ic
            returnMem.push_back(ic);
            loadInMemTrace(IntermRegister::ic, mem_idx_c, ic, AvmMemoryTag::ff);
        }

        mainTrace.push_back(Row{
            .avmMini_clk = clk,
            .avmMini_pc = FF(pc),
            .avmMini_internal_return_ptr = FF(internal_return_ptr),
            .avmMini_sel_halt = FF(1),
            .avmMini_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::ff)),
            .avmMini_ia = ia,
            .avmMini_ib = ib,
            .avmMini_ic = ic,
            .avmMini_mem_op_a = FF(mem_op_a),
            .avmMini_mem_op_b = FF(mem_op_b),
            .avmMini_mem_op_c = FF(mem_op_c),
            .avmMini_mem_idx_a = FF(mem_idx_a),
            .avmMini_mem_idx_b = FF(mem_idx_b),
            .avmMini_mem_idx_c = FF(mem_idx_c),
        });

        if (retSize - pos > 2) { // Guard to prevent overflow if retSize is close to uint32_t maximum value.
            pos += 3;
        } else {
            pos = retSize;
        }
    }
    return returnMem;
}

/**
 * @brief HALT opcode
 *        This opcode effectively stops program execution, and is used in the relation that
 *        ensures the program counter increments on each opcode.
 *        i.e.ythe program counter should freeze and the halt flag is set to 1.
 */
void AvmMiniTraceBuilder::halt()
{
    auto clk = mainTrace.size();

    mainTrace.push_back(Row{
        .avmMini_clk = clk,
        .avmMini_pc = FF(pc),
        .avmMini_internal_return_ptr = FF(internal_return_ptr),
        .avmMini_sel_halt = FF(1),
    });
}

/**
 * @brief JUMP OPCODE
 *        Jumps to a new `jmpDest`
 *        This function must:
 *          - Set the next program counter to the provided `jmpDest`.
 *
 * @param jmpDest - The destination to jump to
 */
void AvmMiniTraceBuilder::jump(uint32_t jmpDest)
{
    auto clk = mainTrace.size();

    mainTrace.push_back(Row{
        .avmMini_clk = clk,
        .avmMini_pc = FF(pc),
        .avmMini_internal_return_ptr = FF(internal_return_ptr),
        .avmMini_sel_jump = FF(1),
        .avmMini_ia = FF(jmpDest),
    });

    // Adjust parameters for the next row
    pc = jmpDest;
}

/**
 * @brief INTERNAL_CALL OPCODE
 *        This opcode effectively jumps to a new `jmpDest` and stores the return program counter
 *        (current program counter + 1) onto a call stack.
 *        This function must:
 *          - Set the next program counter to the provided `jmpDest`.
 *          - Store the current `pc` + 1 onto the call stack (emulated in memory)
 *          - Increment the return stack pointer (a pointer to where the call stack is in memory)
 *
 *        Note: We use intermediate register to perform memory storage operations.
 *
 * @param jmpDest - The destination to jump to
 */
void AvmMiniTraceBuilder::internal_call(uint32_t jmpDest)
{
    auto clk = mainTrace.size();

    // We store the next instruction as the return location
    uint32_t stored_pc = pc + 1;
    internal_call_stack.push(stored_pc);

    // Add the return location to the memory trace
    storeInMemTrace(IntermRegister::ib, internal_return_ptr, FF(stored_pc), AvmMemoryTag::ff);
    memory.at(internal_return_ptr) = stored_pc;

    mainTrace.push_back(Row{
        .avmMini_clk = clk,
        .avmMini_pc = FF(pc),
        .avmMini_internal_return_ptr = FF(internal_return_ptr),
        .avmMini_sel_internal_call = FF(1),
        .avmMini_ia = FF(jmpDest),
        .avmMini_ib = stored_pc,
        .avmMini_mem_op_b = FF(1),
        .avmMini_rwb = FF(1),
        .avmMini_mem_idx_b = FF(internal_return_ptr),
    });

    // Adjust parameters for the next row
    pc = jmpDest;
    internal_return_ptr++;
}

/**
 * @brief INTERNAL_RETURN OPCODE
 *        The opcode returns from an internal call.
 *        This function must:
 *          - Read the return location from the internal_return_ptr
 *          - Set the next program counter to the return location
 *          - Decrement the return stack pointer
 *
 *  TODO(https://github.com/AztecProtocol/aztec-packages/issues/3740): This function MUST come after a call instruction.
 */
void AvmMiniTraceBuilder::internal_return()
{
    auto clk = mainTrace.size();

    // Internal return pointer is decremented
    FF a = memory.at(internal_return_ptr - 1);

    // We want to load the value pointed by the internal pointer
    loadInMemTrace(IntermRegister::ia, internal_return_ptr - 1, FF(a), AvmMemoryTag::ff);

    mainTrace.push_back(Row{
        .avmMini_clk = clk,
        .avmMini_pc = pc,
        .avmMini_internal_return_ptr = FF(internal_return_ptr),
        .avmMini_sel_internal_return = FF(1),
        .avmMini_ia = a,
        .avmMini_mem_op_a = FF(1),
        .avmMini_rwa = FF(0),
        .avmMini_mem_idx_a = FF(internal_return_ptr - 1),
    });

    // We want the next row to be the one pointed by jmpDest
    // The next pc should be from the top of the internal call stack + 1
    pc = internal_call_stack.top();
    internal_call_stack.pop();
    internal_return_ptr--;
}

/**
 * @brief Helper to initialize ffMemory. (Testing purpose mostly.)
 *
 */
void AvmMiniTraceBuilder::setFFMem(size_t idx, FF el, AvmMemoryTag tag)
{
    memory.at(idx) = el;
    memoryTag.at(idx) = tag;
};

/**
 * @brief Finalisation of the memory trace and incorporating it to the main trace.
 *        In particular, sorting the memory trace, setting .m_lastAccess and
 *        adding shifted values (first row). The main trace is moved at the end of
 *        this call.
 *
 * @return The main trace
 */
std::vector<Row> AvmMiniTraceBuilder::finalize()
{
    size_t memTraceSize = memTrace.size();
    size_t mainTraceSize = mainTrace.size();

    // TODO: We will have to handle this through error handling and not an assertion
    // Smaller than N because we have to add an extra initial row to support shifted
    // elements
    assert(memTraceSize < N);
    assert(mainTraceSize < N);

    // Sort memTrace
    std::sort(memTrace.begin(), memTrace.end(), compareMemEntries);

    // Fill the rest with zeros.
    size_t zeroRowsNum = N - mainTraceSize - 1;
    while (zeroRowsNum-- > 0) {
        mainTrace.push_back(Row{});
    }

    mainTrace.at(mainTraceSize - 1).avmMini_last = FF(1);

    for (size_t i = 0; i < memTraceSize; i++) {
        auto const& src = memTrace.at(i);
        auto& dest = mainTrace.at(i);

        dest.memTrace_m_clk = FF(src.m_clk);
        dest.memTrace_m_sub_clk = FF(src.m_sub_clk);
        dest.memTrace_m_addr = FF(src.m_addr);
        dest.memTrace_m_val = src.m_val;
        dest.memTrace_m_rw = FF(static_cast<uint32_t>(src.m_rw));
        dest.memTrace_m_in_tag = FF(static_cast<uint32_t>(src.m_in_tag));
        dest.memTrace_m_tag = FF(static_cast<uint32_t>(src.m_tag));
        dest.memTrace_m_tag_err = FF(static_cast<uint32_t>(src.m_tag_err));
        dest.memTrace_m_one_min_inv = src.m_one_min_inv;

        if (i + 1 < memTraceSize) {
            auto const& next = memTrace.at(i + 1);
            dest.memTrace_m_lastAccess = FF(static_cast<uint32_t>(src.m_addr != next.m_addr));
        } else {
            dest.memTrace_m_lastAccess = FF(1);
            dest.memTrace_m_last = FF(1);
        }
    }

    // Adding extra row for the shifted values at the top of the execution trace.
    Row first_row = Row{ .avmMini_first = FF(1), .memTrace_m_lastAccess = FF(1) };
    mainTrace.insert(mainTrace.begin(), first_row);

    return std::move(mainTrace);
}

} // namespace proof_system