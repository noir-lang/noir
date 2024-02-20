#include <array>
#include <cassert>
#include <cstddef>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <string>
#include <sys/types.h>
#include <vector>

#include "avm_trace.hpp"

namespace bb::avm_trace {

/**
 * @brief Constructor of a trace builder of AVM. Only serves to set the capacity of the
 *        underlying traces.
 */
AvmTraceBuilder::AvmTraceBuilder()
{
    main_trace.reserve(AVM_TRACE_SIZE);
}

/**
 * @brief Resetting the internal state so that a new trace can be rebuilt using the same object.
 *
 */
void AvmTraceBuilder::reset()
{
    main_trace.clear();
    mem_trace_builder.reset();
    alu_trace_builder.reset();
}

/**
 * @brief Addition with direct memory access.
 *
 * @param a_offset An index in memory pointing to the first operand of the addition.
 * @param b_offset An index in memory pointing to the second operand of the addition.
 * @param dst_offset An index in memory pointing to the output of the addition.
 * @param in_tag The instruction memory tag of the operands.
 */
void AvmTraceBuilder::op_add(uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size());

    // Reading from memory and loading into ia resp. ib.
    auto read_a = mem_trace_builder.read_and_load_from_memory(clk, IntermRegister::IA, a_offset, in_tag);
    auto read_b = mem_trace_builder.read_and_load_from_memory(clk, IntermRegister::IB, b_offset, in_tag);
    bool tag_match = read_a.tag_match && read_b.tag_match;

    // a + b = c
    FF a = tag_match ? read_a.val : FF(0);
    FF b = tag_match ? read_b.val : FF(0);
    FF c = alu_trace_builder.op_add(a, b, in_tag, clk);

    // Write into memory value c from intermediate register ic.
    mem_trace_builder.write_into_memory(clk, IntermRegister::IC, dst_offset, c, in_tag);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_pc = FF(pc++),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_sel_op_add = FF(1),
        .avm_main_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .avm_main_ia = a,
        .avm_main_ib = b,
        .avm_main_ic = c,
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_mem_op_c = FF(1),
        .avm_main_rwc = FF(1),
        .avm_main_mem_idx_a = FF(a_offset),
        .avm_main_mem_idx_b = FF(b_offset),
        .avm_main_mem_idx_c = FF(dst_offset),
    });
};

/**
 * @brief Subtraction with direct memory access.
 *
 * @param a_offset An index in memory pointing to the first operand of the subtraction.
 * @param b_offset An index in memory pointing to the second operand of the subtraction.
 * @param dst_offset An index in memory pointing to the output of the subtraction.
 * @param in_tag The instruction memory tag of the operands.
 */
void AvmTraceBuilder::op_sub(uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size());

    // Reading from memory and loading into ia resp. ib.
    auto read_a = mem_trace_builder.read_and_load_from_memory(clk, IntermRegister::IA, a_offset, in_tag);
    auto read_b = mem_trace_builder.read_and_load_from_memory(clk, IntermRegister::IB, b_offset, in_tag);
    bool tag_match = read_a.tag_match && read_b.tag_match;

    // a - b = c
    FF a = tag_match ? read_a.val : FF(0);
    FF b = tag_match ? read_b.val : FF(0);
    FF c = alu_trace_builder.op_sub(a, b, in_tag, clk);

    // Write into memory value c from intermediate register ic.
    mem_trace_builder.write_into_memory(clk, IntermRegister::IC, dst_offset, c, in_tag);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_pc = FF(pc++),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_sel_op_sub = FF(1),
        .avm_main_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .avm_main_ia = a,
        .avm_main_ib = b,
        .avm_main_ic = c,
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_mem_op_c = FF(1),
        .avm_main_rwc = FF(1),
        .avm_main_mem_idx_a = FF(a_offset),
        .avm_main_mem_idx_b = FF(b_offset),
        .avm_main_mem_idx_c = FF(dst_offset),
    });
};

/**
 * @brief Multiplication with direct memory access.
 *
 * @param a_offset An index in memory pointing to the first operand of the multiplication.
 * @param b_offset An index in memory pointing to the second operand of the multiplication.
 * @param dst_offset An index in memory pointing to the output of the multiplication.
 * @param in_tag The instruction memory tag of the operands.
 */
