#include <algorithm>
#include <array>
#include <cassert>
#include <cstddef>
#include <cstdint>
#include <fstream>
#include <functional>
#include <iostream>
#include <limits>
#include <set>
#include <string>
#include <sys/types.h>
#include <unordered_map>
#include <vector>

#include "barretenberg/common/throw_or_abort.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/avm_trace/avm_helper.hpp"
#include "barretenberg/vm/avm_trace/avm_opcode.hpp"
#include "barretenberg/vm/avm_trace/avm_trace.hpp"

namespace bb::avm_trace {

/**
 * @brief Constructor of a trace builder of AVM. Only serves to set the capacity of the
 *        underlying traces and initialize gas values.
 */
AvmTraceBuilder::AvmTraceBuilder(VmPublicInputs public_inputs,
                                 ExecutionHints execution_hints,
                                 uint32_t side_effect_counter)
    // NOTE: we initialise the environment builder here as it requires public inputs
    : kernel_trace_builder(std::move(public_inputs))
    , side_effect_counter(side_effect_counter)
    , initial_side_effect_counter(side_effect_counter)
    , execution_hints(std::move(execution_hints))
{
    main_trace.reserve(AVM_TRACE_SIZE);

    // TODO: think about cast
    gas_trace_builder.set_initial_gas(static_cast<uint32_t>(std::get<KERNEL_INPUTS>(
                                          kernel_trace_builder.public_inputs)[L2_GAS_LEFT_CONTEXT_INPUTS_OFFSET]),
                                      static_cast<uint32_t>(std::get<KERNEL_INPUTS>(
                                          kernel_trace_builder.public_inputs)[DA_GAS_LEFT_CONTEXT_INPUTS_OFFSET]));
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
    bin_trace_builder.reset();
    kernel_trace_builder.reset();
    gas_trace_builder.reset();
    conversion_trace_builder.reset();
    sha256_trace_builder.reset();
    poseidon2_trace_builder.reset();
    keccak_trace_builder.reset();
    pedersen_trace_builder.reset();

    external_call_counter = 0;
}

AvmTraceBuilder::IndirectThreeResolution AvmTraceBuilder::resolve_ind_three(
    uint8_t space_id, uint32_t clk, uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t c_offset)
{
    bool indirect_flag_a = is_operand_indirect(indirect, 0);
    bool indirect_flag_b = is_operand_indirect(indirect, 1);
    bool indirect_flag_c = is_operand_indirect(indirect, 2);

    uint32_t direct_a_offset = a_offset;
    uint32_t direct_b_offset = b_offset;
    uint32_t direct_c_offset = c_offset;

    bool tag_match = true;

    if (indirect_flag_a) {
        auto read_ind_a =
            mem_trace_builder.indirect_read_and_load_from_memory(space_id, clk, IndirectRegister::IND_A, a_offset);
        direct_a_offset = uint32_t(read_ind_a.val);
        tag_match = tag_match && read_ind_a.tag_match;
    }

    if (indirect_flag_b) {
        auto read_ind_b =
            mem_trace_builder.indirect_read_and_load_from_memory(space_id, clk, IndirectRegister::IND_B, b_offset);
        direct_b_offset = uint32_t(read_ind_b.val);
        tag_match = tag_match && read_ind_b.tag_match;
    }

    if (indirect_flag_c) {
        auto read_ind_c =
            mem_trace_builder.indirect_read_and_load_from_memory(space_id, clk, IndirectRegister::IND_C, c_offset);
        direct_c_offset = uint32_t(read_ind_c.val);
        tag_match = tag_match && read_ind_c.tag_match;
    }

    return IndirectThreeResolution{
        .tag_match = tag_match,
        .direct_a_offset = direct_a_offset,
        .direct_b_offset = direct_b_offset,
        .direct_c_offset = direct_c_offset,
        .indirect_flag_a = indirect_flag_a,
        .indirect_flag_b = indirect_flag_b,
        .indirect_flag_c = indirect_flag_c,
    };
}

/**
 * @brief Addition with direct or indirect memory access.
 *
 * @param indirect A byte encoding information about indirect/direct memory access.
 * @param a_offset An index in memory pointing to the first operand of the addition.
 * @param b_offset An index in memory pointing to the second operand of the addition.
 * @param dst_offset An index in memory pointing to the output of the addition.
 * @param in_tag The instruction memory tag of the operands.
 */
void AvmTraceBuilder::op_add(
    uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    auto const res = resolve_ind_three(call_ptr, clk, indirect, a_offset, b_offset, dst_offset);
    bool tag_match = res.tag_match;

    // Reading from memory and loading into ia resp. ib.
    auto read_a = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, res.direct_a_offset, in_tag, in_tag);
    auto read_b = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, res.direct_b_offset, in_tag, in_tag);
    tag_match = read_a.tag_match && read_b.tag_match;

    // a + b = c
    FF a = read_a.val;
    FF b = read_b.val;

    // In case of a memory tag error, we do not perform the computation.
    // Therefore, we do not create any entry in ALU table and store the value 0 as
    // output (c) in memory.
    FF c = tag_match ? alu_trace_builder.op_add(a, b, in_tag, clk) : FF(0);

    // Write into memory value c from intermediate register ic.
    mem_trace_builder.write_into_memory(call_ptr, clk, IntermRegister::IC, res.direct_c_offset, c, in_tag, in_tag);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::ADD);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_alu_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_call_ptr = call_ptr,
        .avm_main_ia = a,
        .avm_main_ib = b,
        .avm_main_ic = c,
        .avm_main_ind_a = res.indirect_flag_a ? FF(a_offset) : FF(0),
        .avm_main_ind_b = res.indirect_flag_b ? FF(b_offset) : FF(0),
        .avm_main_ind_c = res.indirect_flag_c ? FF(dst_offset) : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(res.indirect_flag_a)),
        .avm_main_ind_op_b = FF(static_cast<uint32_t>(res.indirect_flag_b)),
        .avm_main_ind_op_c = FF(static_cast<uint32_t>(res.indirect_flag_c)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(res.direct_a_offset),
        .avm_main_mem_idx_b = FF(res.direct_b_offset),
        .avm_main_mem_idx_c = FF(res.direct_c_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_mem_op_c = FF(1),
        .avm_main_pc = FF(pc++),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_rwc = FF(1),
        .avm_main_sel_op_add = FF(1),
        .avm_main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(in_tag)),
    });
}

/**
 * @brief Subtraction with direct or indirect memory access.
 *
 * @param indirect A byte encoding information about indirect/direct memory access.
 * @param a_offset An index in memory pointing to the first operand of the subtraction.
 * @param b_offset An index in memory pointing to the second operand of the subtraction.
 * @param dst_offset An index in memory pointing to the output of the subtraction.
 * @param in_tag The instruction memory tag of the operands.
 */
void AvmTraceBuilder::op_sub(
    uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    auto const res = resolve_ind_three(call_ptr, clk, indirect, a_offset, b_offset, dst_offset);
    bool tag_match = res.tag_match;

    // Reading from memory and loading into ia resp. ib.
    auto read_a = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, res.direct_a_offset, in_tag, in_tag);
    auto read_b = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, res.direct_b_offset, in_tag, in_tag);
    tag_match = read_a.tag_match && read_b.tag_match;

    // a - b = c
    FF a = read_a.val;
    FF b = read_b.val;

    // In case of a memory tag error, we do not perform the computation.
    // Therefore, we do not create any entry in ALU table and store the value 0 as
    // output (c) in memory.
    FF c = tag_match ? alu_trace_builder.op_sub(a, b, in_tag, clk) : FF(0);

    // Write into memory value c from intermediate register ic.
    mem_trace_builder.write_into_memory(call_ptr, clk, IntermRegister::IC, res.direct_c_offset, c, in_tag, in_tag);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::SUB);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_alu_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_call_ptr = call_ptr,
        .avm_main_ia = a,
        .avm_main_ib = b,
        .avm_main_ic = c,
        .avm_main_ind_a = res.indirect_flag_a ? FF(a_offset) : FF(0),
        .avm_main_ind_b = res.indirect_flag_b ? FF(b_offset) : FF(0),
        .avm_main_ind_c = res.indirect_flag_c ? FF(dst_offset) : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(res.indirect_flag_a)),
        .avm_main_ind_op_b = FF(static_cast<uint32_t>(res.indirect_flag_b)),
        .avm_main_ind_op_c = FF(static_cast<uint32_t>(res.indirect_flag_c)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(res.direct_a_offset),
        .avm_main_mem_idx_b = FF(res.direct_b_offset),
        .avm_main_mem_idx_c = FF(res.direct_c_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_mem_op_c = FF(1),
        .avm_main_pc = FF(pc++),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_rwc = FF(1),
        .avm_main_sel_op_sub = FF(1),
        .avm_main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(in_tag)),
    });
}

/**
 * @brief Multiplication with direct or indirect memory access.
 *
 * @param indirect A byte encoding information about indirect/direct memory access.
 * @param a_offset An index in memory pointing to the first operand of the multiplication.
 * @param b_offset An index in memory pointing to the second operand of the multiplication.
 * @param dst_offset An index in memory pointing to the output of the multiplication.
 * @param in_tag The instruction memory tag of the operands.
 */
void AvmTraceBuilder::op_mul(
    uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    auto const res = resolve_ind_three(call_ptr, clk, indirect, a_offset, b_offset, dst_offset);
    bool tag_match = res.tag_match;

    // Reading from memory and loading into ia resp. ib.
    auto read_a = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, res.direct_a_offset, in_tag, in_tag);
    auto read_b = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, res.direct_b_offset, in_tag, in_tag);
    tag_match = read_a.tag_match && read_b.tag_match;

    // a * b = c
    FF a = read_a.val;
    FF b = read_b.val;

    // In case of a memory tag error, we do not perform the computation.
    // Therefore, we do not create any entry in ALU table and store the value 0 as
    // output (c) in memory.
    FF c = tag_match ? alu_trace_builder.op_mul(a, b, in_tag, clk) : FF(0);

    // Write into memory value c from intermediate register ic.
    mem_trace_builder.write_into_memory(call_ptr, clk, IntermRegister::IC, res.direct_c_offset, c, in_tag, in_tag);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::MUL);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_alu_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_call_ptr = call_ptr,
        .avm_main_ia = a,
        .avm_main_ib = b,
        .avm_main_ic = c,
        .avm_main_ind_a = res.indirect_flag_a ? FF(a_offset) : FF(0),
        .avm_main_ind_b = res.indirect_flag_b ? FF(b_offset) : FF(0),
        .avm_main_ind_c = res.indirect_flag_c ? FF(dst_offset) : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(res.indirect_flag_a)),
        .avm_main_ind_op_b = FF(static_cast<uint32_t>(res.indirect_flag_b)),
        .avm_main_ind_op_c = FF(static_cast<uint32_t>(res.indirect_flag_c)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(res.direct_a_offset),
        .avm_main_mem_idx_b = FF(res.direct_b_offset),
        .avm_main_mem_idx_c = FF(res.direct_c_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_mem_op_c = FF(1),
        .avm_main_pc = FF(pc++),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_rwc = FF(1),
        .avm_main_sel_op_mul = FF(1),
        .avm_main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(in_tag)),
    });
}

/**
 * @brief Finite field division with direct or indirect memory access.
 *
 * @param indirect A byte encoding information about indirect/direct memory access.
 * @param a_offset An index in memory pointing to the first operand of the division.
 * @param b_offset An index in memory pointing to the second operand of the division.
 * @param dst_offset An index in memory pointing to the output of the division.
 * @param in_tag The instruction memory tag of the operands.
 */
void AvmTraceBuilder::op_fdiv(uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    auto const res = resolve_ind_three(call_ptr, clk, indirect, a_offset, b_offset, dst_offset);
    bool tag_match = res.tag_match;

    // Reading from memory and loading into ia resp. ib.
    auto read_a = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, res.direct_a_offset, AvmMemoryTag::FF, AvmMemoryTag::FF);
    auto read_b = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, res.direct_b_offset, AvmMemoryTag::FF, AvmMemoryTag::FF);
    tag_match = read_a.tag_match && read_b.tag_match;

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
    mem_trace_builder.write_into_memory(
        call_ptr, clk, IntermRegister::IC, res.direct_c_offset, c, AvmMemoryTag::FF, AvmMemoryTag::FF);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::FDIV);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_call_ptr = call_ptr,
        .avm_main_ia = tag_match ? a : FF(0),
        .avm_main_ib = tag_match ? b : FF(0),
        .avm_main_ic = tag_match ? c : FF(0),
        .avm_main_ind_a = res.indirect_flag_a ? FF(a_offset) : FF(0),
        .avm_main_ind_b = res.indirect_flag_b ? FF(b_offset) : FF(0),
        .avm_main_ind_c = res.indirect_flag_c ? FF(dst_offset) : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(res.indirect_flag_a)),
        .avm_main_ind_op_b = FF(static_cast<uint32_t>(res.indirect_flag_b)),
        .avm_main_ind_op_c = FF(static_cast<uint32_t>(res.indirect_flag_c)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_inv = tag_match ? inv : FF(1),
        .avm_main_mem_idx_a = FF(res.direct_a_offset),
        .avm_main_mem_idx_b = FF(res.direct_b_offset),
        .avm_main_mem_idx_c = FF(res.direct_c_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_mem_op_c = FF(1),
        .avm_main_op_err = tag_match ? error : FF(1),
        .avm_main_pc = FF(pc++),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
        .avm_main_rwc = FF(1),
        .avm_main_sel_op_fdiv = FF(1),
        .avm_main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
    });
}

/**
 * @brief Bitwise not with direct or indirect memory access.
 *
 * @param indirect A byte encoding information about indirect/direct memory access.
 * @param a_offset An index in memory pointing to the only operand of Not.
 * @param dst_offset An index in memory pointing to the output of Not.
 * @param in_tag The instruction memory tag of the operands.
 */
void AvmTraceBuilder::op_not(uint8_t indirect, uint32_t a_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;
    bool tag_match = true;
    uint32_t direct_a_offset = a_offset;
    uint32_t direct_dst_offset = dst_offset;

    bool indirect_a_flag = is_operand_indirect(indirect, 0);
    bool indirect_c_flag = is_operand_indirect(indirect, 1);

    if (indirect_a_flag) {
        auto read_ind_a =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_A, a_offset);
        tag_match = read_ind_a.tag_match;
        direct_a_offset = uint32_t(read_ind_a.val);
    }

    if (indirect_c_flag) {
        auto read_ind_c =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_C, dst_offset);
        tag_match = tag_match && read_ind_c.tag_match;
        direct_dst_offset = uint32_t(read_ind_c.val);
    }

    // Reading from memory and loading into ia.
    auto read_a =
        mem_trace_builder.read_and_load_from_memory(call_ptr, clk, IntermRegister::IA, direct_a_offset, in_tag, in_tag);
    tag_match = read_a.tag_match && tag_match;
    // ~a = c
    FF a = read_a.val;

    // In case of a memory tag error, we do not perform the computation.
    // Therefore, we do not create any entry in ALU table and store the value 0 as
    // output (c) in memory.
    FF c = tag_match ? alu_trace_builder.op_not(a, in_tag, clk) : FF(0);

    // Write into memory value c from intermediate register ic.
    mem_trace_builder.write_into_memory(call_ptr, clk, IntermRegister::IC, direct_dst_offset, c, in_tag, in_tag);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::NOT);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_alu_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_call_ptr = call_ptr,
        .avm_main_ia = a,
        .avm_main_ic = c,
        .avm_main_ind_a = indirect_a_flag ? FF(a_offset) : FF(0),
        .avm_main_ind_c = indirect_c_flag ? FF(dst_offset) : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(indirect_a_flag)),
        .avm_main_ind_op_c = FF(static_cast<uint32_t>(indirect_c_flag)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(direct_a_offset),
        .avm_main_mem_idx_c = FF(direct_dst_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_c = FF(1),
        .avm_main_pc = FF(pc++),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_rwc = FF(1),
        .avm_main_sel_op_not = FF(1),
        .avm_main_tag_err = FF(static_cast<uint32_t>(!read_a.tag_match)),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(in_tag)),
    });
}

/**
 * @brief Equality with direct or indirect memory access.
 *
 * @param indirect A byte encoding information about indirect/direct memory access.
 * @param a_offset An index in memory pointing to the first operand of the equality.
 * @param b_offset An index in memory pointing to the second operand of the equality.
 * @param dst_offset An index in memory pointing to the output of the equality.
 * @param in_tag The instruction memory tag of the operands.
 */
void AvmTraceBuilder::op_eq(
    uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    auto const res = resolve_ind_three(call_ptr, clk, indirect, a_offset, b_offset, dst_offset);
    bool tag_match = res.tag_match;

    // Reading from memory and loading into ia resp. ib.
    auto read_a = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, res.direct_a_offset, in_tag, AvmMemoryTag::U8);
    auto read_b = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, res.direct_b_offset, in_tag, AvmMemoryTag::U8);
    tag_match = read_a.tag_match && read_b.tag_match;

    FF a = read_a.val;
    FF b = read_b.val;

    // In case of a memory tag error, we do not perform the computation.
    // Therefore, we do not create any entry in ALU table and store the value 0 as
    // output (c) in memory.
    FF c = tag_match ? alu_trace_builder.op_eq(a, b, in_tag, clk) : FF(0);

    // Write into memory value c from intermediate register ic.
    mem_trace_builder.write_into_memory(
        call_ptr, clk, IntermRegister::IC, res.direct_c_offset, c, in_tag, AvmMemoryTag::U8);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::EQ);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_alu_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_call_ptr = call_ptr,
        .avm_main_ia = a,
        .avm_main_ib = b,
        .avm_main_ic = c,
        .avm_main_ind_a = res.indirect_flag_a ? FF(a_offset) : FF(0),
        .avm_main_ind_b = res.indirect_flag_b ? FF(b_offset) : FF(0),
        .avm_main_ind_c = res.indirect_flag_c ? FF(dst_offset) : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(res.indirect_flag_a)),
        .avm_main_ind_op_b = FF(static_cast<uint32_t>(res.indirect_flag_b)),
        .avm_main_ind_op_c = FF(static_cast<uint32_t>(res.indirect_flag_c)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(res.direct_a_offset),
        .avm_main_mem_idx_b = FF(res.direct_b_offset),
        .avm_main_mem_idx_c = FF(res.direct_c_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_mem_op_c = FF(1),
        .avm_main_pc = FF(pc++),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_rwc = FF(1),
        .avm_main_sel_op_eq = FF(1),
        .avm_main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U8)),
    });
}

void AvmTraceBuilder::op_and(
    uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    auto const res = resolve_ind_three(call_ptr, clk, indirect, a_offset, b_offset, dst_offset);
    bool tag_match = res.tag_match;

    // Reading from memory and loading into ia resp. ib.
    auto read_a = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, res.direct_a_offset, in_tag, in_tag);
    auto read_b = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, res.direct_b_offset, in_tag, in_tag);
    tag_match = read_a.tag_match && read_b.tag_match;

    FF a = tag_match ? read_a.val : FF(0);
    FF b = tag_match ? read_b.val : FF(0);

    FF c = tag_match ? bin_trace_builder.op_and(a, b, in_tag, clk) : FF(0);

    // Write into memory value c from intermediate register ic.
    mem_trace_builder.write_into_memory(call_ptr, clk, IntermRegister::IC, res.direct_c_offset, c, in_tag, in_tag);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::AND);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_bin_op_id = FF(0),
        .avm_main_bin_sel = FF(1),
        .avm_main_call_ptr = call_ptr,
        .avm_main_ia = a,
        .avm_main_ib = b,
        .avm_main_ic = c,
        .avm_main_ind_a = res.indirect_flag_a ? FF(a_offset) : FF(0),
        .avm_main_ind_b = res.indirect_flag_b ? FF(b_offset) : FF(0),
        .avm_main_ind_c = res.indirect_flag_c ? FF(dst_offset) : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(res.indirect_flag_a)),
        .avm_main_ind_op_b = FF(static_cast<uint32_t>(res.indirect_flag_b)),
        .avm_main_ind_op_c = FF(static_cast<uint32_t>(res.indirect_flag_c)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(res.direct_a_offset),
        .avm_main_mem_idx_b = FF(res.direct_b_offset),
        .avm_main_mem_idx_c = FF(res.direct_c_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_mem_op_c = FF(1),
        .avm_main_pc = FF(pc++),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_rwc = FF(1),
        .avm_main_sel_op_and = FF(1),
        .avm_main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(in_tag)),
    });
}

void AvmTraceBuilder::op_or(
    uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    auto const res = resolve_ind_three(call_ptr, clk, indirect, a_offset, b_offset, dst_offset);
    bool tag_match = res.tag_match;

    // Reading from memory and loading into ia resp. ib.
    auto read_a = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, res.direct_a_offset, in_tag, in_tag);
    auto read_b = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, res.direct_b_offset, in_tag, in_tag);
    tag_match = read_a.tag_match && read_b.tag_match;

    FF a = tag_match ? read_a.val : FF(0);
    FF b = tag_match ? read_b.val : FF(0);

    FF c = tag_match ? bin_trace_builder.op_or(a, b, in_tag, clk) : FF(0);

    // Write into memory value c from intermediate register ic.
    mem_trace_builder.write_into_memory(call_ptr, clk, IntermRegister::IC, res.direct_c_offset, c, in_tag, in_tag);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::OR);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_bin_op_id = FF(1),
        .avm_main_bin_sel = FF(1),
        .avm_main_call_ptr = call_ptr,
        .avm_main_ia = a,
        .avm_main_ib = b,
        .avm_main_ic = c,
        .avm_main_ind_a = res.indirect_flag_a ? FF(a_offset) : FF(0),
        .avm_main_ind_b = res.indirect_flag_b ? FF(b_offset) : FF(0),
        .avm_main_ind_c = res.indirect_flag_c ? FF(dst_offset) : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(res.indirect_flag_a)),
        .avm_main_ind_op_b = FF(static_cast<uint32_t>(res.indirect_flag_b)),
        .avm_main_ind_op_c = FF(static_cast<uint32_t>(res.indirect_flag_c)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(res.direct_a_offset),
        .avm_main_mem_idx_b = FF(res.direct_b_offset),
        .avm_main_mem_idx_c = FF(res.direct_c_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_mem_op_c = FF(1),
        .avm_main_pc = FF(pc++),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_rwc = FF(1),
        .avm_main_sel_op_or = FF(1),
        .avm_main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(in_tag)),
    });
}

void AvmTraceBuilder::op_xor(
    uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    auto const res = resolve_ind_three(call_ptr, clk, indirect, a_offset, b_offset, dst_offset);
    bool tag_match = res.tag_match;

    // Reading from memory and loading into ia resp. ib.
    auto read_a = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, res.direct_a_offset, in_tag, in_tag);
    auto read_b = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, res.direct_b_offset, in_tag, in_tag);
    tag_match = read_a.tag_match && read_b.tag_match;

    FF a = tag_match ? read_a.val : FF(0);
    FF b = tag_match ? read_b.val : FF(0);

    FF c = tag_match ? bin_trace_builder.op_xor(a, b, in_tag, clk) : FF(0);

    // Write into memory value c from intermediate register ic.
    mem_trace_builder.write_into_memory(call_ptr, clk, IntermRegister::IC, res.direct_c_offset, c, in_tag, in_tag);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::XOR);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_bin_op_id = FF(2),
        .avm_main_bin_sel = FF(1),
        .avm_main_call_ptr = call_ptr,
        .avm_main_ia = a,
        .avm_main_ib = b,
        .avm_main_ic = c,
        .avm_main_ind_a = res.indirect_flag_a ? FF(a_offset) : FF(0),
        .avm_main_ind_b = res.indirect_flag_b ? FF(b_offset) : FF(0),
        .avm_main_ind_c = res.indirect_flag_c ? FF(dst_offset) : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(res.indirect_flag_a)),
        .avm_main_ind_op_b = FF(static_cast<uint32_t>(res.indirect_flag_b)),
        .avm_main_ind_op_c = FF(static_cast<uint32_t>(res.indirect_flag_c)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(res.direct_a_offset),
        .avm_main_mem_idx_b = FF(res.direct_b_offset),
        .avm_main_mem_idx_c = FF(res.direct_c_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_mem_op_c = FF(1),
        .avm_main_pc = FF(pc++),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_rwc = FF(1),
        .avm_main_sel_op_xor = FF(1),
        .avm_main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(in_tag)),
    });
}

void AvmTraceBuilder::op_lt(
    uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    auto const res = resolve_ind_three(call_ptr, clk, indirect, a_offset, b_offset, dst_offset);
    bool tag_match = res.tag_match;

    // Reading from memory and loading into ia resp. ib.
    auto read_a = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, res.direct_a_offset, in_tag, AvmMemoryTag::U8);
    auto read_b = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, res.direct_b_offset, in_tag, AvmMemoryTag::U8);
    tag_match = read_a.tag_match && read_b.tag_match;

    FF a = tag_match ? read_a.val : FF(0);
    FF b = tag_match ? read_b.val : FF(0);

    FF c = tag_match ? alu_trace_builder.op_lt(a, b, in_tag, clk) : FF(0);

    // Write into memory value c from intermediate register ic.
    mem_trace_builder.write_into_memory(
        call_ptr, clk, IntermRegister::IC, res.direct_c_offset, c, in_tag, AvmMemoryTag::U8);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::LT);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_alu_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_call_ptr = call_ptr,
        .avm_main_ia = a,
        .avm_main_ib = b,
        .avm_main_ic = c,
        .avm_main_ind_a = res.indirect_flag_a ? FF(a_offset) : FF(0),
        .avm_main_ind_b = res.indirect_flag_b ? FF(b_offset) : FF(0),
        .avm_main_ind_c = res.indirect_flag_c ? FF(dst_offset) : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(res.indirect_flag_a)),
        .avm_main_ind_op_b = FF(static_cast<uint32_t>(res.indirect_flag_b)),
        .avm_main_ind_op_c = FF(static_cast<uint32_t>(res.indirect_flag_c)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(res.direct_a_offset),
        .avm_main_mem_idx_b = FF(res.direct_b_offset),
        .avm_main_mem_idx_c = FF(res.direct_c_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_mem_op_c = FF(1),
        .avm_main_pc = FF(pc++),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_rwc = FF(1),
        .avm_main_sel_op_lt = FF(1),
        .avm_main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U8)),
    });
}

