#include <array>
#include <cassert>
#include <cstddef>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <string>
#include <sys/types.h>
#include <vector>

#include "AvmMini_trace.hpp"

namespace avm_trace {

/**
 * @brief Constructor of a trace builder of AVM. Only serves to set the capacity of the
 *        underlying traces.
 */
AvmMiniTraceBuilder::AvmMiniTraceBuilder()
{
    main_trace.reserve(AVM_TRACE_SIZE);
}

/**
 * @brief Resetting the internal state so that a new trace can be rebuilt using the same object.
 *
 */
void AvmMiniTraceBuilder::reset()
{
    main_trace.clear();
    mem_trace_builder.reset();
}

/** TODO: Implement for non finite field types
 * @brief Addition with direct memory access.
 *
 * @param a_offset An index in memory pointing to the first operand of the addition.
 * @param b_offset An index in memory pointing to the second operand of the addition.
 * @param dst_offset An index in memory pointing to the output of the addition.
 * @param in_tag The instruction memory tag of the operands.
 */
void AvmMiniTraceBuilder::add(uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size());

    // Reading from memory and loading into ia resp. ib.
    auto read_a = mem_trace_builder.read_and_load_from_memory(clk, IntermRegister::ia, a_offset, in_tag);
    auto read_b = mem_trace_builder.read_and_load_from_memory(clk, IntermRegister::ib, b_offset, in_tag);
    bool tag_match = read_a.tag_match && read_b.tag_match;

    // a + b = c
    FF a = read_a.val;
    FF b = read_b.val;
    FF c = a + b;

    // Write into memory value c from intermediate register ic.
    mem_trace_builder.write_into_memory(clk, IntermRegister::ic, dst_offset, c, in_tag);

    main_trace.push_back(Row{
        .avmMini_clk = clk,
        .avmMini_pc = FF(pc++),
        .avmMini_internal_return_ptr = FF(internal_return_ptr),
        .avmMini_sel_op_add = FF(1),
        .avmMini_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avmMini_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .avmMini_ia = tag_match ? a : FF(0),
        .avmMini_ib = tag_match ? b : FF(0),
        .avmMini_ic = tag_match ? c : FF(0),
        .avmMini_mem_op_a = FF(1),
        .avmMini_mem_op_b = FF(1),
        .avmMini_mem_op_c = FF(1),
        .avmMini_rwc = FF(1),
        .avmMini_mem_idx_a = FF(a_offset),
        .avmMini_mem_idx_b = FF(b_offset),
        .avmMini_mem_idx_c = FF(dst_offset),
    });
};

/** TODO: Implement for non finite field types
 * @brief Subtraction with direct memory access.
 *
 * @param a_offset An index in memory pointing to the first operand of the subtraction.
 * @param b_offset An index in memory pointing to the second operand of the subtraction.
 * @param dst_offset An index in memory pointing to the output of the subtraction.
 * @param in_tag The instruction memory tag of the operands.
 */
void AvmMiniTraceBuilder::sub(uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size());

    // Reading from memory and loading into ia resp. ib.
    auto read_a = mem_trace_builder.read_and_load_from_memory(clk, IntermRegister::ia, a_offset, in_tag);
    auto read_b = mem_trace_builder.read_and_load_from_memory(clk, IntermRegister::ib, b_offset, in_tag);
    bool tag_match = read_a.tag_match && read_b.tag_match;

    // a - b = c
    FF a = read_a.val;
    FF b = read_b.val;
    FF c = a - b;

    // Write into memory value c from intermediate register ic.
    mem_trace_builder.write_into_memory(clk, IntermRegister::ic, dst_offset, c, in_tag);

    main_trace.push_back(Row{
        .avmMini_clk = clk,
        .avmMini_pc = FF(pc++),
        .avmMini_internal_return_ptr = FF(internal_return_ptr),
        .avmMini_sel_op_sub = FF(1),
        .avmMini_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avmMini_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .avmMini_ia = tag_match ? a : FF(0),
        .avmMini_ib = tag_match ? b : FF(0),
        .avmMini_ic = tag_match ? c : FF(0),
        .avmMini_mem_op_a = FF(1),
        .avmMini_mem_op_b = FF(1),
        .avmMini_mem_op_c = FF(1),
        .avmMini_rwc = FF(1),
        .avmMini_mem_idx_a = FF(a_offset),
        .avmMini_mem_idx_b = FF(b_offset),
        .avmMini_mem_idx_c = FF(dst_offset),
    });
};