void AvmTraceBuilder::op_mul(uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size());

    // Reading from memory and loading into ia resp. ib.
    auto read_a = mem_trace_builder.read_and_load_from_memory(clk, IntermRegister::IA, a_offset, in_tag);
    auto read_b = mem_trace_builder.read_and_load_from_memory(clk, IntermRegister::IB, b_offset, in_tag);
    bool tag_match = read_a.tag_match && read_b.tag_match;

    // a * b = c
    FF a = tag_match ? read_a.val : FF(0);
    FF b = tag_match ? read_b.val : FF(0);
    FF c = alu_trace_builder.op_mul(a, b, in_tag, clk);

    // Write into memory value c from intermediate register ic.
    mem_trace_builder.write_into_memory(clk, IntermRegister::IC, dst_offset, c, in_tag);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_pc = FF(pc++),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_sel_op_mul = FF(1),
        .avm_main_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .avm_main_ia = a,
        .avm_main_ib = b,
        .avm_main_ic = c,
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_mem_op_c = FF(1),
        .avm_main_rwc = FF(1),
        .avm_main_mem_idx_a = FF(a_offset),
        .avm_main_mem_idx_b = FF(b_offset),
        .avm_main_mem_idx_c = FF(dst_offset),
    });
}

/** TODO: Implement for non finite field types
 * @brief Division with direct memory access.
 *
 * @param a_offset An index in memory pointing to the first operand of the division.
 * @param b_offset An index in memory pointing to the second operand of the division.
 * @param dst_offset An index in memory pointing to the output of the division.
 * @param in_tag The instruction memory tag of the operands.
 */
void AvmTraceBuilder::op_div(uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size());

    // Reading from memory and loading into ia resp. ib.
    auto read_a = mem_trace_builder.read_and_load_from_memory(clk, IntermRegister::IA, a_offset, in_tag);
    auto read_b = mem_trace_builder.read_and_load_from_memory(clk, IntermRegister::IB, b_offset, in_tag);
    bool tag_match = read_a.tag_match && read_b.tag_match;

    // a * b^(-1) = c
    FF a = read_a.val;
    FF b = read_b.val;
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

    // Write into memory value c from intermediate register ic.
    mem_trace_builder.write_into_memory(clk, IntermRegister::IC, dst_offset, c, in_tag);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_pc = FF(pc++),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_sel_op_div = FF(1),
        .avm_main_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_op_err = tag_match ? error : FF(1),
        .avm_main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .avm_main_inv = tag_match ? inv : FF(1),
        .avm_main_ia = tag_match ? a : FF(0),
        .avm_main_ib = tag_match ? b : FF(0),
        .avm_main_ic = tag_match ? c : FF(0),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_mem_op_c = FF(1),
        .avm_main_rwc = FF(1),
        .avm_main_mem_idx_a = FF(a_offset),
        .avm_main_mem_idx_b = FF(b_offset),
        .avm_main_mem_idx_c = FF(dst_offset),
    });
}

/**
 * @brief Bitwise not with direct memory access.
 *
 * @param a_offset An index in memory pointing to the only operand of Not.
 * @param dst_offset An index in memory pointing to the output of Not.
 * @param in_tag The instruction memory tag of the operands.
 */
void AvmTraceBuilder::op_not(uint32_t a_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size());

    // Reading from memory and loading into ia.
    auto read_a = mem_trace_builder.read_and_load_from_memory(clk, IntermRegister::IA, a_offset, in_tag);

    // ~a = c
    FF a = read_a.tag_match ? read_a.val : FF(0);
    // TODO(4613): If tag_match == false, then the value of c
    // will not be zero which would not satisfy the constraint that
    // ic == 0 whenever tag_err == 1. This constraint might be removed
    // as part of #4613.
    FF c = alu_trace_builder.op_not(a, in_tag, clk);

    // Write into memory value c from intermediate register ic.
    mem_trace_builder.write_into_memory(clk, IntermRegister::IC, dst_offset, c, in_tag);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_pc = FF(pc++),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_sel_op_not = FF(1),
        .avm_main_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_tag_err = FF(static_cast<uint32_t>(!read_a.tag_match)),
        .avm_main_ia = a,
        .avm_main_ic = c,
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_c = FF(1),
        .avm_main_rwc = FF(1),
        .avm_main_mem_idx_a = FF(a_offset),
        .avm_main_mem_idx_c = FF(dst_offset),
    });
};