void AvmTraceBuilder::op_lte(
    uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    auto const res = resolve_ind_three(call_ptr, clk, indirect, a_offset, b_offset, dst_offset);
    bool tag_match = res.tag_match;

    // Reading from memory and loading into ia resp. ib.
    auto read_a = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, res.direct_a_offset, in_tag, AvmMemoryTag::U8);
    auto read_b = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, res.direct_b_offset, in_tag, AvmMemoryTag::U8);
    tag_match = read_a.tag_match && read_b.tag_match;

    FF a = tag_match ? read_a.val : FF(0);
    FF b = tag_match ? read_b.val : FF(0);

    FF c = tag_match ? alu_trace_builder.op_lte(a, b, in_tag, clk) : FF(0);

    // Write into memory value c from intermediate register ic.
    mem_trace_builder.write_into_memory(
        call_ptr, clk, IntermRegister::IC, res.direct_c_offset, c, in_tag, AvmMemoryTag::U8);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::LTE);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_alu_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_call_ptr = call_ptr,
        .avm_main_ia = a,
        .avm_main_ib = b,
        .avm_main_ic = c,
        .avm_main_ind_a = res.indirect_flag_a ? FF(a_offset) : FF(0),
        .avm_main_ind_b = res.indirect_flag_b ? FF(b_offset) : FF(0),
        .avm_main_ind_c = res.indirect_flag_c ? FF(dst_offset) : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(res.indirect_flag_a)),
        .avm_main_ind_op_b = FF(static_cast<uint32_t>(res.indirect_flag_b)),
        .avm_main_ind_op_c = FF(static_cast<uint32_t>(res.indirect_flag_c)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(res.direct_a_offset),
        .avm_main_mem_idx_b = FF(res.direct_b_offset),
        .avm_main_mem_idx_c = FF(res.direct_c_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_mem_op_c = FF(1),
        .avm_main_pc = FF(pc++),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_rwc = FF(1),
        .avm_main_sel_op_lte = FF(1),
        .avm_main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U8)),
    });
}

void AvmTraceBuilder::op_shr(
    uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{

    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    auto const res = resolve_ind_three(call_ptr, clk, indirect, a_offset, b_offset, dst_offset);
    bool tag_match = res.tag_match;

    // Reading from memory and loading into ia resp. ib.
    auto read_a = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, res.direct_a_offset, in_tag, in_tag);
    auto read_b = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, res.direct_b_offset, in_tag, in_tag);
    tag_match = read_a.tag_match && read_b.tag_match;

    FF a = tag_match ? read_a.val : FF(0);
    FF b = tag_match ? read_b.val : FF(0);

    FF c = tag_match ? alu_trace_builder.op_shr(a, b, in_tag, clk) : FF(0);

    // Write into memory value c from intermediate register ic.
    mem_trace_builder.write_into_memory(call_ptr, clk, IntermRegister::IC, res.direct_c_offset, c, in_tag, in_tag);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::SHR);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_alu_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_call_ptr = call_ptr,
        .avm_main_ia = a,
        .avm_main_ib = b,
        .avm_main_ic = c,
        .avm_main_ind_a = res.indirect_flag_a ? FF(a_offset) : FF(0),
        .avm_main_ind_b = res.indirect_flag_b ? FF(b_offset) : FF(0),
        .avm_main_ind_c = res.indirect_flag_c ? FF(dst_offset) : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(res.indirect_flag_a)),
        .avm_main_ind_op_b = FF(static_cast<uint32_t>(res.indirect_flag_b)),
        .avm_main_ind_op_c = FF(static_cast<uint32_t>(res.indirect_flag_c)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(res.direct_a_offset),
        .avm_main_mem_idx_b = FF(res.direct_b_offset),
        .avm_main_mem_idx_c = FF(res.direct_c_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_mem_op_c = FF(1),
        .avm_main_pc = FF(pc++),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_rwc = FF(1),
        .avm_main_sel_op_shr = FF(1),
        .avm_main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(in_tag)),
    });
}

void AvmTraceBuilder::op_shl(
    uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    auto const res = resolve_ind_three(call_ptr, clk, indirect, a_offset, b_offset, dst_offset);
    bool tag_match = res.tag_match;

    // Reading from memory and loading into ia resp. ib.
    auto read_a = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, res.direct_a_offset, in_tag, in_tag);
    auto read_b = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, res.direct_b_offset, in_tag, in_tag);
    tag_match = read_a.tag_match && read_b.tag_match;

    FF a = tag_match ? read_a.val : FF(0);
    FF b = tag_match ? read_b.val : FF(0);

    FF c = tag_match ? alu_trace_builder.op_shl(a, b, in_tag, clk) : FF(0);

    // Write into memory value c from intermediate register ic.
    mem_trace_builder.write_into_memory(call_ptr, clk, IntermRegister::IC, res.direct_c_offset, c, in_tag, in_tag);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::SHL);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_alu_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_call_ptr = call_ptr,
        .avm_main_ia = a,
        .avm_main_ib = b,
        .avm_main_ic = c,
        .avm_main_ind_a = res.indirect_flag_a ? FF(a_offset) : FF(0),
        .avm_main_ind_b = res.indirect_flag_b ? FF(b_offset) : FF(0),
        .avm_main_ind_c = res.indirect_flag_c ? FF(dst_offset) : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(res.indirect_flag_a)),
        .avm_main_ind_op_b = FF(static_cast<uint32_t>(res.indirect_flag_b)),
        .avm_main_ind_op_c = FF(static_cast<uint32_t>(res.indirect_flag_c)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(res.direct_a_offset),
        .avm_main_mem_idx_b = FF(res.direct_b_offset),
        .avm_main_mem_idx_c = FF(res.direct_c_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_mem_op_c = FF(1),
        .avm_main_pc = FF(pc++),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_rwc = FF(1),
        .avm_main_sel_op_shl = FF(1),
        .avm_main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(in_tag)),
    });
}

// TODO: Ensure that the bytecode validation and/or deserialization is
//       enforcing that val complies to the tag.
/**
 * @brief Set a constant from bytecode with direct or indirect memory access.
 *        SET opcode is implemented purely as a memory operation. As val is a
 *        constant passed in the bytecode, the deserialization layer or bytecode
 *        validation circuit is enforcing that the constant complies to in_tag.
 *        Therefore, no range check is required as part of this opcode relation.
 *
 * @param indirect A byte encoding information about indirect/direct memory access.
 * @param val The constant to be written upcasted to u128
 * @param dst_offset Memory destination offset where val is written to
 * @param in_tag The instruction memory tag
 */
void AvmTraceBuilder::op_set(uint8_t indirect, uint128_t val, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto const clk = static_cast<uint32_t>(main_trace.size()) + 1;
    auto const val_ff = FF{ uint256_t::from_uint128(val) };
    uint32_t direct_dst_offset = dst_offset; // Overriden in indirect mode
    bool indirect_dst_flag = is_operand_indirect(indirect, 0);
    bool tag_match = true;

    if (indirect_dst_flag) {
        auto read_ind_c =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_C, dst_offset);
        tag_match = read_ind_c.tag_match;
        direct_dst_offset = uint32_t(read_ind_c.val);
    }

    mem_trace_builder.write_into_memory(
        call_ptr, clk, IntermRegister::IC, direct_dst_offset, val_ff, AvmMemoryTag::U0, in_tag);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::SET);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_call_ptr = call_ptr,
        .avm_main_ic = val_ff,
        .avm_main_ind_c = indirect_dst_flag ? dst_offset : 0,
        .avm_main_ind_op_c = static_cast<uint32_t>(indirect_dst_flag),
        .avm_main_internal_return_ptr = internal_return_ptr,
        .avm_main_mem_idx_c = direct_dst_offset,
        .avm_main_mem_op_activate_gas = 1, // TODO: remove in the long term
        .avm_main_mem_op_c = 1,
        .avm_main_pc = pc++,
        .avm_main_rwc = 1,
        .avm_main_tag_err = static_cast<uint32_t>(!tag_match),
        .avm_main_w_in_tag = static_cast<uint32_t>(in_tag),
    });
}

/**
 * @brief Copy value and tag from a memory cell at position src_offset to the
 *        memory cell at position dst_offset
 *
 * @param indirect A byte encoding information about indirect/direct memory access.
 * @param src_offset Offset of source memory cell
 * @param dst_offset Offset of destination memory cell
 */
void AvmTraceBuilder::op_mov(uint8_t indirect, uint32_t src_offset, uint32_t dst_offset)
{
    auto const clk = static_cast<uint32_t>(main_trace.size()) + 1;
    bool tag_match = true;
    uint32_t direct_src_offset = src_offset;
    uint32_t direct_dst_offset = dst_offset;

    bool indirect_src_flag = is_operand_indirect(indirect, 0);
    bool indirect_dst_flag = is_operand_indirect(indirect, 1);

    if (indirect_src_flag) {
        auto read_ind_a =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_A, src_offset);
        tag_match = read_ind_a.tag_match;
        direct_src_offset = uint32_t(read_ind_a.val);
    }

    if (indirect_dst_flag) {
        auto read_ind_c =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_C, dst_offset);
        tag_match = tag_match && read_ind_c.tag_match;
        direct_dst_offset = uint32_t(read_ind_c.val);
    }

    // Reading from memory and loading into ia without tag check.
    auto const [val, tag] = mem_trace_builder.read_and_load_mov_opcode(call_ptr, clk, direct_src_offset);

    // Write into memory from intermediate register ic.
    mem_trace_builder.write_into_memory(call_ptr, clk, IntermRegister::IC, direct_dst_offset, val, tag, tag);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::MOV);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_call_ptr = call_ptr,
        .avm_main_ia = val,
        .avm_main_ic = val,
        .avm_main_ind_a = indirect_src_flag ? src_offset : 0,
        .avm_main_ind_c = indirect_dst_flag ? dst_offset : 0,
        .avm_main_ind_op_a = static_cast<uint32_t>(indirect_src_flag),
        .avm_main_ind_op_c = static_cast<uint32_t>(indirect_dst_flag),
        .avm_main_internal_return_ptr = internal_return_ptr,
        .avm_main_mem_idx_a = direct_src_offset,
        .avm_main_mem_idx_c = direct_dst_offset,
        .avm_main_mem_op_a = 1,
        .avm_main_mem_op_c = 1,
        .avm_main_pc = pc++,
        .avm_main_r_in_tag = static_cast<uint32_t>(tag),
        .avm_main_rwc = 1,
        .avm_main_sel_mov = 1,
        .avm_main_sel_mov_a = 1,
        .avm_main_tag_err = static_cast<uint32_t>(!tag_match),
        .avm_main_w_in_tag = static_cast<uint32_t>(tag),
    });
}

/**
 * @brief Copy value and tag from a memory cell at position src_offset to the
 *        memory cell at position dst_offset. src_offset is a_offset if the value
 *        defined by cond_offset is non-zero. Otherwise, src_offset is b_offset.
 *
 * @param indirect A byte encoding information about indirect/direct memory access.
 * @param a_offset Offset of first candidate source memory cell
 * @param b_offset Offset of second candidate source memory cell
 * @param cond_offset Offset of the condition determining the source offset (a_offset or b_offset)
 * @param dst_offset Offset of destination memory cell
 */
void AvmTraceBuilder::op_cmov(
    uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t cond_offset, uint32_t dst_offset)
{
    auto const clk = static_cast<uint32_t>(main_trace.size()) + 1;
    bool tag_match = true;
    uint32_t direct_a_offset = a_offset;
    uint32_t direct_b_offset = b_offset;
    uint32_t direct_cond_offset = cond_offset;
    uint32_t direct_dst_offset = dst_offset;

    bool indirect_a_flag = is_operand_indirect(indirect, 0);
    bool indirect_b_flag = is_operand_indirect(indirect, 1);
    bool indirect_cond_flag = is_operand_indirect(indirect, 2);
    bool indirect_dst_flag = is_operand_indirect(indirect, 3);

    if (indirect_a_flag) {
        auto read_ind_a =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_A, a_offset);
        direct_a_offset = uint32_t(read_ind_a.val);
        tag_match = tag_match && read_ind_a.tag_match;
    }

    if (indirect_b_flag) {
        auto read_ind_b =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_B, b_offset);
        direct_b_offset = uint32_t(read_ind_b.val);
        tag_match = tag_match && read_ind_b.tag_match;
    }

    if (indirect_cond_flag) {
        auto read_ind_d =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_D, cond_offset);
        direct_cond_offset = uint32_t(read_ind_d.val);
        tag_match = tag_match && read_ind_d.tag_match;
    }

    if (indirect_dst_flag) {
        auto read_ind_c =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_C, dst_offset);
        direct_dst_offset = uint32_t(read_ind_c.val);
        tag_match = tag_match && read_ind_c.tag_match;
    }

    // Reading from memory and loading into ia or ib without tag check. We also load the conditional value
    // in id without any tag check.
    std::array<AvmMemTraceBuilder::MemEntry, 3> const cmov_res = mem_trace_builder.read_and_load_cmov_opcode(
        call_ptr, clk, direct_a_offset, direct_b_offset, direct_cond_offset);

    AvmMemTraceBuilder::MemEntry const& a_mem_entry = cmov_res.at(0);
    AvmMemTraceBuilder::MemEntry const& b_mem_entry = cmov_res.at(1);
    AvmMemTraceBuilder::MemEntry const& cond_mem_entry = cmov_res.at(2);

    const bool id_zero = cond_mem_entry.val == 0;

    auto const& val = id_zero ? b_mem_entry.val : a_mem_entry.val;
    auto const& tag = id_zero ? b_mem_entry.tag : a_mem_entry.tag;

    // Write into memory from intermediate register ic.
    mem_trace_builder.write_into_memory(call_ptr, clk, IntermRegister::IC, direct_dst_offset, val, tag, tag);

    FF const inv = !id_zero ? cond_mem_entry.val.invert() : 1;

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::CMOV);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_call_ptr = call_ptr,
        .avm_main_ia = a_mem_entry.val,
        .avm_main_ib = b_mem_entry.val,
        .avm_main_ic = val,
        .avm_main_id = cond_mem_entry.val,
        .avm_main_id_zero = static_cast<uint32_t>(id_zero),
        .avm_main_ind_a = indirect_a_flag ? a_offset : 0,
        .avm_main_ind_b = indirect_b_flag ? b_offset : 0,
        .avm_main_ind_c = indirect_dst_flag ? dst_offset : 0,
        .avm_main_ind_d = indirect_cond_flag ? cond_offset : 0,
        .avm_main_ind_op_a = static_cast<uint32_t>(indirect_a_flag),
        .avm_main_ind_op_b = static_cast<uint32_t>(indirect_b_flag),
        .avm_main_ind_op_c = static_cast<uint32_t>(indirect_dst_flag),
        .avm_main_ind_op_d = static_cast<uint32_t>(indirect_cond_flag),
        .avm_main_internal_return_ptr = internal_return_ptr,
        .avm_main_inv = inv,
        .avm_main_mem_idx_a = direct_a_offset,
        .avm_main_mem_idx_b = direct_b_offset,
        .avm_main_mem_idx_c = direct_dst_offset,
        .avm_main_mem_idx_d = direct_cond_offset,
        .avm_main_mem_op_a = 1,
        .avm_main_mem_op_b = 1,
        .avm_main_mem_op_c = 1,
        .avm_main_mem_op_d = 1,
        .avm_main_pc = pc++,
        .avm_main_r_in_tag = static_cast<uint32_t>(tag),
        .avm_main_rwc = 1,
        .avm_main_sel_cmov = 1,
        .avm_main_sel_mov_a = static_cast<uint32_t>(!id_zero),
        .avm_main_sel_mov_b = static_cast<uint32_t>(id_zero),
        .avm_main_tag_err = static_cast<uint32_t>(!tag_match),
        .avm_main_w_in_tag = static_cast<uint32_t>(tag),
    });
}

// Helper function to add kernel lookup operations into the main trace
// TODO: add tag match to kernel_input_lookup opcodes to - it isnt written to - -ve test would catch
Row AvmTraceBuilder::create_kernel_lookup_opcode(
    bool indirect, uint32_t dst_offset, uint32_t selector, FF value, AvmMemoryTag w_tag)
{
    auto const clk = static_cast<uint32_t>(main_trace.size()) + 1;

    bool tag_match = true;
    uint32_t direct_dst_offset = dst_offset;
    if (indirect) {
        auto read_ind_dst =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_A, dst_offset);
        direct_dst_offset = uint32_t(read_ind_dst.val);
        tag_match = tag_match && read_ind_dst.tag_match;
    }

    AvmMemoryTag r_tag = AvmMemoryTag::U0;
    mem_trace_builder.write_into_memory(call_ptr, clk, IntermRegister::IA, direct_dst_offset, value, r_tag, w_tag);

    return Row{
        .avm_main_clk = clk,
        .avm_kernel_kernel_in_offset = selector,
        .avm_main_call_ptr = call_ptr,
        .avm_main_ia = value,
        .avm_main_ind_a = indirect ? FF(dst_offset) : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(indirect)),
        .avm_main_internal_return_ptr = internal_return_ptr,
        .avm_main_mem_idx_a = direct_dst_offset,
        .avm_main_mem_op_a = 1,
        .avm_main_pc = pc++,
        .avm_main_q_kernel_lookup = 1,
        .avm_main_rwa = 1,
        .avm_main_w_in_tag = static_cast<uint32_t>(w_tag),
    };
}

void AvmTraceBuilder::op_storage_address(uint8_t indirect, uint32_t dst_offset)
{
    FF ia_value = kernel_trace_builder.op_storage_address();

    bool indirect_dst_flag = is_operand_indirect(indirect, 0);
    Row row = create_kernel_lookup_opcode(
        indirect_dst_flag, dst_offset, STORAGE_ADDRESS_SELECTOR, ia_value, AvmMemoryTag::FF);
    row.avm_main_sel_op_storage_address = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(static_cast<uint32_t>(row.avm_main_clk), OpCode::STORAGEADDRESS);

    main_trace.push_back(row);
}

void AvmTraceBuilder::op_sender(uint8_t indirect, uint32_t dst_offset)
{
    FF ia_value = kernel_trace_builder.op_sender();

    bool indirect_dst_flag = is_operand_indirect(indirect, 0);
    Row row = create_kernel_lookup_opcode(indirect_dst_flag, dst_offset, SENDER_SELECTOR, ia_value, AvmMemoryTag::FF);
    row.avm_main_sel_op_sender = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(static_cast<uint32_t>(row.avm_main_clk), OpCode::SENDER);

    main_trace.push_back(row);
}

void AvmTraceBuilder::op_address(uint8_t indirect, uint32_t dst_offset)
{
    FF ia_value = kernel_trace_builder.op_address();

    bool indirect_dst_flag = is_operand_indirect(indirect, 0);
    Row row = create_kernel_lookup_opcode(indirect_dst_flag, dst_offset, ADDRESS_SELECTOR, ia_value, AvmMemoryTag::FF);
    row.avm_main_sel_op_address = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(static_cast<uint32_t>(row.avm_main_clk), OpCode::ADDRESS);

    main_trace.push_back(row);
}

void AvmTraceBuilder::op_fee_per_da_gas(uint8_t indirect, uint32_t dst_offset)
{
    FF ia_value = kernel_trace_builder.op_fee_per_da_gas();

    bool indirect_dst_flag = is_operand_indirect(indirect, 0);
    Row row =
        create_kernel_lookup_opcode(indirect_dst_flag, dst_offset, FEE_PER_DA_GAS_SELECTOR, ia_value, AvmMemoryTag::FF);
    row.avm_main_sel_op_fee_per_da_gas = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(static_cast<uint32_t>(row.avm_main_clk), OpCode::FEEPERDAGAS);

    main_trace.push_back(row);
}

void AvmTraceBuilder::op_fee_per_l2_gas(uint8_t indirect, uint32_t dst_offset)
{
    FF ia_value = kernel_trace_builder.op_fee_per_l2_gas();

    bool indirect_dst_flag = is_operand_indirect(indirect, 0);
    Row row =
        create_kernel_lookup_opcode(indirect_dst_flag, dst_offset, FEE_PER_L2_GAS_SELECTOR, ia_value, AvmMemoryTag::FF);
    row.avm_main_sel_op_fee_per_l2_gas = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(static_cast<uint32_t>(row.avm_main_clk), OpCode::FEEPERL2GAS);

    main_trace.push_back(row);
}

void AvmTraceBuilder::op_transaction_fee(uint8_t indirect, uint32_t dst_offset)
{
    FF ia_value = kernel_trace_builder.op_transaction_fee();

    bool indirect_dst_flag = is_operand_indirect(indirect, 0);
    Row row = create_kernel_lookup_opcode(
        indirect_dst_flag, dst_offset, TRANSACTION_FEE_SELECTOR, ia_value, AvmMemoryTag::FF);
    row.avm_main_sel_op_transaction_fee = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(static_cast<uint32_t>(row.avm_main_clk), OpCode::TRANSACTIONFEE);

    main_trace.push_back(row);
}

void AvmTraceBuilder::op_chain_id(uint8_t indirect, uint32_t dst_offset)
{
    FF ia_value = kernel_trace_builder.op_chain_id();

    bool indirect_dst_flag = is_operand_indirect(indirect, 0);
    Row row = create_kernel_lookup_opcode(indirect_dst_flag, dst_offset, CHAIN_ID_SELECTOR, ia_value, AvmMemoryTag::FF);
    row.avm_main_sel_op_chain_id = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(static_cast<uint32_t>(row.avm_main_clk), OpCode::CHAINID);

    main_trace.push_back(row);
}

void AvmTraceBuilder::op_version(uint8_t indirect, uint32_t dst_offset)
{
    FF ia_value = kernel_trace_builder.op_version();

    bool indirect_dst_flag = is_operand_indirect(indirect, 0);
    Row row = create_kernel_lookup_opcode(indirect_dst_flag, dst_offset, VERSION_SELECTOR, ia_value, AvmMemoryTag::FF);
    row.avm_main_sel_op_version = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(static_cast<uint32_t>(row.avm_main_clk), OpCode::VERSION);

    main_trace.push_back(row);
}

void AvmTraceBuilder::op_block_number(uint8_t indirect, uint32_t dst_offset)
{
    FF ia_value = kernel_trace_builder.op_block_number();

    bool indirect_dst_flag = is_operand_indirect(indirect, 0);
    Row row =
        create_kernel_lookup_opcode(indirect_dst_flag, dst_offset, BLOCK_NUMBER_SELECTOR, ia_value, AvmMemoryTag::FF);
    row.avm_main_sel_op_block_number = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(static_cast<uint32_t>(row.avm_main_clk), OpCode::BLOCKNUMBER);

    main_trace.push_back(row);
}

void AvmTraceBuilder::op_coinbase(uint8_t indirect, uint32_t dst_offset)
{
    FF ia_value = kernel_trace_builder.op_coinbase();

    bool indirect_dst_flag = is_operand_indirect(indirect, 0);
    Row row = create_kernel_lookup_opcode(indirect_dst_flag, dst_offset, COINBASE_SELECTOR, ia_value, AvmMemoryTag::FF);
    row.avm_main_sel_op_coinbase = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(static_cast<uint32_t>(row.avm_main_clk), OpCode::COINBASE);

    main_trace.push_back(row);
}

void AvmTraceBuilder::op_timestamp(uint8_t indirect, uint32_t dst_offset)
{
    FF ia_value = kernel_trace_builder.op_timestamp();

    bool indirect_dst_flag = is_operand_indirect(indirect, 0);
    Row row =
        create_kernel_lookup_opcode(indirect_dst_flag, dst_offset, TIMESTAMP_SELECTOR, ia_value, AvmMemoryTag::U64);
    row.avm_main_sel_op_timestamp = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(static_cast<uint32_t>(row.avm_main_clk), OpCode::TIMESTAMP);

    main_trace.push_back(row);
}

// Helper function to add kernel lookup operations into the main trace
Row AvmTraceBuilder::create_kernel_output_opcode(uint8_t indirect, uint32_t clk, uint32_t data_offset)
{
    bool indirect_data_flag = is_operand_indirect(indirect, 0);

    bool tag_match = true;
    uint32_t direct_data_offset = data_offset;
    if (indirect) {
        auto read_ind_dst =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_A, data_offset);
        direct_data_offset = uint32_t(read_ind_dst.val);
        tag_match = tag_match && read_ind_dst.tag_match;
    }

    AvmMemTraceBuilder::MemRead read_a = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, direct_data_offset, AvmMemoryTag::FF, AvmMemoryTag::U0);

    return Row{
        .avm_main_clk = clk,
        .avm_main_ia = read_a.val,
        .avm_main_ind_a = indirect_data_flag ? FF(data_offset) : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(indirect)),
        .avm_main_internal_return_ptr = internal_return_ptr,
        .avm_main_mem_idx_a = direct_data_offset,
        .avm_main_mem_op_a = 1,
        .avm_main_pc = pc++,
        .avm_main_q_kernel_output_lookup = 1,
        .avm_main_r_in_tag = static_cast<uint32_t>(AvmMemoryTag::FF),
        .avm_main_rwa = 0,
    };
}

