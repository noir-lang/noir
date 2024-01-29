#pragma once

#include <stack>

#include "AvmMini_alu_trace.hpp"
#include "AvmMini_common.hpp"
#include "AvmMini_instructions.hpp"
#include "AvmMini_mem_trace.hpp"
#include "barretenberg/common/throw_or_abort.hpp"

#include "barretenberg/relations/generated/AvmMini/avm_mini.hpp"

namespace avm_trace {

// This is the internal context that we keep along the lifecycle of bytecode execution
// to iteratively build the whole trace. This is effectively performing witness generation.
// At the end of circuit building, mainTrace can be moved to AvmMiniCircuitBuilder by calling
// AvmMiniCircuitBuilder::set_trace(rows).
class AvmMiniTraceBuilder {

  public:
    static const size_t CALLSTACK_OFFSET = 896; // TODO(md): Temporary reserved area 896 - 1024

    AvmMiniTraceBuilder();

    std::vector<Row> finalize();
    void reset();

    uint32_t getPc() const { return pc; }

    // Addition with direct memory access.
    void add(uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag);

    // Subtraction with direct memory access.
    void sub(uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag);

    // Multiplication with direct memory access.
    void mul(uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag);

    // Division with direct memory access.
    void div(uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag);

    // Set a constant from bytecode with direct memory access.
    void set(uint128_t val, uint32_t dst_offset, AvmMemoryTag in_tag);

    // Jump to a given program counter.
    void jump(uint32_t jmp_dest);

    // Jump to a given program counter; storing the return location on a call stack.
    // TODO(md): this program counter MUST be an operand to the OPCODE.
    void internal_call(uint32_t jmp_dest);

    // Return from a jump.
    void internal_return();

    // Halt -> stop program execution.
    void halt();

    // CALLDATACOPY opcode with direct memory access, i.e.,
    // M[dst_offset:dst_offset+copy_size] = calldata[cd_offset:cd_offset+copy_size]
    void calldata_copy(uint32_t cd_offset,
                       uint32_t copy_size,
                       uint32_t dst_offset,
                       std::vector<FF> const& call_data_mem);

    // RETURN opcode with direct memory access, i.e.,
    // return(M[ret_offset:ret_offset+ret_size])
    std::vector<FF> return_op(uint32_t ret_offset, uint32_t ret_size);

  private:
    std::vector<Row> main_trace;
    AvmMiniMemTraceBuilder mem_trace_builder;
    AvmMiniAluTraceBuilder alu_trace_builder;

    uint32_t pc = 0;
    uint32_t internal_return_ptr = CALLSTACK_OFFSET;
    std::stack<uint32_t> internal_call_stack = {};
};
} // namespace avm_trace