/**
 * @brief Equality with direct memory access.
 *
 * @param a_offset An index in memory pointing to the first operand of the equality.
 * @param b_offset An index in memory pointing to the second operand of the equality.
 * @param dst_offset An index in memory pointing to the output of the equality.
 * @param in_tag The instruction memory tag of the operands.
 */
void AvmTraceBuilder::op_eq(uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size());

    // Reading from memory and loading into ia resp. ib.
    auto read_a = mem_trace_builder.read_and_load_from_memory(clk, IntermRegister::IA, a_offset, in_tag);
    auto read_b = mem_trace_builder.read_and_load_from_memory(clk, IntermRegister::IB, b_offset, in_tag);
    bool tag_match = read_a.tag_match && read_b.tag_match;

    // c = a == b ? 1 : 0
    FF a = tag_match ? read_a.val : FF(0);
    FF b = tag_match ? read_b.val : FF(0);

    // TODO(4613): If tag_match == false, then the value of c
    // will not be zero which would not satisfy the constraint that
    // ic == 0 whenever tag_err == 1. This constraint might be removed
    // as part of #4613.
    FF c = alu_trace_builder.op_eq(a, b, in_tag, clk);

    // Write into memory value c from intermediate register ic.
    mem_trace_builder.write_into_memory(clk, IntermRegister::IC, dst_offset, c, in_tag);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_pc = FF(pc++),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_sel_op_eq = FF(1),
        .avm_main_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .avm_main_ia = a,
        .avm_main_ib = b,
        .avm_main_ic = c,
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_mem_op_c = FF(1),
        .avm_main_rwc = FF(1),
        .avm_main_mem_idx_a = FF(a_offset),
        .avm_main_mem_idx_b = FF(b_offset),
        .avm_main_mem_idx_c = FF(dst_offset),
    });
}

// TODO: Finish SET opcode implementation. This is a partial implementation
// facilitating testing of arithmetic operations over non finite field types.
// We add an entry in the memory trace and a simplified one in the main trace
// without operation selector.
// TODO: PIL relations for the SET opcode need to be implemented.
// No check is performed that val pertains to type defined by in_tag.
/**
 * @brief Set a constant from bytecode with direct memory access.
 *
 * @param val The constant to be written upcasted to u128
 * @param dst_offset Memory destination offset where val is written to
 * @param in_tag The instruction memory tag
 */
void AvmTraceBuilder::set(uint128_t val, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size());
    auto val_ff = FF{ uint256_t::from_uint128(val) };

    mem_trace_builder.write_into_memory(clk, IntermRegister::IC, dst_offset, val_ff, in_tag);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_pc = FF(pc++),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_ic = val_ff,
        .avm_main_mem_op_c = FF(1),
        .avm_main_rwc = FF(1),
        .avm_main_mem_idx_c = FF(dst_offset),
    });
}

/**
 * @brief CALLDATACOPY opcode with direct memory access, i.e.,
 *        M[dst_offset:dst_offset+copy_size] = calldata[cd_offset:cd_offset+copy_size]
 *        Simplified version with exclusively memory store operations and
 *        values from M_calldata passed by an array and loaded into
 *        intermediate registers.
 *        Assume that caller passes call_data_mem which is large enough so that
 *        no out-of-bound memory issues occur.
 *        TODO: Implement the indirect memory version (maybe not required)
 *        TODO: taking care of intermediate register values consistency and propagating their
 *        values to the next row when not overwritten.
 *        TODO: error handling if dst_offset + copy_size > 2^32 which would lead to
 *              out-of-bound memory write. Similarly, if cd_offset + copy_size is larger
 *              than call_data_mem.size()
 *
 * @param cd_offset The starting index of the region in calldata to be copied.
 * @param copy_size The number of finite field elements to be copied into memory.
 * @param dst_offset The starting index of memory where calldata will be copied to.
 * @param call_data_mem The vector containing calldata.
 */
