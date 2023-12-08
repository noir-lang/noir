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
    ffMemory.fill(FF(0));
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
 * @param m_rw Boolean telling whether it is a load (false) or store operation (true).
 */
void AvmMiniTraceBuilder::insertInMemTrace(uint32_t m_clk, uint32_t m_sub_clk, uint32_t m_addr, FF m_val, bool m_rw)
{
    memTrace.emplace_back(MemoryTraceEntry{
        .m_clk = m_clk,
        .m_sub_clk = m_sub_clk,
        .m_addr = m_addr,
        .m_val = m_val,
        .m_rw = m_rw,
    });
}

// Memory operations need to be performed before the addition of the corresponding row in
// ainTrace, otherwise the m_clk value will be wrong.This applies to : loadAInMemTrace, loadBInMemTrace,
// loadCInMemTrace
//       storeAInMemTrace, storeBInMemTrace, storeCInMemTrace
/**
 * @brief Add a memory trace entry corresponding to a memory load into the intermediate
 *        register Ia.
 *
 * @param addr The memory address
 * @param val The value to be loaded
 */
void AvmMiniTraceBuilder::loadAInMemTrace(uint32_t addr, FF val)
{
    insertInMemTrace(static_cast<uint32_t>(mainTrace.size()), SUB_CLK_LOAD_A, addr, val, false);
}

/**
 * @brief Add a memory trace entry corresponding to a memory load into the intermediate
 *        register Ib.
 *
 * @param addr The memory address
 * @param val The value to be loaded
 */
void AvmMiniTraceBuilder::loadBInMemTrace(uint32_t addr, FF val)
{
    insertInMemTrace(static_cast<uint32_t>(mainTrace.size()), SUB_CLK_LOAD_B, addr, val, false);
}

/**
 * @brief Add a memory trace entry corresponding to a memory load into the intermediate
 *        register Ic.
 *
 * @param addr The memory address
 * @param val The value to be loaded
 */
void AvmMiniTraceBuilder::loadCInMemTrace(uint32_t addr, FF val)
{
    insertInMemTrace(static_cast<uint32_t>(mainTrace.size()), SUB_CLK_LOAD_C, addr, val, false);
}

/**
 * @brief Add a memory trace entry corresponding to a memory store from the intermediate
 *        register Ia.
 *
 * @param addr The memory address
 * @param val The value to be stored
 */
void AvmMiniTraceBuilder::storeAInMemTrace(uint32_t addr, FF val)
{
    insertInMemTrace(static_cast<uint32_t>(mainTrace.size()), SUB_CLK_STORE_A, addr, val, true);
}

/**
 * @brief Add a memory trace entry corresponding to a memory store from the intermediate
 *        register Ib.
 *
 * @param addr The memory address
 * @param val The value to be stored
 */
void AvmMiniTraceBuilder::storeBInMemTrace(uint32_t addr, FF val)
{
    insertInMemTrace(static_cast<uint32_t>(mainTrace.size()), SUB_CLK_STORE_B, addr, val, true);
}

/**
 * @brief Add a memory trace entry corresponding to a memory store from the intermediate
 *        register Ic.
 *
 * @param addr The memory address
 * @param val The value to be stored
 */
void AvmMiniTraceBuilder::storeCInMemTrace(uint32_t addr, FF val)
{
    insertInMemTrace(static_cast<uint32_t>(mainTrace.size()), SUB_CLK_STORE_C, addr, val, true);
}

/**
 * @brief Addition over finite field with direct memory access.
 *
 * @param aOffset An index in ffMemory pointing to the first operand of the addition.
 * @param bOffset An index in ffMemory pointing to the second operand of the addition.
 * @param dstOffset An index in ffMemory pointing to the output of the addition.
 */