Row AvmTraceBuilder::create_kernel_output_opcode_with_metadata(uint8_t indirect,
                                                               uint32_t clk,
                                                               uint32_t data_offset,
                                                               AvmMemoryTag data_r_tag,
                                                               uint32_t metadata_offset,
                                                               AvmMemoryTag metadata_r_tag)
{

    bool indirect_a_flag = is_operand_indirect(indirect, 0);
    bool indirect_b_flag = is_operand_indirect(indirect, 1);

    bool tag_match = true;
    uint32_t direct_data_offset = data_offset;
    uint32_t direct_metadata_offset = metadata_offset;
    if (indirect_a_flag) {
        auto read_a_ind_dst =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_A, data_offset);
        direct_data_offset = static_cast<uint32_t>(read_a_ind_dst.val);

        tag_match = tag_match && read_a_ind_dst.tag_match;
    }
    if (indirect_b_flag) {
        auto read_b_ind_dst = mem_trace_builder.indirect_read_and_load_from_memory(
            call_ptr, clk, IndirectRegister::IND_B, metadata_offset);
        direct_metadata_offset = static_cast<uint32_t>(read_b_ind_dst.val);

        tag_match = tag_match && read_b_ind_dst.tag_match;
    }

    AvmMemTraceBuilder::MemRead read_a = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, direct_data_offset, data_r_tag, AvmMemoryTag::U0);

    AvmMemTraceBuilder::MemRead read_b = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, direct_metadata_offset, metadata_r_tag, AvmMemoryTag::U0);

    return Row{
        .avm_main_clk = clk,
        .avm_main_ia = read_a.val,
        .avm_main_ib = read_b.val,
        .avm_main_ind_a = indirect_a_flag ? data_offset : FF(0),
        .avm_main_ind_b = indirect_b_flag ? metadata_offset : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(indirect_a_flag)),
        .avm_main_ind_op_b = FF(static_cast<uint32_t>(indirect_b_flag)),
        .avm_main_internal_return_ptr = internal_return_ptr,
        .avm_main_mem_idx_a = direct_data_offset,
        .avm_main_mem_idx_b = direct_metadata_offset,
        .avm_main_mem_op_a = 1,
        .avm_main_mem_op_b = 1,
        .avm_main_pc = pc++,
        .avm_main_q_kernel_output_lookup = 1,
        .avm_main_r_in_tag = static_cast<uint32_t>(data_r_tag),
        .avm_main_rwa = 0,
        .avm_main_rwb = 0,
    };
}

Row AvmTraceBuilder::create_kernel_output_opcode_with_set_metadata_output_from_hint(uint8_t indirect,
                                                                                    uint32_t clk,
                                                                                    uint32_t data_offset,
                                                                                    uint32_t metadata_offset)
{

    FF exists = execution_hints.get_side_effect_hints().at(side_effect_counter);
    // TODO: throw error if incorrect

    bool indirect_a_flag = is_operand_indirect(indirect, 0);
    bool indirect_b_flag = is_operand_indirect(indirect, 1);

    bool tag_match = true;
    uint32_t direct_data_offset = data_offset;
    uint32_t direct_metadata_offset = metadata_offset;
    if (indirect_a_flag) {
        auto read_a_ind_dst =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_A, data_offset);
        direct_data_offset = uint32_t(read_a_ind_dst.val);

        tag_match = tag_match && read_a_ind_dst.tag_match;
    }

    if (indirect_b_flag) {
        auto read_b_ind_dst = mem_trace_builder.indirect_read_and_load_from_memory(
            call_ptr, clk, IndirectRegister::IND_B, metadata_offset);
        direct_metadata_offset = uint32_t(read_b_ind_dst.val);

        tag_match = tag_match && read_b_ind_dst.tag_match;
    }

    AvmMemTraceBuilder::MemRead read_a = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, direct_data_offset, AvmMemoryTag::FF, AvmMemoryTag::U8);

    mem_trace_builder.write_into_memory(
        call_ptr, clk, IntermRegister::IB, direct_metadata_offset, exists, AvmMemoryTag::FF, AvmMemoryTag::U8);

    return Row{
        .avm_main_clk = clk,
        .avm_main_ia = read_a.val,
        .avm_main_ib = exists,
        .avm_main_ind_a = indirect_a_flag ? data_offset : FF(0),
        .avm_main_ind_b = indirect_b_flag ? metadata_offset : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(indirect_a_flag)),
        .avm_main_ind_op_b = FF(static_cast<uint32_t>(indirect_b_flag)),
        .avm_main_internal_return_ptr = internal_return_ptr,
        .avm_main_mem_idx_a = direct_data_offset,
        .avm_main_mem_idx_b = direct_metadata_offset,
        .avm_main_mem_op_a = 1,
        .avm_main_mem_op_b = 1,
        .avm_main_pc = pc++,
        .avm_main_q_kernel_output_lookup = 1,
        .avm_main_r_in_tag = static_cast<uint32_t>(AvmMemoryTag::FF),
        .avm_main_rwa = 0,
        .avm_main_rwb = 1,
        .avm_main_w_in_tag = static_cast<uint32_t>(AvmMemoryTag::U8),
    };
}

Row AvmTraceBuilder::create_kernel_output_opcode_with_set_value_from_hint(uint8_t indirect,
                                                                          uint32_t clk,
                                                                          uint32_t data_offset,
                                                                          uint32_t metadata_offset)
{
    FF value = execution_hints.get_side_effect_hints().at(side_effect_counter);
    // TODO: throw error if incorrect

    bool indirect_a_flag = is_operand_indirect(indirect, 0);
    bool indirect_b_flag = is_operand_indirect(indirect, 1);

    bool tag_match = true;
    uint32_t direct_data_offset = data_offset;
    uint32_t direct_metadata_offset = metadata_offset;
    if (indirect) {
        auto read_a_ind_dst =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_A, data_offset);
        auto read_b_ind_dst = mem_trace_builder.indirect_read_and_load_from_memory(
            call_ptr, clk, IndirectRegister::IND_B, metadata_offset);

        direct_data_offset = uint32_t(read_a_ind_dst.val);
        direct_metadata_offset = uint32_t(read_b_ind_dst.val);

        tag_match = tag_match && read_a_ind_dst.tag_match && read_b_ind_dst.tag_match;
    }

    mem_trace_builder.write_into_memory(
        call_ptr, clk, IntermRegister::IA, direct_data_offset, value, AvmMemoryTag::FF, AvmMemoryTag::FF);

    AvmMemTraceBuilder::MemRead read_b = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, direct_metadata_offset, AvmMemoryTag::FF, AvmMemoryTag::FF);

    return Row{
        .avm_main_clk = clk,
        .avm_main_ia = value,
        .avm_main_ib = read_b.val,
        .avm_main_ind_a = indirect_a_flag ? data_offset : FF(0),
        .avm_main_ind_b = indirect_b_flag ? metadata_offset : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(indirect_a_flag)),
        .avm_main_ind_op_b = FF(static_cast<uint32_t>(indirect_b_flag)),
        .avm_main_internal_return_ptr = internal_return_ptr,
        .avm_main_mem_idx_a = direct_data_offset,
        .avm_main_mem_idx_b = direct_metadata_offset,
        .avm_main_mem_op_a = 1,
        .avm_main_mem_op_b = 1,
        .avm_main_pc = pc, // No PC increment here since we do it in the specific ops
        .avm_main_q_kernel_output_lookup = 1,
        .avm_main_r_in_tag = static_cast<uint32_t>(AvmMemoryTag::FF),
        .avm_main_rwa = 1,
        .avm_main_rwb = 0,
        .avm_main_w_in_tag = static_cast<uint32_t>(AvmMemoryTag::FF),
    };
}

void AvmTraceBuilder::op_emit_note_hash(uint8_t indirect, uint32_t note_hash_offset)
{
    auto const clk = static_cast<uint32_t>(main_trace.size()) + 1;

    Row row = create_kernel_output_opcode(indirect, clk, note_hash_offset);
    kernel_trace_builder.op_emit_note_hash(clk, side_effect_counter, row.avm_main_ia);
    row.avm_main_sel_op_emit_note_hash = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::EMITNOTEHASH);

    main_trace.push_back(row);
    side_effect_counter++;
}

void AvmTraceBuilder::op_emit_nullifier(uint8_t indirect, uint32_t nullifier_offset)
{
    auto const clk = static_cast<uint32_t>(main_trace.size()) + 1;

    Row row = create_kernel_output_opcode(indirect, clk, nullifier_offset);
    kernel_trace_builder.op_emit_nullifier(clk, side_effect_counter, row.avm_main_ia);
    row.avm_main_sel_op_emit_nullifier = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::EMITNULLIFIER);

    main_trace.push_back(row);
    side_effect_counter++;
}

void AvmTraceBuilder::op_emit_l2_to_l1_msg(uint8_t indirect, uint32_t recipient_offset, uint32_t msg_offset)
{
    auto const clk = static_cast<uint32_t>(main_trace.size()) + 1;

    // Note: unorthadox order - as seen in L2ToL1Message struct in TS
    Row row = create_kernel_output_opcode_with_metadata(
        indirect, clk, msg_offset, AvmMemoryTag::FF, recipient_offset, AvmMemoryTag::FF);
    kernel_trace_builder.op_emit_l2_to_l1_msg(clk, side_effect_counter, row.avm_main_ia, row.avm_main_ib);
    row.avm_main_sel_op_emit_l2_to_l1_msg = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::SENDL2TOL1MSG);

    main_trace.push_back(row);
    side_effect_counter++;
}

void AvmTraceBuilder::op_emit_unencrypted_log(uint8_t indirect, uint32_t log_offset)
{
    auto const clk = static_cast<uint32_t>(main_trace.size()) + 1;

    Row row = create_kernel_output_opcode(indirect, clk, log_offset);
    kernel_trace_builder.op_emit_unencrypted_log(clk, side_effect_counter, row.avm_main_ia);
    row.avm_main_sel_op_emit_unencrypted_log = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::EMITUNENCRYPTEDLOG);

    main_trace.push_back(row);
    side_effect_counter++;
}

// State output opcodes that include metadata
void AvmTraceBuilder::op_l1_to_l2_msg_exists(uint8_t indirect, uint32_t log_offset, uint32_t dest_offset)
{
    auto const clk = static_cast<uint32_t>(main_trace.size()) + 1;

    Row row = create_kernel_output_opcode_with_set_metadata_output_from_hint(indirect, clk, log_offset, dest_offset);
    kernel_trace_builder.op_l1_to_l2_msg_exists(
        clk, side_effect_counter, row.avm_main_ia, /*safe*/ static_cast<uint32_t>(row.avm_main_ib));
    row.avm_main_sel_op_l1_to_l2_msg_exists = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::L1TOL2MSGEXISTS);

    main_trace.push_back(row);
    side_effect_counter++;
}

void AvmTraceBuilder::op_note_hash_exists(uint8_t indirect, uint32_t note_offset, uint32_t dest_offset)
{
    auto const clk = static_cast<uint32_t>(main_trace.size()) + 1;

    Row row = create_kernel_output_opcode_with_set_metadata_output_from_hint(indirect, clk, note_offset, dest_offset);
    kernel_trace_builder.op_note_hash_exists(
        clk, side_effect_counter, row.avm_main_ia, /*safe*/ static_cast<uint32_t>(row.avm_main_ib));
    row.avm_main_sel_op_note_hash_exists = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::NOTEHASHEXISTS);

    main_trace.push_back(row);
    side_effect_counter++;
}

void AvmTraceBuilder::op_nullifier_exists(uint8_t indirect, uint32_t nullifier_offset, uint32_t dest_offset)
{
    auto const clk = static_cast<uint32_t>(main_trace.size()) + 1;

    Row row =
        create_kernel_output_opcode_with_set_metadata_output_from_hint(indirect, clk, nullifier_offset, dest_offset);
    kernel_trace_builder.op_nullifier_exists(
        clk, side_effect_counter, row.avm_main_ia, /*safe*/ static_cast<uint32_t>(row.avm_main_ib));
    row.avm_main_sel_op_nullifier_exists = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::NULLIFIEREXISTS);

    main_trace.push_back(row);
    side_effect_counter++;
}

void AvmTraceBuilder::op_sload(uint8_t indirect, uint32_t slot_offset, uint32_t size, uint32_t dest_offset)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    // TODO: align usage of indirect with simulator
    // TODO: support indirect slot offset
    bool dest_offset_is_indirect = is_operand_indirect(indirect, 1);

    auto direct_dest_offset = dest_offset;
    if (dest_offset_is_indirect) {
        auto read_ind_dest_offset =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_A, dest_offset);
        direct_dest_offset = uint32_t(read_ind_dest_offset.val);
    }
    auto read_dest_value = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, direct_dest_offset, AvmMemoryTag::FF, AvmMemoryTag::FF);

    AvmMemTraceBuilder::MemRead read_slot = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, slot_offset, AvmMemoryTag::FF, AvmMemoryTag::FF);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_ia = read_dest_value.val,
        .avm_main_ib = read_slot.val,
        .avm_main_ind_a = dest_offset_is_indirect ? dest_offset : 0,
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(dest_offset_is_indirect)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(direct_dest_offset),
        .avm_main_mem_idx_b = FF(slot_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_pc = pc, // No PC increment here since this is the same opcode as the rows created below
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
    });
    clk++;

    for (uint32_t i = 0; i < size; i++) {
        FF value = execution_hints.get_side_effect_hints().at(side_effect_counter);

        mem_trace_builder.write_into_memory(
            call_ptr, clk, IntermRegister::IA, direct_dest_offset + i, value, AvmMemoryTag::FF, AvmMemoryTag::FF);

        auto row = Row{
            .avm_main_clk = clk,
            .avm_main_ia = value,
            .avm_main_ib = read_slot.val + i, // slot increments each time
            .avm_main_internal_return_ptr = internal_return_ptr,
            .avm_main_mem_idx_a = direct_dest_offset + i,
            .avm_main_mem_op_a = 1,
            .avm_main_pc = pc, // No PC increment here since this is the same opcode for all loop iterations
            .avm_main_q_kernel_output_lookup = 1,
            .avm_main_r_in_tag = static_cast<uint32_t>(AvmMemoryTag::FF),
            .avm_main_rwa = 1,
            .avm_main_sel_op_sload = FF(1),
            .avm_main_w_in_tag = static_cast<uint32_t>(AvmMemoryTag::FF),
        };

        // Output storage read to kernel outputs (performs lookup)
        kernel_trace_builder.op_sload(clk, side_effect_counter, row.avm_main_ib, row.avm_main_ia);

        // Constrain gas cost
        gas_trace_builder.constrain_gas_lookup(clk, OpCode::SLOAD);

        main_trace.push_back(row);
        side_effect_counter++;
        clk++;
    }
    pc++;
}

void AvmTraceBuilder::op_sstore(uint8_t indirect, uint32_t src_offset, uint32_t size, uint32_t slot_offset)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    // TODO: align usage of indirect with simulator
    // TODO: support indirect slot offset
    bool src_offset_is_indirect = is_operand_indirect(indirect, 0);

    // Resolve loads and indirect
    auto direct_src_offset = src_offset;
    if (src_offset_is_indirect) {
        auto read_ind_src_offset =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_A, src_offset);
        direct_src_offset = uint32_t(read_ind_src_offset.val);
    }
    auto read_src_value = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, direct_src_offset, AvmMemoryTag::FF, AvmMemoryTag::FF);

    auto read_slot = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, slot_offset, AvmMemoryTag::FF, AvmMemoryTag::FF);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_ia = read_src_value.val,
        .avm_main_ib = read_slot.val,
        .avm_main_ind_a = src_offset_is_indirect ? src_offset : 0,
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(src_offset_is_indirect)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(direct_src_offset),
        .avm_main_mem_idx_b = FF(slot_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_pc = pc, // No PC increment here since this is the same opcode as the rows created below
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
    });
    clk++;

    for (uint32_t i = 0; i < size; i++) {
        auto read_a = mem_trace_builder.read_and_load_from_memory(
            call_ptr, clk, IntermRegister::IA, direct_src_offset + i, AvmMemoryTag::FF, AvmMemoryTag::U0);

        Row row = Row{
            .avm_main_clk = clk,
            .avm_main_ia = read_a.val,
            .avm_main_ib = read_slot.val + i, // slot increments each time
            .avm_main_internal_return_ptr = internal_return_ptr,
            .avm_main_mem_idx_a = direct_src_offset + i,
            .avm_main_mem_op_a = 1,
            .avm_main_pc = pc,
            .avm_main_q_kernel_output_lookup = 1,
            .avm_main_r_in_tag = static_cast<uint32_t>(AvmMemoryTag::FF),
        };
        row.avm_main_sel_op_sstore = FF(1);
        kernel_trace_builder.op_sstore(clk, side_effect_counter, row.avm_main_ib, row.avm_main_ia);

        // Constrain gas cost
        gas_trace_builder.constrain_gas_lookup(clk, OpCode::SSTORE);

        main_trace.push_back(row);
        side_effect_counter++;
        clk++;
    }
    pc++;
}

/**
 * @brief Cast an element pointed by the address a_offset into type specified by dst_tag and
          store the result in address given by dst_offset.
 *
 * @param indirect A byte encoding information about indirect/direct memory access.
 * @param a_offset Offset of source memory cell.
 * @param dst_offset Offset of destination memory cell.
 * @param dst_tag Destination tag specifying the type the source value must be casted to.
 */
void AvmTraceBuilder::op_cast(uint8_t indirect, uint32_t a_offset, uint32_t dst_offset, AvmMemoryTag dst_tag)
{
    auto const clk = static_cast<uint32_t>(main_trace.size()) + 1;
    bool tag_match = true;
    uint32_t direct_a_offset = a_offset;
    uint32_t direct_dst_offset = dst_offset;

    bool indirect_a_flag = is_operand_indirect(indirect, 0);
    bool indirect_dst_flag = is_operand_indirect(indirect, 1);

    if (indirect_a_flag) {
        auto read_ind_a =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_A, a_offset);
        direct_a_offset = uint32_t(read_ind_a.val);
        tag_match = tag_match && read_ind_a.tag_match;
    }

    if (indirect_dst_flag) {
        auto read_ind_c =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_C, dst_offset);
        direct_dst_offset = uint32_t(read_ind_c.val);
        tag_match = tag_match && read_ind_c.tag_match;
    }

    // Reading from memory and loading into ia
    auto memEntry = mem_trace_builder.read_and_load_cast_opcode(call_ptr, clk, direct_a_offset, dst_tag);
    FF a = memEntry.val;

    // In case of a memory tag error, we do not perform the computation.
    // Therefore, we do not create any entry in ALU table and store the value 0 as
    // output (c) in memory.
    FF c = tag_match ? alu_trace_builder.op_cast(a, dst_tag, clk) : FF(0);

    // Write into memory value c from intermediate register ic.
    mem_trace_builder.write_into_memory(call_ptr, clk, IntermRegister::IC, direct_dst_offset, c, memEntry.tag, dst_tag);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::CAST);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_alu_in_tag = FF(static_cast<uint32_t>(dst_tag)),
        .avm_main_call_ptr = call_ptr,
        .avm_main_ia = a,
        .avm_main_ic = c,
        .avm_main_ind_a = indirect_a_flag ? FF(a_offset) : FF(0),
        .avm_main_ind_c = indirect_dst_flag ? FF(dst_offset) : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(indirect_a_flag)),
        .avm_main_ind_op_c = FF(static_cast<uint32_t>(indirect_dst_flag)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(direct_a_offset),
        .avm_main_mem_idx_c = FF(direct_dst_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_c = FF(1),
        .avm_main_pc = FF(pc++),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(memEntry.tag)),
        .avm_main_rwc = FF(1),
        .avm_main_sel_op_cast = FF(1),
        .avm_main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(dst_tag)),
    });
}
/**
 * @brief Integer division with direct or indirect memory access.
 *
 * @param indirect A byte encoding information about indirect/direct memory access.
 * @param a_offset An index in memory pointing to the first operand of the division.
 * @param b_offset An index in memory pointing to the second operand of the division.
 * @param dst_offset An index in memory pointing to the output of the division.
 * @param in_tag The instruction memory tag of the operands.
 */
void AvmTraceBuilder::op_div(
    uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    auto const res = resolve_ind_three(call_ptr, clk, indirect, a_offset, b_offset, dst_offset);
    bool tag_match = res.tag_match;

    // Reading from memory and loading into ia resp. ib.
    auto read_a = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, res.direct_a_offset, in_tag, in_tag);
    auto read_b = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, res.direct_b_offset, in_tag, in_tag);
    tag_match = read_a.tag_match && read_b.tag_match;

    // a / b = c
    FF a = read_a.val;
    FF b = read_b.val;

    // In case of a memory tag error, we do not perform the computation.
    // Therefore, we do not create any entry in ALU table and store the value 0 as
    // output (c) in memory.
    FF c;
    FF inv;
    FF error;

    if (!b.is_zero()) {
        // If b is not zero, we prove it is not by providing its inverse as well
        inv = b.invert();
        c = tag_match ? alu_trace_builder.op_div(a, b, in_tag, clk) : FF(0);
        error = 0;
    } else {
        inv = 1;
        c = 0;
        error = 1;
    }

    // Write into memory value c from intermediate register ic.
    mem_trace_builder.write_into_memory(call_ptr, clk, IntermRegister::IC, res.direct_c_offset, c, in_tag, in_tag);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::DIV);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_alu_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_call_ptr = call_ptr,
        .avm_main_ia = a,
        .avm_main_ib = b,
        .avm_main_ic = c,
        .avm_main_ind_a = res.indirect_flag_a ? FF(a_offset) : FF(0),
        .avm_main_ind_b = res.indirect_flag_b ? FF(b_offset) : FF(0),
        .avm_main_ind_c = res.indirect_flag_c ? FF(dst_offset) : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(res.indirect_flag_a)),
        .avm_main_ind_op_b = FF(static_cast<uint32_t>(res.indirect_flag_b)),
        .avm_main_ind_op_c = FF(static_cast<uint32_t>(res.indirect_flag_c)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_inv = tag_match ? inv : FF(1),
        .avm_main_mem_idx_a = FF(res.direct_a_offset),
        .avm_main_mem_idx_b = FF(res.direct_b_offset),
        .avm_main_mem_idx_c = FF(res.direct_c_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_mem_op_c = FF(1),
        .avm_main_op_err = tag_match ? error : FF(1),
        .avm_main_pc = FF(pc++),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .avm_main_rwc = FF(1),
        .avm_main_sel_op_div = FF(1),
        .avm_main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(in_tag)),
    });
}

/**
 * @brief CALLDATACOPY opcode with direct or indirect memory access, i.e.,
 *        direct: M[dst_offset:dst_offset+copy_size] = calldata[cd_offset:cd_offset+copy_size]
 *        indirect: M[M[dst_offset]:M[dst_offset]+copy_size] = calldata[cd_offset:cd_offset+copy_size]
 *        Simplified version with exclusively memory store operations and
 *        values from calldata passed by an array and loaded into
 *        intermediate registers.
 *        Assume that caller passes call_data_mem which is large enough so that
 *        no out-of-bound memory issues occur.
 *        TODO: error handling if dst_offset + copy_size > 2^32 which would lead to
 *              out-of-bound memory write. Similarly, if cd_offset + copy_size is larger
 *              than call_data_mem.size()
 *
 * @param indirect A byte encoding information about indirect/direct memory access.
 * @param cd_offset The starting index of the region in calldata to be copied.
 * @param copy_size The number of finite field elements to be copied into memory.
 * @param dst_offset The starting index of memory where calldata will be copied to.
 * @param call_data_mem The vector containing calldata.
 */