void AvmTraceBuilder::calldata_copy(uint32_t cd_offset,
                                    uint32_t copy_size,
                                    uint32_t dst_offset,
                                    std::vector<FF> const& call_data_mem)
{
    // We parallelize storing memory operations in chunk of 3, i.e., 1 per intermediate register.
    // The variable pos is an index pointing to the first storing operation (pertaining to intermediate
    // register Ia) relative to cd_offset:
    // cd_offset + pos:       Ia memory store operation
    // cd_offset + pos + 1:   Ib memory store operation
    // cd_offset + pos + 2:   Ic memory store operation

    uint32_t pos = 0;

    while (pos < copy_size) {
        FF ib(0);
        FF ic(0);
        uint32_t mem_op_b(0);
        uint32_t mem_op_c(0);
        uint32_t mem_idx_b(0);
        uint32_t mem_idx_c(0);
        uint32_t rwb(0);
        uint32_t rwc(0);
        auto clk = static_cast<uint32_t>(main_trace.size());

        FF ia = call_data_mem.at(cd_offset + pos);
        uint32_t mem_op_a(1);
        uint32_t mem_idx_a = dst_offset + pos;
        uint32_t rwa = 1;

        // Storing from Ia
        mem_trace_builder.write_into_memory(clk, IntermRegister::IA, mem_idx_a, ia, AvmMemoryTag::FF);

        if (copy_size - pos > 1) {
            ib = call_data_mem.at(cd_offset + pos + 1);
            mem_op_b = 1;
            mem_idx_b = dst_offset + pos + 1;
            rwb = 1;

            // Storing from Ib
            mem_trace_builder.write_into_memory(clk, IntermRegister::IB, mem_idx_b, ib, AvmMemoryTag::FF);
        }

        if (copy_size - pos > 2) {
            ic = call_data_mem.at(cd_offset + pos + 2);
            mem_op_c = 1;
            mem_idx_c = dst_offset + pos + 2;
            rwc = 1;

            // Storing from Ic
            mem_trace_builder.write_into_memory(clk, IntermRegister::IC, mem_idx_c, ic, AvmMemoryTag::FF);
        }

        main_trace.push_back(Row{
            .avm_main_clk = clk,
            .avm_main_pc = FF(pc++),
            .avm_main_internal_return_ptr = FF(internal_return_ptr),
            .avm_main_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
            .avm_main_ia = ia,
            .avm_main_ib = ib,
            .avm_main_ic = ic,
            .avm_main_mem_op_a = FF(mem_op_a),
            .avm_main_mem_op_b = FF(mem_op_b),
            .avm_main_mem_op_c = FF(mem_op_c),
            .avm_main_rwa = FF(rwa),
            .avm_main_rwb = FF(rwb),
            .avm_main_rwc = FF(rwc),
            .avm_main_mem_idx_a = FF(mem_idx_a),
            .avm_main_mem_idx_b = FF(mem_idx_b),
            .avm_main_mem_idx_c = FF(mem_idx_c),
        });

        if (copy_size - pos > 2) { // Guard to prevent overflow if copy_size is close to uint32_t maximum value.
            pos += 3;
        } else {
            pos = copy_size;
        }
    }
}

/**
 * @brief RETURN opcode with direct memory access, i.e.,
 *        return(M[ret_offset:ret_offset+ret_size])
 *        Simplified version with exclusively memory load operations into
 *        intermediate registers and then values are copied to the returned vector.
 *        TODO: Implement the indirect memory version (maybe not required)
 *        TODO: taking care of flagging this row as the last one? Special STOP flag?
 *        TODO: error handling if ret_offset + ret_size > 2^32 which would lead to
 *              out-of-bound memory read.
 *
 * @param ret_offset The starting index of the memory region to be returned.
 * @param ret_size The number of elements to be returned.
 * @return The returned memory region as a std::vector.
 */