void AvmMiniTraceBuilder::add(uint32_t aOffset, uint32_t bOffset, uint32_t dstOffset)
{
    // a + b = c
    FF a = ffMemory.at(aOffset);
    FF b = ffMemory.at(bOffset);
    FF c = a + b;
    ffMemory.at(dstOffset) = c;

    auto clk = mainTrace.size();

    // Loading into Ia
    loadAInMemTrace(aOffset, a);

    // Loading into Ib
    loadBInMemTrace(bOffset, b);

    // Storing from Ic
    storeCInMemTrace(dstOffset, c);

    mainTrace.push_back(Row{
        .avmMini_clk = clk,
        .avmMini_sel_op_add = FF(1),
        .avmMini_ia = a,
        .avmMini_ib = b,
        .avmMini_ic = c,
        .avmMini_mem_op_a = FF(1),
        .avmMini_mem_op_b = FF(1),
        .avmMini_mem_op_c = FF(1),
        .avmMini_rwc = FF(1),
        .avmMini_mem_idx_a = FF(aOffset),
        .avmMini_mem_idx_b = FF(bOffset),
        .avmMini_mem_idx_c = FF(dstOffset),
    });
};

/**
 * @brief Subtraction over finite field with direct memory access.
 *
 * @param aOffset An index in ffMemory pointing to the first operand of the subtraction.
 * @param bOffset An index in ffMemory pointing to the second operand of the subtraction.
 * @param dstOffset An index in ffMemory pointing to the output of the subtraction.
 */
void AvmMiniTraceBuilder::sub(uint32_t aOffset, uint32_t bOffset, uint32_t dstOffset)
{
    // a - b = c
    FF a = ffMemory.at(aOffset);
    FF b = ffMemory.at(bOffset);
    FF c = a - b;
    ffMemory.at(dstOffset) = c;

    auto clk = mainTrace.size();

    // Loading into Ia
    loadAInMemTrace(aOffset, a);

    // Loading into Ib
    loadBInMemTrace(bOffset, b);

    // Storing from Ic
    storeCInMemTrace(dstOffset, c);

    mainTrace.push_back(Row{
        .avmMini_clk = clk,
        .avmMini_sel_op_sub = FF(1),
        .avmMini_ia = a,
        .avmMini_ib = b,
        .avmMini_ic = c,
        .avmMini_mem_op_a = FF(1),
        .avmMini_mem_op_b = FF(1),
        .avmMini_mem_op_c = FF(1),
        .avmMini_rwc = FF(1),
        .avmMini_mem_idx_a = FF(aOffset),
        .avmMini_mem_idx_b = FF(bOffset),
        .avmMini_mem_idx_c = FF(dstOffset),
    });
};

/**
 * @brief Multiplication over finite field with direct memory access.
 *
 * @param aOffset An index in ffMemory pointing to the first operand of the multiplication.
 * @param bOffset An index in ffMemory pointing to the second operand of the multiplication.
 * @param dstOffset An index in ffMemory pointing to the output of the multiplication.
 */