void AvmTraceBuilder::calldata_copy(
    uint8_t indirect, uint32_t cd_offset, uint32_t copy_size, uint32_t dst_offset, std::vector<FF> const& call_data_mem)
{
    // We parallelize storing memory operations in chunk of 3, i.e., 1 per intermediate register.
    // The variable pos is an index pointing to the first storing operation (pertaining to intermediate
    // register Ia) relative to cd_offset:
    // cd_offset + pos:       Ia memory store operation
    // cd_offset + pos + 1:   Ib memory store operation
    // cd_offset + pos + 2:   Ic memory store operation

    uint32_t pos = 0;
    uint32_t direct_dst_offset = dst_offset; // Will be overwritten in indirect mode.

    while (pos < copy_size) {
        FF ib(0);
        FF ic(0);
        uint32_t mem_op_b(0);
        uint32_t mem_op_c(0);
        uint32_t mem_idx_b(0);
        uint32_t mem_idx_c(0);
        uint32_t rwb(0);
        uint32_t rwc(0);
        auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

        FF ia = call_data_mem.at(cd_offset + pos);
        uint32_t mem_op_a(1);
        uint32_t rwa = 1;

        bool indirect_flag = false;
        bool tag_match = true;

        if (pos == 0 && is_operand_indirect(indirect, 0)) {
            indirect_flag = true;
            auto ind_read = mem_trace_builder.indirect_read_and_load_from_memory(
                call_ptr, clk, IndirectRegister::IND_A, dst_offset);
            direct_dst_offset = uint32_t(ind_read.val);
            tag_match = ind_read.tag_match;
        }

        uint32_t mem_idx_a = direct_dst_offset + pos;

        // Storing from Ia
        mem_trace_builder.write_into_memory(
            call_ptr, clk, IntermRegister::IA, mem_idx_a, ia, AvmMemoryTag::U0, AvmMemoryTag::FF);

        if (copy_size - pos > 1) {
            ib = call_data_mem.at(cd_offset + pos + 1);
            mem_op_b = 1;
            mem_idx_b = direct_dst_offset + pos + 1;
            rwb = 1;

            // Storing from Ib
            mem_trace_builder.write_into_memory(
                call_ptr, clk, IntermRegister::IB, mem_idx_b, ib, AvmMemoryTag::U0, AvmMemoryTag::FF);
        }

        if (copy_size - pos > 2) {
            ic = call_data_mem.at(cd_offset + pos + 2);
            mem_op_c = 1;
            mem_idx_c = direct_dst_offset + pos + 2;
            rwc = 1;

            // Storing from Ic
            mem_trace_builder.write_into_memory(
                call_ptr, clk, IntermRegister::IC, mem_idx_c, ic, AvmMemoryTag::U0, AvmMemoryTag::FF);
        }

        // Constrain gas cost on the first row
        if (pos == 0) {
            gas_trace_builder.constrain_gas_lookup(clk, OpCode::CALLDATACOPY);
        }

        main_trace.push_back(Row{
            .avm_main_clk = clk,
            .avm_main_call_ptr = call_ptr,
            .avm_main_ia = ia,
            .avm_main_ib = ib,
            .avm_main_ic = ic,
            .avm_main_ind_a = indirect_flag ? FF(dst_offset) : FF(0),
            .avm_main_ind_op_a = FF(static_cast<uint32_t>(indirect_flag)),
            .avm_main_internal_return_ptr = FF(internal_return_ptr),
            .avm_main_mem_idx_a = FF(mem_idx_a),
            .avm_main_mem_idx_b = FF(mem_idx_b),
            .avm_main_mem_idx_c = FF(mem_idx_c),
            .avm_main_mem_op_a = FF(mem_op_a),
            .avm_main_mem_op_activate_gas = FF(static_cast<uint32_t>(
                pos == 0)), // TODO: remove in the long term. This activate gas only for the first row.
            .avm_main_mem_op_b = FF(mem_op_b),
            .avm_main_mem_op_c = FF(mem_op_c),
            .avm_main_pc = FF(pc),
            .avm_main_rwa = FF(rwa),
            .avm_main_rwb = FF(rwb),
            .avm_main_rwc = FF(rwc),
            .avm_main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
            .avm_main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
        });

        if (copy_size - pos > 2) { // Guard to prevent overflow if copy_size is close to uint32_t maximum value.
            pos += 3;
        } else {
            pos = copy_size;
        }
    }

    pc++;
}

// Credit to SEAN for coming up with this revert opcode
std::vector<FF> AvmTraceBuilder::op_revert(uint8_t indirect, uint32_t ret_offset, uint32_t ret_size)
{
    return return_op(indirect, ret_offset, ret_size);
}

/**
 * @brief RETURN opcode with direct and indirect memory access, i.e.,
 *        direct:   return(M[ret_offset:ret_offset+ret_size])
 *        indirect: return(M[M[ret_offset]:M[ret_offset]+ret_size])
 *        Simplified version with exclusively memory load operations into
 *        intermediate registers and then values are copied to the returned vector.
 *        TODO: taking care of flagging this row as the last one? Special STOP flag?
 *        TODO: error handling if ret_offset + ret_size > 2^32 which would lead to
 *              out-of-bound memory read.
 *
 * @param indirect A byte encoding information about indirect/direct memory access.
 * @param ret_offset The starting index of the memory region to be returned.
 * @param ret_size The number of elements to be returned.
 * @return The returned memory region as a std::vector.
 */
std::vector<FF> AvmTraceBuilder::return_op(uint8_t indirect, uint32_t ret_offset, uint32_t ret_size)
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
    // In indirect mode, ret_offset is first resolved by the first indirect load.

    uint32_t pos = 0;
    std::vector<FF> returnMem;
    uint32_t direct_ret_offset = ret_offset; // Will be overwritten in indirect mode.

    while (pos < ret_size) {
        FF ib(0);
        FF ic(0);
        uint32_t mem_op_b(0);
        uint32_t mem_op_c(0);
        uint32_t mem_idx_b(0);
        uint32_t mem_idx_c(0);
        auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

        uint32_t mem_op_a(1);
        bool indirect_flag = false;
        bool tag_match = true;

        if (pos == 0 && is_operand_indirect(indirect, 0)) {
            indirect_flag = true;
            auto ind_read = mem_trace_builder.indirect_read_and_load_from_memory(
                call_ptr, clk, IndirectRegister::IND_A, ret_offset);
            direct_ret_offset = uint32_t(ind_read.val);
            tag_match = ind_read.tag_match;
        }

        uint32_t mem_idx_a = direct_ret_offset + pos;

        // Reading and loading to Ia
        auto read_a = mem_trace_builder.read_and_load_from_memory(
            call_ptr, clk, IntermRegister::IA, mem_idx_a, AvmMemoryTag::FF, AvmMemoryTag::FF);
        tag_match = tag_match && read_a.tag_match;

        FF ia = read_a.val;
        returnMem.push_back(ia);

        if (ret_size - pos > 1) {
            mem_op_b = 1;
            mem_idx_b = direct_ret_offset + pos + 1;

            // Reading and loading to Ib
            auto read_b = mem_trace_builder.read_and_load_from_memory(
                call_ptr, clk, IntermRegister::IB, mem_idx_b, AvmMemoryTag::FF, AvmMemoryTag::FF);
            tag_match = tag_match && read_b.tag_match;
            ib = read_b.val;
            returnMem.push_back(ib);
        }

        if (ret_size - pos > 2) {
            mem_op_c = 1;
            mem_idx_c = direct_ret_offset + pos + 2;

            // Reading and loading to Ic
            auto read_c = mem_trace_builder.read_and_load_from_memory(
                call_ptr, clk, IntermRegister::IC, mem_idx_c, AvmMemoryTag::FF, AvmMemoryTag::FF);
            tag_match = tag_match && read_c.tag_match;
            ic = read_c.val;
            returnMem.push_back(ic);
        }

        // Constrain gas cost on the first row
        if (pos == 0) {
            gas_trace_builder.constrain_gas_lookup(clk, OpCode::RETURN);
        }

        main_trace.push_back(Row{
            .avm_main_clk = clk,
            .avm_main_call_ptr = call_ptr,
            .avm_main_ia = ia,
            .avm_main_ib = ib,
            .avm_main_ic = ic,
            .avm_main_ind_a = indirect_flag ? FF(ret_offset) : FF(0),
            .avm_main_ind_op_a = FF(static_cast<uint32_t>(indirect_flag)),
            .avm_main_internal_return_ptr = FF(internal_return_ptr),
            .avm_main_mem_idx_a = FF(mem_idx_a),
            .avm_main_mem_idx_b = FF(mem_idx_b),
            .avm_main_mem_idx_c = FF(mem_idx_c),
            .avm_main_mem_op_a = FF(mem_op_a),
            .avm_main_mem_op_activate_gas = FF(static_cast<uint32_t>(
                pos == 0)), // TODO: remove in the long term. This activate gas only for the first row.
            .avm_main_mem_op_b = FF(mem_op_b),
            .avm_main_mem_op_c = FF(mem_op_c),
            .avm_main_pc = FF(pc),
            .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
            .avm_main_sel_halt = FF(1),
            .avm_main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
            .avm_main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
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
    auto clk = main_trace.size() + 1;

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_call_ptr = call_ptr,
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_pc = FF(pc),
        .avm_main_sel_halt = FF(1),
    });

    pc = UINT32_MAX; // This ensures that no subsequent opcode will be executed.
}

void AvmTraceBuilder::execute_gasleft(OpCode opcode, uint8_t indirect, uint32_t dst_offset)
{
    assert(opcode == OpCode::L2GASLEFT || opcode == OpCode::DAGASLEFT);

    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;
    bool tag_match = true;

    uint32_t direct_dst_offset = dst_offset;

    bool indirect_dst_flag = is_operand_indirect(indirect, 0);

    if (indirect_dst_flag) {
        auto read_ind_dst =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_A, dst_offset);
        direct_dst_offset = uint32_t(read_ind_dst.val);
        tag_match = tag_match && read_ind_dst.tag_match;
    }

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, opcode);

    uint32_t gas_remaining = 0;

    if (opcode == OpCode::L2GASLEFT) {
        gas_remaining = gas_trace_builder.get_l2_gas_left();
    } else {
        gas_remaining = gas_trace_builder.get_da_gas_left();
    }

    // Write into memory from intermediate register ia.
    mem_trace_builder.write_into_memory(call_ptr,
                                        clk,
                                        IntermRegister::IA,
                                        direct_dst_offset,
                                        gas_remaining,
                                        AvmMemoryTag::U0,
                                        AvmMemoryTag::FF); // TODO: probably will be U32 in final version

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_call_ptr = call_ptr,
        .avm_main_ia = gas_remaining,
        .avm_main_ind_a = indirect_dst_flag ? FF(dst_offset) : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(indirect_dst_flag)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(direct_dst_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_pc = FF(pc++),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U0)),
        .avm_main_rwa = FF(1),
        .avm_main_sel_op_dagasleft = (opcode == OpCode::DAGASLEFT) ? FF(1) : FF(0),
        .avm_main_sel_op_l2gasleft = (opcode == OpCode::L2GASLEFT) ? FF(1) : FF(0),
        .avm_main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)), // TODO: probably will be U32 in final version
                                                                          // Should the circuit (pil) constrain U32?
    });
}

void AvmTraceBuilder::op_l2gasleft(uint8_t indirect, uint32_t dst_offset)
{
    execute_gasleft(OpCode::L2GASLEFT, indirect, dst_offset);
}

void AvmTraceBuilder::op_dagasleft(uint8_t indirect, uint32_t dst_offset)
{
    execute_gasleft(OpCode::DAGASLEFT, indirect, dst_offset);
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
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::JUMP);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_call_ptr = call_ptr,
        .avm_main_ia = FF(jmp_dest),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_pc = FF(pc),
        .avm_main_sel_jump = FF(1),
    });

    // Adjust parameters for the next row
    pc = jmp_dest;
}

/**
 * @brief JUMPI OPCODE
 *        Jumps to a new `jmp_dest` if M[cond_offset] > 0
 *        This function sets the next program counter to the provided `jmp_dest` if condition > 0.
 *        Otherwise, program counter is incremented.
 *
 * @param indirect A byte encoding information about indirect/direct memory access.
 * @param jmp_dest The destination to jump to
 * @param cond_offset Offset of the condition
 */
void AvmTraceBuilder::jumpi(uint8_t indirect, uint32_t jmp_dest, uint32_t cond_offset)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    bool tag_match = true;
    uint32_t direct_cond_offset = cond_offset;

    bool indirect_cond_flag = is_operand_indirect(indirect, 0);

    if (indirect_cond_flag) {
        auto read_ind_d =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_D, cond_offset);
        direct_cond_offset = uint32_t(read_ind_d.val);
        tag_match = tag_match && read_ind_d.tag_match;
    }

    // Specific JUMPI loading of conditional value into intermediate register id without any tag constraint.
    auto read_d = mem_trace_builder.read_and_load_jumpi_opcode(call_ptr, clk, direct_cond_offset);

    const bool id_zero = read_d.val == 0;
    FF const inv = !id_zero ? read_d.val.invert() : 1;
    uint32_t next_pc = !id_zero ? jmp_dest : pc + 1;

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::JUMPI);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_call_ptr = call_ptr,
        .avm_main_ia = FF(next_pc),
        .avm_main_id = read_d.val,
        .avm_main_id_zero = static_cast<uint32_t>(id_zero),
        .avm_main_ind_d = indirect_cond_flag ? cond_offset : 0,
        .avm_main_ind_op_d = static_cast<uint32_t>(indirect_cond_flag),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_inv = inv,
        .avm_main_mem_idx_d = direct_cond_offset,
        .avm_main_mem_op_d = 1,
        .avm_main_pc = FF(pc),
        .avm_main_r_in_tag = static_cast<uint32_t>(read_d.tag),
        .avm_main_sel_jumpi = FF(1),
        .avm_main_tag_err = static_cast<uint32_t>(!tag_match),
        .avm_main_w_in_tag = static_cast<uint32_t>(read_d.tag),
    });

    // Adjust parameters for the next row
    pc = next_pc;
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
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    // We store the next instruction as the return location
    mem_trace_builder.write_into_memory(INTERNAL_CALL_SPACE_ID,
                                        clk,
                                        IntermRegister::IB,
                                        internal_return_ptr,
                                        FF(pc + 1),
                                        AvmMemoryTag::U0,
                                        AvmMemoryTag::U32);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::INTERNALCALL);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_call_ptr = call_ptr,
        .avm_main_ia = FF(jmp_dest),
        .avm_main_ib = FF(pc + 1),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_b = FF(internal_return_ptr),
        .avm_main_mem_op_b = FF(1),
        .avm_main_pc = FF(pc),
        .avm_main_rwb = FF(1),
        .avm_main_sel_internal_call = FF(1),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U32)),
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
 *  TODO(https://github.com/AztecProtocol/aztec-packages/issues/3740): This function MUST come after a call
 * instruction.
 */
void AvmTraceBuilder::internal_return()
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    // Internal return pointer is decremented
    // We want to load the value pointed by the internal pointer
    auto read_a = mem_trace_builder.read_and_load_from_memory(
        INTERNAL_CALL_SPACE_ID, clk, IntermRegister::IA, internal_return_ptr - 1, AvmMemoryTag::U32, AvmMemoryTag::U0);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::INTERNALRETURN);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_call_ptr = call_ptr,
        .avm_main_ia = read_a.val,
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(internal_return_ptr - 1),
        .avm_main_mem_op_a = FF(1),
        .avm_main_pc = pc,
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U32)),
        .avm_main_rwa = FF(0),
        .avm_main_sel_internal_return = FF(1),
        .avm_main_tag_err = FF(static_cast<uint32_t>(!read_a.tag_match)),
    });

    pc = uint32_t(read_a.val);
    internal_return_ptr--;
}

// TODO(ilyas: #6383): Temporary way to bulk write slices
void AvmTraceBuilder::write_slice_to_memory(uint8_t space_id,
                                            uint32_t clk,
                                            uint32_t dst_offset,
                                            AvmMemoryTag r_tag,
                                            AvmMemoryTag w_tag,
                                            FF internal_return_ptr,
                                            std::vector<FF> const& slice)
{
    // We have 4 registers that we are able to use to write to memory within a single main trace row
    auto register_order = std::array{ IntermRegister::IA, IntermRegister::IB, IntermRegister::IC, IntermRegister::ID };
    // If the slice size isnt a multiple of 4, we still need an extra row to write the remainder
    uint32_t const num_main_rows =
        static_cast<uint32_t>(slice.size()) / 4 + static_cast<uint32_t>(slice.size() % 4 != 0);
    for (uint32_t i = 0; i < num_main_rows; i++) {
        Row main_row{
            .avm_main_clk = clk + i,
            .avm_main_internal_return_ptr = FF(internal_return_ptr),
            .avm_main_pc = FF(pc),
            .avm_main_r_in_tag = FF(static_cast<uint32_t>(r_tag)),
            .avm_main_w_in_tag = FF(static_cast<uint32_t>(w_tag)),
        };
        // Write 4 values to memory in each_row
        for (uint32_t j = 0; j < 4; j++) {
            auto offset = i * 4 + j;
            // If we exceed the slice size, we break
            if (offset >= slice.size()) {
                break;
            }
            mem_trace_builder.write_into_memory(
                space_id, clk + i, register_order[j], dst_offset + offset, slice.at(offset), r_tag, w_tag);
            // This looks a bit gross, but it is fine for now.
            if (j == 0) {
                main_row.avm_main_ia = slice.at(offset);
                main_row.avm_main_mem_idx_a = FF(dst_offset + offset);
                main_row.avm_main_mem_op_a = FF(1);
                main_row.avm_main_rwa = FF(1);
            } else if (j == 1) {
                main_row.avm_main_ib = slice.at(offset);
                main_row.avm_main_mem_idx_b = FF(dst_offset + offset);
                main_row.avm_main_mem_op_b = FF(1);
                main_row.avm_main_rwb = FF(1);
            } else if (j == 2) {
                main_row.avm_main_ic = slice.at(offset);
                main_row.avm_main_mem_idx_c = FF(dst_offset + offset);
                main_row.avm_main_mem_op_c = FF(1);
                main_row.avm_main_rwc = FF(1);
            } else {
                main_row.avm_main_id = slice.at(offset);
                main_row.avm_main_mem_idx_d = FF(dst_offset + offset);
                main_row.avm_main_mem_op_d = FF(1);
                main_row.avm_main_rwd = FF(1);
            }
        }
        main_trace.emplace_back(main_row);
    }
}

template <typename MEM, size_t T> std::array<MEM, T> vec_to_arr(std::vector<MEM> const& vec)
{
    std::array<MEM, T> arr;
    ASSERT(T == vec.size());
    for (size_t i = 0; i < T; i++) {
        arr[i] = vec[i];
    }
    return arr;
}
// TODO(ilyas: #6383): Temporary way to bulk read slices
template <typename MEM>
uint32_t AvmTraceBuilder::read_slice_to_memory(uint8_t space_id,
                                               uint32_t clk,
                                               uint32_t src_offset,
                                               AvmMemoryTag r_tag,
                                               AvmMemoryTag w_tag,
                                               FF internal_return_ptr,
                                               size_t slice_len,
                                               std::vector<MEM>& slice)
{
    // We have 4 registers that we are able to use to read from memory within a single main trace row
    auto register_order = std::array{ IntermRegister::IA, IntermRegister::IB, IntermRegister::IC, IntermRegister::ID };
    // If the slice size isnt a multiple of 4, we still need an extra row to write the remainder
    uint32_t const num_main_rows = static_cast<uint32_t>(slice_len) / 4 + static_cast<uint32_t>(slice_len % 4 != 0);
    for (uint32_t i = 0; i < num_main_rows; i++) {
        Row main_row{
            .avm_main_clk = clk + i,
            .avm_main_internal_return_ptr = FF(internal_return_ptr),
            .avm_main_pc = FF(pc),
            .avm_main_r_in_tag = FF(static_cast<uint32_t>(r_tag)),
            .avm_main_w_in_tag = FF(static_cast<uint32_t>(w_tag)),
        };
        // Write 4 values to memory in each_row
        for (uint32_t j = 0; j < 4; j++) {
            auto offset = i * 4 + j;
            // If we exceed the slice size, we break
            if (offset >= slice_len) {
                break;
            }
            auto mem_read = mem_trace_builder.read_and_load_from_memory(
                space_id, clk + i, register_order[j], src_offset + offset, r_tag, w_tag);
            slice.emplace_back(MEM(mem_read.val));
            // This looks a bit gross, but it is fine for now.
            if (j == 0) {
                main_row.avm_main_ia = slice.at(offset);
                main_row.avm_main_mem_idx_a = FF(src_offset + offset);
                main_row.avm_main_mem_op_a = FF(1);
                main_row.avm_main_tag_err = FF(static_cast<uint32_t>(!mem_read.tag_match));
            } else if (j == 1) {
                main_row.avm_main_ib = slice.at(offset);
                main_row.avm_main_mem_idx_b = FF(src_offset + offset);
                main_row.avm_main_mem_op_b = FF(1);
                main_row.avm_main_tag_err = FF(static_cast<uint32_t>(!mem_read.tag_match));
            } else if (j == 2) {
                main_row.avm_main_ic = slice.at(offset);
                main_row.avm_main_mem_idx_c = FF(src_offset + offset);
                main_row.avm_main_mem_op_c = FF(1);
                main_row.avm_main_tag_err = FF(static_cast<uint32_t>(!mem_read.tag_match));
            } else {
                main_row.avm_main_id = slice.at(offset);
                main_row.avm_main_mem_idx_d = FF(src_offset + offset);
                main_row.avm_main_mem_op_d = FF(1);
                main_row.avm_main_tag_err = FF(static_cast<uint32_t>(!mem_read.tag_match));
            }
        }
        main_trace.emplace_back(main_row);
    }
    return num_main_rows;
}

/**
 * @brief External Call with direct or indirect memory access.
 *
 * TODO: Use the indirect later to support all the indirect accesses
 * NOTE: we do not constrain this here as it's behaviour will change fully once we have a full enqueued function
 * call in one vm circuit
 * @param indirect byte encoding information about indirect/direct memory access.
 * @param gas_offset An index in memory pointing to the first of the gas value tuple (l2Gas, daGas)
 * @param addr_offset An index in memory pointing to the target contract address
 * @param args_offset An index in memory pointing to the first value of the input array for the external call
 * @param args_size The number of values in the input array for the external call
 * @param ret_offset An index in memory pointing to where the first value of the external calls return value should be
 * stored.
 * @param ret_size The number of values in the return array
 * @param success_offset An index in memory pointing to where the success flag (U8) of the external call should be
 * stored
 * @param function_selector_offset An index in memory pointing to the function selector of the external call (TEMP)
 */
void AvmTraceBuilder::op_call([[maybe_unused]] uint8_t indirect,
                              [[maybe_unused]] uint32_t gas_offset,
                              [[maybe_unused]] uint32_t addr_offset,
                              [[maybe_unused]] uint32_t args_offset,
                              [[maybe_unused]] uint32_t args_size,
                              [[maybe_unused]] uint32_t ret_offset,
                              [[maybe_unused]] uint32_t ret_size,
                              [[maybe_unused]] uint32_t success_offset,
                              [[maybe_unused]] uint32_t function_selector_offset)
{
    // pc++;
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;
    const ExternalCallHint& hint = execution_hints.externalcall_hints.at(external_call_counter);
    // We can load up to 4 things per row
    auto register_order = std::array{ IntermRegister::IA, IntermRegister::IB, IntermRegister::IC, IntermRegister::ID };
    // Constrain gas cost
    gas_trace_builder.constrain_gas_for_external_call(
        clk, static_cast<uint32_t>(hint.l2_gas_used), static_cast<uint32_t>(hint.da_gas_used));
    // Indirect is ZEROTH, SECOND and FOURTH bit  COME BACK TO MAKING THIS ALL SUPPORTED
    auto read_ind_gas_offset =
        mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_A, gas_offset);
    auto read_ind_args_offset =
        mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_C, args_offset);

    std::vector<uint32_t> first_row_load = {
        uint32_t(read_ind_gas_offset.val),
        addr_offset,
        uint32_t(read_ind_args_offset.val),
    };
    std::vector<FF> first_row_values = {};
    for (uint32_t j = 0; j < first_row_load.size(); j++) {
        // We just read and load to set up the constraints, we dont actually use these values for now.
        // info("Register order ", register_order[j]);
        auto mem_read = mem_trace_builder.read_and_load_from_memory(
            call_ptr, clk, register_order[j], first_row_load[j], AvmMemoryTag::FF, AvmMemoryTag::U0);
        first_row_values.emplace_back(mem_read.val);
    }

    // We read the input and output addresses in one row as they should contain FF elements
    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_ia = first_row_values[0], /* gas_offset */
        .avm_main_ib = first_row_values[1], /* addr_offset */
        .avm_main_ic = first_row_values[2], /* args_offset */
        .avm_main_ind_a = gas_offset,
        .avm_main_ind_c = args_offset,
        .avm_main_ind_op_a = FF(1),
        .avm_main_ind_op_c = FF(1),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = read_ind_gas_offset.val,
        .avm_main_mem_idx_b = addr_offset,
        .avm_main_mem_idx_c = read_ind_args_offset.val,
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_mem_op_c = FF(1),
        .avm_main_pc = FF(pc++),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
        .avm_main_sel_external_call = FF(1),
    });
    clk++;
    // Read the rest on a separate line, remember that the 4th operand is indirect
    auto read_ind_ret_offset =
        mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_A, ret_offset);
    // We just read and load to set up the constraints, we dont actually use these values for now.
    auto mem_read_ret = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, uint32_t(read_ind_ret_offset.val), AvmMemoryTag::FF, AvmMemoryTag::U0);
    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_ia = mem_read_ret.val, /* ret_offset */
        .avm_main_ind_a = ret_offset,
        .avm_main_ind_op_a = FF(1),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = read_ind_ret_offset.val,
        .avm_main_mem_op_a = FF(1),
        .avm_main_pc = FF(pc),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
    });
    clk++;
    auto mem_read_success = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, success_offset, AvmMemoryTag::U32, AvmMemoryTag::U0);
    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_ia = mem_read_success.val, /* success_offset */
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(success_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_pc = FF(pc),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U32)),
    });
    clk++;
    write_slice_to_memory(call_ptr,
                          clk,
                          uint32_t(read_ind_ret_offset.val),
                          AvmMemoryTag::U0,
                          AvmMemoryTag::FF,
                          internal_return_ptr,
                          hint.return_data);
    clk++;
    write_slice_to_memory(
        call_ptr, clk, success_offset, AvmMemoryTag::U0, AvmMemoryTag::U8, internal_return_ptr, { hint.success });
    external_call_counter++;
}