std::vector<FF> AvmTraceBuilder::return_op(uint32_t ret_offset, uint32_t ret_size)
{
    if (ret_size == 0) {
        halt();
        return {};
    }

    // We parallelize loading memory operations in chunk of 3, i.e., 1 per intermediate register.
    // The variable pos is an index pointing to the first storing operation (pertaining to intermediate
    // register Ia) relative to ret_offset:
    // ret_offset + pos:       Ia memory load operation
    // ret_offset + pos + 1:   Ib memory load operation
    // ret_offset + pos + 2:   Ic memory load operation

    uint32_t pos = 0;
    std::vector<FF> returnMem;

    while (pos < ret_size) {
        FF ib(0);
        FF ic(0);
        uint32_t mem_op_b(0);
        uint32_t mem_op_c(0);
        uint32_t mem_idx_b(0);
        uint32_t mem_idx_c(0);
        auto clk = static_cast<uint32_t>(main_trace.size());

        uint32_t mem_op_a(1);
        uint32_t mem_idx_a = ret_offset + pos;

        // Reading and loading to Ia
        auto read_a = mem_trace_builder.read_and_load_from_memory(clk, IntermRegister::IA, mem_idx_a, AvmMemoryTag::FF);
        bool tag_match = read_a.tag_match;

        FF ia = read_a.val;
        returnMem.push_back(ia);

        if (ret_size - pos > 1) {
            mem_op_b = 1;
            mem_idx_b = ret_offset + pos + 1;

            // Reading and loading to Ib
            auto read_b =
                mem_trace_builder.read_and_load_from_memory(clk, IntermRegister::IB, mem_idx_b, AvmMemoryTag::FF);
            tag_match = tag_match && read_b.tag_match;
            FF ib = read_b.val;
            returnMem.push_back(ib);
        }

        if (ret_size - pos > 2) {
            mem_op_c = 1;
            mem_idx_c = ret_offset + pos + 2;

            // Reading and loading to Ic
            auto read_c =
                mem_trace_builder.read_and_load_from_memory(clk, IntermRegister::IC, mem_idx_c, AvmMemoryTag::FF);
            tag_match = tag_match && read_c.tag_match;
            FF ic = read_c.val;
            returnMem.push_back(ic);
        }

        main_trace.push_back(Row{
            .avm_main_clk = clk,
            .avm_main_pc = FF(pc),
            .avm_main_internal_return_ptr = FF(internal_return_ptr),
            .avm_main_sel_halt = FF(1),
            .avm_main_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
            .avm_main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
            .avm_main_ia = tag_match ? ia : FF(0),
            .avm_main_ib = tag_match ? ib : FF(0),
            .avm_main_ic = tag_match ? ic : FF(0),
            .avm_main_mem_op_a = FF(mem_op_a),
            .avm_main_mem_op_b = FF(mem_op_b),
            .avm_main_mem_op_c = FF(mem_op_c),
            .avm_main_mem_idx_a = FF(mem_idx_a),
            .avm_main_mem_idx_b = FF(mem_idx_b),
            .avm_main_mem_idx_c = FF(mem_idx_c),
        });

        if (ret_size - pos > 2) { // Guard to prevent overflow if ret_size is close to uint32_t maximum value.
            pos += 3;
        } else {
            pos = ret_size;
        }
    }
    pc = UINT32_MAX; // This ensures that no subsequent opcode will be executed.
    return returnMem;
}

/**
 * @brief HALT opcode
 *        This opcode effectively stops program execution, and is used in the relation that
 *        ensures the program counter increments on each opcode.
 *        i.e. the program counter should freeze and the halt flag is set to 1.
 */
void AvmTraceBuilder::halt()
{
    auto clk = main_trace.size();

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_pc = FF(pc),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_sel_halt = FF(1),
    });

    pc = UINT32_MAX; // This ensures that no subsequent opcode will be executed.
}

/**
 * @brief JUMP OPCODE
 *        Jumps to a new `jmp_dest`
 *        This function must:
 *          - Set the next program counter to the provided `jmp_dest`.
 *
 * @param jmp_dest - The destination to jump to
 */
void AvmTraceBuilder::jump(uint32_t jmp_dest)
{
    auto clk = main_trace.size();

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_pc = FF(pc),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_sel_jump = FF(1),
        .avm_main_ia = FF(jmp_dest),
    });

    // Adjust parameters for the next row
    pc = jmp_dest;
}

/**
 * @brief INTERNAL_CALL OPCODE
 *        This opcode effectively jumps to a new `jmp_dest` and stores the return program counter
 *        (current program counter + 1) onto a call stack.
 *        This function must:
 *          - Set the next program counter to the provided `jmp_dest`.
 *          - Store the current `pc` + 1 onto the call stack (emulated in memory)
 *          - Increment the return stack pointer (a pointer to where the call stack is in memory)
 *
 *        Note: We use intermediate register to perform memory storage operations.
 *
 * @param jmp_dest - The destination to jump to
 */