/** TODO: Implement for non finite field types
 * @brief Multiplication with direct memory access.
 *
 * @param a_offset An index in memory pointing to the first operand of the multiplication.
 * @param b_offset An index in memory pointing to the second operand of the multiplication.
 * @param dst_offset An index in memory pointing to the output of the multiplication.
 * @param in_tag The instruction memory tag of the operands.
 */
void AvmMiniTraceBuilder::mul(uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size());

    // Reading from memory and loading into ia resp. ib.
    auto read_a = mem_trace_builder.read_and_load_from_memory(clk, IntermRegister::ia, a_offset, in_tag);
    auto read_b = mem_trace_builder.read_and_load_from_memory(clk, IntermRegister::ib, b_offset, in_tag);
    bool tag_match = read_a.tag_match && read_b.tag_match;

    // a * b = c
    FF a = read_a.val;
    FF b = read_b.val;
    FF c = a * b;

    // Write into memory value c from intermediate register ic.
    mem_trace_builder.write_into_memory(clk, IntermRegister::ic, dst_offset, c, in_tag);

    main_trace.push_back(Row{
        .avmMini_clk = clk,
        .avmMini_pc = FF(pc++),
        .avmMini_internal_return_ptr = FF(internal_return_ptr),
        .avmMini_sel_op_mul = FF(1),
        .avmMini_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avmMini_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .avmMini_ia = tag_match ? a : FF(0),
        .avmMini_ib = tag_match ? b : FF(0),
        .avmMini_ic = tag_match ? c : FF(0),
        .avmMini_mem_op_a = FF(1),
        .avmMini_mem_op_b = FF(1),
        .avmMini_mem_op_c = FF(1),
        .avmMini_rwc = FF(1),
        .avmMini_mem_idx_a = FF(a_offset),
        .avmMini_mem_idx_b = FF(b_offset),
        .avmMini_mem_idx_c = FF(dst_offset),
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
void AvmMiniTraceBuilder::div(uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size());

    // Reading from memory and loading into ia resp. ib.
    auto read_a = mem_trace_builder.read_and_load_from_memory(clk, IntermRegister::ia, a_offset, in_tag);
    auto read_b = mem_trace_builder.read_and_load_from_memory(clk, IntermRegister::ib, b_offset, in_tag);
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
    mem_trace_builder.write_into_memory(clk, IntermRegister::ic, dst_offset, c, in_tag);

    main_trace.push_back(Row{
        .avmMini_clk = clk,
        .avmMini_pc = FF(pc++),
        .avmMini_internal_return_ptr = FF(internal_return_ptr),
        .avmMini_sel_op_div = FF(1),
        .avmMini_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avmMini_op_err = tag_match ? error : FF(1),
        .avmMini_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .avmMini_inv = tag_match ? inv : FF(1),
        .avmMini_ia = tag_match ? a : FF(0),
        .avmMini_ib = tag_match ? b : FF(0),
        .avmMini_ic = tag_match ? c : FF(0),
        .avmMini_mem_op_a = FF(1),
        .avmMini_mem_op_b = FF(1),
        .avmMini_mem_op_c = FF(1),
        .avmMini_rwc = FF(1),
        .avmMini_mem_idx_a = FF(a_offset),
        .avmMini_mem_idx_b = FF(b_offset),
        .avmMini_mem_idx_c = FF(dst_offset),
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
 *
 * @param cd_offset The starting index of the region in calldata to be copied.
 * @param copy_size The number of finite field elements to be copied into memory.
 * @param dst_offset The starting index of memory where calldata will be copied to.
 * @param call_data_mem The vector containing calldata.
 */
void AvmMiniTraceBuilder::call_data_copy(uint32_t cd_offset,
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
        mem_trace_builder.write_into_memory(clk, IntermRegister::ia, mem_idx_a, ia, AvmMemoryTag::ff);

        if (copy_size - pos > 1) {
            ib = call_data_mem.at(cd_offset + pos + 1);
            mem_op_b = 1;
            mem_idx_b = dst_offset + pos + 1;
            rwb = 1;

            // Storing from Ib
            mem_trace_builder.write_into_memory(clk, IntermRegister::ib, mem_idx_b, ib, AvmMemoryTag::ff);
        }

        if (copy_size - pos > 2) {
            ic = call_data_mem.at(cd_offset + pos + 2);
            mem_op_c = 1;
            mem_idx_c = dst_offset + pos + 2;
            rwc = 1;

            // Storing from Ic
            mem_trace_builder.write_into_memory(clk, IntermRegister::ic, mem_idx_c, ic, AvmMemoryTag::ff);
        }

        main_trace.push_back(Row{
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
 *
 * @param ret_offset The starting index of the memory region to be returned.
 * @param ret_size The number of elements to be returned.
 * @return The returned memory region as a std::vector.
 */

std::vector<FF> AvmMiniTraceBuilder::return_op(uint32_t ret_offset, uint32_t ret_size)
{
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
        auto read_a = mem_trace_builder.read_and_load_from_memory(clk, IntermRegister::ia, mem_idx_a, AvmMemoryTag::ff);
        FF ia = read_a.val;
        returnMem.push_back(ia);

        if (ret_size - pos > 1) {
            mem_op_b = 1;
            mem_idx_b = ret_offset + pos + 1;

            // Reading and loading to Ib
            auto read_b =
                mem_trace_builder.read_and_load_from_memory(clk, IntermRegister::ib, mem_idx_b, AvmMemoryTag::ff);
            FF ib = read_b.val;
            returnMem.push_back(ib);
        }

        if (ret_size - pos > 2) {
            mem_op_c = 1;
            mem_idx_c = ret_offset + pos + 2;

            // Reading and loading to Ic
            auto read_c =
                mem_trace_builder.read_and_load_from_memory(clk, IntermRegister::ic, mem_idx_c, AvmMemoryTag::ff);
            FF ic = read_c.val;
            returnMem.push_back(ic);
        }

        main_trace.push_back(Row{
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

        if (ret_size - pos > 2) { // Guard to prevent overflow if ret_size is close to uint32_t maximum value.
            pos += 3;
        } else {
            pos = ret_size;
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
    auto clk = main_trace.size();

    main_trace.push_back(Row{
        .avmMini_clk = clk,
        .avmMini_pc = FF(pc),
        .avmMini_internal_return_ptr = FF(internal_return_ptr),
        .avmMini_sel_halt = FF(1),
    });
}

/**
 * @brief JUMP OPCODE
 *        Jumps to a new `jmp_dest`
 *        This function must:
 *          - Set the next program counter to the provided `jmp_dest`.
 *
 * @param jmp_dest - The destination to jump to
 */
void AvmMiniTraceBuilder::jump(uint32_t jmp_dest)
{
    auto clk = main_trace.size();

    main_trace.push_back(Row{
        .avmMini_clk = clk,
        .avmMini_pc = FF(pc),
        .avmMini_internal_return_ptr = FF(internal_return_ptr),
        .avmMini_sel_jump = FF(1),
        .avmMini_ia = FF(jmp_dest),
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
void AvmMiniTraceBuilder::internal_call(uint32_t jmp_dest)
{
    auto clk = static_cast<uint32_t>(main_trace.size());

    // We store the next instruction as the return location
    uint32_t stored_pc = pc + 1;
    internal_call_stack.push(stored_pc);

    // Add the return location to the memory trace
    mem_trace_builder.write_into_memory(clk, IntermRegister::ib, internal_return_ptr, FF(stored_pc), AvmMemoryTag::ff);

    main_trace.push_back(Row{
        .avmMini_clk = clk,
        .avmMini_pc = FF(pc),
        .avmMini_internal_return_ptr = FF(internal_return_ptr),
        .avmMini_sel_internal_call = FF(1),
        .avmMini_ia = FF(jmp_dest),
        .avmMini_ib = stored_pc,
        .avmMini_mem_op_b = FF(1),
        .avmMini_rwb = FF(1),
        .avmMini_mem_idx_b = FF(internal_return_ptr),
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
void AvmMiniTraceBuilder::internal_return()
{
    auto clk = static_cast<uint32_t>(main_trace.size());

    // Internal return pointer is decremented
    // We want to load the value pointed by the internal pointer
    auto read_a =
        mem_trace_builder.read_and_load_from_memory(clk, IntermRegister::ia, internal_return_ptr - 1, AvmMemoryTag::ff);

    main_trace.push_back(Row{
        .avmMini_clk = clk,
        .avmMini_pc = pc,
        .avmMini_internal_return_ptr = FF(internal_return_ptr),
        .avmMini_sel_internal_return = FF(1),
        .avmMini_ia = read_a.val,
        .avmMini_mem_op_a = FF(1),
        .avmMini_rwa = FF(0),
        .avmMini_mem_idx_a = FF(internal_return_ptr - 1),
    });

    // We want the next row to be the one pointed by jmp_dest
    // The next pc should be from the top of the internal call stack + 1
    pc = internal_call_stack.top();
    internal_call_stack.pop();
    internal_return_ptr--;
}

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
    auto mem_trace = mem_trace_builder.finalize();
    size_t mem_trace_size = mem_trace.size();
    size_t main_trace_size = main_trace.size();

    // TODO: We will have to handle this through error handling and not an assertion
    // Smaller than N because we have to add an extra initial row to support shifted
    // elements
    assert(mem_trace_size < AVM_TRACE_SIZE);
    assert(main_trace_size < AVM_TRACE_SIZE);

    // Fill the rest with zeros.
    size_t zero_rows_num = AVM_TRACE_SIZE - main_trace_size - 1;
    while (zero_rows_num-- > 0) {
        main_trace.push_back(Row{});
    }

    main_trace.at(main_trace_size - 1).avmMini_last = FF(1);

    for (size_t i = 0; i < mem_trace_size; i++) {
        auto const& src = mem_trace.at(i);
        auto& dest = main_trace.at(i);

        dest.memTrace_m_clk = FF(src.m_clk);
        dest.memTrace_m_sub_clk = FF(src.m_sub_clk);
        dest.memTrace_m_addr = FF(src.m_addr);
        dest.memTrace_m_val = src.m_val;
        dest.memTrace_m_rw = FF(static_cast<uint32_t>(src.m_rw));
        dest.memTrace_m_in_tag = FF(static_cast<uint32_t>(src.m_in_tag));
        dest.memTrace_m_tag = FF(static_cast<uint32_t>(src.m_tag));
        dest.memTrace_m_tag_err = FF(static_cast<uint32_t>(src.m_tag_err));
        dest.memTrace_m_one_min_inv = src.m_one_min_inv;

        if (i + 1 < mem_trace_size) {
            auto const& next = mem_trace.at(i + 1);
            dest.memTrace_m_lastAccess = FF(static_cast<uint32_t>(src.m_addr != next.m_addr));
        } else {
            dest.memTrace_m_lastAccess = FF(1);
            dest.memTrace_m_last = FF(1);
        }
    }

    // Adding extra row for the shifted values at the top of the execution trace.
    Row first_row = Row{ .avmMini_first = FF(1), .memTrace_m_lastAccess = FF(1) };
    main_trace.insert(main_trace.begin(), first_row);

    auto trace = std::move(main_trace);
    reset();

    return trace;
}

} // namespace avm_trace