void AvmTraceBuilder::op_get_contract_instance(uint8_t indirect, uint32_t address_offset, uint32_t dst_offset)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;
    bool tag_match = true;
    uint32_t direct_address_offset = address_offset;
    uint32_t direct_dst_offset = dst_offset;

    bool indirect_address_flag = is_operand_indirect(indirect, 0);
    bool indirect_dst_flag = is_operand_indirect(indirect, 1);

    if (indirect_address_flag) {
        auto read_ind_address = mem_trace_builder.indirect_read_and_load_from_memory(
            call_ptr, clk, IndirectRegister::IND_A, address_offset);
        direct_address_offset = uint32_t(read_ind_address.val);
        tag_match = tag_match && read_ind_address.tag_match;
    }

    if (indirect_dst_flag) {
        auto read_ind_dst =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_B, dst_offset);
        direct_dst_offset = uint32_t(read_ind_dst.val);
        tag_match = tag_match && read_ind_dst.tag_match;
    }

    auto read_address = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, direct_address_offset, AvmMemoryTag::FF, AvmMemoryTag::U0);
    auto read_dst = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, direct_dst_offset, AvmMemoryTag::FF, AvmMemoryTag::U0);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::GETCONTRACTINSTANCE);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_ia = read_address.val,
        .avm_main_ib = read_dst.val,
        .avm_main_ind_a = indirect_address_flag ? address_offset : 0,
        .avm_main_ind_b = indirect_dst_flag ? dst_offset : 0,
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(indirect_address_flag)),
        .avm_main_ind_op_b = FF(static_cast<uint32_t>(indirect_dst_flag)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(direct_address_offset),
        .avm_main_mem_idx_b = FF(direct_dst_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_activate_gas = FF(1), // TODO: remove in the long term
        .avm_main_mem_op_b = FF(1),
        .avm_main_pc = FF(pc++),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
        .avm_main_sel_op_get_contract_instance = FF(1),
    });
    clk++;
    // Read the contract instance
    ContractInstanceHint contract_instance = execution_hints.contract_instance_hints.at(read_address.val);

    // NOTE: we don't write the first entry (the contract instance's address/key) to memory
    std::vector<FF> contract_instance_vec = { contract_instance.instance_found_in_address,
                                              contract_instance.salt,
                                              contract_instance.deployer_addr,
                                              contract_instance.contract_class_id,
                                              contract_instance.initialisation_hash,
                                              contract_instance.public_key_hash };
    write_slice_to_memory(call_ptr,
                          clk,
                          direct_dst_offset,
                          AvmMemoryTag::U0,
                          AvmMemoryTag::FF,
                          internal_return_ptr,
                          contract_instance_vec);
}

/**
 * @brief To_Radix_LE with direct or indirect memory access.
 *
 * @param indirect A byte encoding information about indirect/direct memory access.
 * @param src_offset An index in memory pointing to the input of the To_Radix_LE conversion.
 * @param dst_offset An index in memory pointing to the output of the To_Radix_LE conversion.
 * @param radix A strict upper bound of each converted limb, i.e., 0 <= limb < radix.
 * @param num_limbs The number of limbs to the value into.
 */
void AvmTraceBuilder::op_to_radix_le(
    uint8_t indirect, uint32_t src_offset, uint32_t dst_offset, uint32_t radix, uint32_t num_limbs)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;
    bool tag_match = true;
    uint32_t direct_src_offset = src_offset;
    uint32_t direct_dst_offset = dst_offset;

    bool indirect_src_flag = is_operand_indirect(indirect, 0);
    bool indirect_dst_flag = is_operand_indirect(indirect, 1);

    if (indirect_src_flag) {
        auto read_ind_src =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_A, src_offset);
        direct_src_offset = uint32_t(read_ind_src.val);
        tag_match = tag_match && read_ind_src.tag_match;
    }

    if (indirect_dst_flag) {
        auto read_ind_dst =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_B, dst_offset);
        direct_dst_offset = uint32_t(read_ind_dst.val);
        tag_match = tag_match && read_ind_dst.tag_match;
    }

    auto read_src = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, direct_src_offset, AvmMemoryTag::FF, AvmMemoryTag::U8);
    // Read in the memory address of where the first limb should be stored (the read_tag must be U32 and write tag
    // U8)
    auto read_dst = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, direct_dst_offset, AvmMemoryTag::FF, AvmMemoryTag::U8);

    FF input = read_src.val;
    FF dst_addr = read_dst.val;

    // In case of a memory tag error, we do not perform the computation.
    // Therefore, we do not create any entry in gadget table and return a vector of 0
    std::vector<uint8_t> res = tag_match ? conversion_trace_builder.op_to_radix_le(input, radix, num_limbs, clk)
                                         : std::vector<uint8_t>(num_limbs, 0);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::TORADIXLE);

    // This is the row that contains the selector to trigger the sel_op_radix_le
    // In this row, we read the input value and the destination address into register A and B respectively
    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_call_ptr = call_ptr,
        .avm_main_ia = input,
        .avm_main_ib = dst_addr,
        .avm_main_ic = radix,
        .avm_main_id = num_limbs,
        .avm_main_ind_a = indirect_src_flag ? src_offset : 0,
        .avm_main_ind_b = indirect_dst_flag ? dst_offset : 0,
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(indirect_src_flag)),
        .avm_main_ind_op_b = FF(static_cast<uint32_t>(indirect_dst_flag)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(direct_src_offset),
        .avm_main_mem_idx_b = FF(direct_dst_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_pc = FF(pc++),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
        .avm_main_sel_op_radix_le = FF(1),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U8)),
    });
    // Increment the clock so we dont write at the same clock cycle
    // Instead we temporarily encode the writes into the subsequent rows of the main trace
    clk++;

    // MemTrace, write into memory value b from intermediate register ib.
    std::vector<FF> ff_res = {};
    ff_res.reserve(res.size());
    for (auto const& limb : res) {
        ff_res.emplace_back(limb);
    }
    write_slice_to_memory(
        call_ptr, clk, direct_dst_offset, AvmMemoryTag::FF, AvmMemoryTag::U8, FF(internal_return_ptr), ff_res);
}

/**
 * @brief SHA256 Compression with direct or indirect memory access.
 *
 * @param indirect byte encoding information about indirect/direct memory access.
 * @param h_init_offset An index in memory pointing to the first U32 value of the state array to be used in the next
 * instance of sha256 compression.
 * @param input_offset An index in memory pointing to the first U32 value of the input array to be used in the next
 * instance of sha256 compression.
 * @param output_offset An index in memory pointing to where the first U32 value of the output array should be stored.
 */
void AvmTraceBuilder::op_sha256_compression(uint8_t indirect,
                                            uint32_t output_offset,
                                            uint32_t h_init_offset,
                                            uint32_t input_offset)
{
    // The clk plays a crucial role in this function as we attempt to write across multiple lines in the main trace.
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    // Resolve the indirect flags, the results of this function are used to determine the memory offsets
    // that point to the starting memory addresses for the input and output values.
    // Note::This function will add memory reads at clk in the mem_trace_builder
    auto const res = resolve_ind_three(call_ptr, clk, indirect, h_init_offset, input_offset, output_offset);

    auto read_a = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, res.direct_a_offset, AvmMemoryTag::U32, AvmMemoryTag::U32);
    auto read_b = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, res.direct_b_offset, AvmMemoryTag::U32, AvmMemoryTag::U32);
    auto read_c = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IC, res.direct_c_offset, AvmMemoryTag::U32, AvmMemoryTag::U32);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::SHA256COMPRESSION);

    // Since the above adds mem_reads in the mem_trace_builder at clk, we need to follow up resolving the reads in
    // the main trace at the same clk cycle to preserve the cross-table permutation
    //
    // TODO<#6383>: We put the first value of each of the input, output (which is 0 at this point) and h_init arrays
    // into the main trace at the intermediate registers simply for the permutation check, in the future this will
    // change.
    // Note: we could avoid output being zero if we loaded the input and state beforehand (with a new function that
    // did not lay down constraints), but this is a simplification
    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_ia = read_a.val, // First element of output (trivially 0)
        .avm_main_ib = read_b.val, // First element of state
        .avm_main_ic = read_c.val, // First element of input
        .avm_main_ind_a = res.indirect_flag_a ? FF(h_init_offset) : FF(0),
        .avm_main_ind_b = res.indirect_flag_b ? FF(input_offset) : FF(0),
        .avm_main_ind_c = res.indirect_flag_a ? FF(output_offset) : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(res.indirect_flag_a)),
        .avm_main_ind_op_b = FF(static_cast<uint32_t>(res.indirect_flag_b)),
        .avm_main_ind_op_c = FF(static_cast<uint32_t>(res.indirect_flag_c)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(res.direct_a_offset),
        .avm_main_mem_idx_b = FF(res.direct_b_offset),
        .avm_main_mem_idx_c = FF(res.direct_c_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_mem_op_c = FF(1),
        .avm_main_pc = FF(pc++),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U32)),
        .avm_main_sel_op_sha256 = FF(1),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U32)),
    });
    // We store the current clk this main trace row occurred so that we can line up the sha256 gadget operation at
    // the same clk later.
    auto sha_op_clk = clk;
    // We need to increment the clk
    clk++;
    // State array input is fixed to 256 bits
    std::vector<uint32_t> h_init_vec;
    // Input for hash is expanded to 512 bits
    std::vector<uint32_t> input_vec;
    // Read results are written to h_init array.
    read_slice_to_memory<uint32_t>(call_ptr,
                                   clk,
                                   res.direct_a_offset,
                                   AvmMemoryTag::U32,
                                   AvmMemoryTag::U32,
                                   FF(internal_return_ptr),
                                   8,
                                   h_init_vec);

    // Increment the clock by 2 since (8 reads / 4 reads per row = 2)
    clk += 2;
    // Read results are written to input array
    read_slice_to_memory<uint32_t>(call_ptr,
                                   clk,
                                   res.direct_b_offset,
                                   AvmMemoryTag::U32,
                                   AvmMemoryTag::U32,
                                   FF(internal_return_ptr),
                                   16,
                                   input_vec);
    // Increment the clock by 4 since (16 / 4 = 4)
    clk += 4;

    // Now that we have read all the values, we can perform the operation to get the resulting witness.
    // Note: We use the sha_op_clk to ensure that the sha256 operation is performed at the same clock cycle as the
    // main trace that has the selector
    std::array<uint32_t, 8> h_init = vec_to_arr<uint32_t, 8>(h_init_vec);
    std::array<uint32_t, 16> input = vec_to_arr<uint32_t, 16>(input_vec);

    std::array<uint32_t, 8> result = sha256_trace_builder.sha256_compression(h_init, input, sha_op_clk);
    // We convert the results to field elements here
    std::vector<FF> ff_result;
    for (uint32_t i = 0; i < 8; i++) {
        ff_result.emplace_back(result[i]);
    }

    // Write the result to memory after
    write_slice_to_memory(
        call_ptr, clk, res.direct_c_offset, AvmMemoryTag::U32, AvmMemoryTag::U32, FF(internal_return_ptr), ff_result);
}

/**
 * @brief SHA256 Hash with direct or indirect memory access.
 * This function is temporary until we have transitioned to sha256Compression
 * @param indirect byte encoding information about indirect/direct memory access.
 * @param output_offset An index in memory pointing to where the first U32 value of the output array should be stored.
 * @param input_offset An index in memory pointing to the first U8 value of the state array to be used in the next
 * instance of sha256.
 * @param input_size_offset An index in memory pointing to the U32 value of the input size.
 */
void AvmTraceBuilder::op_sha256(uint8_t indirect,
                                uint32_t output_offset,
                                uint32_t input_offset,
                                uint32_t input_size_offset)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;
    bool tag_match = true;
    uint32_t direct_src_offset = input_offset;
    uint32_t direct_dst_offset = output_offset;

    bool indirect_src_flag = is_operand_indirect(indirect, 1);
    bool indirect_dst_flag = is_operand_indirect(indirect, 0);

    if (indirect_src_flag) {
        auto read_ind_src =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_A, input_offset);
        direct_src_offset = uint32_t(read_ind_src.val);
        tag_match = tag_match && read_ind_src.tag_match;
    }

    if (indirect_dst_flag) {
        auto read_ind_dst =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_C, output_offset);
        direct_dst_offset = uint32_t(read_ind_dst.val);
        tag_match = tag_match && read_ind_dst.tag_match;
    }
    // Note we load the input and output onto one line in the main trace and the length on the next line
    // We do this so we can load two different AvmMemoryTags (u8 for the I/O and u32 for the length)
    auto input_read = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, direct_src_offset, AvmMemoryTag::U8, AvmMemoryTag::U8);
    auto output_read = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IC, direct_dst_offset, AvmMemoryTag::U8, AvmMemoryTag::U8);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::SHA256);

    // Store the clock time that we will use to line up the gadget later
    auto sha256_op_clk = clk;
    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_ia = input_read.val,  // First element of input
        .avm_main_ic = output_read.val, // First element of output
        .avm_main_ind_a = indirect_src_flag ? FF(input_offset) : FF(0),
        .avm_main_ind_c = indirect_dst_flag ? FF(output_offset) : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(indirect_src_flag)),
        .avm_main_ind_op_c = FF(static_cast<uint32_t>(indirect_dst_flag)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(direct_src_offset), // input
        .avm_main_mem_idx_c = FF(direct_dst_offset), // output
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_c = FF(1),
        .avm_main_pc = FF(pc++),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U8)),
        .avm_main_sel_op_sha256 = FF(1),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U8)),
    });
    clk++;
    auto input_length_read = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, input_size_offset, AvmMemoryTag::U32, AvmMemoryTag::U32);
    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_ib = input_length_read.val, // Message Length
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_b = FF(input_size_offset), // length
        .avm_main_mem_op_b = FF(1),
        .avm_main_pc = FF(pc),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U32)),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U32)),
    });
    clk++;

    std::vector<uint8_t> input;
    input.reserve(uint32_t(input_length_read.val));

    // We unroll this loop because the function typically expects arrays and for this temporary sha256 function we
    // have a dynamic amount of input so we will use a vector.
    auto register_order = std::array{ IntermRegister::IA, IntermRegister::IB, IntermRegister::IC, IntermRegister::ID };
    // If the slice size isnt a multiple of 4, we still need an extra row to write the remainder
    uint32_t const num_main_rows = static_cast<uint32_t>(input_length_read.val) / 4 +
                                   static_cast<uint32_t>(uint32_t(input_length_read.val) % 4 != 0);
    for (uint32_t i = 0; i < num_main_rows; i++) {
        Row main_row{
            .avm_main_clk = clk + i,
            .avm_main_internal_return_ptr = FF(internal_return_ptr),
            .avm_main_pc = FF(pc),
            .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U8)),
            .avm_main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U8)),
        };
        // Write 4 values to memory in each_row
        for (uint32_t j = 0; j < 4; j++) {
            auto offset = i * 4 + j;
            // If we exceed the slice size, we break
            if (offset >= uint32_t(input_length_read.val)) {
                break;
            }
            auto mem_read = mem_trace_builder.read_and_load_from_memory(
                call_ptr, clk + i, register_order[j], direct_src_offset + offset, AvmMemoryTag::U8, AvmMemoryTag::U8);
            input.emplace_back(uint8_t(mem_read.val));
            // This looks a bit gross, but it is fine for now.
            if (j == 0) {
                main_row.avm_main_ia = input.at(offset);
                main_row.avm_main_mem_idx_a = FF(direct_src_offset + offset);
                main_row.avm_main_mem_op_a = FF(1);
                main_row.avm_main_tag_err = FF(static_cast<uint32_t>(!mem_read.tag_match));
            } else if (j == 1) {
                main_row.avm_main_ib = input.at(offset);
                main_row.avm_main_mem_idx_b = FF(direct_src_offset + offset);
                main_row.avm_main_mem_op_b = FF(1);
                main_row.avm_main_tag_err = FF(static_cast<uint32_t>(!mem_read.tag_match));
            } else if (j == 2) {
                main_row.avm_main_ic = input.at(offset);
                main_row.avm_main_mem_idx_c = FF(direct_src_offset + offset);
                main_row.avm_main_mem_op_c = FF(1);
                main_row.avm_main_tag_err = FF(static_cast<uint32_t>(!mem_read.tag_match));
            } else {
                main_row.avm_main_id = input.at(offset);
                main_row.avm_main_mem_idx_d = FF(direct_src_offset + offset);
                main_row.avm_main_mem_op_d = FF(1);
                main_row.avm_main_tag_err = FF(static_cast<uint32_t>(!mem_read.tag_match));
            }
        }
        main_trace.emplace_back(main_row);
    }

    clk += num_main_rows;

    std::array<uint8_t, 32> result = sha256_trace_builder.sha256(input, sha256_op_clk);
    // We convert the results to field elements here
    std::vector<FF> ff_result;
    for (uint32_t i = 0; i < 32; i++) {
        ff_result.emplace_back(result[i]);
    }
    // Write the result to memory after
    write_slice_to_memory(
        call_ptr, clk, direct_dst_offset, AvmMemoryTag::U8, AvmMemoryTag::U8, FF(internal_return_ptr), ff_result);
}

/**
 * @brief Poseidon2 Permutation with direct or indirect memory access.
 *
 * @param indirect byte encoding information about indirect/direct memory access.
 * @param input_offset An index in memory pointing to the first Field value of the input array to be used in the next
 * instance of poseidon2 permutation.
 * @param output_offset An index in memory pointing to where the first Field value of the output array should be stored.
 */
void AvmTraceBuilder::op_poseidon2_permutation(uint8_t indirect, uint32_t input_offset, uint32_t output_offset)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    // Resolve the indirect flags, the results of this function are used to determine the memory offsets
    // that point to the starting memory addresses for the input, output and h_init values
    // Note::This function will add memory reads at clk in the mem_trace_builder
    bool tag_match = true;
    uint32_t direct_src_offset = input_offset;
    uint32_t direct_dst_offset = output_offset;

    bool indirect_src_flag = is_operand_indirect(indirect, 0);
    bool indirect_dst_flag = is_operand_indirect(indirect, 1);

    if (indirect_src_flag) {
        auto read_ind_src =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_A, input_offset);
        direct_src_offset = uint32_t(read_ind_src.val);
        tag_match = tag_match && read_ind_src.tag_match;
    }

    if (indirect_dst_flag) {
        auto read_ind_dst =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_B, output_offset);
        direct_dst_offset = uint32_t(read_ind_dst.val);
        tag_match = tag_match && read_ind_dst.tag_match;
    }

    auto read_a = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, direct_src_offset, AvmMemoryTag::FF, AvmMemoryTag::FF);
    // Read in the memory address of where the first limb should be stored
    auto read_b = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, direct_dst_offset, AvmMemoryTag::FF, AvmMemoryTag::FF);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::POSEIDON2);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_ia = read_a.val, // First element of input
        .avm_main_ib = read_b.val, // First element of output (trivially zero)
        .avm_main_ind_a = indirect_src_flag ? FF(input_offset) : FF(0),
        .avm_main_ind_b = indirect_dst_flag ? FF(output_offset) : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(indirect_src_flag)),
        .avm_main_ind_op_b = FF(static_cast<uint32_t>(indirect_dst_flag)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(direct_src_offset),
        .avm_main_mem_idx_b = FF(direct_dst_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_pc = FF(pc++),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
        .avm_main_sel_op_poseidon2 = FF(1),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
    });
    // We store the current clk this main trace row occurred so that we can line up the poseidon2 gadget operation
    // at the same clk later.
    auto poseidon_op_clk = clk;

    // We need to increment the clk
    clk++;
    // Read results are written to input array.
    std::vector<FF> input_vec;
    read_slice_to_memory<FF>(
        call_ptr, clk, direct_src_offset, AvmMemoryTag::FF, AvmMemoryTag::FF, FF(internal_return_ptr), 4, input_vec);

    // Increment the clock by 1 since (4 reads / 4 reads per row = 1)
    clk += 1;
    std::array<FF, 4> input = vec_to_arr<FF, 4>(input_vec);
    std::array<FF, 4> result = poseidon2_trace_builder.poseidon2_permutation(input, poseidon_op_clk);
    std::vector<FF> ff_result;
    for (uint32_t i = 0; i < 4; i++) {
        ff_result.emplace_back(result[i]);
    }
    // // Write the result to memory after
    write_slice_to_memory(
        call_ptr, clk, direct_dst_offset, AvmMemoryTag::FF, AvmMemoryTag::FF, FF(internal_return_ptr), ff_result);
}

/**
 * @brief Keccakf1600  with direct or indirect memory access.
 * This function temporarily has the same interface as the kecccak opcode for compatibility, when the keccak migration
 * is complete (to keccakf1600) We will update this function call as we will not likely need input_size_offset
 * @param indirect byte encoding information about indirect/direct memory access.
 * @param output_offset An index in memory pointing to where the first u64 value of the output array should be stored.
 * @param input_offset An index in memory pointing to the first u64 value of the input array to be used in the next
 * instance of poseidon2 permutation.
 * @param input_size offset An index in memory pointing to the size of the input array. Temporary while we maintain the
 * same interface as keccak (this is fixed to 25)
 */
void AvmTraceBuilder::op_keccakf1600(uint8_t indirect,
                                     uint32_t output_offset,
                                     uint32_t input_offset,
                                     uint32_t input_size_offset)
{
    // What happens if the input_size_offset is > 25 when the state is more that that?
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;
    // bool tag_match = res.tag_match;
    bool tag_match = true;
    uint32_t direct_src_offset = input_offset;
    uint32_t direct_dst_offset = output_offset;

    bool indirect_src_flag = is_operand_indirect(indirect, 1);
    bool indirect_dst_flag = is_operand_indirect(indirect, 0);

    if (indirect_src_flag) {
        auto read_ind_src =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_A, input_offset);
        direct_src_offset = uint32_t(read_ind_src.val);
        tag_match = tag_match && read_ind_src.tag_match;
    }

    if (indirect_dst_flag) {
        auto read_ind_dst =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_C, output_offset);
        direct_dst_offset = uint32_t(read_ind_dst.val);
        tag_match = tag_match && read_ind_dst.tag_match;
    }

    auto input_read = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, direct_src_offset, AvmMemoryTag::U64, AvmMemoryTag::U64);
    auto output_read = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IC, direct_dst_offset, AvmMemoryTag::U64, AvmMemoryTag::U64);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::KECCAKF1600);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_ia = input_read.val,  // First element of input
        .avm_main_ic = output_read.val, // First element of output
        .avm_main_ind_a = indirect_src_flag ? FF(input_offset) : FF(0),
        .avm_main_ind_c = indirect_dst_flag ? FF(output_offset) : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(indirect_src_flag)),
        .avm_main_ind_op_c = FF(static_cast<uint32_t>(indirect_dst_flag)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(direct_src_offset), // input
        .avm_main_mem_idx_c = FF(direct_dst_offset), // output
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_c = FF(1),
        .avm_main_pc = FF(pc++),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U64)),
        .avm_main_sel_op_keccak = FF(1),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U64)),
    });
    // We store the current clk this main trace row occurred so that we can line up the keccak gadget operation
    // at the same clk later.
    auto keccak_op_clk = clk;
    // We need to increment the clk
    clk++;
    auto input_length_read = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, input_size_offset, AvmMemoryTag::U32, AvmMemoryTag::U32);
    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_ib = input_length_read.val, // Message Length
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_b = FF(input_size_offset), // length
        .avm_main_mem_op_b = FF(1),
        .avm_main_pc = FF(pc),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U32)),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U32)),
    });
    clk++;
    // Array input is fixed to 1600 bits
    std::vector<uint64_t> input_vec;
    // Read results are written to input array
    read_slice_to_memory<uint64_t>(
        call_ptr, clk, direct_src_offset, AvmMemoryTag::U64, AvmMemoryTag::U64, FF(internal_return_ptr), 25, input_vec);

    std::array<uint64_t, 25> input = vec_to_arr<uint64_t, 25>(input_vec);
    // Increment the clock by 7 since (25 reads / 4 reads per row = 7)
    clk += 7;

    // Now that we have read all the values, we can perform the operation to get the resulting witness.
    // Note: We use the keccak_op_clk to ensure that the keccakf1600 operation is performed at the same clock cycle
    // as the main trace that has the selector
    std::array<uint64_t, 25> result = keccak_trace_builder.keccakf1600(keccak_op_clk, input);
    // We convert the results to field elements here
    std::vector<FF> ff_result;
    for (uint32_t i = 0; i < 25; i++) {
        ff_result.emplace_back(result[i]);
    }

    // Write the result to memory after
    write_slice_to_memory(
        call_ptr, clk, direct_dst_offset, AvmMemoryTag::U64, AvmMemoryTag::U64, FF(internal_return_ptr), ff_result);
}

/**
 * @brief Keccak  with direct or indirect memory access.
 * Keccak is TEMPORARY while we wait for the transition to keccakf1600, so we do the minimal to store the result
 * @param indirect byte encoding information about indirect/direct memory access.
 * @param output_offset An index in memory pointing to where the first u8 value of the output array should be stored.
 * @param input_offset An index in memory pointing to the first u8 value of the input array to be used in the next
 * instance of poseidon2 permutation.
 * @param input_size offset An index in memory pointing to the size of the input array.
 */