void AvmTraceBuilder::internal_call(uint32_t jmp_dest)
{
    auto clk = static_cast<uint32_t>(main_trace.size());

    // We store the next instruction as the return location
    uint32_t stored_pc = pc + 1;
    internal_call_stack.push(stored_pc);

    // Add the return location to the memory trace
    mem_trace_builder.write_into_memory(clk, IntermRegister::IB, internal_return_ptr, FF(stored_pc), AvmMemoryTag::FF);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_pc = FF(pc),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_sel_internal_call = FF(1),
        .avm_main_ia = FF(jmp_dest),
        .avm_main_ib = stored_pc,
        .avm_main_mem_op_b = FF(1),
        .avm_main_rwb = FF(1),
        .avm_main_mem_idx_b = FF(internal_return_ptr),
    });

    // Adjust parameters for the next row
    pc = jmp_dest;
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
void AvmTraceBuilder::internal_return()
{
    auto clk = static_cast<uint32_t>(main_trace.size());

    // Internal return pointer is decremented
    // We want to load the value pointed by the internal pointer
    auto read_a =
        mem_trace_builder.read_and_load_from_memory(clk, IntermRegister::IA, internal_return_ptr - 1, AvmMemoryTag::FF);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_pc = pc,
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_sel_internal_return = FF(1),
        .avm_main_tag_err = FF(static_cast<uint32_t>(!read_a.tag_match)),
        .avm_main_ia = read_a.tag_match ? read_a.val : FF(0),
        .avm_main_mem_op_a = FF(1),
        .avm_main_rwa = FF(0),
        .avm_main_mem_idx_a = FF(internal_return_ptr - 1),
    });

    // We want the next row to be the one pointed by jmp_dest
    // The next pc should be from the top of the internal call stack + 1
    pc = internal_call_stack.top();
    internal_call_stack.pop();
    internal_return_ptr--;
}

// Finalise Lookup Counts
//
// For log derivative lookups, we require a column that contains the number of times each lookup is consumed
// As we build the trace, we keep track of the reads made in a mapping, so that they can be applied to the
// counts column here
//
// NOTE: its coupled to pil - this is not the final iteration
void AvmTraceBuilder::finalise_mem_trace_lookup_counts(std::map<uint32_t, uint32_t> const& tag_err_lookup_counts)
{
    for (auto const& [clk, count] : tag_err_lookup_counts) {
        main_trace.at(clk).equiv_tag_err_counts = count;
    }
}

/**
 * @brief Finalisation of the memory trace and incorporating it to the main trace.
 *        In particular, sorting the memory trace, setting .m_lastAccess and
 *        adding shifted values (first row). The main trace is moved at the end of
 *        this call.
 *
 * @return The main trace
 */