void AvmMiniTraceBuilder::mul(uint32_t aOffset, uint32_t bOffset, uint32_t dstOffset)
{
    // a * b = c
    FF a = ffMemory.at(aOffset);
    FF b = ffMemory.at(bOffset);
    FF c = a * b;
    ffMemory.at(dstOffset) = c;

    auto clk = mainTrace.size();

    // Loading into Ia
    loadAInMemTrace(aOffset, a);

    // Loading into Ib
    loadBInMemTrace(bOffset, b);

    // Storing from Ic
    storeCInMemTrace(dstOffset, c);

    mainTrace.push_back(Row{
        .avmMini_clk = clk,
        .avmMini_sel_op_mul = FF(1),
        .avmMini_ia = a,
        .avmMini_ib = b,
        .avmMini_ic = c,
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
 * @brief Division over finite field with direct memory access.
 *
 * @param aOffset An index in ffMemory pointing to the first operand of the division.
 * @param bOffset An index in ffMemory pointing to the second operand of the division.
 * @param dstOffset An index in ffMemory pointing to the output of the division.
 */
void AvmMiniTraceBuilder::div(uint32_t aOffset, uint32_t bOffset, uint32_t dstOffset)
{
    // a * b^(-1) = c
    FF a = ffMemory.at(aOffset);
    FF b = ffMemory.at(bOffset);
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

    ffMemory.at(dstOffset) = c;

    auto clk = mainTrace.size();

    // Loading into Ia
    loadAInMemTrace(aOffset, a);

    // Loading into Ib
    loadBInMemTrace(bOffset, b);

    // Storing from Ic
    storeCInMemTrace(dstOffset, c);

    mainTrace.push_back(Row{
        .avmMini_clk = clk,
        .avmMini_sel_op_div = FF(1),
        .avmMini_op_err = error,
        .avmMini_inv = inv,
        .avmMini_ia = a,
        .avmMini_ib = b,
        .avmMini_ic = c,
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
        ffMemory.at(mem_idx_a) = ia;
        storeAInMemTrace(mem_idx_a, ia);

        if (copySize - pos > 1) {
            ib = callDataMem.at(cdOffset + pos + 1);
            mem_op_b = 1;
            mem_idx_b = dstOffset + pos + 1;
            rwb = 1;

            // Storing from Ib
            ffMemory.at(mem_idx_b) = ib;
            storeBInMemTrace(mem_idx_b, ib);
        }

        if (copySize - pos > 2) {
            ic = callDataMem.at(cdOffset + pos + 2);
            mem_op_c = 1;
            mem_idx_c = dstOffset + pos + 2;
            rwc = 1;

            // Storing from Ic
            ffMemory.at(mem_idx_c) = ic;
            storeCInMemTrace(mem_idx_c, ic);
        }

        mainTrace.push_back(Row{
            .avmMini_clk = clk,
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
        FF ia = ffMemory.at(mem_idx_a);

        // Loading from Ia
        returnMem.push_back(ia);
        loadAInMemTrace(mem_idx_a, ia);

        if (retSize - pos > 1) {
            mem_op_b = 1;
            mem_idx_b = retOffset + pos + 1;
            ib = ffMemory.at(mem_idx_b);

            // Loading from Ib
            returnMem.push_back(ib);
            loadBInMemTrace(mem_idx_b, ib);
        }

        if (retSize - pos > 2) {
            mem_op_c = 1;
            mem_idx_c = retOffset + pos + 2;
            ic = ffMemory.at(mem_idx_c);

            // Loading from Ic
            returnMem.push_back(ic);
            loadCInMemTrace(mem_idx_c, ic);
        }

        mainTrace.push_back(Row{
            .avmMini_clk = clk,
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
 * @brief Helper to initialize ffMemory. (Testing purpose mostly.)
 *
 */
void AvmMiniTraceBuilder::setFFMem(size_t idx, FF el)
{
    ffMemory.at(idx) = el;
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

    size_t lastIndex = (memTraceSize > mainTraceSize) ? memTraceSize - 1 : mainTraceSize - 1;
    mainTrace.at(lastIndex).avmMini_last = FF(1);

    for (size_t i = 0; i < memTraceSize; i++) {
        auto const& src = memTrace.at(i);
        auto& dest = mainTrace.at(i);

        dest.memTrace_m_clk = FF(src.m_clk);
        dest.memTrace_m_sub_clk = FF(src.m_sub_clk);
        dest.memTrace_m_addr = FF(src.m_addr);
        dest.memTrace_m_val = src.m_val;
        dest.memTrace_m_rw = FF(static_cast<uint32_t>(src.m_rw));

        if (i + 1 < memTraceSize) {
            auto const& next = memTrace.at(i + 1);
            dest.memTrace_m_lastAccess = FF(static_cast<uint32_t>(src.m_addr != next.m_addr));
        } else {
            dest.memTrace_m_lastAccess = FF(1);
        }
    }

    // Adding extra row for the shifted values at the top of the execution trace.
    Row first_row = Row{ .avmMini_first = 1 };
    mainTrace.insert(mainTrace.begin(), first_row);

    return std::move(mainTrace);
}

} // namespace proof_system