void AvmTraceBuilder::op_keccak(uint8_t indirect,
                                uint32_t output_offset,
                                uint32_t input_offset,
                                uint32_t input_size_offset)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;
    bool tag_match = true;
    uint32_t direct_src_offset = input_offset;
    uint32_t direct_dst_offset = output_offset;

    bool indirect_src_flag = is_operand_indirect(indirect, 1);
    bool indirect_dst_flag = is_operand_indirect(indirect, 0);

    if (indirect_src_flag) {
        auto read_ind_src =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_A, input_offset);
        direct_src_offset = uint32_t(read_ind_src.val);
        tag_match = tag_match && read_ind_src.tag_match;
    }

    if (indirect_dst_flag) {
        auto read_ind_dst =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_C, output_offset);
        direct_dst_offset = uint32_t(read_ind_dst.val);
        tag_match = tag_match && read_ind_dst.tag_match;
    }
    // Note we load the input and output onto one line in the main trace and the length on the next line
    // We do this so we can load two different AvmMemoryTags (u8 for the I/O and u32 for the length)
    auto input_read = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, direct_src_offset, AvmMemoryTag::U8, AvmMemoryTag::U8);
    auto output_read = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IC, direct_dst_offset, AvmMemoryTag::U8, AvmMemoryTag::U8);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::KECCAK);

    // Store the clock time that we will use to line up the gadget later
    auto keccak_op_clk = clk;
    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_ia = input_read.val,  // First element of input
        .avm_main_ic = output_read.val, // First element of output
        .avm_main_ind_a = indirect_src_flag ? FF(input_offset) : FF(0),
        .avm_main_ind_c = indirect_dst_flag ? FF(output_offset) : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(indirect_src_flag)),
        .avm_main_ind_op_c = FF(static_cast<uint32_t>(indirect_dst_flag)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(direct_src_offset), // input
        .avm_main_mem_idx_c = FF(direct_dst_offset), // output
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_c = FF(1),
        .avm_main_pc = FF(pc++),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U8)),
        .avm_main_sel_op_keccak = FF(1),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U8)),
    });
    clk++;
    auto input_length_read = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, input_size_offset, AvmMemoryTag::U32, AvmMemoryTag::U32);
    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_ib = input_length_read.val, // Message Length
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_b = FF(input_size_offset), // length
        .avm_main_mem_op_b = FF(1),
        .avm_main_pc = FF(pc),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U32)),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U32)),
    });
    clk++;

    std::vector<uint8_t> input;
    input.reserve(uint32_t(input_length_read.val));

    uint32_t num_main_rows = read_slice_to_memory<uint8_t>(
        call_ptr, clk, direct_src_offset, AvmMemoryTag::U8, AvmMemoryTag::U8, FF(internal_return_ptr), 4, input);

    clk += num_main_rows;

    std::array<uint8_t, 32> result = keccak_trace_builder.keccak(keccak_op_clk, input, uint32_t(input_length_read.val));
    // We convert the results to field elements here
    std::vector<FF> ff_result;
    for (uint32_t i = 0; i < 32; i++) {
        ff_result.emplace_back(result[i]);
    }
    // Write the result to memory after
    write_slice_to_memory(
        call_ptr, clk, direct_dst_offset, AvmMemoryTag::U8, AvmMemoryTag::U8, FF(internal_return_ptr), ff_result);
}

/**
 * @brief Pedersen Hash  with direct or indirect memory access.
 * @param indirect byte encoding information about indirect/direct memory access.
 * @param gen_ctx_offset An index in memory pointing to where the u32 offset for the pedersen hash generators.
 * @param input_offset An index in memory pointing to the first FF value of the input array to be used in the next
 * @param input_size offset An index in memory pointing to the size of the input array.
 */
void AvmTraceBuilder::op_pedersen_hash(uint8_t indirect,
                                       uint32_t gen_ctx_offset,
                                       uint32_t output_offset,
                                       uint32_t input_offset,
                                       uint32_t input_size_offset)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;
    bool tag_match = true;
    uint32_t direct_src_offset = input_offset;
    bool indirect_src_flag = is_operand_indirect(indirect, 2);

    if (indirect_src_flag) {
        auto read_ind_src =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_A, input_offset);
        direct_src_offset = uint32_t(read_ind_src.val);
        tag_match = tag_match && read_ind_src.tag_match;
    }

    auto input_read = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, direct_src_offset, AvmMemoryTag::FF, AvmMemoryTag::FF);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::PEDERSEN);

    uint32_t pedersen_clk = clk;
    // We read the input and output addresses in one row as they should contain FF elements
    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_ia = input_read.val, // First element of input
        .avm_main_ind_a = indirect_src_flag ? FF(input_offset) : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(indirect_src_flag)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(direct_src_offset), // input
        .avm_main_mem_op_a = FF(1),
        .avm_main_pc = FF(pc++),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
        .avm_main_sel_op_pedersen = FF(1),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
    });
    clk++;
    // We read the input size and gen_ctx addresses in one row as they should contain U32 elements
    auto input_size_read = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, input_size_offset, AvmMemoryTag::U32, AvmMemoryTag::U32);
    auto gen_ctx_read = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, gen_ctx_offset, AvmMemoryTag::U32, AvmMemoryTag::U32);
    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_ia = input_size_read.val,
        .avm_main_ib = gen_ctx_read.val,
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(input_size_offset),
        .avm_main_mem_idx_b = FF(gen_ctx_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_pc = FF(pc),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U32)),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U32)),
    });
    clk++;

    std::vector<FF> inputs;
    uint32_t num_main_rows = read_slice_to_memory<FF>(call_ptr,
                                                      clk,
                                                      direct_src_offset,
                                                      AvmMemoryTag::FF,
                                                      AvmMemoryTag::FF,
                                                      FF(internal_return_ptr),
                                                      uint32_t(input_size_read.val),
                                                      inputs);
    clk += num_main_rows;
    FF output = pedersen_trace_builder.pedersen_hash(inputs, uint32_t(gen_ctx_read.val), pedersen_clk);
    write_slice_to_memory(
        call_ptr, clk, output_offset, AvmMemoryTag::FF, AvmMemoryTag::FF, FF(internal_return_ptr), { output });
}

void AvmTraceBuilder::op_ec_add(uint8_t indirect,
                                uint32_t lhs_x_offset,
                                uint32_t lhs_y_offset,
                                uint32_t lhs_is_inf_offset,
                                uint32_t rhs_x_offset,
                                uint32_t rhs_y_offset,
                                uint32_t rhs_is_inf_offset,
                                uint32_t output_offset)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;
    // Load lhs point
    auto lhs_x_read = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, lhs_x_offset, AvmMemoryTag::FF, AvmMemoryTag::U0);
    auto lhs_y_read = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, lhs_y_offset, AvmMemoryTag::FF, AvmMemoryTag::U0);
    // Load rhs point
    auto rhs_x_read = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IC, rhs_x_offset, AvmMemoryTag::FF, AvmMemoryTag::U0);
    auto rhs_y_read = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::ID, rhs_y_offset, AvmMemoryTag::FF, AvmMemoryTag::U0);

    // Save this clk time to line up with the gadget op.
    auto ecc_clk = clk;
    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_ia = lhs_x_read.val,
        .avm_main_ib = lhs_y_read.val,
        .avm_main_ic = rhs_x_read.val,
        .avm_main_id = rhs_y_read.val,
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(lhs_x_offset),
        .avm_main_mem_idx_b = FF(lhs_y_offset),
        .avm_main_mem_idx_c = FF(rhs_x_offset),
        .avm_main_mem_idx_d = FF(rhs_y_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_mem_op_c = FF(1),
        .avm_main_mem_op_d = FF(1),
        .avm_main_pc = FF(pc++),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
    });
    clk++;
    // Load the infinite bools separately since they have a different memory tag
    auto lhs_is_inf_read = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, lhs_is_inf_offset, AvmMemoryTag::U8, AvmMemoryTag::U0);
    auto rhs_is_inf_read = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, rhs_is_inf_offset, AvmMemoryTag::U8, AvmMemoryTag::U0);

    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_ia = lhs_is_inf_read.val,
        .avm_main_ib = rhs_is_inf_read.val,
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(lhs_is_inf_offset),
        .avm_main_mem_idx_b = FF(rhs_is_inf_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_pc = FF(pc),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U8)),
    });
    clk++;
    grumpkin::g1::affine_element lhs = uint8_t(lhs_is_inf_read.val) == 1
                                           ? grumpkin::g1::affine_element::infinity()
                                           : grumpkin::g1::affine_element{ lhs_x_read.val, lhs_y_read.val };
    grumpkin::g1::affine_element rhs = uint8_t(rhs_is_inf_read.val) == 1
                                           ? grumpkin::g1::affine_element::infinity()
                                           : grumpkin::g1::affine_element{ rhs_x_read.val, rhs_y_read.val };
    auto result = ecc_trace_builder.embedded_curve_add(lhs, rhs, ecc_clk);
    // Write across two lines since we have different mem_tags
    uint32_t direct_output_offset = output_offset;
    bool indirect_flag_output = is_operand_indirect(indirect, 6);
    if (indirect_flag_output) {
        auto read_ind_output =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_A, output_offset);
        direct_output_offset = uint32_t(read_ind_output.val);
    }

    mem_trace_builder.write_into_memory(
        call_ptr, clk, IntermRegister::IA, direct_output_offset, result.x, AvmMemoryTag::U0, AvmMemoryTag::FF);
    mem_trace_builder.write_into_memory(
        call_ptr, clk, IntermRegister::IB, direct_output_offset + 1, result.y, AvmMemoryTag::U0, AvmMemoryTag::FF);
    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_ia = result.x,
        .avm_main_ib = result.y,
        .avm_main_ind_a = indirect_flag_output ? FF(output_offset) : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(indirect_flag_output)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(direct_output_offset),
        .avm_main_mem_idx_b = FF(direct_output_offset + 1),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_pc = FF(pc),
        .avm_main_rwa = FF(1),
        .avm_main_rwb = FF(1),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
    });
    clk++;
    write_slice_to_memory(call_ptr,
                          clk,
                          direct_output_offset + 2,
                          AvmMemoryTag::U8,
                          AvmMemoryTag::U8,
                          FF(internal_return_ptr),
                          { result.is_point_at_infinity() });
}

// This function is a bit overloaded with logic around reconstructing points and scalars that could probably be moved to
// the gadget at some stage (although this is another temporary gadget..)
void AvmTraceBuilder::op_variable_msm(uint8_t indirect,
                                      uint32_t points_offset,
                                      uint32_t scalars_offset,
                                      uint32_t output_offset,
                                      uint32_t point_length_offset)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;
    // This will all get refactored as part of the indirection refactor
    bool tag_match = true;
    uint32_t direct_points_offset = points_offset;
    uint32_t direct_scalars_offset = scalars_offset;
    uint32_t direct_output_offset = output_offset;
    // Resolve the indirects
    bool indirect_points_flag = is_operand_indirect(indirect, 0);
    bool indirect_scalars_flag = is_operand_indirect(indirect, 1);
    bool indirect_output_flag = is_operand_indirect(indirect, 2);

    // Read in the points first
    if (indirect_points_flag) {
        auto read_ind_a =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_A, points_offset);
        direct_points_offset = uint32_t(read_ind_a.val);
        tag_match = tag_match && read_ind_a.tag_match;
    }

    auto read_points = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, direct_points_offset, AvmMemoryTag::FF, AvmMemoryTag::U0);

    // Read in the scalars
    if (indirect_scalars_flag) {
        auto read_ind_b = mem_trace_builder.indirect_read_and_load_from_memory(
            call_ptr, clk, IndirectRegister::IND_B, scalars_offset);
        direct_scalars_offset = uint32_t(read_ind_b.val);
        tag_match = tag_match && read_ind_b.tag_match;
    }
    auto read_scalars = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, direct_scalars_offset, AvmMemoryTag::FF, AvmMemoryTag::U0);

    // In the refactor we will have the read_slice function handle indirects as well
    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_ia = read_points.val,
        .avm_main_ib = read_scalars.val,
        .avm_main_ind_a = indirect_points_flag ? FF(points_offset) : FF(0),
        .avm_main_ind_b = indirect_scalars_flag ? FF(scalars_offset) : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(indirect_points_flag)),
        .avm_main_ind_op_b = FF(static_cast<uint32_t>(indirect_scalars_flag)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(direct_points_offset),
        .avm_main_mem_idx_b = FF(direct_scalars_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_pc = FF(pc++),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
        .avm_main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
    });
    clk++;

    // Read the points length (different row since it has a different memory tag)
    auto points_length_read = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, point_length_offset, AvmMemoryTag::U32, AvmMemoryTag::U0);
    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_ia = points_length_read.val,
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(point_length_offset),
        .avm_main_mem_op_a = FF(1),
        .avm_main_pc = FF(pc),
        .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U32)),
        .avm_main_tag_err = FF(static_cast<uint32_t>(!points_length_read.tag_match)),
    });
    clk++;

    // Points are stored as [x1, y1, inf1, x2, y2, inf2, ...] with the types [FF, FF, U8, FF, FF, U8, ...]
    uint32_t num_points = uint32_t(points_length_read.val) / 3; // 3 elements per point
    // We need to split up the reads due to the memory tags,
    std::vector<FF> points_coords_vec;
    std::vector<FF> points_inf_vec;
    std::vector<FF> scalars_vec;
    // Read the coordinates first, +2 since we read 2 points per row
    for (uint32_t i = 0; i < num_points; i += 2) {
        // We can read up to 4 coordinates per row (x1,y1,x2,y2)
        // Each pair of coordinates are separated by 3 memory addressess
        auto point_x1_read = mem_trace_builder.read_and_load_from_memory(
            call_ptr, clk, IntermRegister::IA, direct_points_offset + i * 3, AvmMemoryTag::FF, AvmMemoryTag::U0);
        auto point_y1_read = mem_trace_builder.read_and_load_from_memory(
            call_ptr, clk, IntermRegister::IB, direct_points_offset + i * 3 + 1, AvmMemoryTag::FF, AvmMemoryTag::U0);
        auto point_x2_read = mem_trace_builder.read_and_load_from_memory(
            call_ptr, clk, IntermRegister::IC, direct_points_offset + (i + 1) * 3, AvmMemoryTag::FF, AvmMemoryTag::U0);
        auto point_y2_read = mem_trace_builder.read_and_load_from_memory(call_ptr,
                                                                         clk,
                                                                         IntermRegister::ID,
                                                                         direct_points_offset + (i + 1) * 3 + 1,
                                                                         AvmMemoryTag::FF,
                                                                         AvmMemoryTag::U0);
        bool tag_match =
            point_x1_read.tag_match && point_y1_read.tag_match && point_x2_read.tag_match && point_y2_read.tag_match;
        points_coords_vec.insert(points_coords_vec.end(),
                                 { point_x1_read.val, point_y1_read.val, point_x2_read.val, point_y2_read.val });
        main_trace.push_back(Row{
            .avm_main_clk = clk,
            .avm_main_ia = point_x1_read.val,
            .avm_main_ib = point_y1_read.val,
            .avm_main_ic = point_x2_read.val,
            .avm_main_id = point_y2_read.val,
            .avm_main_internal_return_ptr = FF(internal_return_ptr),
            .avm_main_mem_idx_a = FF(direct_points_offset + i * 3),
            .avm_main_mem_idx_b = FF(direct_points_offset + i * 3 + 1),
            .avm_main_mem_idx_c = FF(direct_points_offset + (i + 1) * 3),
            .avm_main_mem_idx_d = FF(direct_points_offset + (i + 1) * 3 + 1),
            .avm_main_mem_op_a = FF(1),
            .avm_main_mem_op_b = FF(1),
            .avm_main_mem_op_c = FF(1),
            .avm_main_mem_op_d = FF(1),
            .avm_main_pc = FF(pc),
            .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
            .avm_main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        });
        clk++;
    }
    // Read the Infinities flags, +4 since we read 4 points row
    for (uint32_t i = 0; i < num_points; i += 4) {
        // We can read up to 4 infinities per row
        // Each infinity flag is separated by 3 memory addressess
        uint32_t offset = direct_points_offset + i * 3 + 2;
        auto point_inf1_read = mem_trace_builder.read_and_load_from_memory(
            call_ptr, clk, IntermRegister::IA, offset, AvmMemoryTag::U8, AvmMemoryTag::U0);
        offset += 3;

        auto point_inf2_read = mem_trace_builder.read_and_load_from_memory(
            call_ptr, clk, IntermRegister::IB, offset, AvmMemoryTag::U8, AvmMemoryTag::U0);
        offset += 3;

        auto point_inf3_read = mem_trace_builder.read_and_load_from_memory(
            call_ptr, clk, IntermRegister::IC, offset, AvmMemoryTag::U8, AvmMemoryTag::U0);
        offset += 3;

        auto point_inf4_read = mem_trace_builder.read_and_load_from_memory(
            call_ptr, clk, IntermRegister::ID, offset, AvmMemoryTag::U8, AvmMemoryTag::U0);

        points_inf_vec.insert(points_inf_vec.end(),
                              { point_inf1_read.val, point_inf2_read.val, point_inf3_read.val, point_inf4_read.val });
        bool tag_match = point_inf1_read.tag_match && point_inf2_read.tag_match && point_inf3_read.tag_match &&
                         point_inf4_read.tag_match;
        main_trace.push_back(Row{
            .avm_main_clk = clk,
            .avm_main_ia = point_inf1_read.val,
            .avm_main_ib = point_inf2_read.val,
            .avm_main_ic = point_inf3_read.val,
            .avm_main_id = point_inf4_read.val,
            .avm_main_internal_return_ptr = FF(internal_return_ptr),
            .avm_main_mem_idx_a = FF(direct_points_offset + i * 3 + 2),
            .avm_main_mem_idx_b = FF(direct_points_offset + (i + 1) * 3 + 2),
            .avm_main_mem_idx_c = FF(direct_points_offset + (i + 2) * 3 + 2),
            .avm_main_mem_idx_d = FF(direct_points_offset + (i + 3) * 3 + 2),
            .avm_main_mem_op_a = FF(1),
            .avm_main_mem_op_b = FF(1),
            .avm_main_mem_op_c = FF(1),
            .avm_main_mem_op_d = FF(1),
            .avm_main_pc = FF(pc),
            .avm_main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U8)),
            .avm_main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        });
        clk++;
    }
    // Scalar read length is num_points* 2 since scalars are stored as lo and hi limbs
    uint32_t scalar_read_length = num_points * 2;
    auto num_scalar_rows = read_slice_to_memory(call_ptr,
                                                clk,
                                                direct_scalars_offset,
                                                AvmMemoryTag::FF,
                                                AvmMemoryTag::U0,
                                                FF(internal_return_ptr),
                                                scalar_read_length,
                                                scalars_vec);
    clk += num_scalar_rows;
    // Reconstruct Grumpkin points
    std::vector<grumpkin::g1::affine_element> points;
    for (size_t i = 0; i < num_points; i++) {
        grumpkin::g1::Fq x = points_coords_vec[i * 2];
        grumpkin::g1::Fq y = points_coords_vec[i * 2 + 1];
        bool is_inf = points_inf_vec[i] == 1;
        if (is_inf) {
            points.emplace_back(grumpkin::g1::affine_element::infinity());
        } else {
            points.emplace_back(x, y);
        }
    }
    // Reconstruct Grumpkin scalars
    // Scalars are stored as [lo1, hi1, lo2, hi2, ...] with the types [FF, FF, FF, FF, ...]
    std::vector<grumpkin::fr> scalars;
    for (size_t i = 0; i < num_points; i++) {
        FF lo = scalars_vec[i * 2];
        FF hi = scalars_vec[i * 2 + 1];
        // hi is shifted 128 bits
        uint256_t scalar = (uint256_t(hi) << 128) + uint256_t(lo);
        scalars.emplace_back(scalar);
    }
    // Perform the variable MSM - could just put the logic in here since there are no constraints.
    auto result = ecc_trace_builder.variable_msm(points, scalars, clk);
    // Write the result back to memory [x, y, inf] with tags [FF, FF, U8]
    if (indirect_output_flag) {
        auto read_ind_a =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_A, output_offset);
        direct_output_offset = uint32_t(read_ind_a.val);
    }
    mem_trace_builder.write_into_memory(
        call_ptr, clk, IntermRegister::IA, direct_output_offset, result.x, AvmMemoryTag::U0, AvmMemoryTag::FF);
    mem_trace_builder.write_into_memory(
        call_ptr, clk, IntermRegister::IB, direct_output_offset + 1, result.y, AvmMemoryTag::U0, AvmMemoryTag::FF);
    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_ia = result.x,
        .avm_main_ib = result.y,
        .avm_main_ind_a = indirect_output_flag ? FF(output_offset) : FF(0),
        .avm_main_ind_op_a = FF(static_cast<uint32_t>(indirect_output_flag)),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(direct_output_offset),
        .avm_main_mem_idx_b = FF(direct_output_offset + 1),
        .avm_main_mem_op_a = FF(1),
        .avm_main_mem_op_b = FF(1),
        .avm_main_pc = FF(pc),
        .avm_main_rwa = FF(1),
        .avm_main_rwb = FF(1),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
    });
    clk++;
    // Write the infinity
    mem_trace_builder.write_into_memory(call_ptr,
                                        clk,
                                        IntermRegister::IA,
                                        direct_output_offset + 2,
                                        result.is_point_at_infinity(),
                                        AvmMemoryTag::U0,
                                        AvmMemoryTag::U8);
    main_trace.push_back(Row{
        .avm_main_clk = clk,
        .avm_main_ia = static_cast<uint8_t>(result.is_point_at_infinity()),
        .avm_main_internal_return_ptr = FF(internal_return_ptr),
        .avm_main_mem_idx_a = FF(direct_output_offset + 2),
        .avm_main_mem_op_a = FF(1),
        .avm_main_pc = FF(pc),
        .avm_main_rwa = FF(1),
        .avm_main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U8)),
    });
}
// Finalise Lookup Counts
//
// For log derivative lookups, we require a column that contains the number of times each lookup is consumed
// As we build the trace, we keep track of the reads made in a mapping, so that they can be applied to the
// counts column here
//
// NOTE: its coupled to pil - this is not the final iteration
void AvmTraceBuilder::finalise_mem_trace_lookup_counts()
{
    for (auto const& [clk, count] : mem_trace_builder.m_tag_err_lookup_counts) {
        main_trace.at(clk).incl_main_tag_err_counts = count;
    }
}

namespace {
// WARNING: FOR TESTING ONLY
// Generates the minimal lookup table for the binary trace
uint32_t finalize_bin_trace_lookup_for_testing(std::vector<Row>& main_trace, AvmBinaryTraceBuilder& bin_trace_builder)
{
    // Generate ByteLength Lookup table of instruction tags to the number of bytes
    // {U8: 1, U16: 2, U32: 4, U64: 8, U128: 16}
    for (auto const& [clk, count] : bin_trace_builder.byte_operation_counter) {
        // from the clk we can derive the a and b inputs
        auto b = static_cast<uint8_t>(clk);
        auto a = static_cast<uint8_t>(clk >> 8);
        auto op_id = static_cast<uint8_t>(clk >> 16);
        uint8_t bit_op = 0;
        if (op_id == 0) {
            bit_op = a & b;
        } else if (op_id == 1) {
            bit_op = a | b;
        } else {
            bit_op = a ^ b;
        }
        if (clk > (main_trace.size() - 1)) {
            main_trace.push_back(Row{
                .avm_main_clk = FF(clk),
                .avm_byte_lookup_bin_sel = FF(1),
                .avm_byte_lookup_table_input_a = a,
                .avm_byte_lookup_table_input_b = b,
                .avm_byte_lookup_table_op_id = op_id,
                .avm_byte_lookup_table_output = bit_op,
                .lookup_byte_operations_counts = count,
            });
        } else {
            main_trace.at(clk).lookup_byte_operations_counts = count;
            main_trace.at(clk).avm_byte_lookup_bin_sel = FF(1);
            main_trace.at(clk).avm_byte_lookup_table_op_id = op_id;
            main_trace.at(clk).avm_byte_lookup_table_input_a = a;
            main_trace.at(clk).avm_byte_lookup_table_input_b = b;
            main_trace.at(clk).avm_byte_lookup_table_output = bit_op;
        }
        // Add the counter value stored throughout the execution
    }
    return static_cast<uint32_t>(main_trace.size());
}

constexpr size_t L2_HI_GAS_COUNTS_IDX = 0;
constexpr size_t L2_LO_GAS_COUNTS_IDX = 1;
constexpr size_t DA_HI_GAS_COUNTS_IDX = 2;
constexpr size_t DA_LO_GAS_COUNTS_IDX = 3;

// WARNING: FOR TESTING ONLY
// Generates the lookup table for the range checks without doing a full 2**16 rows
uint32_t finalize_rng_chks_for_testing(
    std::vector<Row>& main_trace,
    AvmAluTraceBuilder const& alu_trace_builder,
    AvmMemTraceBuilder const& mem_trace_builder,
    std::unordered_map<uint16_t, uint32_t> const& mem_rng_check_lo_counts,
    std::unordered_map<uint16_t, uint32_t> const& mem_rng_check_mid_counts,
    std::unordered_map<uint8_t, uint32_t> const& mem_rng_check_hi_counts,
    std::array<std::unordered_map<uint16_t, uint32_t>, 4> const& rem_gas_rng_check_counts)
{
    // Build the main_trace, and add any new rows with specific clks that line up with lookup reads

    // Is there a "spread-like" operator in cpp or can I make it generic of the first param of the unordered map
    std::vector<std::unordered_map<uint8_t, uint32_t>> u8_rng_chks = { alu_trace_builder.u8_range_chk_counters[0],
                                                                       alu_trace_builder.u8_range_chk_counters[1],
                                                                       alu_trace_builder.u8_pow_2_counters[0],
                                                                       alu_trace_builder.u8_pow_2_counters[1],
                                                                       std::move(mem_rng_check_hi_counts) };

    std::vector<std::reference_wrapper<std::unordered_map<uint16_t, uint32_t> const>> u16_rng_chks;

    u16_rng_chks.emplace_back(mem_rng_check_lo_counts);
    u16_rng_chks.emplace_back(mem_rng_check_mid_counts);
    for (size_t i = 0; i < 4; i++) {
        u16_rng_chks.emplace_back(rem_gas_rng_check_counts[i]);
    }

    for (size_t i = 0; i < 15; i++) {
        u16_rng_chks.emplace_back(alu_trace_builder.u16_range_chk_counters[i]);
    }

    auto custom_clk = std::set<uint32_t>{};
    for (auto const& row : u8_rng_chks) {
        for (auto const& [key, value] : row) {
            custom_clk.insert(key);
        }
    }

    for (auto const& row : alu_trace_builder.u16_range_chk_counters) {
        for (auto const& [key, value] : row) {
            custom_clk.insert(key);
        }
    }

    for (auto row : u16_rng_chks) {
        for (auto const& [key, value] : row.get()) {
            custom_clk.insert(key);
        }
    }

    for (auto const& row : alu_trace_builder.div_u64_range_chk_counters) {
        for (auto const& [key, value] : row) {
            custom_clk.insert(key);
        }
    }

    for (auto const& [clk, count] : mem_trace_builder.m_tag_err_lookup_counts) {
        custom_clk.insert(clk);
    }

    auto old_size = main_trace.size();
    for (auto const& clk : custom_clk) {
        if (clk >= old_size) {
            main_trace.push_back(Row{ .avm_main_clk = FF(clk) });
        }
    }

    return static_cast<uint32_t>(main_trace.size());
}
} // anonymous namespace