std::vector<Row> AvmTraceBuilder::finalize()
{
    auto mem_trace = mem_trace_builder.finalize();
    auto alu_trace = alu_trace_builder.finalize();
    size_t mem_trace_size = mem_trace.size();
    size_t main_trace_size = main_trace.size();
    size_t alu_trace_size = alu_trace.size();

    // Get tag_err counts from the mem_trace_builder
    this->finalise_mem_trace_lookup_counts(mem_trace_builder.m_tag_err_lookup_counts);

    // TODO: We will have to handle this through error handling and not an assertion
    // Smaller than N because we have to add an extra initial row to support shifted
    // elements
    assert(mem_trace_size < AVM_TRACE_SIZE);
    assert(main_trace_size < AVM_TRACE_SIZE);
    assert(alu_trace_size < AVM_TRACE_SIZE);

    // Fill the rest with zeros.
    size_t zero_rows_num = AVM_TRACE_SIZE - main_trace_size - 1;
    while (zero_rows_num-- > 0) {
        main_trace.push_back({});
    }

    main_trace.at(main_trace_size - 1).avm_main_last = FF(1);

    // Memory trace inclusion
    for (size_t i = 0; i < mem_trace_size; i++) {
        auto const& src = mem_trace.at(i);
        auto& dest = main_trace.at(i);

        dest.avm_mem_m_clk = FF(src.m_clk);
        dest.avm_mem_m_sub_clk = FF(src.m_sub_clk);
        dest.avm_mem_m_addr = FF(src.m_addr);
        dest.avm_mem_m_val = src.m_val;
        dest.avm_mem_m_rw = FF(static_cast<uint32_t>(src.m_rw));
        dest.avm_mem_m_in_tag = FF(static_cast<uint32_t>(src.m_in_tag));
        dest.avm_mem_m_tag = FF(static_cast<uint32_t>(src.m_tag));
        dest.avm_mem_m_tag_err = FF(static_cast<uint32_t>(src.m_tag_err));
        dest.avm_mem_m_one_min_inv = src.m_one_min_inv;

        if (i + 1 < mem_trace_size) {
            auto const& next = mem_trace.at(i + 1);
            dest.avm_mem_m_lastAccess = FF(static_cast<uint32_t>(src.m_addr != next.m_addr));
        } else {
            dest.avm_mem_m_lastAccess = FF(1);
            dest.avm_mem_m_last = FF(1);
        }
    }

    // Alu trace inclusion
    for (size_t i = 0; i < alu_trace_size; i++) {
        auto const& src = alu_trace.at(i);
        auto& dest = main_trace.at(i);

        dest.avm_alu_alu_clk = FF(static_cast<uint32_t>(src.alu_clk));

        dest.avm_alu_alu_op_add = FF(static_cast<uint32_t>(src.alu_op_add));
        dest.avm_alu_alu_op_sub = FF(static_cast<uint32_t>(src.alu_op_sub));
        dest.avm_alu_alu_op_mul = FF(static_cast<uint32_t>(src.alu_op_mul));
        dest.avm_alu_alu_op_not = FF(static_cast<uint32_t>(src.alu_op_not));
        dest.avm_alu_alu_op_eq = FF(static_cast<uint32_t>(src.alu_op_eq));

        dest.avm_alu_alu_ff_tag = FF(static_cast<uint32_t>(src.alu_ff_tag));
        dest.avm_alu_alu_u8_tag = FF(static_cast<uint32_t>(src.alu_u8_tag));
        dest.avm_alu_alu_u16_tag = FF(static_cast<uint32_t>(src.alu_u16_tag));
        dest.avm_alu_alu_u32_tag = FF(static_cast<uint32_t>(src.alu_u32_tag));
        dest.avm_alu_alu_u64_tag = FF(static_cast<uint32_t>(src.alu_u64_tag));
        dest.avm_alu_alu_u128_tag = FF(static_cast<uint32_t>(src.alu_u128_tag));

        dest.avm_alu_alu_ia = src.alu_ia;
        dest.avm_alu_alu_ib = src.alu_ib;
        dest.avm_alu_alu_ic = src.alu_ic;

        dest.avm_alu_alu_cf = FF(static_cast<uint32_t>(src.alu_cf));

        dest.avm_alu_alu_u8_r0 = FF(src.alu_u8_r0);
        dest.avm_alu_alu_u8_r1 = FF(src.alu_u8_r1);

        dest.avm_alu_alu_u16_r0 = FF(src.alu_u16_reg.at(0));
        dest.avm_alu_alu_u16_r1 = FF(src.alu_u16_reg.at(1));
        dest.avm_alu_alu_u16_r2 = FF(src.alu_u16_reg.at(2));
        dest.avm_alu_alu_u16_r3 = FF(src.alu_u16_reg.at(3));
        dest.avm_alu_alu_u16_r4 = FF(src.alu_u16_reg.at(4));
        dest.avm_alu_alu_u16_r5 = FF(src.alu_u16_reg.at(5));
        dest.avm_alu_alu_u16_r6 = FF(src.alu_u16_reg.at(6));
        dest.avm_alu_alu_u16_r7 = FF(src.alu_u16_reg.at(7));

        dest.avm_alu_alu_u64_r0 = FF(src.alu_u64_r0);
        dest.avm_alu_alu_op_eq_diff_inv = FF(src.alu_op_eq_diff_inv);
    }

    // Adding extra row for the shifted values at the top of the execution trace.
    Row first_row = Row{ .avm_main_first = FF(1), .avm_mem_m_lastAccess = FF(1) };
    main_trace.insert(main_trace.begin(), first_row);

    auto trace = std::move(main_trace);
    reset();

    return trace;
}

} // namespace bb::avm_trace