/**
 * @brief Finalisation of the memory trace and incorporating it to the main trace.
 *        In particular, sorting the memory trace, setting .m_lastAccess and
 *        adding shifted values (first row). The main trace is moved at the end of
 *        this call.
 *
 * @return The main trace
 */
std::vector<Row> AvmTraceBuilder::finalize(uint32_t min_trace_size, bool range_check_required)
{
    auto mem_trace = mem_trace_builder.finalize();
    auto alu_trace = alu_trace_builder.finalize();
    auto conv_trace = conversion_trace_builder.finalize();
    auto sha256_trace = sha256_trace_builder.finalize();
    auto poseidon2_trace = poseidon2_trace_builder.finalize();
    auto keccak_trace = keccak_trace_builder.finalize();
    auto pedersen_trace = pedersen_trace_builder.finalize();
    auto bin_trace = bin_trace_builder.finalize();
    auto gas_trace = gas_trace_builder.finalize();
    size_t mem_trace_size = mem_trace.size();
    size_t main_trace_size = main_trace.size();
    size_t alu_trace_size = alu_trace.size();
    size_t conv_trace_size = conv_trace.size();
    size_t sha256_trace_size = sha256_trace.size();
    size_t poseidon2_trace_size = poseidon2_trace.size();
    size_t keccak_trace_size = keccak_trace.size();
    size_t pedersen_trace_size = pedersen_trace.size();
    size_t bin_trace_size = bin_trace.size();
    size_t gas_trace_size = gas_trace.size();

    // Data structure to collect all lookup counts pertaining to 16-bit/32-bit range checks in memory trace
    std::unordered_map<uint16_t, uint32_t> mem_rng_check_lo_counts;
    std::unordered_map<uint16_t, uint32_t> mem_rng_check_mid_counts;
    std::unordered_map<uint8_t, uint32_t> mem_rng_check_hi_counts;

    // Main Trace needs to be at least as big as the biggest subtrace.
    // If the bin_trace_size has entries, we need the main_trace to be as big as our byte lookup table (3 *
    // 2**16 long)
    size_t const lookup_table_size = (bin_trace_size > 0 && range_check_required) ? 3 * (1 << 16) : 0;
    size_t const range_check_size = range_check_required ? UINT16_MAX + 1 : 0;
    std::vector<size_t> trace_sizes = { mem_trace_size,     main_trace_size,      alu_trace_size,
                                        range_check_size,   conv_trace_size,      lookup_table_size,
                                        sha256_trace_size,  poseidon2_trace_size, pedersen_trace_size,
                                        gas_trace_size + 1, KERNEL_INPUTS_LENGTH, KERNEL_OUTPUTS_LENGTH,
                                        min_trace_size,     GAS_COST_TABLE.size() };
    auto trace_size = std::max_element(trace_sizes.begin(), trace_sizes.end());

    // We only need to pad with zeroes to the size to the largest trace here, pow_2 padding is handled in the
    // subgroup_size check in bb
    // Resize the main_trace to accomodate a potential lookup, filling with default empty rows.
    main_trace_size = *trace_size;
    main_trace.resize(*trace_size, {});

    main_trace.at(*trace_size - 1).avm_main_last = FF(1);

    // Memory trace inclusion

    // We compute in the main loop the timestamp and global address for next row.
    // Perform initialization for index 0 outside of the loop provided that mem trace exists.
    if (mem_trace_size > 0) {
        main_trace.at(0).avm_mem_tsp =
            FF(AvmMemTraceBuilder::NUM_SUB_CLK * mem_trace.at(0).m_clk + mem_trace.at(0).m_sub_clk);
        main_trace.at(0).avm_mem_glob_addr =
            FF(mem_trace.at(0).m_addr + (static_cast<uint64_t>(mem_trace.at(0).m_space_id) << 32));
    }

    for (size_t i = 0; i < mem_trace_size; i++) {
        auto const& src = mem_trace.at(i);
        auto& dest = main_trace.at(i);

        dest.avm_mem_mem_sel = FF(1);
        dest.avm_mem_clk = FF(src.m_clk);
        dest.avm_mem_addr = FF(src.m_addr);
        dest.avm_mem_space_id = FF(src.m_space_id);
        dest.avm_mem_val = src.m_val;
        dest.avm_mem_rw = FF(static_cast<uint32_t>(src.m_rw));
        dest.avm_mem_r_in_tag = FF(static_cast<uint32_t>(src.r_in_tag));
        dest.avm_mem_w_in_tag = FF(static_cast<uint32_t>(src.w_in_tag));
        dest.avm_mem_tag = FF(static_cast<uint32_t>(src.m_tag));
        dest.avm_mem_tag_err = FF(static_cast<uint32_t>(src.m_tag_err));
        dest.avm_mem_one_min_inv = src.m_one_min_inv;
        dest.avm_mem_sel_mov_a = FF(static_cast<uint32_t>(src.m_sel_mov_a));
        dest.avm_mem_sel_mov_b = FF(static_cast<uint32_t>(src.m_sel_mov_b));
        dest.avm_mem_sel_cmov = FF(static_cast<uint32_t>(src.m_sel_cmov));

        dest.incl_mem_tag_err_counts = FF(static_cast<uint32_t>(src.m_tag_err_count_relevant));

        switch (src.m_sub_clk) {
        case AvmMemTraceBuilder::SUB_CLK_LOAD_A:
        case AvmMemTraceBuilder::SUB_CLK_STORE_A:
            dest.avm_mem_op_a = 1;
            break;
        case AvmMemTraceBuilder::SUB_CLK_LOAD_B:
        case AvmMemTraceBuilder::SUB_CLK_STORE_B:
            dest.avm_mem_op_b = 1;
            break;
        case AvmMemTraceBuilder::SUB_CLK_LOAD_C:
        case AvmMemTraceBuilder::SUB_CLK_STORE_C:
            dest.avm_mem_op_c = 1;
            break;
        case AvmMemTraceBuilder::SUB_CLK_LOAD_D:
        case AvmMemTraceBuilder::SUB_CLK_STORE_D:
            dest.avm_mem_op_d = 1;
            break;
        case AvmMemTraceBuilder::SUB_CLK_IND_LOAD_A:
            dest.avm_mem_ind_op_a = 1;
            break;
        case AvmMemTraceBuilder::SUB_CLK_IND_LOAD_B:
            dest.avm_mem_ind_op_b = 1;
            break;
        case AvmMemTraceBuilder::SUB_CLK_IND_LOAD_C:
            dest.avm_mem_ind_op_c = 1;
            break;
        case AvmMemTraceBuilder::SUB_CLK_IND_LOAD_D:
            dest.avm_mem_ind_op_d = 1;
            break;
        default:
            break;
        }

        if (src.m_sel_cmov) {
            dest.avm_mem_skip_check_tag = dest.avm_mem_op_d + dest.avm_mem_op_a * (-dest.avm_mem_sel_mov_a + 1) +
                                          dest.avm_mem_op_b * (-dest.avm_mem_sel_mov_b + 1);
        }

        if (i + 1 < mem_trace_size) {
            auto const& next = mem_trace.at(i + 1);
            auto& dest_next = main_trace.at(i + 1);
            dest_next.avm_mem_tsp = FF(AvmMemTraceBuilder::NUM_SUB_CLK * next.m_clk + next.m_sub_clk);
            dest_next.avm_mem_glob_addr = FF(next.m_addr + (static_cast<uint64_t>(next.m_space_id) << 32));

            FF diff{};
            if (dest_next.avm_mem_glob_addr == dest.avm_mem_glob_addr) {
                diff = dest_next.avm_mem_tsp - dest.avm_mem_tsp;
            } else {
                diff = dest_next.avm_mem_glob_addr - dest.avm_mem_glob_addr;
                dest.avm_mem_lastAccess = FF(1);
            }
            dest.avm_mem_rng_chk_sel = FF(1);

            // Decomposition of diff
            auto const diff_64 = uint64_t(diff);
            auto const diff_hi = static_cast<uint8_t>(diff_64 >> 32);
            auto const diff_mid = static_cast<uint16_t>((diff_64 & UINT32_MAX) >> 16);
            auto const diff_lo = static_cast<uint16_t>(diff_64 & UINT16_MAX);
            dest.avm_mem_diff_hi = FF(diff_hi);
            dest.avm_mem_diff_mid = FF(diff_mid);
            dest.avm_mem_diff_lo = FF(diff_lo);

            // Add the range checks counts
            mem_rng_check_hi_counts[diff_hi]++;
            mem_rng_check_mid_counts[diff_mid]++;
            mem_rng_check_lo_counts[diff_lo]++;
        } else {
            dest.avm_mem_lastAccess = FF(1);
            dest.avm_mem_last = FF(1);
        }
    }

    // Alu trace inclusion
    for (size_t i = 0; i < alu_trace_size; i++) {
        auto const& src = alu_trace.at(i);
        auto& dest = main_trace.at(i);

        dest.avm_alu_clk = FF(static_cast<uint32_t>(src.alu_clk));

        dest.avm_alu_op_add = FF(static_cast<uint32_t>(src.alu_op_add));
        dest.avm_alu_op_sub = FF(static_cast<uint32_t>(src.alu_op_sub));
        dest.avm_alu_op_mul = FF(static_cast<uint32_t>(src.alu_op_mul));
        dest.avm_alu_op_not = FF(static_cast<uint32_t>(src.alu_op_not));
        dest.avm_alu_op_eq = FF(static_cast<uint32_t>(src.alu_op_eq));
        dest.avm_alu_op_lt = FF(static_cast<uint32_t>(src.alu_op_lt));
        dest.avm_alu_op_lte = FF(static_cast<uint32_t>(src.alu_op_lte));
        dest.avm_alu_op_cast = FF(static_cast<uint32_t>(src.alu_op_cast));
        dest.avm_alu_op_cast_prev = FF(static_cast<uint32_t>(src.alu_op_cast_prev));
        dest.avm_alu_cmp_sel = FF(static_cast<uint8_t>(src.alu_op_lt) + static_cast<uint8_t>(src.alu_op_lte));
        dest.avm_alu_rng_chk_sel = FF(static_cast<uint8_t>(src.rng_chk_sel));
        dest.avm_alu_op_shr = FF(static_cast<uint8_t>(src.alu_op_shr));
        dest.avm_alu_op_shl = FF(static_cast<uint8_t>(src.alu_op_shl));
        dest.avm_alu_op_div = FF(static_cast<uint8_t>(src.alu_op_div));

        dest.avm_alu_ff_tag = FF(static_cast<uint32_t>(src.alu_ff_tag));
        dest.avm_alu_u8_tag = FF(static_cast<uint32_t>(src.alu_u8_tag));
        dest.avm_alu_u16_tag = FF(static_cast<uint32_t>(src.alu_u16_tag));
        dest.avm_alu_u32_tag = FF(static_cast<uint32_t>(src.alu_u32_tag));
        dest.avm_alu_u64_tag = FF(static_cast<uint32_t>(src.alu_u64_tag));
        dest.avm_alu_u128_tag = FF(static_cast<uint32_t>(src.alu_u128_tag));

        dest.avm_alu_in_tag = dest.avm_alu_u8_tag + FF(2) * dest.avm_alu_u16_tag + FF(3) * dest.avm_alu_u32_tag +
                              FF(4) * dest.avm_alu_u64_tag + FF(5) * dest.avm_alu_u128_tag +
                              FF(6) * dest.avm_alu_ff_tag;

        dest.avm_alu_ia = src.alu_ia;
        dest.avm_alu_ib = src.alu_ib;
        dest.avm_alu_ic = src.alu_ic;

        dest.avm_alu_cf = FF(static_cast<uint32_t>(src.alu_cf));

        dest.avm_alu_u8_r0 = FF(src.alu_u8_r0);
        dest.avm_alu_u8_r1 = FF(src.alu_u8_r1);

        dest.avm_alu_u16_r0 = FF(src.alu_u16_reg.at(0));
        dest.avm_alu_u16_r1 = FF(src.alu_u16_reg.at(1));
        dest.avm_alu_u16_r2 = FF(src.alu_u16_reg.at(2));
        dest.avm_alu_u16_r3 = FF(src.alu_u16_reg.at(3));
        dest.avm_alu_u16_r4 = FF(src.alu_u16_reg.at(4));
        dest.avm_alu_u16_r5 = FF(src.alu_u16_reg.at(5));
        dest.avm_alu_u16_r6 = FF(src.alu_u16_reg.at(6));
        dest.avm_alu_u16_r7 = FF(src.alu_u16_reg.at(7));
        dest.avm_alu_u16_r8 = FF(src.alu_u16_reg.at(8));
        dest.avm_alu_u16_r9 = FF(src.alu_u16_reg.at(9));
        dest.avm_alu_u16_r10 = FF(src.alu_u16_reg.at(10));
        dest.avm_alu_u16_r11 = FF(src.alu_u16_reg.at(11));
        dest.avm_alu_u16_r12 = FF(src.alu_u16_reg.at(12));
        dest.avm_alu_u16_r13 = FF(src.alu_u16_reg.at(13));
        dest.avm_alu_u16_r14 = FF(src.alu_u16_reg.at(14));

        dest.avm_alu_div_rng_chk_selector = FF(static_cast<uint8_t>(src.div_u64_range_chk_sel));
        dest.avm_alu_div_u16_r0 = FF(src.div_u64_range_chk.at(0));
        dest.avm_alu_div_u16_r1 = FF(src.div_u64_range_chk.at(1));
        dest.avm_alu_div_u16_r2 = FF(src.div_u64_range_chk.at(2));
        dest.avm_alu_div_u16_r3 = FF(src.div_u64_range_chk.at(3));
        dest.avm_alu_div_u16_r4 = FF(src.div_u64_range_chk.at(4));
        dest.avm_alu_div_u16_r5 = FF(src.div_u64_range_chk.at(5));
        dest.avm_alu_div_u16_r6 = FF(src.div_u64_range_chk.at(6));
        dest.avm_alu_div_u16_r7 = FF(src.div_u64_range_chk.at(7));
        dest.avm_alu_op_eq_diff_inv = FF(src.alu_op_eq_diff_inv);

        // Not all rows in ALU are enabled with a selector. For instance,
        // multiplication over u128 and cast is taking two lines.
        if (AvmAluTraceBuilder::is_alu_row_enabled(src)) {
            dest.avm_alu_alu_sel = FF(1);
        }

        if (dest.avm_alu_cmp_sel == FF(1) || dest.avm_alu_rng_chk_sel == FF(1)) {
            dest.avm_alu_a_lo = FF(src.hi_lo_limbs.at(0));
            dest.avm_alu_a_hi = FF(src.hi_lo_limbs.at(1));
            dest.avm_alu_b_lo = FF(src.hi_lo_limbs.at(2));
            dest.avm_alu_b_hi = FF(src.hi_lo_limbs.at(3));
            dest.avm_alu_p_sub_a_lo = FF(src.hi_lo_limbs.at(4));
            dest.avm_alu_p_sub_a_hi = FF(src.hi_lo_limbs.at(5));
            dest.avm_alu_p_sub_b_lo = FF(src.hi_lo_limbs.at(6));
            dest.avm_alu_p_sub_b_hi = FF(src.hi_lo_limbs.at(7));
            dest.avm_alu_res_lo = FF(src.hi_lo_limbs.at(8));
            dest.avm_alu_res_hi = FF(src.hi_lo_limbs.at(9));
            dest.avm_alu_p_a_borrow = FF(static_cast<uint8_t>(src.p_a_borrow));
            dest.avm_alu_p_b_borrow = FF(static_cast<uint8_t>(src.p_b_borrow));
            dest.avm_alu_borrow = FF(static_cast<uint8_t>(src.borrow));
            dest.avm_alu_cmp_rng_ctr = FF(static_cast<uint8_t>(src.cmp_rng_ctr));
            dest.avm_alu_rng_chk_lookup_selector = FF(1);
        }
        if (dest.avm_alu_op_div == FF(1)) {
            dest.avm_alu_op_div_std = uint256_t(src.alu_ia) >= uint256_t(src.alu_ib);
            dest.avm_alu_op_div_a_lt_b = uint256_t(src.alu_ia) < uint256_t(src.alu_ib);
            dest.avm_alu_rng_chk_lookup_selector = FF(1);
            dest.avm_alu_a_lo = FF(src.hi_lo_limbs.at(0));
            dest.avm_alu_a_hi = FF(src.hi_lo_limbs.at(1));
            dest.avm_alu_b_lo = FF(src.hi_lo_limbs.at(2));
            dest.avm_alu_b_hi = FF(src.hi_lo_limbs.at(3));
            dest.avm_alu_p_sub_a_lo = FF(src.hi_lo_limbs.at(4));
            dest.avm_alu_p_sub_a_hi = FF(src.hi_lo_limbs.at(5));
            dest.avm_alu_remainder = src.remainder;
            dest.avm_alu_divisor_lo = src.divisor_lo;
            dest.avm_alu_divisor_hi = src.divisor_hi;
            dest.avm_alu_quotient_lo = src.quotient_lo;
            dest.avm_alu_quotient_hi = src.quotient_hi;
            dest.avm_alu_partial_prod_lo = src.partial_prod_lo;
            dest.avm_alu_partial_prod_hi = src.partial_prod_hi;
        }

        if (dest.avm_alu_op_add == FF(1) || dest.avm_alu_op_sub == FF(1) || dest.avm_alu_op_mul == FF(1)) {
            dest.avm_alu_rng_chk_lookup_selector = FF(1);
        }

        if (dest.avm_alu_op_cast == FF(1)) {
            dest.avm_alu_a_lo = FF(src.hi_lo_limbs.at(0));
            dest.avm_alu_a_hi = FF(src.hi_lo_limbs.at(1));
            dest.avm_alu_p_sub_a_lo = FF(src.hi_lo_limbs.at(2));
            dest.avm_alu_p_sub_a_hi = FF(src.hi_lo_limbs.at(3));
            dest.avm_alu_p_a_borrow = FF(static_cast<uint8_t>(src.p_a_borrow));
            dest.avm_alu_rng_chk_lookup_selector = FF(1);
        }

        if (dest.avm_alu_op_cast_prev == FF(1)) {
            dest.avm_alu_a_lo = FF(src.hi_lo_limbs.at(0));
            dest.avm_alu_a_hi = FF(src.hi_lo_limbs.at(1));
            dest.avm_alu_rng_chk_lookup_selector = FF(1);
        }

        // Multiplication over u128 expands over two rows.
        if (dest.avm_alu_op_mul == FF(1) && dest.avm_alu_u128_tag) {
            main_trace.at(i + 1).avm_alu_rng_chk_lookup_selector = FF(1);
        }
        if (src.alu_op_shr || src.alu_op_shl) {
            dest.avm_alu_a_lo = FF(src.hi_lo_limbs[0]);
            dest.avm_alu_a_hi = FF(src.hi_lo_limbs[1]);
            dest.avm_alu_b_lo = FF(src.hi_lo_limbs[2]);
            dest.avm_alu_b_hi = FF(src.hi_lo_limbs[3]);
            dest.avm_alu_shift_sel = FF(1);
            dest.avm_alu_shift_lt_bit_len = FF(static_cast<uint8_t>(src.shift_lt_bit_len));
            dest.avm_alu_t_sub_s_bits = FF(src.mem_tag_sub_shift);
            dest.avm_alu_two_pow_s = FF(uint256_t(1) << dest.avm_alu_ib);
            dest.avm_alu_two_pow_t_sub_s = FF(uint256_t(1) << uint256_t(dest.avm_alu_t_sub_s_bits));
            dest.avm_alu_rng_chk_lookup_selector = FF(1);
        }
    }

    // Add Conversion Gadget table
    for (size_t i = 0; i < conv_trace_size; i++) {
        auto const& src = conv_trace.at(i);
        auto& dest = main_trace.at(i);
        dest.avm_conversion_to_radix_le_sel = FF(static_cast<uint8_t>(src.to_radix_le_sel));
        dest.avm_conversion_clk = FF(src.conversion_clk);
        dest.avm_conversion_input = src.input;
        dest.avm_conversion_radix = FF(src.radix);
        dest.avm_conversion_num_limbs = FF(src.num_limbs);
    }

    // Add SHA256 Gadget table
    for (size_t i = 0; i < sha256_trace_size; i++) {
        auto const& src = sha256_trace.at(i);
        auto& dest = main_trace.at(i);
        dest.avm_sha256_clk = FF(src.clk);
        dest.avm_sha256_input = src.input[0];
        // TODO: This will need to be enabled later
        // dest.avm_sha256_output = src.output[0];
        dest.avm_sha256_sha256_compression_sel = FF(1);
        dest.avm_sha256_state = src.state[0];
    }

    // Add Poseidon2 Gadget table
    for (size_t i = 0; i < poseidon2_trace_size; i++) {
        auto const& src = poseidon2_trace.at(i);
        auto& dest = main_trace.at(i);
        dest.avm_poseidon2_clk = FF(src.clk);
        dest.avm_poseidon2_input = src.input[0];
        // TODO: This will need to be enabled later
        // dest.avm_poseidon2_output = src.output[0];
        dest.avm_poseidon2_poseidon_perm_sel = FF(1);
    }

    // Add KeccakF1600 Gadget table
    for (size_t i = 0; i < keccak_trace_size; i++) {
        auto const& src = keccak_trace.at(i);
        auto& dest = main_trace.at(i);
        dest.avm_keccakf1600_clk = FF(src.clk);
        dest.avm_keccakf1600_input = FF(src.input[0]);
        // TODO: This will need to be enabled later
        // dest.avm_keccakf1600_output = src.output[0];
        dest.avm_keccakf1600_keccakf1600_sel = FF(1);
    }

    // Add Pedersen Gadget table
    for (size_t i = 0; i < pedersen_trace_size; i++) {
        auto const& src = pedersen_trace.at(i);
        auto& dest = main_trace.at(i);
        dest.avm_pedersen_clk = FF(src.clk);
        dest.avm_pedersen_input = FF(src.input[0]);
        dest.avm_pedersen_pedersen_sel = FF(1);
    }

    // Add Binary Trace table
    for (size_t i = 0; i < bin_trace_size; i++) {
        auto const& src = bin_trace.at(i);
        auto& dest = main_trace.at(i);
        dest.avm_binary_clk = src.binary_clk;
        dest.avm_binary_bin_sel = static_cast<uint8_t>(src.bin_sel);
        dest.avm_binary_acc_ia = src.acc_ia;
        dest.avm_binary_acc_ib = src.acc_ib;
        dest.avm_binary_acc_ic = src.acc_ic;
        dest.avm_binary_in_tag = src.in_tag;
        dest.avm_binary_op_id = src.op_id;
        dest.avm_binary_ia_bytes = src.bin_ia_bytes;
        dest.avm_binary_ib_bytes = src.bin_ib_bytes;
        dest.avm_binary_ic_bytes = src.bin_ic_bytes;
        dest.avm_binary_start = FF(static_cast<uint8_t>(src.start));
        dest.avm_binary_mem_tag_ctr = src.mem_tag_ctr;
        dest.avm_binary_mem_tag_ctr_inv = src.mem_tag_ctr_inv;
    }

    // Only generate precomputed byte tables if we are actually going to use them in this main trace.
    if (bin_trace_size > 0) {
        if (!range_check_required) {
            finalize_bin_trace_lookup_for_testing(main_trace, bin_trace_builder);
        } else {
            // Generate Lookup Table of all combinations of 2, 8-bit numbers and op_id.
            for (uint32_t op_id = 0; op_id < 3; op_id++) {
                for (uint32_t input_a = 0; input_a <= UINT8_MAX; input_a++) {
                    for (uint32_t input_b = 0; input_b <= UINT8_MAX; input_b++) {
                        auto a = static_cast<uint8_t>(input_a);
                        auto b = static_cast<uint8_t>(input_b);

                        // Derive a unique row index given op_id, a, and b.
                        auto main_trace_index = (op_id << 16) + (input_a << 8) + b;

                        main_trace.at(main_trace_index).avm_byte_lookup_bin_sel = FF(1);
                        main_trace.at(main_trace_index).avm_byte_lookup_table_op_id = op_id;
                        main_trace.at(main_trace_index).avm_byte_lookup_table_input_a = a;
                        main_trace.at(main_trace_index).avm_byte_lookup_table_input_b = b;
                        // Add the counter value stored throughout the execution
                        main_trace.at(main_trace_index).lookup_byte_operations_counts =
                            bin_trace_builder.byte_operation_counter[main_trace_index];
                        if (op_id == 0) {
                            main_trace.at(main_trace_index).avm_byte_lookup_table_output = a & b;
                        } else if (op_id == 1) {
                            main_trace.at(main_trace_index).avm_byte_lookup_table_output = a | b;
                        } else {
                            main_trace.at(main_trace_index).avm_byte_lookup_table_output = a ^ b;
                        }
                    }
                }
            }
        }
        // Generate ByteLength Lookup table of instruction tags to the number of bytes
        // {U8: 1, U16: 2, U32: 4, U64: 8, U128: 16}
        for (uint8_t avm_in_tag = 0; avm_in_tag < 5; avm_in_tag++) {
            // The +1 here is because the instruction tags we care about (i.e excl U0 and FF) has the range
            // [1,5]
            main_trace.at(avm_in_tag).avm_byte_lookup_bin_sel = FF(1);
            main_trace.at(avm_in_tag).avm_byte_lookup_table_in_tags = avm_in_tag + 1;
            main_trace.at(avm_in_tag).avm_byte_lookup_table_byte_lengths = static_cast<uint8_t>(pow(2, avm_in_tag));
            main_trace.at(avm_in_tag).lookup_byte_lengths_counts =
                bin_trace_builder.byte_length_counter[avm_in_tag + 1];
        }
    }

    /////////// GAS ACCOUNTING //////////////////////////

    // Add the gas cost table to the main trace
    // TODO: do i need a way to produce an interupt that will stop the execution of the trace when the gas left
    // becomes zero in the gas_trace_builder Does all of the gas trace information need to be added to this main
    // machine?????

    // Add the gas accounting for each row
    // We can assume that the gas trace will never be larger than the main trace
    // We infer that a row is active for gas (.avm_main_gas_cost_active = 1) based on the presence
    // of a gas entry row.
    // Set the initial gas
    auto& first_opcode_row = main_trace.at(0);
    first_opcode_row.avm_main_l2_gas_remaining = gas_trace_builder.initial_l2_gas;
    first_opcode_row.avm_main_da_gas_remaining = gas_trace_builder.initial_da_gas;
    uint32_t current_clk = 1;
    uint32_t current_l2_gas_remaining = gas_trace_builder.initial_l2_gas;
    uint32_t current_da_gas_remaining = gas_trace_builder.initial_da_gas;

    // Data structure to collect all lookup counts pertaining to 16-bit range checks related to remaining gas
    std::array<std::unordered_map<uint16_t, uint32_t>, 4> rem_gas_rng_check_counts;

    std::unordered_map<uint16_t, uint32_t> l2_rem_gas_rng_check_hi_counts;
    std::unordered_map<uint16_t, uint32_t> l2_rem_gas_rng_check_lo_counts;
    std::unordered_map<uint16_t, uint32_t> da_rem_gas_rng_check_hi_counts;
    std::unordered_map<uint16_t, uint32_t> da_rem_gas_rng_check_lo_counts;

    // Assume that gas_trace entries are ordered by a strictly increasing clk sequence.
    for (auto const& gas_entry : gas_trace) {

        // Filling potential gap between two gas_trace entries
        // Remaining gas values remain unchanged.
        while (gas_entry.clk > current_clk) {
            auto& next = main_trace.at(current_clk);
            next.avm_main_l2_gas_remaining = current_l2_gas_remaining;
            next.avm_main_da_gas_remaining = current_da_gas_remaining;
            current_clk++;
        }

        auto& dest = main_trace.at(gas_entry.clk - 1);
        auto& next = main_trace.at(gas_entry.clk);

        // Write each of the relevant gas accounting values
        dest.avm_main_opcode_val = static_cast<uint8_t>(gas_entry.opcode);
        dest.avm_main_l2_gas_op = gas_entry.l2_gas_cost;
        dest.avm_main_da_gas_op = gas_entry.da_gas_cost;

        // If gas remaining is increasing, it means we underflowed in uint32_t
        bool l2_out_of_gas = current_l2_gas_remaining < gas_entry.remaining_l2_gas;
        bool da_out_of_gas = current_da_gas_remaining < gas_entry.remaining_da_gas;

        uint32_t abs_l2_gas_remaining = l2_out_of_gas ? -gas_entry.remaining_l2_gas : gas_entry.remaining_l2_gas;
        uint32_t abs_da_gas_remaining = da_out_of_gas ? -gas_entry.remaining_da_gas : gas_entry.remaining_da_gas;

        dest.avm_main_abs_l2_rem_gas_hi = abs_l2_gas_remaining >> 16;
        dest.avm_main_abs_da_rem_gas_hi = abs_da_gas_remaining >> 16;
        dest.avm_main_abs_l2_rem_gas_lo = static_cast<uint16_t>(abs_l2_gas_remaining);
        dest.avm_main_abs_da_rem_gas_lo = static_cast<uint16_t>(abs_da_gas_remaining);

        // TODO: gas is not constrained for external call at this time
        if (gas_entry.opcode != OpCode::CALL) {
            dest.avm_main_gas_cost_active = FF(1);

            // lookups counting
            rem_gas_rng_check_counts[L2_HI_GAS_COUNTS_IDX][static_cast<uint16_t>(dest.avm_main_abs_l2_rem_gas_hi)]++;
            rem_gas_rng_check_counts[L2_LO_GAS_COUNTS_IDX][static_cast<uint16_t>(dest.avm_main_abs_l2_rem_gas_lo)]++;
            rem_gas_rng_check_counts[DA_HI_GAS_COUNTS_IDX][static_cast<uint16_t>(dest.avm_main_abs_da_rem_gas_hi)]++;
            rem_gas_rng_check_counts[DA_LO_GAS_COUNTS_IDX][static_cast<uint16_t>(dest.avm_main_abs_da_rem_gas_lo)]++;
        }

        dest.avm_main_l2_out_of_gas = static_cast<uint32_t>(l2_out_of_gas);
        dest.avm_main_da_out_of_gas = static_cast<uint32_t>(da_out_of_gas);

        current_l2_gas_remaining = gas_entry.remaining_l2_gas;
        current_da_gas_remaining = gas_entry.remaining_da_gas;
        next.avm_main_l2_gas_remaining =
            l2_out_of_gas ? FF::modulus - uint256_t(abs_l2_gas_remaining) : current_l2_gas_remaining;
        next.avm_main_da_gas_remaining =
            da_out_of_gas ? FF::modulus - uint256_t(abs_da_gas_remaining) : current_da_gas_remaining;

        current_clk++;
    }

    // Pad the rest of the trace with the same gas remaining
    for (size_t i = current_clk; i < main_trace_size; i++) {
        auto& dest = main_trace.at(i);
        dest.avm_main_l2_gas_remaining = current_l2_gas_remaining;
        dest.avm_main_da_gas_remaining = current_da_gas_remaining;
    }

    /////////// END OF GAS ACCOUNTING //////////////////////////

    // Adding extra row for the shifted values at the top of the execution trace.
    Row first_row = Row{ .avm_main_first = FF(1), .avm_mem_lastAccess = FF(1) };
    main_trace.insert(main_trace.begin(), first_row);
    auto const old_trace_size = main_trace.size();

    auto new_trace_size = range_check_required ? old_trace_size
                                               : finalize_rng_chks_for_testing(main_trace,
                                                                               alu_trace_builder,
                                                                               mem_trace_builder,
                                                                               mem_rng_check_lo_counts,
                                                                               mem_rng_check_mid_counts,
                                                                               mem_rng_check_hi_counts,
                                                                               rem_gas_rng_check_counts);
    for (size_t i = 0; i < new_trace_size; i++) {
        auto& r = main_trace.at(i);

        if ((r.avm_main_sel_op_add == FF(1) || r.avm_main_sel_op_sub == FF(1) || r.avm_main_sel_op_mul == FF(1) ||
             r.avm_main_sel_op_eq == FF(1) || r.avm_main_sel_op_not == FF(1) || r.avm_main_sel_op_lt == FF(1) ||
             r.avm_main_sel_op_lte == FF(1) || r.avm_main_sel_op_cast == FF(1) || r.avm_main_sel_op_shr == FF(1) ||
             r.avm_main_sel_op_shl == FF(1) || r.avm_main_sel_op_div == FF(1)) &&
            r.avm_main_tag_err == FF(0) && r.avm_main_op_err == FF(0)) {
            r.avm_main_alu_sel = FF(1);
        }

        if (r.avm_main_sel_internal_call == FF(1) || r.avm_main_sel_internal_return == FF(1)) {
            r.avm_main_space_id = INTERNAL_CALL_SPACE_ID;
        } else {
            r.avm_main_space_id = r.avm_main_call_ptr;
        };

        r.avm_main_clk = i >= old_trace_size ? r.avm_main_clk : FF(i);
        auto counter = i >= old_trace_size ? static_cast<uint32_t>(r.avm_main_clk) : static_cast<uint32_t>(i);
        r.incl_main_tag_err_counts = mem_trace_builder.m_tag_err_lookup_counts[static_cast<uint32_t>(counter)];

        if (counter <= UINT8_MAX) {
            r.lookup_u8_0_counts = alu_trace_builder.u8_range_chk_counters[0][static_cast<uint8_t>(counter)];
            r.lookup_u8_1_counts = alu_trace_builder.u8_range_chk_counters[1][static_cast<uint8_t>(counter)];
            r.lookup_pow_2_0_counts = alu_trace_builder.u8_pow_2_counters[0][static_cast<uint8_t>(counter)];
            r.lookup_pow_2_1_counts = alu_trace_builder.u8_pow_2_counters[1][static_cast<uint8_t>(counter)];
            r.lookup_mem_rng_chk_hi_counts = mem_rng_check_hi_counts[static_cast<uint8_t>(counter)];
            r.avm_main_sel_rng_8 = FF(1);
            r.avm_main_table_pow_2 = uint256_t(1) << uint256_t(counter);
        }

        if (counter <= UINT16_MAX) {
            // We add to the clk here in case our trace is smaller than our range checks
            // There might be a cleaner way to do this in the future as this only applies
            // when our trace (excluding range checks) is < 2**16
            r.lookup_u16_0_counts = alu_trace_builder.u16_range_chk_counters[0][static_cast<uint16_t>(counter)];
            r.lookup_u16_1_counts = alu_trace_builder.u16_range_chk_counters[1][static_cast<uint16_t>(counter)];
            r.lookup_u16_2_counts = alu_trace_builder.u16_range_chk_counters[2][static_cast<uint16_t>(counter)];
            r.lookup_u16_3_counts = alu_trace_builder.u16_range_chk_counters[3][static_cast<uint16_t>(counter)];
            r.lookup_u16_4_counts = alu_trace_builder.u16_range_chk_counters[4][static_cast<uint16_t>(counter)];
            r.lookup_u16_5_counts = alu_trace_builder.u16_range_chk_counters[5][static_cast<uint16_t>(counter)];
            r.lookup_u16_6_counts = alu_trace_builder.u16_range_chk_counters[6][static_cast<uint16_t>(counter)];
            r.lookup_u16_7_counts = alu_trace_builder.u16_range_chk_counters[7][static_cast<uint16_t>(counter)];
            r.lookup_u16_8_counts = alu_trace_builder.u16_range_chk_counters[8][static_cast<uint16_t>(counter)];
            r.lookup_u16_9_counts = alu_trace_builder.u16_range_chk_counters[9][static_cast<uint16_t>(counter)];
            r.lookup_u16_10_counts = alu_trace_builder.u16_range_chk_counters[10][static_cast<uint16_t>(counter)];
            r.lookup_u16_11_counts = alu_trace_builder.u16_range_chk_counters[11][static_cast<uint16_t>(counter)];
            r.lookup_u16_12_counts = alu_trace_builder.u16_range_chk_counters[12][static_cast<uint16_t>(counter)];
            r.lookup_u16_13_counts = alu_trace_builder.u16_range_chk_counters[13][static_cast<uint16_t>(counter)];
            r.lookup_u16_14_counts = alu_trace_builder.u16_range_chk_counters[14][static_cast<uint16_t>(counter)];

            r.lookup_mem_rng_chk_mid_counts = mem_rng_check_mid_counts[static_cast<uint16_t>(counter)];
            r.lookup_mem_rng_chk_lo_counts = mem_rng_check_lo_counts[static_cast<uint16_t>(counter)];

            r.lookup_div_u16_0_counts = alu_trace_builder.div_u64_range_chk_counters[0][static_cast<uint16_t>(counter)];
            r.lookup_div_u16_1_counts = alu_trace_builder.div_u64_range_chk_counters[1][static_cast<uint16_t>(counter)];
            r.lookup_div_u16_2_counts = alu_trace_builder.div_u64_range_chk_counters[2][static_cast<uint16_t>(counter)];
            r.lookup_div_u16_3_counts = alu_trace_builder.div_u64_range_chk_counters[3][static_cast<uint16_t>(counter)];
            r.lookup_div_u16_4_counts = alu_trace_builder.div_u64_range_chk_counters[4][static_cast<uint16_t>(counter)];
            r.lookup_div_u16_5_counts = alu_trace_builder.div_u64_range_chk_counters[5][static_cast<uint16_t>(counter)];
            r.lookup_div_u16_6_counts = alu_trace_builder.div_u64_range_chk_counters[6][static_cast<uint16_t>(counter)];
            r.lookup_div_u16_7_counts = alu_trace_builder.div_u64_range_chk_counters[7][static_cast<uint16_t>(counter)];

            r.range_check_l2_gas_hi_counts =
                rem_gas_rng_check_counts[L2_HI_GAS_COUNTS_IDX][static_cast<uint16_t>(counter)];
            r.range_check_l2_gas_lo_counts =
                rem_gas_rng_check_counts[L2_LO_GAS_COUNTS_IDX][static_cast<uint16_t>(counter)];
            r.range_check_da_gas_hi_counts =
                rem_gas_rng_check_counts[DA_HI_GAS_COUNTS_IDX][static_cast<uint16_t>(counter)];
            r.range_check_da_gas_lo_counts =
                rem_gas_rng_check_counts[DA_LO_GAS_COUNTS_IDX][static_cast<uint16_t>(counter)];

            r.avm_main_sel_rng_16 = FF(1);
        }
    }

    // Write the kernel trace into the main trace
    // 1. The write offsets are constrained to be non changing over the entire trace, so we fill in the values until
    // we
    //    hit an operation that changes one of the write_offsets (a relevant opcode)
    // 2. Upon hitting the clk of each kernel operation we copy the values into the main trace
    // 3. When an increment is required, we increment the value in the next row, then continue the process until the
    // end
    // 4. Whenever we hit the last row, we zero all write_offsets such that the shift relation will succeed
    std::vector<AvmKernelTraceBuilder::KernelTraceEntry> kernel_trace = kernel_trace_builder.finalize();
    size_t kernel_padding_main_trace_bottom = 1;

    // Index 1 corresponds here to the first active row of the main execution trace, as
    // we already prepended the extra row for shifted columns. Therefore, initialization
    // of side_effect_counter occurs occurs on this row.
    main_trace.at(1).avm_kernel_side_effect_counter = initial_side_effect_counter;

    // External loop iterates over the kernel entries which are sorted by increasing clk.
    // Internal loop iterates to fill the gap in main trace between each kernel entries.
    for (auto const& src : kernel_trace) {
        // Check the clock and iterate through the main trace until we hit the clock
        auto clk = src.clk;

        // Until the next kernel changing instruction is encountered we set all of the values of the offset arrays
        // to be the same as the previous row This satisfies the `offset' - (offset + operation_selector) = 0`
        // constraints
        for (size_t j = kernel_padding_main_trace_bottom; j < clk; j++) {
            auto const& prev = main_trace.at(j);
            auto& dest = main_trace.at(j + 1);

            dest.avm_kernel_note_hash_exist_write_offset = prev.avm_kernel_note_hash_exist_write_offset;
            dest.avm_kernel_emit_note_hash_write_offset = prev.avm_kernel_emit_note_hash_write_offset;
            dest.avm_kernel_nullifier_exists_write_offset = prev.avm_kernel_nullifier_exists_write_offset;
            dest.avm_kernel_nullifier_non_exists_write_offset = prev.avm_kernel_nullifier_non_exists_write_offset;
            dest.avm_kernel_emit_nullifier_write_offset = prev.avm_kernel_emit_nullifier_write_offset;
            dest.avm_kernel_emit_l2_to_l1_msg_write_offset = prev.avm_kernel_emit_l2_to_l1_msg_write_offset;
            dest.avm_kernel_emit_unencrypted_log_write_offset = prev.avm_kernel_emit_unencrypted_log_write_offset;
            dest.avm_kernel_l1_to_l2_msg_exists_write_offset = prev.avm_kernel_l1_to_l2_msg_exists_write_offset;
            dest.avm_kernel_sload_write_offset = prev.avm_kernel_sload_write_offset;
            dest.avm_kernel_sstore_write_offset = prev.avm_kernel_sstore_write_offset;
            dest.avm_kernel_side_effect_counter = prev.avm_kernel_side_effect_counter;
        }

        Row& curr = main_trace.at(clk);

        // Read in values from kernel trace
        // Lookup values
        curr.avm_kernel_kernel_in_offset = src.kernel_in_offset;
        curr.avm_kernel_kernel_out_offset = src.kernel_out_offset;
        curr.avm_main_q_kernel_lookup = static_cast<uint32_t>(src.q_kernel_lookup);
        curr.avm_main_q_kernel_output_lookup = static_cast<uint32_t>(src.q_kernel_output_lookup);

        // Operation selectors
        curr.avm_main_sel_op_note_hash_exists = static_cast<uint32_t>(src.op_note_hash_exists);
        curr.avm_main_sel_op_emit_note_hash = static_cast<uint32_t>(src.op_emit_note_hash);
        curr.avm_main_sel_op_nullifier_exists = static_cast<uint32_t>(src.op_nullifier_exists);
        curr.avm_main_sel_op_emit_nullifier = static_cast<uint32_t>(src.op_emit_nullifier);
        curr.avm_main_sel_op_l1_to_l2_msg_exists = static_cast<uint32_t>(src.op_l1_to_l2_msg_exists);
        curr.avm_main_sel_op_emit_unencrypted_log = static_cast<uint32_t>(src.op_emit_unencrypted_log);
        curr.avm_main_sel_op_emit_l2_to_l1_msg = static_cast<uint32_t>(src.op_emit_l2_to_l1_msg);
        curr.avm_main_sel_op_sload = static_cast<uint32_t>(src.op_sload);
        curr.avm_main_sel_op_sstore = static_cast<uint32_t>(src.op_sstore);

        if (clk < old_trace_size) {
            Row& next = main_trace.at(clk + 1);

            // Increment the write offset counter for the following row
            next.avm_kernel_note_hash_exist_write_offset =
                curr.avm_kernel_note_hash_exist_write_offset + static_cast<FF>(src.op_note_hash_exists);
            next.avm_kernel_emit_note_hash_write_offset =
                curr.avm_kernel_emit_note_hash_write_offset + static_cast<FF>(src.op_emit_note_hash);
            next.avm_kernel_emit_nullifier_write_offset =
                curr.avm_kernel_emit_nullifier_write_offset + static_cast<FF>(src.op_emit_nullifier);
            next.avm_kernel_nullifier_exists_write_offset =
                curr.avm_kernel_nullifier_exists_write_offset +
                (static_cast<FF>(src.op_nullifier_exists) * curr.avm_main_ib);
            next.avm_kernel_nullifier_non_exists_write_offset =
                curr.avm_kernel_nullifier_non_exists_write_offset +
                (static_cast<FF>(src.op_nullifier_exists) * (FF(1) - curr.avm_main_ib));
            next.avm_kernel_l1_to_l2_msg_exists_write_offset =
                curr.avm_kernel_l1_to_l2_msg_exists_write_offset + static_cast<FF>(src.op_l1_to_l2_msg_exists);
            next.avm_kernel_emit_l2_to_l1_msg_write_offset =
                curr.avm_kernel_emit_l2_to_l1_msg_write_offset + static_cast<FF>(src.op_emit_l2_to_l1_msg);
            next.avm_kernel_emit_unencrypted_log_write_offset =
                curr.avm_kernel_emit_unencrypted_log_write_offset + static_cast<FF>(src.op_emit_unencrypted_log);
            next.avm_kernel_sload_write_offset = curr.avm_kernel_sload_write_offset + static_cast<FF>(src.op_sload);
            next.avm_kernel_sstore_write_offset = curr.avm_kernel_sstore_write_offset + static_cast<FF>(src.op_sstore);

            // The side effect counter will increment regardless of the offset value
            next.avm_kernel_side_effect_counter = curr.avm_kernel_side_effect_counter + 1;
        }

        kernel_padding_main_trace_bottom = clk + 1;
    }

    // Pad out the main trace from the bottom of the main trace until the end
    for (size_t i = kernel_padding_main_trace_bottom + 1; i < old_trace_size; ++i) {

        Row const& prev = main_trace.at(i - 1);
        Row& dest = main_trace.at(i);

        // Setting all of the counters to 0 after the IS_LAST check so we can satisfy the constraints until the end
        if (i == old_trace_size) {
            dest.avm_kernel_note_hash_exist_write_offset = 0;
            dest.avm_kernel_emit_note_hash_write_offset = 0;
            dest.avm_kernel_nullifier_exists_write_offset = 0;
            dest.avm_kernel_nullifier_non_exists_write_offset = 0;
            dest.avm_kernel_emit_nullifier_write_offset = 0;
            dest.avm_kernel_l1_to_l2_msg_exists_write_offset = 0;
            dest.avm_kernel_emit_unencrypted_log_write_offset = 0;
            dest.avm_kernel_emit_l2_to_l1_msg_write_offset = 0;
            dest.avm_kernel_sload_write_offset = 0;
            dest.avm_kernel_sstore_write_offset = 0;
            dest.avm_kernel_side_effect_counter = 0;
        } else {
            dest.avm_kernel_note_hash_exist_write_offset = prev.avm_kernel_note_hash_exist_write_offset;
            dest.avm_kernel_emit_note_hash_write_offset = prev.avm_kernel_emit_note_hash_write_offset;
            dest.avm_kernel_nullifier_exists_write_offset = prev.avm_kernel_nullifier_exists_write_offset;
            dest.avm_kernel_nullifier_non_exists_write_offset = prev.avm_kernel_nullifier_non_exists_write_offset;
            dest.avm_kernel_emit_nullifier_write_offset = prev.avm_kernel_emit_nullifier_write_offset;
            dest.avm_kernel_l1_to_l2_msg_exists_write_offset = prev.avm_kernel_l1_to_l2_msg_exists_write_offset;
            dest.avm_kernel_emit_unencrypted_log_write_offset = prev.avm_kernel_emit_unencrypted_log_write_offset;
            dest.avm_kernel_emit_l2_to_l1_msg_write_offset = prev.avm_kernel_emit_l2_to_l1_msg_write_offset;
            dest.avm_kernel_sload_write_offset = prev.avm_kernel_sload_write_offset;
            dest.avm_kernel_sstore_write_offset = prev.avm_kernel_sstore_write_offset;
            dest.avm_kernel_side_effect_counter = prev.avm_kernel_side_effect_counter;
        }
    }

    // Public Input Columns Inclusion
    // Crucial to add these columns after the extra row was added.

    // Write lookup counts for inputs
    for (uint32_t i = 0; i < KERNEL_INPUTS_LENGTH; i++) {
        auto value = kernel_trace_builder.kernel_input_selector_counter.find(i);
        if (value != kernel_trace_builder.kernel_input_selector_counter.end()) {
            auto& dest = main_trace.at(i);
            dest.lookup_into_kernel_counts = FF(value->second);
            dest.avm_kernel_q_public_input_kernel_add_to_table = FF(1);
        }
    }

    // Copy the kernel input public inputs
    for (size_t i = 0; i < KERNEL_INPUTS_LENGTH; i++) {
        main_trace.at(i).avm_kernel_kernel_inputs = std::get<KERNEL_INPUTS>(kernel_trace_builder.public_inputs).at(i);
    }

    // Write lookup counts for outputs
    for (uint32_t i = 0; i < KERNEL_OUTPUTS_LENGTH; i++) {
        auto value = kernel_trace_builder.kernel_output_selector_counter.find(i);
        if (value != kernel_trace_builder.kernel_output_selector_counter.end()) {
            auto& dest = main_trace.at(i);
            dest.kernel_output_lookup_counts = FF(value->second);
            dest.avm_kernel_q_public_input_kernel_out_add_to_table = FF(1);
        }
    }

    // Copy the kernel outputs counts into the main trace
    for (size_t i = 0; i < KERNEL_OUTPUTS_LENGTH; i++) {
        main_trace.at(i).avm_kernel_kernel_value_out =
            std::get<KERNEL_OUTPUTS_VALUE>(kernel_trace_builder.public_inputs).at(i);

        main_trace.at(i).avm_kernel_kernel_side_effect_out =
            std::get<KERNEL_OUTPUTS_SIDE_EFFECT_COUNTER>(kernel_trace_builder.public_inputs).at(i);

        main_trace.at(i).avm_kernel_kernel_metadata_out =
            std::get<KERNEL_OUTPUTS_METADATA>(kernel_trace_builder.public_inputs).at(i);
    }

    // Get tag_err counts from the mem_trace_builder
    if (range_check_required) {
        finalise_mem_trace_lookup_counts();
    }

    // Add the gas costs table to the main trace
    // For each opcode we write its l2 gas cost and da gas cost
    for (auto const& [opcode, gas_entry] : GAS_COST_TABLE) {
        auto& dest = main_trace.at(static_cast<size_t>(opcode));

        dest.avm_gas_gas_cost_sel = FF(1);
        dest.avm_gas_l2_gas_fixed_table = gas_entry.l2_fixed_gas_cost;
        dest.avm_gas_da_gas_fixed_table = gas_entry.da_fixed_gas_cost;
    }

    // Finalise gas left lookup counts
    for (auto const& [opcode, count] : gas_trace_builder.gas_opcode_lookup_counter) {
        main_trace.at(static_cast<uint8_t>(opcode)).lookup_opcode_gas_counts = count;
    }

    auto trace = std::move(main_trace);
    reset();

    return trace;
}

} // namespace bb::avm_trace
