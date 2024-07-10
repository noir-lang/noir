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
#include "barretenberg/vm/avm_trace/fixed_gas.hpp"
#include "barretenberg/vm/avm_trace/fixed_powers.hpp"
#include "barretenberg/vm/avm_trace/gadgets/avm_slice_trace.hpp"

namespace bb::avm_trace {

/**************************************************************************************************
 *                              HELPERS IN ANONYMOUS NAMESPACE
 **************************************************************************************************/
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
                .main_clk = FF(clk),
                .byte_lookup_sel_bin = FF(1),
                .byte_lookup_table_input_a = a,
                .byte_lookup_table_input_b = b,
                .byte_lookup_table_op_id = op_id,
                .byte_lookup_table_output = bit_op,
                .lookup_byte_operations_counts = count,
            });
        } else {
            main_trace.at(clk).lookup_byte_operations_counts = count;
            main_trace.at(clk).byte_lookup_sel_bin = FF(1);
            main_trace.at(clk).byte_lookup_table_op_id = op_id;
            main_trace.at(clk).byte_lookup_table_input_a = a;
            main_trace.at(clk).byte_lookup_table_input_b = b;
            main_trace.at(clk).byte_lookup_table_output = bit_op;
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
            main_trace.push_back(Row{ .main_clk = FF(clk) });
        }
    }

    return static_cast<uint32_t>(main_trace.size());
}

/**
 * @brief Returns an array of mem_offsets and tags them with their given Addressing Mode (direct/indirect) based on the
 * given indirect byte.
 * @tparam N The number of memory offsets to resolve.
 */
template <size_t N>
std::array<AddressWithMode, N> unpack_indirects(uint8_t indirect, std::array<uint32_t, N> mem_offsets)
{
    std::array<AddressWithMode, N> addr_mode_arr;

    for (size_t i = 0; i < N; i++) {
        // No need to type this as a bool as is implied by the (& 1).
        uint8_t indirect_bit = (indirect >> i) & 1;
        // Cast straight to AddressingMode, saves having to have a branching statement here.
        auto addr_mode = static_cast<AddressingMode>(indirect_bit);
        addr_mode_arr[i] = { addr_mode, mem_offsets[i] };
    }
    return addr_mode_arr;
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
} // anonymous namespace

/**************************************************************************************************
 *                                   HELPERS
 **************************************************************************************************/

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
        .main_clk = clk,
        .main_call_ptr = call_ptr,
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_pc = FF(pc),
        .main_sel_op_halt = FF(1),
    });

    pc = UINT32_MAX; // This ensures that no subsequent opcode will be executed.
}

/**
 * @brief Loads a value from memory into a given intermediate register at a specified clock cycle.
 * Handles both direct and indirect memory access.
 * @tparam reg The intermediate register to load the value into.
 */
AvmTraceBuilder::MemOp AvmTraceBuilder::constrained_read_from_memory(uint8_t space_id,
                                                                     uint32_t clk,
                                                                     AddressWithMode addr,
                                                                     AvmMemoryTag read_tag,
                                                                     AvmMemoryTag write_tag,
                                                                     IntermRegister reg)
{
    // Get the same matching indirect register for the given intermediate register.
    // This is a hack that we can replace with a mapping of IntermediateRegister to IndirectRegister.
    auto indirect_reg = static_cast<IndirectRegister>(reg);
    // Set up direct and indirect offsets that may be overwritten
    uint32_t direct_offset = addr.offset;
    uint32_t indirect_offset = 0;
    bool tag_match = true;
    bool is_indirect = false;
    if (addr.mode == AddressingMode::INDIRECT) {
        is_indirect = true;
        indirect_offset = direct_offset;
        auto read_ind =
            mem_trace_builder.indirect_read_and_load_from_memory(space_id, clk, indirect_reg, indirect_offset);
        if (!read_ind.tag_match) {
            tag_match = false;
        }
        direct_offset = uint32_t(read_ind.val);
    }
    auto read_dir = mem_trace_builder.read_and_load_from_memory(space_id, clk, reg, direct_offset, read_tag, write_tag);

    return MemOp{
        .is_indirect = is_indirect,
        .indirect_address = indirect_offset,
        .direct_address = direct_offset,
        .tag = read_tag,
        .tag_match = tag_match && read_dir.tag_match,
        .val = read_dir.val,
    };
}

/**
 * @brief Writes a value to memory from a given intermediate register at a specified clock cycle.
 * Handles both direct and indirect memory access.
 * @tparam reg The intermediate register to write the value from.
 */
AvmTraceBuilder::MemOp AvmTraceBuilder::constrained_write_to_memory(uint8_t space_id,
                                                                    uint32_t clk,
                                                                    AddressWithMode addr,
                                                                    FF const& value,
                                                                    AvmMemoryTag read_tag,
                                                                    AvmMemoryTag write_tag,
                                                                    IntermRegister reg)
{
    auto indirect_reg = static_cast<IndirectRegister>(reg);
    uint32_t direct_offset = addr.offset;
    uint32_t indirect_offset = 0;
    bool tag_match = true;
    bool is_indirect = false;
    if (addr.mode == AddressingMode::INDIRECT) {
        is_indirect = true;
        indirect_offset = direct_offset;
        auto read_ind =
            mem_trace_builder.indirect_read_and_load_from_memory(space_id, clk, indirect_reg, indirect_offset);
        if (!read_ind.tag_match) {
            tag_match = false;
        }
        direct_offset = uint32_t(read_ind.val);
    }
    mem_trace_builder.write_into_memory(space_id, clk, reg, direct_offset, value, read_tag, write_tag);
    return MemOp{ .is_indirect = is_indirect,
                  .indirect_address = indirect_offset,
                  .direct_address = direct_offset,
                  .tag = write_tag,
                  .tag_match = tag_match,
                  .val = value };
}

// TODO(ilyas: #6383): Temporary way to bulk read slices
template <typename MEM>
uint32_t AvmTraceBuilder::read_slice_to_memory(uint8_t space_id,
                                               uint32_t clk,
                                               AddressWithMode addr,
                                               AvmMemoryTag r_tag,
                                               AvmMemoryTag w_tag,
                                               FF internal_return_ptr,
                                               size_t slice_len,
                                               std::vector<MEM>& slice)
{
    // If the mem_op is indirect, it goes into register A
    bool is_indirect = addr.mode == AddressingMode::INDIRECT;
    auto src_offset = addr.offset;
    // We have 4 registers that we are able to use to read from memory within a single main trace row
    auto register_order = std::array{ IntermRegister::IA, IntermRegister::IB, IntermRegister::IC, IntermRegister::ID };
    // If the slice size isnt a multiple of 4, we still need an extra row to write the remainder
    uint32_t const num_main_rows = static_cast<uint32_t>(slice_len) / 4 + static_cast<uint32_t>(slice_len % 4 != 0);
    for (uint32_t i = 0; i < num_main_rows; i++) {
        Row main_row{
            .main_clk = clk + i,
            .main_internal_return_ptr = FF(internal_return_ptr),
            .main_pc = FF(pc),
            .main_r_in_tag = FF(static_cast<uint32_t>(r_tag)),
            .main_w_in_tag = FF(static_cast<uint32_t>(w_tag)),
        };
        // Write 4 values to memory in each_row
        for (uint32_t j = 0; j < 4; j++) {
            auto offset = i * 4 + j;
            // If we exceed the slice size, we break
            if (offset >= slice_len) {
                break;
            }
            MemOp mem_read;
            if (is_indirect) {
                // If the first address is indirect we read it into register A, this can only happen once per slice read
                mem_read = constrained_read_from_memory(space_id, clk + i, addr, r_tag, w_tag, IntermRegister::IA);
                // Set this to false for the rest of the reads
                is_indirect = false;
                src_offset = mem_read.direct_address;
            } else {
                auto mem_load = mem_trace_builder.read_and_load_from_memory(
                    space_id, clk + i, register_order[j], src_offset + offset, r_tag, w_tag);
                mem_read = MemOp{
                    .is_indirect = false,
                    .indirect_address = 0,
                    .direct_address = src_offset + offset,
                    .tag = r_tag,
                    .tag_match = mem_load.tag_match,
                    .val = MEM(mem_load.val),
                };
            }
            slice.emplace_back(MEM(mem_read.val));
            // This looks a bit gross, but it is fine for now.
            if (j == 0) {
                main_row.main_ia = slice.at(offset);
                main_row.main_ind_addr_a = FF(mem_read.indirect_address);
                main_row.main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(mem_read.is_indirect));
                main_row.main_mem_addr_a = FF(mem_read.direct_address);
                main_row.main_sel_mem_op_a = FF(1);
                main_row.main_tag_err = FF(static_cast<uint32_t>(!mem_read.tag_match));
            } else if (j == 1) {
                main_row.main_ib = slice.at(offset);
                main_row.main_mem_addr_b = FF(mem_read.direct_address);
                main_row.main_sel_mem_op_b = FF(1);
                main_row.main_tag_err = FF(static_cast<uint32_t>(!mem_read.tag_match));
            } else if (j == 2) {
                main_row.main_ic = slice.at(offset);
                main_row.main_mem_addr_c = FF(mem_read.direct_address);
                main_row.main_sel_mem_op_c = FF(1);
                main_row.main_tag_err = FF(static_cast<uint32_t>(!mem_read.tag_match));
            } else {
                main_row.main_id = slice.at(offset);
                main_row.main_mem_addr_d = FF(mem_read.direct_address);
                main_row.main_sel_mem_op_d = FF(1);
                main_row.main_tag_err = FF(static_cast<uint32_t>(!mem_read.tag_match));
            }
        }
        main_trace.emplace_back(main_row);
    }
    return num_main_rows;
}

// TODO(ilyas: #6383): Temporary way to bulk write slices
uint32_t AvmTraceBuilder::write_slice_to_memory(uint8_t space_id,
                                                uint32_t clk,
                                                AddressWithMode addr,
                                                AvmMemoryTag r_tag,
                                                AvmMemoryTag w_tag,
                                                FF internal_return_ptr,
                                                std::vector<FF> const& slice)
{
    bool is_indirect = addr.mode == AddressingMode::INDIRECT;
    auto dst_offset = addr.offset;
    // We have 4 registers that we are able to use to write to memory within a single main trace row
    auto register_order = std::array{ IntermRegister::IA, IntermRegister::IB, IntermRegister::IC, IntermRegister::ID };
    // If the slice size isnt a multiple of 4, we still need an extra row to write the remainder
    uint32_t const num_main_rows =
        static_cast<uint32_t>(slice.size()) / 4 + static_cast<uint32_t>(slice.size() % 4 != 0);
    for (uint32_t i = 0; i < num_main_rows; i++) {
        Row main_row{
            .main_clk = clk + i,
            .main_internal_return_ptr = FF(internal_return_ptr),
            .main_pc = FF(pc),
            .main_r_in_tag = FF(static_cast<uint32_t>(r_tag)),
            .main_w_in_tag = FF(static_cast<uint32_t>(w_tag)),
        };
        // Write 4 values to memory in each_row
        for (uint32_t j = 0; j < 4; j++) {
            auto offset = i * 4 + j;
            // If we exceed the slice size, we break
            if (offset >= slice.size()) {
                break;
            }
            MemOp mem_write;
            if (is_indirect) {
                mem_write = constrained_write_to_memory(
                    space_id, clk + i, addr, slice.at(offset), r_tag, w_tag, IntermRegister::IA);
                // Ensure futures calls are direct
                is_indirect = false;
                dst_offset = mem_write.direct_address;
            } else {
                mem_trace_builder.write_into_memory(
                    space_id, clk + i, register_order[j], dst_offset + offset, slice.at(offset), r_tag, w_tag);
                mem_write = MemOp{
                    .is_indirect = false,
                    .indirect_address = 0,
                    .direct_address = dst_offset + offset,
                    .tag = w_tag,
                    .tag_match = true,
                    .val = slice.at(offset),
                };
            }
            // This looks a bit gross, but it is fine for now.
            if (j == 0) {
                main_row.main_ia = slice.at(offset);
                main_row.main_ind_addr_a = FF(mem_write.indirect_address);
                main_row.main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(mem_write.is_indirect));
                main_row.main_mem_addr_a = FF(mem_write.direct_address);
                main_row.main_sel_mem_op_a = FF(1);
                main_row.main_rwa = FF(1);
            } else if (j == 1) {
                main_row.main_ib = slice.at(offset);
                main_row.main_mem_addr_b = FF(mem_write.direct_address);
                main_row.main_sel_mem_op_b = FF(1);
                main_row.main_rwb = FF(1);
            } else if (j == 2) {
                main_row.main_ic = slice.at(offset);
                main_row.main_mem_addr_c = FF(mem_write.direct_address);
                main_row.main_sel_mem_op_c = FF(1);
                main_row.main_rwc = FF(1);
            } else {
                main_row.main_id = slice.at(offset);
                main_row.main_mem_addr_d = FF(mem_write.direct_address);
                main_row.main_sel_mem_op_d = FF(1);
                main_row.main_rwd = FF(1);
            }
        }
        main_trace.emplace_back(main_row);
    }
    return num_main_rows;
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

/**
 * @brief Constructor of a trace builder of AVM. Only serves to set the capacity of the
 *        underlying traces and initialize gas values.
 */
AvmTraceBuilder::AvmTraceBuilder(VmPublicInputs public_inputs,
                                 ExecutionHints execution_hints,
                                 uint32_t side_effect_counter,
                                 std::vector<FF> calldata)
    // NOTE: we initialise the environment builder here as it requires public inputs
    : kernel_trace_builder(std::move(public_inputs))
    , calldata(std::move(calldata))
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

/**************************************************************************************************
 *                            COMPUTE - ARITHMETIC
 **************************************************************************************************/

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

    // Resolve any potential indirects in the order they are encoded in the indirect byte.
    auto [resolved_a, resolved_b, resolved_c] = unpack_indirects<3>(indirect, { a_offset, b_offset, dst_offset });

    // Reading from memory and loading into ia resp. ib.
    auto read_a = constrained_read_from_memory(call_ptr, clk, resolved_a, in_tag, in_tag, IntermRegister::IA);
    auto read_b = constrained_read_from_memory(call_ptr, clk, resolved_b, in_tag, in_tag, IntermRegister::IB);

    bool tag_match = read_a.tag_match && read_b.tag_match;

    // a + b = c
    FF a = read_a.val;
    FF b = read_b.val;

    // In case of a memory tag error, we do not perform the computation.
    // Therefore, we do not create any entry in ALU table and store the value 0 as
    // output (c) in memory.
    FF c = tag_match ? alu_trace_builder.op_add(a, b, in_tag, clk) : FF(0);

    // Write into memory value c from intermediate register ic.
    auto write_c = constrained_write_to_memory(call_ptr, clk, resolved_c, c, in_tag, in_tag, IntermRegister::IC);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::ADD);

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_alu_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .main_call_ptr = call_ptr,
        .main_ia = read_a.val,
        .main_ib = read_b.val,
        .main_ic = write_c.val,
        .main_ind_addr_a = FF(read_a.indirect_address),
        .main_ind_addr_b = FF(read_b.indirect_address),
        .main_ind_addr_c = FF(write_c.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(read_a.direct_address),
        .main_mem_addr_b = FF(read_b.direct_address),
        .main_mem_addr_c = FF(write_c.direct_address),
        .main_pc = FF(pc++),
        .main_r_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .main_rwc = FF(1),
        .main_sel_mem_op_a = FF(1),
        .main_sel_mem_op_b = FF(1),
        .main_sel_mem_op_c = FF(1),
        .main_sel_op_add = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(read_a.is_indirect)),
        .main_sel_resolve_ind_addr_b = FF(static_cast<uint32_t>(read_b.is_indirect)),
        .main_sel_resolve_ind_addr_c = FF(static_cast<uint32_t>(write_c.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .main_w_in_tag = FF(static_cast<uint32_t>(in_tag)),
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

    // Resolve any potential indirects in the order they are encoded in the indirect byte.
    auto [resolved_a, resolved_b, resolved_c] = unpack_indirects<3>(indirect, { a_offset, b_offset, dst_offset });

    // Reading from memory and loading into ia resp. ib.
    auto read_a = constrained_read_from_memory(call_ptr, clk, resolved_a, in_tag, in_tag, IntermRegister::IA);
    auto read_b = constrained_read_from_memory(call_ptr, clk, resolved_b, in_tag, in_tag, IntermRegister::IB);

    bool tag_match = read_a.tag_match && read_b.tag_match;

    // a - b = c
    FF a = read_a.val;
    FF b = read_b.val;

    // In case of a memory tag error, we do not perform the computation.
    // Therefore, we do not create any entry in ALU table and store the value 0 as
    // output (c) in memory.
    FF c = tag_match ? alu_trace_builder.op_sub(a, b, in_tag, clk) : FF(0);

    // Write into memory value c from intermediate register ic.
    auto write_c = constrained_write_to_memory(call_ptr, clk, resolved_c, c, in_tag, in_tag, IntermRegister::IC);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::SUB);

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_alu_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .main_call_ptr = call_ptr,
        .main_ia = read_a.val,
        .main_ib = read_b.val,
        .main_ic = write_c.val,
        .main_ind_addr_a = FF(read_a.indirect_address),
        .main_ind_addr_b = FF(read_b.indirect_address),
        .main_ind_addr_c = FF(write_c.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(read_a.direct_address),
        .main_mem_addr_b = FF(read_b.direct_address),
        .main_mem_addr_c = FF(write_c.direct_address),
        .main_pc = FF(pc++),
        .main_r_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .main_rwc = FF(1),
        .main_sel_mem_op_a = FF(1),
        .main_sel_mem_op_b = FF(1),
        .main_sel_mem_op_c = FF(1),
        .main_sel_op_sub = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(read_a.is_indirect)),
        .main_sel_resolve_ind_addr_b = FF(static_cast<uint32_t>(read_b.is_indirect)),
        .main_sel_resolve_ind_addr_c = FF(static_cast<uint32_t>(write_c.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .main_w_in_tag = FF(static_cast<uint32_t>(in_tag)),
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

    // Resolve any potential indirects in the order they are encoded in the indirect byte.
    auto [resolved_a, resolved_b, resolved_c] = unpack_indirects<3>(indirect, { a_offset, b_offset, dst_offset });

    // Reading from memory and loading into ia resp. ib.
    auto read_a = constrained_read_from_memory(call_ptr, clk, resolved_a, in_tag, in_tag, IntermRegister::IA);
    auto read_b = constrained_read_from_memory(call_ptr, clk, resolved_b, in_tag, in_tag, IntermRegister::IB);

    bool tag_match = read_a.tag_match && read_b.tag_match;

    // a * b = c
    FF a = read_a.val;
    FF b = read_b.val;

    // In case of a memory tag error, we do not perform the computation.
    // Therefore, we do not create any entry in ALU table and store the value 0 as
    // output (c) in memory.
    FF c = tag_match ? alu_trace_builder.op_mul(a, b, in_tag, clk) : FF(0);

    // Write into memory value c from intermediate register ic.
    auto write_c = constrained_write_to_memory(call_ptr, clk, resolved_c, c, in_tag, in_tag, IntermRegister::IC);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::MUL);

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_alu_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .main_call_ptr = call_ptr,
        .main_ia = read_a.val,
        .main_ib = read_b.val,
        .main_ic = write_c.val,
        .main_ind_addr_a = FF(read_a.indirect_address),
        .main_ind_addr_b = FF(read_b.indirect_address),
        .main_ind_addr_c = FF(write_c.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(read_a.direct_address),
        .main_mem_addr_b = FF(read_b.direct_address),
        .main_mem_addr_c = FF(write_c.direct_address),
        .main_pc = FF(pc++),
        .main_r_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .main_rwc = FF(1),
        .main_sel_mem_op_a = FF(1),
        .main_sel_mem_op_b = FF(1),
        .main_sel_mem_op_c = FF(1),
        .main_sel_op_mul = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(read_a.is_indirect)),
        .main_sel_resolve_ind_addr_b = FF(static_cast<uint32_t>(read_b.is_indirect)),
        .main_sel_resolve_ind_addr_c = FF(static_cast<uint32_t>(write_c.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .main_w_in_tag = FF(static_cast<uint32_t>(in_tag)),
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

    auto [resolved_a, resolved_b, resolved_dst] = unpack_indirects<3>(indirect, { a_offset, b_offset, dst_offset });

    // Reading from memory and loading into ia resp. ib.
    auto read_a = constrained_read_from_memory(call_ptr, clk, resolved_a, in_tag, in_tag, IntermRegister::IA);
    auto read_b = constrained_read_from_memory(call_ptr, clk, resolved_b, in_tag, in_tag, IntermRegister::IB);
    bool tag_match = read_a.tag_match && read_b.tag_match;

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
    auto write_dst = constrained_write_to_memory(call_ptr, clk, resolved_dst, c, in_tag, in_tag, IntermRegister::IC);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::DIV);

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_alu_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .main_call_ptr = call_ptr,
        .main_ia = read_a.val,
        .main_ib = read_b.val,
        .main_ic = c,
        .main_ind_addr_a = FF(read_a.indirect_address),
        .main_ind_addr_b = FF(read_b.indirect_address),
        .main_ind_addr_c = FF(write_dst.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_inv = tag_match ? inv : FF(1),
        .main_mem_addr_a = FF(read_a.direct_address),
        .main_mem_addr_b = FF(read_b.direct_address),
        .main_mem_addr_c = FF(write_dst.direct_address),
        .main_op_err = tag_match ? error : FF(1),
        .main_pc = FF(pc++),
        .main_r_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .main_rwc = FF(1),
        .main_sel_mem_op_a = FF(1),
        .main_sel_mem_op_b = FF(1),
        .main_sel_mem_op_c = FF(1),
        .main_sel_op_div = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(read_a.is_indirect)),
        .main_sel_resolve_ind_addr_b = FF(static_cast<uint32_t>(read_b.is_indirect)),
        .main_sel_resolve_ind_addr_c = FF(static_cast<uint32_t>(write_dst.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .main_w_in_tag = FF(static_cast<uint32_t>(in_tag)),
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

    // Resolve any potential indirects in the order they are encoded in the indirect byte.
    auto [resolved_a, resolved_b, resolved_c] = unpack_indirects<3>(indirect, { a_offset, b_offset, dst_offset });

    // Reading from memory and loading into ia resp. ib.
    auto read_a =
        constrained_read_from_memory(call_ptr, clk, resolved_a, AvmMemoryTag::FF, AvmMemoryTag::FF, IntermRegister::IA);
    auto read_b =
        constrained_read_from_memory(call_ptr, clk, resolved_b, AvmMemoryTag::FF, AvmMemoryTag::FF, IntermRegister::IB);

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
    auto write_c = constrained_write_to_memory(
        call_ptr, clk, resolved_c, c, AvmMemoryTag::FF, AvmMemoryTag::FF, IntermRegister::IC);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::FDIV);

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_call_ptr = call_ptr,
        .main_ia = tag_match ? read_a.val : FF(0),
        .main_ib = tag_match ? read_b.val : FF(0),
        .main_ic = tag_match ? write_c.val : FF(0),
        .main_ind_addr_a = FF(read_a.indirect_address),
        .main_ind_addr_b = FF(read_b.indirect_address),
        .main_ind_addr_c = FF(write_c.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_inv = tag_match ? inv : FF(1),
        .main_mem_addr_a = FF(read_a.direct_address),
        .main_mem_addr_b = FF(read_b.direct_address),
        .main_mem_addr_c = FF(write_c.direct_address),
        .main_op_err = tag_match ? error : FF(1),
        .main_pc = FF(pc++),
        .main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
        .main_rwc = FF(1),
        .main_sel_mem_op_a = FF(1),
        .main_sel_mem_op_b = FF(1),
        .main_sel_mem_op_c = FF(1),
        .main_sel_op_fdiv = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(read_a.is_indirect)),
        .main_sel_resolve_ind_addr_b = FF(static_cast<uint32_t>(read_b.is_indirect)),
        .main_sel_resolve_ind_addr_c = FF(static_cast<uint32_t>(write_c.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
    });
}

/**************************************************************************************************
 *                            COMPUTE - COMPARATORS
 **************************************************************************************************/

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

    auto [resolved_a, resolved_b, resolved_c] = unpack_indirects<3>(indirect, { a_offset, b_offset, dst_offset });

    // Reading from memory and loading into ia resp. ib.
    auto read_a = constrained_read_from_memory(call_ptr, clk, resolved_a, in_tag, AvmMemoryTag::U8, IntermRegister::IA);
    auto read_b = constrained_read_from_memory(call_ptr, clk, resolved_b, in_tag, AvmMemoryTag::U8, IntermRegister::IB);
    bool tag_match = read_a.tag_match && read_b.tag_match;

    FF a = read_a.val;
    FF b = read_b.val;

    // In case of a memory tag error, we do not perform the computation.
    // Therefore, we do not create any entry in ALU table and store the value 0 as
    // output (c) in memory.
    FF c = tag_match ? alu_trace_builder.op_eq(a, b, in_tag, clk) : FF(0);

    // Write into memory value c from intermediate register ic.
    auto write_c =
        constrained_write_to_memory(call_ptr, clk, resolved_c, c, in_tag, AvmMemoryTag::U8, IntermRegister::IC);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::EQ);

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_alu_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .main_call_ptr = call_ptr,
        .main_ia = read_a.val,
        .main_ib = read_b.val,
        .main_ic = write_c.val,
        .main_ind_addr_a = FF(read_a.indirect_address),
        .main_ind_addr_b = FF(read_b.indirect_address),
        .main_ind_addr_c = FF(write_c.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(read_a.direct_address),
        .main_mem_addr_b = FF(read_b.direct_address),
        .main_mem_addr_c = FF(write_c.direct_address),
        .main_pc = FF(pc++),
        .main_r_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .main_rwc = FF(1),
        .main_sel_mem_op_a = FF(1),
        .main_sel_mem_op_b = FF(1),
        .main_sel_mem_op_c = FF(1),
        .main_sel_op_eq = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(read_a.is_indirect)),
        .main_sel_resolve_ind_addr_b = FF(static_cast<uint32_t>(read_b.is_indirect)),
        .main_sel_resolve_ind_addr_c = FF(static_cast<uint32_t>(write_c.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U8)),
    });
}

void AvmTraceBuilder::op_lt(
    uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    auto [resolved_a, resolved_b, resolved_c] = unpack_indirects<3>(indirect, { a_offset, b_offset, dst_offset });

    auto read_a = constrained_read_from_memory(call_ptr, clk, resolved_a, in_tag, AvmMemoryTag::U8, IntermRegister::IA);
    auto read_b = constrained_read_from_memory(call_ptr, clk, resolved_b, in_tag, AvmMemoryTag::U8, IntermRegister::IB);
    bool tag_match = read_a.tag_match && read_b.tag_match;

    FF a = tag_match ? read_a.val : FF(0);
    FF b = tag_match ? read_b.val : FF(0);

    FF c = tag_match ? alu_trace_builder.op_lt(a, b, in_tag, clk) : FF(0);

    // Write into memory value c from intermediate register ic.
    auto write_c =
        constrained_write_to_memory(call_ptr, clk, resolved_c, c, in_tag, AvmMemoryTag::U8, IntermRegister::IC);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::LT);

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_alu_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .main_call_ptr = call_ptr,
        .main_ia = read_a.val,
        .main_ib = read_b.val,
        .main_ic = write_c.val,
        .main_ind_addr_a = FF(read_a.indirect_address),
        .main_ind_addr_b = FF(read_b.indirect_address),
        .main_ind_addr_c = FF(write_c.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(read_a.direct_address),
        .main_mem_addr_b = FF(read_b.direct_address),
        .main_mem_addr_c = FF(write_c.direct_address),
        .main_pc = FF(pc++),
        .main_r_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .main_rwc = FF(1),
        .main_sel_mem_op_a = FF(1),
        .main_sel_mem_op_b = FF(1),
        .main_sel_mem_op_c = FF(1),
        .main_sel_op_lt = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(read_a.is_indirect)),
        .main_sel_resolve_ind_addr_b = FF(static_cast<uint32_t>(read_b.is_indirect)),
        .main_sel_resolve_ind_addr_c = FF(static_cast<uint32_t>(write_c.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U8)),
    });
}

void AvmTraceBuilder::op_lte(
    uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    auto [resolved_a, resolved_b, resolved_c] = unpack_indirects<3>(indirect, { a_offset, b_offset, dst_offset });

    // Reading from memory and loading into ia resp. ib.
    auto read_a = constrained_read_from_memory(call_ptr, clk, resolved_a, in_tag, AvmMemoryTag::U8, IntermRegister::IA);
    auto read_b = constrained_read_from_memory(call_ptr, clk, resolved_b, in_tag, AvmMemoryTag::U8, IntermRegister::IB);
    bool tag_match = read_a.tag_match && read_b.tag_match;

    FF a = tag_match ? read_a.val : FF(0);
    FF b = tag_match ? read_b.val : FF(0);

    FF c = tag_match ? alu_trace_builder.op_lte(a, b, in_tag, clk) : FF(0);

    // Write into memory value c from intermediate register ic.
    auto write_c =
        constrained_write_to_memory(call_ptr, clk, resolved_c, c, in_tag, AvmMemoryTag::U8, IntermRegister::IC);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::LTE);

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_alu_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .main_call_ptr = call_ptr,
        .main_ia = read_a.val,
        .main_ib = read_b.val,
        .main_ic = write_c.val,
        .main_ind_addr_a = FF(read_a.indirect_address),
        .main_ind_addr_b = FF(read_b.indirect_address),
        .main_ind_addr_c = FF(write_c.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(read_a.direct_address),
        .main_mem_addr_b = FF(read_b.direct_address),
        .main_mem_addr_c = FF(write_c.direct_address),
        .main_pc = FF(pc++),
        .main_r_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .main_rwc = FF(1),
        .main_sel_mem_op_a = FF(1),
        .main_sel_mem_op_b = FF(1),
        .main_sel_mem_op_c = FF(1),
        .main_sel_op_lte = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(read_a.is_indirect)),
        .main_sel_resolve_ind_addr_b = FF(static_cast<uint32_t>(read_b.is_indirect)),
        .main_sel_resolve_ind_addr_c = FF(static_cast<uint32_t>(write_c.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U8)),
    });
}

/**************************************************************************************************
 *                            COMPUTE - BITWISE
 **************************************************************************************************/

void AvmTraceBuilder::op_and(
    uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    auto [resolved_a, resolved_b, resolved_c] = unpack_indirects<3>(indirect, { a_offset, b_offset, dst_offset });

    // Reading from memory and loading into ia resp. ib.
    auto read_a = constrained_read_from_memory(call_ptr, clk, resolved_a, in_tag, in_tag, IntermRegister::IA);
    auto read_b = constrained_read_from_memory(call_ptr, clk, resolved_b, in_tag, in_tag, IntermRegister::IB);
    bool tag_match = read_a.tag_match && read_b.tag_match;

    FF a = tag_match ? read_a.val : FF(0);
    FF b = tag_match ? read_b.val : FF(0);

    FF c = tag_match ? bin_trace_builder.op_and(a, b, in_tag, clk) : FF(0);

    // Write into memory value c from intermediate register ic.
    auto write_c = constrained_write_to_memory(call_ptr, clk, resolved_c, c, in_tag, in_tag, IntermRegister::IC);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::AND);

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_bin_op_id = FF(0),
        .main_call_ptr = call_ptr,
        .main_ia = read_a.val,
        .main_ib = read_b.val,
        .main_ic = write_c.val,
        .main_ind_addr_a = FF(read_a.indirect_address),
        .main_ind_addr_b = FF(read_b.indirect_address),
        .main_ind_addr_c = FF(write_c.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(read_a.direct_address),
        .main_mem_addr_b = FF(read_b.direct_address),
        .main_mem_addr_c = FF(write_c.direct_address),
        .main_pc = FF(pc++),
        .main_r_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .main_rwc = FF(1),
        .main_sel_bin = FF(1),
        .main_sel_mem_op_a = FF(1),
        .main_sel_mem_op_b = FF(1),
        .main_sel_mem_op_c = FF(1),
        .main_sel_op_and = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(read_a.is_indirect)),
        .main_sel_resolve_ind_addr_b = FF(static_cast<uint32_t>(read_b.is_indirect)),
        .main_sel_resolve_ind_addr_c = FF(static_cast<uint32_t>(write_c.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .main_w_in_tag = FF(static_cast<uint32_t>(in_tag)),
    });
}

void AvmTraceBuilder::op_or(
    uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;
    auto [resolved_a, resolved_b, resolved_c] = unpack_indirects<3>(indirect, { a_offset, b_offset, dst_offset });

    // Reading from memory and loading into ia resp. ib.
    auto read_a = constrained_read_from_memory(call_ptr, clk, resolved_a, in_tag, in_tag, IntermRegister::IA);
    auto read_b = constrained_read_from_memory(call_ptr, clk, resolved_b, in_tag, in_tag, IntermRegister::IB);
    bool tag_match = read_a.tag_match && read_b.tag_match;

    FF a = tag_match ? read_a.val : FF(0);
    FF b = tag_match ? read_b.val : FF(0);

    FF c = tag_match ? bin_trace_builder.op_or(a, b, in_tag, clk) : FF(0);

    // Write into memory value c from intermediate register ic.
    auto write_c = constrained_write_to_memory(call_ptr, clk, resolved_c, c, in_tag, in_tag, IntermRegister::IC);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::OR);

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_bin_op_id = FF(1),
        .main_call_ptr = call_ptr,
        .main_ia = read_a.val,
        .main_ib = read_b.val,
        .main_ic = write_c.val,
        .main_ind_addr_a = FF(read_a.indirect_address),
        .main_ind_addr_b = FF(read_b.indirect_address),
        .main_ind_addr_c = FF(write_c.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(read_a.direct_address),
        .main_mem_addr_b = FF(read_b.direct_address),
        .main_mem_addr_c = FF(write_c.direct_address),
        .main_pc = FF(pc++),
        .main_r_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .main_rwc = FF(1),
        .main_sel_bin = FF(1),
        .main_sel_mem_op_a = FF(1),
        .main_sel_mem_op_b = FF(1),
        .main_sel_mem_op_c = FF(1),
        .main_sel_op_or = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(read_a.is_indirect)),
        .main_sel_resolve_ind_addr_b = FF(static_cast<uint32_t>(read_b.is_indirect)),
        .main_sel_resolve_ind_addr_c = FF(static_cast<uint32_t>(write_c.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .main_w_in_tag = FF(static_cast<uint32_t>(in_tag)),
    });
}

void AvmTraceBuilder::op_xor(
    uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    auto [resolved_a, resolved_b, resolved_c] = unpack_indirects<3>(indirect, { a_offset, b_offset, dst_offset });

    // Reading from memory and loading into ia resp. ib.
    auto read_a = constrained_read_from_memory(call_ptr, clk, resolved_a, in_tag, in_tag, IntermRegister::IA);
    auto read_b = constrained_read_from_memory(call_ptr, clk, resolved_b, in_tag, in_tag, IntermRegister::IB);
    bool tag_match = read_a.tag_match && read_b.tag_match;

    FF a = tag_match ? read_a.val : FF(0);
    FF b = tag_match ? read_b.val : FF(0);

    FF c = tag_match ? bin_trace_builder.op_xor(a, b, in_tag, clk) : FF(0);

    // Write into memory value c from intermediate register ic.
    auto write_c = constrained_write_to_memory(call_ptr, clk, resolved_c, c, in_tag, in_tag, IntermRegister::IC);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::XOR);

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_bin_op_id = FF(2),
        .main_call_ptr = call_ptr,
        .main_ia = read_a.val,
        .main_ib = read_b.val,
        .main_ic = write_c.val,
        .main_ind_addr_a = FF(read_a.indirect_address),
        .main_ind_addr_b = FF(read_b.indirect_address),
        .main_ind_addr_c = FF(write_c.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(read_a.direct_address),
        .main_mem_addr_b = FF(read_b.direct_address),
        .main_mem_addr_c = FF(write_c.direct_address),
        .main_pc = FF(pc++),
        .main_r_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .main_rwc = FF(1),
        .main_sel_bin = FF(1),
        .main_sel_mem_op_a = FF(1),
        .main_sel_mem_op_b = FF(1),
        .main_sel_mem_op_c = FF(1),
        .main_sel_op_xor = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(read_a.is_indirect)),
        .main_sel_resolve_ind_addr_b = FF(static_cast<uint32_t>(read_b.is_indirect)),
        .main_sel_resolve_ind_addr_c = FF(static_cast<uint32_t>(write_c.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .main_w_in_tag = FF(static_cast<uint32_t>(in_tag)),
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

    // Resolve any potential indirects in the order they are encoded in the indirect byte.
    auto [resolved_a, resolved_c] = unpack_indirects<2>(indirect, { a_offset, dst_offset });

    // Reading from memory and loading into ia
    auto read_a = constrained_read_from_memory(call_ptr, clk, resolved_a, in_tag, in_tag, IntermRegister::IA);

    bool tag_match = read_a.tag_match;
    // ~a = c
    FF a = read_a.val;

    // In case of a memory tag error, we do not perform the computation.
    // Therefore, we do not create any entry in ALU table and store the value 0 as
    // output (c) in memory.
    FF c = tag_match ? alu_trace_builder.op_not(a, in_tag, clk) : FF(0);

    // Write into memory value c from intermediate register ic.
    auto write_c = constrained_write_to_memory(call_ptr, clk, resolved_c, c, in_tag, in_tag, IntermRegister::IC);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::NOT);

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_alu_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .main_call_ptr = call_ptr,
        .main_ia = read_a.val,
        .main_ic = write_c.val,
        .main_ind_addr_a = FF(read_a.indirect_address),
        .main_ind_addr_c = FF(write_c.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(read_a.direct_address),
        .main_mem_addr_c = FF(write_c.direct_address),
        .main_pc = FF(pc++),
        .main_r_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .main_rwc = FF(1),
        .main_sel_mem_op_a = FF(1),
        .main_sel_mem_op_c = FF(1),
        .main_sel_op_not = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(read_a.is_indirect)),
        .main_sel_resolve_ind_addr_c = FF(static_cast<uint32_t>(write_c.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!read_a.tag_match)),
        .main_w_in_tag = FF(static_cast<uint32_t>(in_tag)),
    });
}

void AvmTraceBuilder::op_shl(
    uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    auto [resolved_a, resolved_b, resolved_c] = unpack_indirects<3>(indirect, { a_offset, b_offset, dst_offset });

    // Reading from memory and loading into ia resp. ib.
    auto read_a = constrained_read_from_memory(call_ptr, clk, resolved_a, in_tag, in_tag, IntermRegister::IA);
    auto read_b = constrained_read_from_memory(call_ptr, clk, resolved_b, in_tag, in_tag, IntermRegister::IB);
    bool tag_match = read_a.tag_match && read_b.tag_match;

    FF a = tag_match ? read_a.val : FF(0);
    FF b = tag_match ? read_b.val : FF(0);

    FF c = tag_match ? alu_trace_builder.op_shl(a, b, in_tag, clk) : FF(0);

    // Write into memory value c from intermediate register ic.
    auto write_c = constrained_write_to_memory(call_ptr, clk, resolved_c, c, in_tag, in_tag, IntermRegister::IC);
    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::SHL);

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_alu_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .main_call_ptr = call_ptr,
        .main_ia = read_a.val,
        .main_ib = read_b.val,
        .main_ic = write_c.val,
        .main_ind_addr_a = FF(read_a.indirect_address),
        .main_ind_addr_b = FF(read_b.indirect_address),
        .main_ind_addr_c = FF(write_c.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(read_a.direct_address),
        .main_mem_addr_b = FF(read_b.direct_address),
        .main_mem_addr_c = FF(write_c.direct_address),
        .main_pc = FF(pc++),
        .main_r_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .main_rwc = FF(1),
        .main_sel_mem_op_a = FF(1),
        .main_sel_mem_op_b = FF(1),
        .main_sel_mem_op_c = FF(1),
        .main_sel_op_shl = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(read_a.is_indirect)),
        .main_sel_resolve_ind_addr_b = FF(static_cast<uint32_t>(read_b.is_indirect)),
        .main_sel_resolve_ind_addr_c = FF(static_cast<uint32_t>(write_c.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .main_w_in_tag = FF(static_cast<uint32_t>(in_tag)),
    });
}

void AvmTraceBuilder::op_shr(
    uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag)
{

    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    auto [resolved_a, resolved_b, resolved_c] = unpack_indirects<3>(indirect, { a_offset, b_offset, dst_offset });

    // Reading from memory and loading into ia resp. ib.
    auto read_a = constrained_read_from_memory(call_ptr, clk, resolved_a, in_tag, in_tag, IntermRegister::IA);
    auto read_b = constrained_read_from_memory(call_ptr, clk, resolved_b, in_tag, in_tag, IntermRegister::IB);
    bool tag_match = read_a.tag_match && read_b.tag_match;

    FF a = tag_match ? read_a.val : FF(0);
    FF b = tag_match ? read_b.val : FF(0);

    FF c = tag_match ? alu_trace_builder.op_shr(a, b, in_tag, clk) : FF(0);

    // Write into memory value c from intermediate register ic.
    auto write_c = constrained_write_to_memory(call_ptr, clk, resolved_c, c, in_tag, in_tag, IntermRegister::IC);
    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::SHR);

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_alu_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .main_call_ptr = call_ptr,
        .main_ia = read_a.val,
        .main_ib = read_b.val,
        .main_ic = write_c.val,
        .main_ind_addr_a = FF(read_a.indirect_address),
        .main_ind_addr_b = FF(read_b.indirect_address),
        .main_ind_addr_c = FF(write_c.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(read_a.direct_address),
        .main_mem_addr_b = FF(read_b.direct_address),
        .main_mem_addr_c = FF(write_c.direct_address),
        .main_pc = FF(pc++),
        .main_r_in_tag = FF(static_cast<uint32_t>(in_tag)),
        .main_rwc = FF(1),
        .main_sel_mem_op_a = FF(1),
        .main_sel_mem_op_b = FF(1),
        .main_sel_mem_op_c = FF(1),
        .main_sel_op_shr = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(read_a.is_indirect)),
        .main_sel_resolve_ind_addr_b = FF(static_cast<uint32_t>(read_b.is_indirect)),
        .main_sel_resolve_ind_addr_c = FF(static_cast<uint32_t>(write_c.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .main_w_in_tag = FF(static_cast<uint32_t>(in_tag)),
    });
}

/**************************************************************************************************
 *                            COMPUTE - TYPE CONVERSIONS
 **************************************************************************************************/

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
        .main_clk = clk,
        .main_alu_in_tag = FF(static_cast<uint32_t>(dst_tag)),
        .main_call_ptr = call_ptr,
        .main_ia = a,
        .main_ic = c,
        .main_ind_addr_a = indirect_a_flag ? FF(a_offset) : FF(0),
        .main_ind_addr_c = indirect_dst_flag ? FF(dst_offset) : FF(0),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(direct_a_offset),
        .main_mem_addr_c = FF(direct_dst_offset),
        .main_pc = FF(pc++),
        .main_r_in_tag = FF(static_cast<uint32_t>(memEntry.tag)),
        .main_rwc = FF(1),
        .main_sel_mem_op_a = FF(1),
        .main_sel_mem_op_c = FF(1),
        .main_sel_op_cast = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(indirect_a_flag)),
        .main_sel_resolve_ind_addr_c = FF(static_cast<uint32_t>(indirect_dst_flag)),
        .main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        .main_w_in_tag = FF(static_cast<uint32_t>(dst_tag)),
    });
}

/**************************************************************************************************
 *                            EXECUTION ENVIRONMENT
 **************************************************************************************************/

// Helper function to add kernel lookup operations into the main trace
// TODO: add tag match to kernel_input_lookup opcodes to - it isnt written to - -ve test would catch
/**
 * @brief Create a kernel lookup opcode object
 *
 * Used for looking up into the kernel inputs (context) - {caller, address, etc.}
 *
 * @param indirect - Perform indirect memory resolution
 * @param dst_offset - Memory address to write the lookup result to
 * @param selector - The index of the kernel input lookup column
 * @param value - The value read from the memory address
 * @param w_tag - The memory tag of the value read
 * @return Row
 */
Row AvmTraceBuilder::create_kernel_lookup_opcode(
    uint8_t indirect, uint32_t dst_offset, uint32_t selector, FF value, AvmMemoryTag w_tag)
{
    auto const clk = static_cast<uint32_t>(main_trace.size()) + 1;

    auto [resolved_dst] = unpack_indirects<1>(indirect, { dst_offset });
    auto write_dst =
        constrained_write_to_memory(call_ptr, clk, resolved_dst, value, AvmMemoryTag::U0, w_tag, IntermRegister::IA);

    return Row{
        .main_clk = clk,
        .kernel_kernel_in_offset = selector,
        .main_call_ptr = call_ptr,
        .main_ia = value,
        .main_ind_addr_a = FF(write_dst.indirect_address),
        .main_internal_return_ptr = internal_return_ptr,
        .main_mem_addr_a = FF(write_dst.direct_address),
        .main_pc = pc++,
        .main_rwa = 1,
        .main_sel_mem_op_a = 1,
        .main_sel_q_kernel_lookup = 1,
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(write_dst.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!write_dst.tag_match)),
        .main_w_in_tag = static_cast<uint32_t>(w_tag),
    };
}

void AvmTraceBuilder::op_address(uint8_t indirect, uint32_t dst_offset)
{
    FF ia_value = kernel_trace_builder.op_address();
    Row row = create_kernel_lookup_opcode(indirect, dst_offset, ADDRESS_SELECTOR, ia_value, AvmMemoryTag::FF);
    row.main_sel_op_address = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(static_cast<uint32_t>(row.main_clk), OpCode::ADDRESS);

    main_trace.push_back(row);
}

void AvmTraceBuilder::op_storage_address(uint8_t indirect, uint32_t dst_offset)
{
    FF ia_value = kernel_trace_builder.op_storage_address();
    Row row = create_kernel_lookup_opcode(indirect, dst_offset, STORAGE_ADDRESS_SELECTOR, ia_value, AvmMemoryTag::FF);
    row.main_sel_op_storage_address = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(static_cast<uint32_t>(row.main_clk), OpCode::STORAGEADDRESS);

    main_trace.push_back(row);
}

void AvmTraceBuilder::op_sender(uint8_t indirect, uint32_t dst_offset)
{
    FF ia_value = kernel_trace_builder.op_sender();
    Row row = create_kernel_lookup_opcode(indirect, dst_offset, SENDER_SELECTOR, ia_value, AvmMemoryTag::FF);
    row.main_sel_op_sender = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(static_cast<uint32_t>(row.main_clk), OpCode::SENDER);

    main_trace.push_back(row);
}

void AvmTraceBuilder::op_function_selector(uint8_t indirect, uint32_t dst_offset)
{
    FF ia_value = kernel_trace_builder.op_function_selector();
    Row row =
        create_kernel_lookup_opcode(indirect, dst_offset, FUNCTION_SELECTOR_SELECTOR, ia_value, AvmMemoryTag::U32);
    row.main_sel_op_function_selector = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(static_cast<uint32_t>(row.main_clk), OpCode::FUNCTIONSELECTOR);

    main_trace.push_back(row);
}

void AvmTraceBuilder::op_transaction_fee(uint8_t indirect, uint32_t dst_offset)
{
    FF ia_value = kernel_trace_builder.op_transaction_fee();
    Row row = create_kernel_lookup_opcode(indirect, dst_offset, TRANSACTION_FEE_SELECTOR, ia_value, AvmMemoryTag::FF);
    row.main_sel_op_transaction_fee = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(static_cast<uint32_t>(row.main_clk), OpCode::TRANSACTIONFEE);

    main_trace.push_back(row);
}

/**************************************************************************************************
 *                            EXECUTION ENVIRONMENT - GLOBALS
 **************************************************************************************************/

void AvmTraceBuilder::op_chain_id(uint8_t indirect, uint32_t dst_offset)
{
    FF ia_value = kernel_trace_builder.op_chain_id();
    Row row = create_kernel_lookup_opcode(indirect, dst_offset, CHAIN_ID_SELECTOR, ia_value, AvmMemoryTag::FF);
    row.main_sel_op_chain_id = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(static_cast<uint32_t>(row.main_clk), OpCode::CHAINID);

    main_trace.push_back(row);
}

void AvmTraceBuilder::op_version(uint8_t indirect, uint32_t dst_offset)
{
    FF ia_value = kernel_trace_builder.op_version();
    Row row = create_kernel_lookup_opcode(indirect, dst_offset, VERSION_SELECTOR, ia_value, AvmMemoryTag::FF);
    row.main_sel_op_version = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(static_cast<uint32_t>(row.main_clk), OpCode::VERSION);

    main_trace.push_back(row);
}

void AvmTraceBuilder::op_block_number(uint8_t indirect, uint32_t dst_offset)
{
    FF ia_value = kernel_trace_builder.op_block_number();
    Row row = create_kernel_lookup_opcode(indirect, dst_offset, BLOCK_NUMBER_SELECTOR, ia_value, AvmMemoryTag::FF);
    row.main_sel_op_block_number = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(static_cast<uint32_t>(row.main_clk), OpCode::BLOCKNUMBER);

    main_trace.push_back(row);
}

void AvmTraceBuilder::op_timestamp(uint8_t indirect, uint32_t dst_offset)
{
    FF ia_value = kernel_trace_builder.op_timestamp();
    Row row = create_kernel_lookup_opcode(indirect, dst_offset, TIMESTAMP_SELECTOR, ia_value, AvmMemoryTag::U64);
    row.main_sel_op_timestamp = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(static_cast<uint32_t>(row.main_clk), OpCode::TIMESTAMP);

    main_trace.push_back(row);
}

void AvmTraceBuilder::op_coinbase(uint8_t indirect, uint32_t dst_offset)
{
    FF ia_value = kernel_trace_builder.op_coinbase();
    Row row = create_kernel_lookup_opcode(indirect, dst_offset, COINBASE_SELECTOR, ia_value, AvmMemoryTag::FF);
    row.main_sel_op_coinbase = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(static_cast<uint32_t>(row.main_clk), OpCode::COINBASE);

    main_trace.push_back(row);
}

void AvmTraceBuilder::op_fee_per_l2_gas(uint8_t indirect, uint32_t dst_offset)
{
    FF ia_value = kernel_trace_builder.op_fee_per_l2_gas();
    Row row = create_kernel_lookup_opcode(indirect, dst_offset, FEE_PER_L2_GAS_SELECTOR, ia_value, AvmMemoryTag::FF);
    row.main_sel_op_fee_per_l2_gas = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(static_cast<uint32_t>(row.main_clk), OpCode::FEEPERL2GAS);

    main_trace.push_back(row);
}

void AvmTraceBuilder::op_fee_per_da_gas(uint8_t indirect, uint32_t dst_offset)
{
    FF ia_value = kernel_trace_builder.op_fee_per_da_gas();
    Row row = create_kernel_lookup_opcode(indirect, dst_offset, FEE_PER_DA_GAS_SELECTOR, ia_value, AvmMemoryTag::FF);
    row.main_sel_op_fee_per_da_gas = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(static_cast<uint32_t>(row.main_clk), OpCode::FEEPERDAGAS);

    main_trace.push_back(row);
}

/**************************************************************************************************
 *                            EXECUTION ENVIRONMENT - CALLDATA
 **************************************************************************************************/

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
 */
void AvmTraceBuilder::op_calldata_copy(uint8_t indirect, uint32_t cd_offset, uint32_t copy_size, uint32_t dst_offset)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    uint32_t direct_dst_offset = dst_offset; // Will be overwritten in indirect mode.

    bool indirect_flag = false;
    bool tag_match = true;

    // The only memory operation performed from the main trace is a possible indirect load for resolving the
    // direct destination offset stored in main_mem_addr_c.
    // All the other memory operations are triggered by the slice gadget.

    if (is_operand_indirect(indirect, 0)) {
        indirect_flag = true;
        auto ind_read =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_C, dst_offset);
        direct_dst_offset = uint32_t(ind_read.val);
        tag_match = ind_read.tag_match;
    }

    if (tag_match) {
        slice_trace_builder.create_calldata_copy_slice(
            calldata, clk, call_ptr, cd_offset, copy_size, direct_dst_offset);
        mem_trace_builder.write_calldata_copy(calldata, clk, call_ptr, cd_offset, copy_size, direct_dst_offset);
    }

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::CALLDATACOPY);

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_call_ptr = call_ptr,
        .main_ia = cd_offset,
        .main_ib = copy_size,
        .main_ind_addr_c = indirect_flag ? dst_offset : 0,
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_c = direct_dst_offset,
        .main_pc = pc++,
        .main_r_in_tag = static_cast<uint32_t>(AvmMemoryTag::FF),
        .main_sel_op_calldata_copy = 1,
        .main_sel_resolve_ind_addr_c = static_cast<uint32_t>(indirect_flag),
        .main_sel_slice_gadget = static_cast<uint32_t>(tag_match),
        .main_tag_err = static_cast<uint32_t>(!tag_match),
        .main_w_in_tag = static_cast<uint32_t>(AvmMemoryTag::FF),
    });
}

/**************************************************************************************************
 *                            MACHINE STATE - GAS
 **************************************************************************************************/

// Helper for "gas left" related opcodes
void AvmTraceBuilder::execute_gasleft(OpCode opcode, uint8_t indirect, uint32_t dst_offset)
{
    assert(opcode == OpCode::L2GASLEFT || opcode == OpCode::DAGASLEFT);

    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    auto [resolved_dst] = unpack_indirects<1>(indirect, { dst_offset });

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, opcode);

    uint32_t gas_remaining = 0;

    if (opcode == OpCode::L2GASLEFT) {
        gas_remaining = gas_trace_builder.get_l2_gas_left();
    } else {
        gas_remaining = gas_trace_builder.get_da_gas_left();
    }

    // Write into memory from intermediate register ia.
    // TODO: probably will be U32 in final version
    auto write_dst = constrained_write_to_memory(
        call_ptr, clk, resolved_dst, gas_remaining, AvmMemoryTag::U0, AvmMemoryTag::FF, IntermRegister::IA);

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_call_ptr = call_ptr,
        .main_ia = gas_remaining,
        .main_ind_addr_a = FF(write_dst.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(write_dst.direct_address),
        .main_pc = FF(pc++),
        .main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U0)),
        .main_rwa = FF(1),
        .main_sel_mem_op_a = FF(1),
        .main_sel_op_dagasleft = (opcode == OpCode::DAGASLEFT) ? FF(1) : FF(0),
        .main_sel_op_l2gasleft = (opcode == OpCode::L2GASLEFT) ? FF(1) : FF(0),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(is_operand_indirect(indirect, 0))),
        .main_tag_err = FF(static_cast<uint32_t>(!write_dst.tag_match)),
        .main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)), // TODO: probably will be U32 in final version
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

/**************************************************************************************************
 *                            MACHINE STATE - INTERNAL CONTROL FLOW
 **************************************************************************************************/

/**
 * @brief JUMP OPCODE
 *        Jumps to a new `jmp_dest`
 *        This function must:
 *          - Set the next program counter to the provided `jmp_dest`.
 *
 * @param jmp_dest - The destination to jump to
 */
void AvmTraceBuilder::op_jump(uint32_t jmp_dest)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::JUMP);

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_call_ptr = call_ptr,
        .main_ia = FF(jmp_dest),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_pc = FF(pc),
        .main_sel_op_jump = FF(1),
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
void AvmTraceBuilder::op_jumpi(uint8_t indirect, uint32_t jmp_dest, uint32_t cond_offset)
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
        .main_clk = clk,
        .main_call_ptr = call_ptr,
        .main_ia = FF(next_pc),
        .main_id = read_d.val,
        .main_id_zero = static_cast<uint32_t>(id_zero),
        .main_ind_addr_d = indirect_cond_flag ? cond_offset : 0,
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_inv = inv,
        .main_mem_addr_d = direct_cond_offset,
        .main_pc = FF(pc),
        .main_r_in_tag = static_cast<uint32_t>(read_d.tag),
        .main_sel_mem_op_d = 1,
        .main_sel_op_jumpi = FF(1),
        .main_sel_resolve_ind_addr_d = static_cast<uint32_t>(indirect_cond_flag),
        .main_tag_err = static_cast<uint32_t>(!tag_match),
        .main_w_in_tag = static_cast<uint32_t>(read_d.tag),
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
void AvmTraceBuilder::op_internal_call(uint32_t jmp_dest)
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
        .main_clk = clk,
        .main_call_ptr = call_ptr,
        .main_ia = FF(jmp_dest),
        .main_ib = FF(pc + 1),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_b = FF(internal_return_ptr),
        .main_pc = FF(pc),
        .main_rwb = FF(1),
        .main_sel_mem_op_b = FF(1),
        .main_sel_op_internal_call = FF(1),
        .main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U32)),
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
void AvmTraceBuilder::op_internal_return()
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    // Internal return pointer is decremented
    // We want to load the value pointed by the internal pointer
    auto read_a = mem_trace_builder.read_and_load_from_memory(
        INTERNAL_CALL_SPACE_ID, clk, IntermRegister::IA, internal_return_ptr - 1, AvmMemoryTag::U32, AvmMemoryTag::U0);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::INTERNALRETURN);

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_call_ptr = call_ptr,
        .main_ia = read_a.val,
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(internal_return_ptr - 1),
        .main_pc = pc,
        .main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U32)),
        .main_rwa = FF(0),
        .main_sel_mem_op_a = FF(1),
        .main_sel_op_internal_return = FF(1),
        .main_tag_err = FF(static_cast<uint32_t>(!read_a.tag_match)),
    });

    pc = uint32_t(read_a.val);
    internal_return_ptr--;
}

/**************************************************************************************************
 *                            MACHINE STATE - MEMORY
 **************************************************************************************************/

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
    auto [resolved_c] = unpack_indirects<1>(indirect, { dst_offset });

    auto write_c =
        constrained_write_to_memory(call_ptr, clk, resolved_c, val_ff, AvmMemoryTag::U0, in_tag, IntermRegister::IC);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::SET);

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_call_ptr = call_ptr,
        .main_ic = write_c.val,
        .main_ind_addr_c = FF(write_c.indirect_address),
        .main_internal_return_ptr = internal_return_ptr,
        .main_mem_addr_c = FF(write_c.direct_address),
        .main_pc = pc++,
        .main_rwc = 1,
        .main_sel_mem_op_activate_gas = 1, // TODO: remove in the long term
        .main_sel_mem_op_c = 1,
        .main_sel_resolve_ind_addr_c = FF(static_cast<uint32_t>(write_c.is_indirect)),
        .main_tag_err = static_cast<uint32_t>(!write_c.tag_match),
        .main_w_in_tag = static_cast<uint32_t>(in_tag),
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
        .main_clk = clk,
        .main_call_ptr = call_ptr,
        .main_ia = val,
        .main_ic = val,
        .main_ind_addr_a = indirect_src_flag ? src_offset : 0,
        .main_ind_addr_c = indirect_dst_flag ? dst_offset : 0,
        .main_internal_return_ptr = internal_return_ptr,
        .main_mem_addr_a = direct_src_offset,
        .main_mem_addr_c = direct_dst_offset,
        .main_pc = pc++,
        .main_r_in_tag = static_cast<uint32_t>(tag),
        .main_rwc = 1,
        .main_sel_mem_op_a = 1,
        .main_sel_mem_op_c = 1,
        .main_sel_mov_ia_to_ic = 1,
        .main_sel_op_mov = 1,
        .main_sel_resolve_ind_addr_a = static_cast<uint32_t>(indirect_src_flag),
        .main_sel_resolve_ind_addr_c = static_cast<uint32_t>(indirect_dst_flag),
        .main_tag_err = static_cast<uint32_t>(!tag_match),
        .main_w_in_tag = static_cast<uint32_t>(tag),
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
        .main_clk = clk,
        .main_call_ptr = call_ptr,
        .main_ia = a_mem_entry.val,
        .main_ib = b_mem_entry.val,
        .main_ic = val,
        .main_id = cond_mem_entry.val,
        .main_id_zero = static_cast<uint32_t>(id_zero),
        .main_ind_addr_a = indirect_a_flag ? a_offset : 0,
        .main_ind_addr_b = indirect_b_flag ? b_offset : 0,
        .main_ind_addr_c = indirect_dst_flag ? dst_offset : 0,
        .main_ind_addr_d = indirect_cond_flag ? cond_offset : 0,
        .main_internal_return_ptr = internal_return_ptr,
        .main_inv = inv,
        .main_mem_addr_a = direct_a_offset,
        .main_mem_addr_b = direct_b_offset,
        .main_mem_addr_c = direct_dst_offset,
        .main_mem_addr_d = direct_cond_offset,
        .main_pc = pc++,
        .main_r_in_tag = static_cast<uint32_t>(tag),
        .main_rwc = 1,
        .main_sel_mem_op_a = 1,
        .main_sel_mem_op_b = 1,
        .main_sel_mem_op_c = 1,
        .main_sel_mem_op_d = 1,
        .main_sel_mov_ia_to_ic = static_cast<uint32_t>(!id_zero),
        .main_sel_mov_ib_to_ic = static_cast<uint32_t>(id_zero),
        .main_sel_op_cmov = 1,
        .main_sel_resolve_ind_addr_a = static_cast<uint32_t>(indirect_a_flag),
        .main_sel_resolve_ind_addr_b = static_cast<uint32_t>(indirect_b_flag),
        .main_sel_resolve_ind_addr_c = static_cast<uint32_t>(indirect_dst_flag),
        .main_sel_resolve_ind_addr_d = static_cast<uint32_t>(indirect_cond_flag),
        .main_tag_err = static_cast<uint32_t>(!tag_match),
        .main_w_in_tag = static_cast<uint32_t>(tag),
    });
}

/**************************************************************************************************
 *                   HELPERS FOR WORLD STATE AND ACCRUED SUBSTATE
 **************************************************************************************************/

/**
 * @brief Create a kernel output opcode object
 *
 * Used for writing to the kernel app outputs - {new_note_hash, new_nullifier, etc.}
 *
 * @param indirect - Perform indirect memory resolution
 * @param clk - The trace clk
 * @param data_offset - The memory address to read the output from
 * @return Row
 */
Row AvmTraceBuilder::create_kernel_output_opcode(uint8_t indirect, uint32_t clk, uint32_t data_offset)
{
    auto [resolved_data] = unpack_indirects<1>(indirect, { data_offset });
    auto read_a = constrained_read_from_memory(
        call_ptr, clk, resolved_data, AvmMemoryTag::FF, AvmMemoryTag::U0, IntermRegister::IA);
    bool tag_match = read_a.tag_match;

    return Row{
        .main_clk = clk,
        .main_ia = read_a.val,
        .main_ind_addr_a = FF(read_a.indirect_address),
        .main_internal_return_ptr = internal_return_ptr,
        .main_mem_addr_a = FF(read_a.direct_address),
        .main_pc = pc++,
        .main_r_in_tag = static_cast<uint32_t>(AvmMemoryTag::FF),
        .main_rwa = 0,
        .main_sel_mem_op_a = 1,
        .main_sel_q_kernel_output_lookup = 1,
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(read_a.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
    };
}

/**
 * @brief Create a kernel output opcode with metadata object
 *
 * Used for writing to the kernel app outputs with extra metadata - {sload, sstore} (value, slot)
 *
 * @param indirect - Perform indirect memory resolution
 * @param clk - The trace clk
 * @param data_offset - The offset of the main value to output
 * @param data_r_tag - The data type of the value
 * @param metadata_offset - The offset of the metadata (slot in the sload example)
 * @param metadata_r_tag - The data type of the metadata
 * @return Row
 */
Row AvmTraceBuilder::create_kernel_output_opcode_with_metadata(uint8_t indirect,
                                                               uint32_t clk,
                                                               uint32_t data_offset,
                                                               AvmMemoryTag data_r_tag,
                                                               uint32_t metadata_offset,
                                                               AvmMemoryTag metadata_r_tag)
{
    auto [resolved_data, resolved_metadata] = unpack_indirects<2>(indirect, { data_offset, metadata_offset });

    auto read_a =
        constrained_read_from_memory(call_ptr, clk, resolved_data, data_r_tag, AvmMemoryTag::U0, IntermRegister::IA);
    auto read_b = constrained_read_from_memory(
        call_ptr, clk, resolved_metadata, metadata_r_tag, AvmMemoryTag::U0, IntermRegister::IB);
    bool tag_match = read_a.tag_match && read_b.tag_match;

    return Row{
        .main_clk = clk,
        .main_ia = read_a.val,
        .main_ib = read_b.val,
        .main_ind_addr_a = FF(read_a.indirect_address),
        .main_ind_addr_b = FF(read_b.indirect_address),
        .main_internal_return_ptr = internal_return_ptr,
        .main_mem_addr_a = FF(read_a.direct_address),
        .main_mem_addr_b = FF(read_b.direct_address),
        .main_pc = pc++,
        .main_r_in_tag = static_cast<uint32_t>(data_r_tag),
        .main_rwa = 0,
        .main_rwb = 0,
        .main_sel_mem_op_a = 1,
        .main_sel_mem_op_b = 1,
        .main_sel_q_kernel_output_lookup = 1,
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(read_a.is_indirect)),
        .main_sel_resolve_ind_addr_b = FF(static_cast<uint32_t>(read_b.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
    };
}

/**
 * @brief Create a kernel output opcode with set metadata output object
 *
 * Used for writing output opcode where one metadata value is written and comes from a hint
 * {note_hash_exists, nullifier_exists, etc. } Where a boolean output if it exists must also be written
 *
 * @param indirect - Perform indirect memory resolution
 * @param clk - The trace clk
 * @param data_offset - The offset of the main value to output
 * @param metadata_offset - The offset of the metadata (slot in the sload example)
 * @return Row
 */
Row AvmTraceBuilder::create_kernel_output_opcode_with_set_metadata_output_from_hint(uint8_t indirect,
                                                                                    uint32_t clk,
                                                                                    uint32_t data_offset,
                                                                                    uint32_t metadata_offset)
{
    FF exists = execution_hints.get_side_effect_hints().at(side_effect_counter);
    // TODO: throw error if incorrect

    auto [resolved_data, resolved_metadata] = unpack_indirects<2>(indirect, { data_offset, metadata_offset });
    auto read_a = constrained_read_from_memory(
        call_ptr, clk, resolved_data, AvmMemoryTag::FF, AvmMemoryTag::U8, IntermRegister::IA);

    auto write_b = constrained_write_to_memory(
        call_ptr, clk, resolved_metadata, exists, AvmMemoryTag::FF, AvmMemoryTag::U8, IntermRegister::IB);
    bool tag_match = read_a.tag_match && write_b.tag_match;

    return Row{
        .main_clk = clk,
        .main_ia = read_a.val,
        .main_ib = write_b.val,
        .main_ind_addr_a = FF(read_a.indirect_address),
        .main_ind_addr_b = FF(write_b.indirect_address),
        .main_internal_return_ptr = internal_return_ptr,
        .main_mem_addr_a = FF(read_a.direct_address),
        .main_mem_addr_b = FF(write_b.direct_address),
        .main_pc = pc++,
        .main_r_in_tag = static_cast<uint32_t>(AvmMemoryTag::FF),
        .main_rwa = 0,
        .main_rwb = 1,
        .main_sel_mem_op_a = 1,
        .main_sel_mem_op_b = 1,
        .main_sel_q_kernel_output_lookup = 1,
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(read_a.is_indirect)),
        .main_sel_resolve_ind_addr_b = FF(static_cast<uint32_t>(write_b.is_indirect)),
        .main_tag_err = static_cast<uint32_t>(!tag_match),
        .main_w_in_tag = static_cast<uint32_t>(AvmMemoryTag::U8),
    };
}

/**
 * @brief Create a kernel output opcode with set metadata output object
 *
 * Used for writing output opcode where one value is written and comes from a hint
 * {sload}
 *
 * @param indirect - Perform indirect memory resolution
 * @param clk - The trace clk
 * @param data_offset - The offset of the main value to output
 * @param metadata_offset - The offset of the metadata (slot in the sload example)
 * @return Row
 */
Row AvmTraceBuilder::create_kernel_output_opcode_with_set_value_from_hint(uint8_t indirect,
                                                                          uint32_t clk,
                                                                          uint32_t data_offset,
                                                                          uint32_t metadata_offset)
{
    FF value = execution_hints.get_side_effect_hints().at(side_effect_counter);
    // TODO: throw error if incorrect

    auto [resolved_data, resolved_metadata] = unpack_indirects<2>(indirect, { data_offset, metadata_offset });
    auto write_a = constrained_write_to_memory(
        call_ptr, clk, resolved_data, value, AvmMemoryTag::FF, AvmMemoryTag::FF, IntermRegister::IA);
    auto read_b = constrained_read_from_memory(
        call_ptr, clk, resolved_metadata, AvmMemoryTag::FF, AvmMemoryTag::FF, IntermRegister::IB);
    bool tag_match = write_a.tag_match && read_b.tag_match;

    return Row{
        .main_clk = clk,
        .main_ia = write_a.val,
        .main_ib = read_b.val,
        .main_ind_addr_a = FF(write_a.indirect_address),
        .main_ind_addr_b = FF(read_b.indirect_address),
        .main_internal_return_ptr = internal_return_ptr,
        .main_mem_addr_a = FF(write_a.direct_address),
        .main_mem_addr_b = FF(read_b.direct_address),
        .main_pc = pc, // No PC increment here since we do it in the specific ops
        .main_r_in_tag = static_cast<uint32_t>(AvmMemoryTag::FF),
        .main_rwa = 1,
        .main_rwb = 0,
        .main_sel_mem_op_a = 1,
        .main_sel_mem_op_b = 1,
        .main_sel_q_kernel_output_lookup = 1,
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(write_a.is_indirect)),
        .main_sel_resolve_ind_addr_b = FF(static_cast<uint32_t>(read_b.is_indirect)),
        .main_tag_err = static_cast<uint32_t>(!tag_match),
        .main_w_in_tag = static_cast<uint32_t>(AvmMemoryTag::FF),
    };
}

/**************************************************************************************************
 *                              WORLD STATE
 **************************************************************************************************/

void AvmTraceBuilder::op_sload(uint8_t indirect, uint32_t slot_offset, uint32_t size, uint32_t dest_offset)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    auto [resolved_slot, resolved_dest] = unpack_indirects<2>(indirect, { slot_offset, dest_offset });
    auto read_slot = constrained_read_from_memory(
        call_ptr, clk, resolved_slot, AvmMemoryTag::FF, AvmMemoryTag::U0, IntermRegister::IA);

    // Read the slot value that we will write hints to in a row
    main_trace.push_back(Row{
        .main_clk = clk,
        .main_ia = read_slot.val,
        .main_ind_addr_a = FF(read_slot.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(read_slot.direct_address),
        .main_pc = pc, // No PC increment here since this is the same opcode as the rows created below
        .main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
        .main_sel_mem_op_a = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(read_slot.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!read_slot.tag_match)),
    });
    clk++;

    AddressWithMode write_dst = resolved_dest;
    // Loop over the size and write the hints to memory
    for (uint32_t i = 0; i < size; i++) {
        FF value = execution_hints.get_side_effect_hints().at(side_effect_counter);

        auto write_a = constrained_write_to_memory(
            call_ptr, clk, write_dst, value, AvmMemoryTag::U0, AvmMemoryTag::FF, IntermRegister::IA);

        auto row = Row{
            .main_clk = clk,
            .main_ia = value,
            .main_ib = read_slot.val + i, // slot increments each time
            .main_ind_addr_a = write_a.indirect_address,
            .main_internal_return_ptr = internal_return_ptr,
            .main_mem_addr_a = write_a.direct_address, // direct address incremented at end of the loop
            .main_pc = pc, // No PC increment here since this is the same opcode for all loop iterations
            .main_rwa = 1,
            .main_sel_mem_op_a = 1,
            .main_sel_op_sload = FF(1),
            .main_sel_q_kernel_output_lookup = 1,
            .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(write_a.is_indirect)),
            .main_tag_err = FF(static_cast<uint32_t>(!write_a.tag_match)),
            .main_w_in_tag = static_cast<uint32_t>(AvmMemoryTag::FF),
        };

        // Output storage read to kernel outputs (performs lookup)
        // Tuples of (slot, value) in the kernel lookup
        kernel_trace_builder.op_sload(clk, side_effect_counter, row.main_ib, row.main_ia);

        // Constrain gas cost
        gas_trace_builder.constrain_gas_lookup(clk, OpCode::SLOAD);

        main_trace.push_back(row);

        debug("sload side-effect cnt: ", side_effect_counter);
        side_effect_counter++;
        clk++;

        // After the first loop, all future write destinations are direct, increment the direct address
        write_dst = AddressWithMode{ AddressingMode::DIRECT, write_a.direct_address + 1 };
    }
    pc++;
}

void AvmTraceBuilder::op_sstore(uint8_t indirect, uint32_t src_offset, uint32_t size, uint32_t slot_offset)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    auto [resolved_src, resolved_slot] = unpack_indirects<2>(indirect, { src_offset, slot_offset });

    auto read_slot = constrained_read_from_memory(
        call_ptr, clk, resolved_slot, AvmMemoryTag::FF, AvmMemoryTag::FF, IntermRegister::IA);

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_ia = read_slot.val,
        .main_ind_addr_a = FF(read_slot.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(read_slot.direct_address),
        .main_pc = pc, // No PC increment here since this is the same opcode as the rows created below
        .main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
        .main_sel_mem_op_a = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(read_slot.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!read_slot.tag_match)),
        .main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
    });
    clk++;

    AddressWithMode read_src = resolved_src;

    // This loop reads a _size_ number of elements from memory and places them into a tuple of (ele, slot)
    // in the kernel lookup.
    for (uint32_t i = 0; i < size; i++) {
        auto read_a = constrained_read_from_memory(
            call_ptr, clk, read_src, AvmMemoryTag::FF, AvmMemoryTag::U0, IntermRegister::IA);

        Row row = Row{
            .main_clk = clk,
            .main_ia = read_a.val,
            .main_ib = read_slot.val + i, // slot increments each time
            .main_ind_addr_a = read_a.indirect_address,
            .main_internal_return_ptr = internal_return_ptr,
            .main_mem_addr_a = read_a.direct_address, // direct address incremented at end of the loop
            .main_pc = pc,
            .main_r_in_tag = static_cast<uint32_t>(AvmMemoryTag::FF),
            .main_sel_mem_op_a = 1,
            .main_sel_q_kernel_output_lookup = 1,
            .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(read_a.is_indirect)),
            .main_tag_err = FF(static_cast<uint32_t>(!read_a.tag_match)),
        };
        row.main_sel_op_sstore = FF(1);
        kernel_trace_builder.op_sstore(clk, side_effect_counter, row.main_ib, row.main_ia);

        // Constrain gas cost
        gas_trace_builder.constrain_gas_lookup(clk, OpCode::SSTORE);

        main_trace.push_back(row);

        debug("sstore side-effect cnt: ", side_effect_counter);
        side_effect_counter++;
        clk++;
        // All future reads are direct, increment the direct address
        read_src = AddressWithMode{ AddressingMode::DIRECT, read_a.direct_address + 1 };
    }
    pc++;
}

void AvmTraceBuilder::op_note_hash_exists(uint8_t indirect, uint32_t note_hash_offset, uint32_t dest_offset)
{
    auto const clk = static_cast<uint32_t>(main_trace.size()) + 1;

    Row row =
        create_kernel_output_opcode_with_set_metadata_output_from_hint(indirect, clk, note_hash_offset, dest_offset);
    kernel_trace_builder.op_note_hash_exists(
        clk, side_effect_counter, row.main_ia, /*safe*/ static_cast<uint32_t>(row.main_ib));
    row.main_sel_op_note_hash_exists = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::NOTEHASHEXISTS);

    main_trace.push_back(row);

    debug("note_hash_exists side-effect cnt: ", side_effect_counter);
    side_effect_counter++;
}

void AvmTraceBuilder::op_emit_note_hash(uint8_t indirect, uint32_t note_hash_offset)
{
    auto const clk = static_cast<uint32_t>(main_trace.size()) + 1;

    Row row = create_kernel_output_opcode(indirect, clk, note_hash_offset);
    kernel_trace_builder.op_emit_note_hash(clk, side_effect_counter, row.main_ia);
    row.main_sel_op_emit_note_hash = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::EMITNOTEHASH);

    main_trace.push_back(row);

    debug("emit_note_hash side-effect cnt: ", side_effect_counter);
    side_effect_counter++;
}

void AvmTraceBuilder::op_nullifier_exists(uint8_t indirect, uint32_t nullifier_offset, uint32_t dest_offset)
{
    auto const clk = static_cast<uint32_t>(main_trace.size()) + 1;

    Row row =
        create_kernel_output_opcode_with_set_metadata_output_from_hint(indirect, clk, nullifier_offset, dest_offset);
    kernel_trace_builder.op_nullifier_exists(
        clk, side_effect_counter, row.main_ia, /*safe*/ static_cast<uint32_t>(row.main_ib));
    row.main_sel_op_nullifier_exists = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::NULLIFIEREXISTS);

    main_trace.push_back(row);

    debug("nullifier_exists side-effect cnt: ", side_effect_counter);
    side_effect_counter++;
}

void AvmTraceBuilder::op_emit_nullifier(uint8_t indirect, uint32_t nullifier_offset)
{
    auto const clk = static_cast<uint32_t>(main_trace.size()) + 1;

    Row row = create_kernel_output_opcode(indirect, clk, nullifier_offset);
    kernel_trace_builder.op_emit_nullifier(clk, side_effect_counter, row.main_ia);
    row.main_sel_op_emit_nullifier = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::EMITNULLIFIER);

    main_trace.push_back(row);

    debug("emit_nullifier side-effect cnt: ", side_effect_counter);
    side_effect_counter++;
}

void AvmTraceBuilder::op_l1_to_l2_msg_exists(uint8_t indirect, uint32_t log_offset, uint32_t dest_offset)
{
    auto const clk = static_cast<uint32_t>(main_trace.size()) + 1;

    Row row = create_kernel_output_opcode_with_set_metadata_output_from_hint(indirect, clk, log_offset, dest_offset);
    kernel_trace_builder.op_l1_to_l2_msg_exists(
        clk, side_effect_counter, row.main_ia, /*safe*/ static_cast<uint32_t>(row.main_ib));
    row.main_sel_op_l1_to_l2_msg_exists = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::L1TOL2MSGEXISTS);

    main_trace.push_back(row);

    debug("l1_to_l2_msg_exists side-effect cnt: ", side_effect_counter);
    side_effect_counter++;
}

void AvmTraceBuilder::op_get_contract_instance(uint8_t indirect, uint32_t address_offset, uint32_t dst_offset)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    auto [resolved_address_offset, resolved_dst_offset] = unpack_indirects<2>(indirect, { address_offset, dst_offset });
    auto read_address = constrained_read_from_memory(
        call_ptr, clk, resolved_address_offset, AvmMemoryTag::FF, AvmMemoryTag::U0, IntermRegister::IA);
    bool tag_match = read_address.tag_match;

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::GETCONTRACTINSTANCE);

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_ia = read_address.val,
        .main_ind_addr_a = FF(read_address.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(read_address.direct_address),
        .main_pc = FF(pc++),
        .main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
        .main_sel_mem_op_a = FF(1),
        .main_sel_mem_op_activate_gas = FF(1), // TODO: remove in the long term
        .main_sel_op_get_contract_instance = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(read_address.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
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
                          resolved_dst_offset,
                          AvmMemoryTag::U0,
                          AvmMemoryTag::FF,
                          internal_return_ptr,
                          contract_instance_vec);

    debug("contract_instance cnt: ", side_effect_counter);
    side_effect_counter++;
}

/**************************************************************************************************
 *                              ACCRUED SUBSTATE
 **************************************************************************************************/

void AvmTraceBuilder::op_emit_unencrypted_log(uint8_t indirect,
                                              uint32_t log_offset,
                                              [[maybe_unused]] uint32_t log_size_offset)
{
    auto const clk = static_cast<uint32_t>(main_trace.size()) + 1;

    // FIXME: read (and constrain) log_size_offset
    // FIXME: we need to constrain the log_size_offset mem read (and tag check), not just one field!
    Row row = create_kernel_output_opcode(indirect, clk, log_offset);
    kernel_trace_builder.op_emit_unencrypted_log(clk, side_effect_counter, row.main_ia);
    row.main_sel_op_emit_unencrypted_log = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::EMITUNENCRYPTEDLOG);

    main_trace.push_back(row);

    debug("emit_unencrypted_log side-effect cnt: ", side_effect_counter);
    side_effect_counter++;
}

void AvmTraceBuilder::op_emit_l2_to_l1_msg(uint8_t indirect, uint32_t recipient_offset, uint32_t content_offset)
{
    auto const clk = static_cast<uint32_t>(main_trace.size()) + 1;

    // Note: unorthadox order - as seen in L2ToL1Message struct in TS
    Row row = create_kernel_output_opcode_with_metadata(
        indirect, clk, content_offset, AvmMemoryTag::FF, recipient_offset, AvmMemoryTag::FF);
    kernel_trace_builder.op_emit_l2_to_l1_msg(clk, side_effect_counter, row.main_ia, row.main_ib);
    row.main_sel_op_emit_l2_to_l1_msg = FF(1);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::SENDL2TOL1MSG);

    main_trace.push_back(row);

    debug("emit_l2_to_l1_msg side-effect cnt: ", side_effect_counter);
    side_effect_counter++;
}

/**************************************************************************************************
 *                            CONTROL FLOW - CONTRACT CALLS
 **************************************************************************************************/

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
 * @param ret_offset An index in memory pointing to where the first value of the external calls return value should
 * be stored.
 * @param ret_size The number of values in the return array
 * @param success_offset An index in memory pointing to where the success flag (U8) of the external call should be
 * stored
 * @param function_selector_offset An index in memory pointing to the function selector of the external call (TEMP)
 */
void AvmTraceBuilder::op_call(uint8_t indirect,
                              uint32_t gas_offset,
                              uint32_t addr_offset,
                              uint32_t args_offset,
                              uint32_t args_size,
                              uint32_t ret_offset,
                              uint32_t ret_size,
                              uint32_t success_offset,
                              [[maybe_unused]] uint32_t function_selector_offset)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;
    const ExternalCallHint& hint = execution_hints.externalcall_hints.at(external_call_counter);

    gas_trace_builder.constrain_gas_for_external_call(
        clk, static_cast<uint32_t>(hint.l2_gas_used), static_cast<uint32_t>(hint.da_gas_used));

    auto [resolved_gas_offset,
          resolved_addr_offset,
          resolved_args_offset,
          resolved_args_size,
          resolved_ret_offset,
          resolved_success_offset] =
        unpack_indirects<6>(indirect, { gas_offset, addr_offset, args_offset, args_size, ret_offset, success_offset });

    // Should read the address next to read_gas as well (tuple of gas values (l2Gas, daGas))
    auto read_gas_l2 = constrained_read_from_memory(
        call_ptr, clk, resolved_gas_offset, AvmMemoryTag::FF, AvmMemoryTag::U0, IntermRegister::IA);
    auto read_gas_da = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, read_gas_l2.direct_address + 1, AvmMemoryTag::FF, AvmMemoryTag::U0);
    auto read_addr = constrained_read_from_memory(
        call_ptr, clk, resolved_addr_offset, AvmMemoryTag::FF, AvmMemoryTag::U0, IntermRegister::IC);
    auto read_args = constrained_read_from_memory(
        call_ptr, clk, resolved_args_offset, AvmMemoryTag::FF, AvmMemoryTag::U0, IntermRegister::ID);
    bool tag_match = read_gas_l2.tag_match && read_gas_da.tag_match && read_addr.tag_match && read_args.tag_match;

    // We read the input and output addresses in one row as they should contain FF elements
    main_trace.push_back(Row{
        .main_clk = clk,
        .main_ia = read_gas_l2.val, /* gas_offset_l2 */
        .main_ib = read_gas_da.val, /* gas_offset_da */
        .main_ic = read_addr.val,   /* addr_offset */
        .main_id = read_args.val,   /* args_offset */
        .main_ind_addr_a = FF(read_gas_l2.indirect_address),
        .main_ind_addr_c = FF(read_addr.indirect_address),
        .main_ind_addr_d = FF(read_args.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(read_gas_l2.direct_address),
        .main_mem_addr_b = FF(read_gas_l2.direct_address + 1),
        .main_mem_addr_c = FF(read_addr.direct_address),
        .main_mem_addr_d = FF(read_args.direct_address),
        .main_pc = FF(pc),
        .main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
        .main_sel_mem_op_a = FF(1),
        .main_sel_mem_op_b = FF(1),
        .main_sel_mem_op_c = FF(1),
        .main_sel_mem_op_d = FF(1),
        .main_sel_op_external_call = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(read_gas_l2.is_indirect)),
        .main_sel_resolve_ind_addr_c = FF(static_cast<uint32_t>(read_addr.is_indirect)),
        .main_sel_resolve_ind_addr_d = FF(static_cast<uint32_t>(read_args.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
    });
    clk++;
    // The return data hint is used for now, we check it has the same length as the ret_size
    ASSERT(hint.return_data.size() == ret_size);
    // Write the return data to memory
    uint32_t num_rows = write_slice_to_memory(
        call_ptr, clk, resolved_ret_offset, AvmMemoryTag::U0, AvmMemoryTag::FF, internal_return_ptr, hint.return_data);
    clk += num_rows;
    // Write the success flag to memory
    write_slice_to_memory(call_ptr,
                          clk,
                          resolved_success_offset,
                          AvmMemoryTag::U0,
                          AvmMemoryTag::U8,
                          internal_return_ptr,
                          { hint.success });
    external_call_counter++;
    pc++;
    // Adjust the side_effect_counter to the the value at the end of the external call.
    side_effect_counter = static_cast<uint32_t>(hint.end_side_effect_counter);
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
std::vector<FF> AvmTraceBuilder::op_return(uint8_t indirect, uint32_t ret_offset, uint32_t ret_size)
{
    if (ret_size == 0) {
        halt();
        return {};
    }

    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    uint32_t direct_ret_offset = ret_offset; // Will be overwritten in indirect mode.

    bool indirect_flag = false;
    bool tag_match = true;

    // The only memory operation performed from the main trace is a possible indirect load for resolving the
    // direct destination offset stored in main_mem_addr_c.
    // All the other memory operations are triggered by the slice gadget.

    if (is_operand_indirect(indirect, 0)) {
        indirect_flag = true;
        auto ind_read =
            mem_trace_builder.indirect_read_and_load_from_memory(call_ptr, clk, IndirectRegister::IND_C, ret_offset);
        direct_ret_offset = uint32_t(ind_read.val);
        tag_match = ind_read.tag_match;
    }

    if (tag_match) {
        returndata = mem_trace_builder.read_return_opcode(clk, call_ptr, direct_ret_offset, ret_size);
        slice_trace_builder.create_return_slice(returndata, clk, call_ptr, direct_ret_offset, ret_size);
    }

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::RETURN);

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_call_ptr = call_ptr,
        .main_ib = ret_size,
        .main_ind_addr_c = indirect_flag ? ret_offset : 0,
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_c = direct_ret_offset,
        .main_pc = pc,
        .main_r_in_tag = static_cast<uint32_t>(AvmMemoryTag::FF),
        .main_sel_op_external_return = 1,
        .main_sel_op_halt = 1,
        .main_sel_resolve_ind_addr_c = static_cast<uint32_t>(indirect_flag),
        .main_sel_slice_gadget = static_cast<uint32_t>(tag_match),
        .main_tag_err = static_cast<uint32_t>(!tag_match),
        .main_w_in_tag = static_cast<uint32_t>(AvmMemoryTag::FF),
    });

    pc = UINT32_MAX; // This ensures that no subsequent opcode will be executed.
    return returndata;
}

std::vector<FF> AvmTraceBuilder::op_revert(uint8_t indirect, uint32_t ret_offset, uint32_t ret_size)
{
    return op_return(indirect, ret_offset, ret_size);
}

/**************************************************************************************************
 *                                   GADGETS
 **************************************************************************************************/

/**
 * @brief Keccak  with direct or indirect memory access.
 * Keccak is TEMPORARY while we wait for the transition to keccakf1600, so we do the minimal to store the result
 * @param indirect byte encoding information about indirect/direct memory access.
 * @param output_offset An index in memory pointing to where the first u8 value of the output array should be
 * stored.
 * @param input_offset An index in memory pointing to the first u8 value of the input array to be used
 * @param input_size offset An index in memory pointing to the size of the input array.
 */
void AvmTraceBuilder::op_keccak(uint8_t indirect,
                                uint32_t output_offset,
                                uint32_t input_offset,
                                uint32_t input_size_offset)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;
    auto [resolved_output_offset, resolved_input_offset, resolved_input_size_offset] =
        unpack_indirects<3>(indirect, { output_offset, input_offset, input_size_offset });

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::KECCAK);

    // Read the input length first
    auto input_length_read = constrained_read_from_memory(
        call_ptr, clk, resolved_input_size_offset, AvmMemoryTag::U32, AvmMemoryTag::U0, IntermRegister::IB);

    // Store the clock time that we will use to line up the gadget later
    auto keccak_op_clk = clk;
    main_trace.push_back(Row{
        .main_clk = clk,
        .main_ib = input_length_read.val, // Message Length
        .main_ind_addr_b = FF(input_length_read.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_b = FF(input_length_read.direct_address), // length
        .main_pc = FF(pc++),
        .main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U32)),
        .main_sel_mem_op_b = FF(1),
        .main_sel_op_keccak = FF(1),
        .main_sel_resolve_ind_addr_b = FF(static_cast<uint32_t>(input_length_read.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!input_length_read.tag_match)),
    });
    clk++;

    std::vector<uint8_t> input;
    input.reserve(uint32_t(input_length_read.val));
    // Read the slice length from memory
    uint32_t num_main_rows = read_slice_to_memory<uint8_t>(call_ptr,
                                                           clk,
                                                           resolved_input_offset,
                                                           AvmMemoryTag::U8,
                                                           AvmMemoryTag::U8,
                                                           FF(internal_return_ptr),
                                                           uint32_t(input_length_read.val),
                                                           input);

    clk += num_main_rows;

    std::array<uint8_t, 32> result = keccak_trace_builder.keccak(keccak_op_clk, input, uint32_t(input_length_read.val));
    // We convert the results to field elements here
    std::vector<FF> ff_result;
    for (uint32_t i = 0; i < 32; i++) {
        ff_result.emplace_back(result[i]);
    }
    // Write the result to memory after
    write_slice_to_memory(
        call_ptr, clk, resolved_output_offset, AvmMemoryTag::U8, AvmMemoryTag::U8, FF(internal_return_ptr), ff_result);
}

/**
 * @brief Poseidon2 Permutation with direct or indirect memory access.
 *
 * @param indirect byte encoding information about indirect/direct memory access.
 * @param input_offset An index in memory pointing to the first Field value of the input array to be used in the
 * next instance of poseidon2 permutation.
 * @param output_offset An index in memory pointing to where the first Field value of the output array should be
 * stored.
 */
void AvmTraceBuilder::op_poseidon2_permutation(uint8_t indirect, uint32_t input_offset, uint32_t output_offset)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;

    // Resolve the indirect flags, the results of this function are used to determine the memory offsets
    // that point to the starting memory addresses for the input, output and h_init values
    // Note::This function will add memory reads at clk in the mem_trace_builder
    auto [resolved_input_offset, resolved_output_offset] =
        unpack_indirects<2>(indirect, { input_offset, output_offset });

    auto read_a = constrained_read_from_memory(
        call_ptr, clk, resolved_input_offset, AvmMemoryTag::FF, AvmMemoryTag::U0, IntermRegister::IA);
    auto read_b = constrained_read_from_memory(
        call_ptr, clk, resolved_output_offset, AvmMemoryTag::FF, AvmMemoryTag::U0, IntermRegister::IB);
    bool tag_match = read_a.tag_match && read_b.tag_match;

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::POSEIDON2);

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_ia = read_a.val, // First element of input
        .main_ib = read_b.val, // First element of output (trivially zero)
        .main_ind_addr_a = FF(read_a.indirect_address),
        .main_ind_addr_b = FF(read_b.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(read_a.direct_address),
        .main_mem_addr_b = FF(read_b.direct_address),
        .main_pc = FF(pc++),
        .main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
        .main_sel_mem_op_a = FF(1),
        .main_sel_mem_op_b = FF(1),
        .main_sel_op_poseidon2 = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(read_a.is_indirect)),
        .main_sel_resolve_ind_addr_b = FF(static_cast<uint32_t>(read_b.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
    });
    // We store the current clk this main trace row occurred so that we can line up the poseidon2 gadget operation
    // at the same clk later.
    auto poseidon_op_clk = clk;

    // We need to increment the clk
    clk++;
    // Read results are written to input array.
    std::vector<FF> input_vec;
    read_slice_to_memory<FF>(call_ptr,
                             clk,
                             resolved_input_offset,
                             AvmMemoryTag::FF,
                             AvmMemoryTag::U0,
                             FF(internal_return_ptr),
                             4,
                             input_vec);

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
        call_ptr, clk, resolved_output_offset, AvmMemoryTag::U0, AvmMemoryTag::FF, FF(internal_return_ptr), ff_result);
}

/**
 * @brief SHA256 Hash with direct or indirect memory access.
 * This function is temporary until we have transitioned to sha256Compression
 * @param indirect byte encoding information about indirect/direct memory access.
 * @param output_offset An index in memory pointing to where the first U32 value of the output array should be
 * stored.
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
    auto [resolved_output_offset, resolved_input_offset, resolved_input_size_offset] =
        unpack_indirects<3>(indirect, { output_offset, input_offset, input_size_offset });

    gas_trace_builder.constrain_gas_lookup(clk, OpCode::SHA256);

    auto input_length_read = constrained_read_from_memory(
        call_ptr, clk, resolved_input_size_offset, AvmMemoryTag::U32, AvmMemoryTag::U0, IntermRegister::IB);

    // Store the clock time that we will use to line up the gadget later
    auto sha256_op_clk = clk;
    main_trace.push_back(Row{
        .main_clk = clk,
        .main_ib = input_length_read.val, // Message Length
        .main_ind_addr_b = FF(input_length_read.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_b = FF(input_length_read.direct_address),
        .main_pc = FF(pc++),
        .main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U32)),
        .main_sel_mem_op_b = FF(1),
        .main_sel_op_sha256 = FF(1),
        .main_sel_resolve_ind_addr_b = FF(static_cast<uint32_t>(input_length_read.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!input_length_read.tag_match)),
    });
    clk++;

    std::vector<uint8_t> input;
    input.reserve(uint32_t(input_length_read.val));
    uint32_t num_main_rows = read_slice_to_memory<uint8_t>(call_ptr,
                                                           clk,
                                                           resolved_input_offset,
                                                           AvmMemoryTag::U8,
                                                           AvmMemoryTag::U0,
                                                           FF(internal_return_ptr),
                                                           uint32_t(input_length_read.val),
                                                           input);
    clk += num_main_rows;
    //
    std::array<uint8_t, 32> result = sha256_trace_builder.sha256(input, sha256_op_clk);

    std::vector<FF> ff_result;
    for (uint32_t i = 0; i < 32; i++) {
        ff_result.emplace_back(result[i]);
    }
    // Write the result to memory after
    write_slice_to_memory(
        call_ptr, clk, resolved_output_offset, AvmMemoryTag::U0, AvmMemoryTag::U8, FF(internal_return_ptr), ff_result);
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
    auto [resolved_gen_ctx_offset, resolved_output_offset, resolved_input_offset, resolved_input_size_offset] =
        unpack_indirects<4>(indirect, { gen_ctx_offset, output_offset, input_offset, input_size_offset });
    auto input_read = constrained_read_from_memory(
        call_ptr, clk, resolved_input_offset, AvmMemoryTag::FF, AvmMemoryTag::U0, IntermRegister::IA);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::PEDERSEN);

    uint32_t pedersen_clk = clk;
    // We read the input and output addresses in one row as they should contain FF elements
    main_trace.push_back(Row{
        .main_clk = clk,
        .main_ia = input_read.val, // First element of input
        .main_ind_addr_a = FF(input_read.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(input_read.direct_address),
        .main_pc = FF(pc++),
        .main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
        .main_sel_mem_op_a = FF(1),
        .main_sel_op_pedersen = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(input_read.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!input_read.tag_match)),
    });
    clk++;
    // We read the input size and gen_ctx addresses in one row as they should contain U32 elements
    auto input_size_read = constrained_read_from_memory(
        call_ptr, clk, resolved_input_size_offset, AvmMemoryTag::U32, AvmMemoryTag::U0, IntermRegister::IA);
    auto gen_ctx_read = constrained_read_from_memory(
        call_ptr, clk, resolved_gen_ctx_offset, AvmMemoryTag::U32, AvmMemoryTag::U0, IntermRegister::IB);
    main_trace.push_back(Row{
        .main_clk = clk,
        .main_ia = input_size_read.val,
        .main_ib = gen_ctx_read.val,
        .main_ind_addr_a = FF(input_size_read.indirect_address),
        .main_ind_addr_b = FF(gen_ctx_read.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(input_size_read.direct_address),
        .main_mem_addr_b = FF(gen_ctx_read.direct_address),
        .main_pc = FF(pc),
        .main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U32)),
        .main_sel_mem_op_a = FF(1),
        .main_sel_mem_op_b = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(input_size_read.is_indirect)),
        .main_sel_resolve_ind_addr_b = FF(static_cast<uint32_t>(gen_ctx_read.is_indirect)),
    });
    clk++;

    std::vector<FF> inputs;
    uint32_t num_main_rows = read_slice_to_memory<FF>(call_ptr,
                                                      clk,
                                                      resolved_input_offset,
                                                      AvmMemoryTag::FF,
                                                      AvmMemoryTag::FF,
                                                      FF(internal_return_ptr),
                                                      uint32_t(input_size_read.val),
                                                      inputs);
    clk += num_main_rows;
    FF output = pedersen_trace_builder.pedersen_hash(inputs, uint32_t(gen_ctx_read.val), pedersen_clk);
    write_slice_to_memory(
        call_ptr, clk, resolved_output_offset, AvmMemoryTag::FF, AvmMemoryTag::FF, FF(internal_return_ptr), { output });
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
    auto [resolved_lhs_x_offset,
          resolved_lhs_y_offset,
          resolved_lhs_is_inf_offset,
          resolved_rhs_x_offset,
          resolved_rhs_y_offset,
          resolved_rhs_is_inf_offset,
          resolved_output_offset] = unpack_indirects<7>(indirect,
                                                        { lhs_x_offset,
                                                          lhs_y_offset,
                                                          lhs_is_inf_offset,
                                                          rhs_x_offset,
                                                          rhs_y_offset,
                                                          rhs_is_inf_offset,
                                                          output_offset });
    // Load lhs point
    auto lhs_x_read = constrained_read_from_memory(
        call_ptr, clk, resolved_lhs_x_offset, AvmMemoryTag::FF, AvmMemoryTag::U0, IntermRegister::IA);
    auto lhs_y_read = constrained_read_from_memory(
        call_ptr, clk, resolved_lhs_y_offset, AvmMemoryTag::FF, AvmMemoryTag::U0, IntermRegister::IB);
    // Load rhs point
    auto rhs_x_read = constrained_read_from_memory(
        call_ptr, clk, resolved_rhs_x_offset, AvmMemoryTag::FF, AvmMemoryTag::U0, IntermRegister::IC);
    auto rhs_y_read = constrained_read_from_memory(
        call_ptr, clk, resolved_rhs_y_offset, AvmMemoryTag::FF, AvmMemoryTag::U0, IntermRegister::ID);
    bool tag_match = lhs_x_read.tag_match && lhs_y_read.tag_match && rhs_x_read.tag_match && rhs_y_read.tag_match;

    // Save this clk time to line up with the gadget op.
    auto ecc_clk = clk;
    main_trace.push_back(Row{
        .main_clk = clk,
        .main_ia = lhs_x_read.val,
        .main_ib = lhs_y_read.val,
        .main_ic = rhs_x_read.val,
        .main_id = rhs_y_read.val,
        .main_ind_addr_a = FF(lhs_x_read.indirect_address),
        .main_ind_addr_b = FF(lhs_y_read.indirect_address),
        .main_ind_addr_c = FF(rhs_x_read.indirect_address),
        .main_ind_addr_d = FF(rhs_y_read.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(lhs_x_read.direct_address),
        .main_mem_addr_b = FF(lhs_y_read.direct_address),
        .main_mem_addr_c = FF(rhs_x_read.direct_address),
        .main_mem_addr_d = FF(rhs_y_read.direct_address),
        .main_pc = FF(pc++),
        .main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
        .main_sel_mem_op_a = FF(1),
        .main_sel_mem_op_b = FF(1),
        .main_sel_mem_op_c = FF(1),
        .main_sel_mem_op_d = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(lhs_x_read.is_indirect)),
        .main_sel_resolve_ind_addr_b = FF(static_cast<uint32_t>(lhs_y_read.is_indirect)),
        .main_sel_resolve_ind_addr_c = FF(static_cast<uint32_t>(rhs_x_read.is_indirect)),
        .main_sel_resolve_ind_addr_d = FF(static_cast<uint32_t>(rhs_y_read.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
    });
    clk++;
    // Load the infinite bools separately since they have a different memory tag
    auto lhs_is_inf_read = constrained_read_from_memory(
        call_ptr, clk, resolved_lhs_is_inf_offset, AvmMemoryTag::U8, AvmMemoryTag::U0, IntermRegister::IA);
    auto rhs_is_inf_read = constrained_read_from_memory(
        call_ptr, clk, resolved_rhs_is_inf_offset, AvmMemoryTag::U8, AvmMemoryTag::U0, IntermRegister::IB);
    bool tag_match_inf = lhs_is_inf_read.tag_match && rhs_is_inf_read.tag_match;

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_ia = lhs_is_inf_read.val,
        .main_ib = rhs_is_inf_read.val,
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(lhs_is_inf_offset),
        .main_mem_addr_b = FF(rhs_is_inf_offset),
        .main_pc = FF(pc),
        .main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U8)),
        .main_sel_mem_op_a = FF(1),
        .main_sel_mem_op_b = FF(1),
        .main_tag_err = FF(static_cast<uint32_t>(!tag_match_inf)),
    });
    clk++;
    grumpkin::g1::affine_element lhs = uint8_t(lhs_is_inf_read.val) == 1
                                           ? grumpkin::g1::affine_element::infinity()
                                           : grumpkin::g1::affine_element{ lhs_x_read.val, lhs_y_read.val };
    grumpkin::g1::affine_element rhs = uint8_t(rhs_is_inf_read.val) == 1
                                           ? grumpkin::g1::affine_element::infinity()
                                           : grumpkin::g1::affine_element{ rhs_x_read.val, rhs_y_read.val };
    auto result = ecc_trace_builder.embedded_curve_add(lhs, rhs, ecc_clk);

    // Write point coordinates
    auto write_x = constrained_write_to_memory(
        call_ptr, clk, resolved_output_offset, result.x, AvmMemoryTag::U0, AvmMemoryTag::FF, IntermRegister::IA);
    // Write y (directly) using the write_x.direct_address + 1
    mem_trace_builder.write_into_memory(
        call_ptr, clk, IntermRegister::IB, write_x.direct_address + 1, result.y, AvmMemoryTag::U0, AvmMemoryTag::FF);
    main_trace.push_back(Row{
        .main_clk = clk,
        .main_ia = result.x,
        .main_ib = result.y,
        .main_ind_addr_a = FF(write_x.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(write_x.direct_address),
        .main_mem_addr_b = FF(write_x.direct_address + 1),
        .main_pc = FF(pc),
        .main_rwa = FF(1),
        .main_rwb = FF(1),
        .main_sel_mem_op_a = FF(1),
        .main_sel_mem_op_b = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(write_x.is_indirect)),
        .main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
    });
    clk++;
    mem_trace_builder.write_into_memory(call_ptr,
                                        clk,
                                        IntermRegister::IA,
                                        write_x.direct_address + 2,
                                        result.is_point_at_infinity(),
                                        AvmMemoryTag::U0,
                                        AvmMemoryTag::U8);

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_ia = result.is_point_at_infinity(),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(write_x.direct_address + 2),
        .main_pc = FF(pc),
        .main_rwa = FF(1),
        .main_sel_mem_op_a = FF(1),
        .main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U8)),
    });
}

void AvmTraceBuilder::op_variable_msm(uint8_t indirect,
                                      uint32_t points_offset,
                                      uint32_t scalars_offset,
                                      uint32_t output_offset,
                                      uint32_t point_length_offset)
{
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;
    auto [resolved_points_offset, resolved_scalars_offset, resolved_output_offset] =
        unpack_indirects<3>(indirect, { points_offset, scalars_offset, output_offset });

    auto points_length_read = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IA, point_length_offset, AvmMemoryTag::U32, AvmMemoryTag::U0);

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_ia = points_length_read.val,
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(point_length_offset),
        .main_pc = FF(pc),
        .main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U32)),
        .main_sel_mem_op_a = FF(1),
        .main_tag_err = FF(static_cast<uint32_t>(!points_length_read.tag_match)),
    });
    clk++;

    // Points are stored as [x1, y1, inf1, x2, y2, inf2, ...] with the types [FF, FF, U8, FF, FF, U8, ...]
    uint32_t num_points = uint32_t(points_length_read.val) / 3; // 3 elements per point
    // We need to split up the reads due to the memory tags,
    std::vector<FF> points_coords_vec;
    std::vector<FF> points_inf_vec;
    std::vector<FF> scalars_vec;
    AddressWithMode coords_offset = resolved_points_offset;
    // Loading the points is a bit more complex since we need to read the coordinates and the infinity flags
    // separately The current circuit constraints does not allow for multiple memory tags to be loaded from within
    // the same row. If we could we would be able to replace the following loops with a single read_slice_to_memory
    // call. For now we load the coordinates first and then the infinity flags, and finally splice them together
    // when creating the points

    // Read the coordinates first, +2 since we read 2 points per row, the first load could be indirect
    for (uint32_t i = 0; i < num_points; i++) {
        auto point_x1_read = constrained_read_from_memory(
            call_ptr, clk, coords_offset, AvmMemoryTag::FF, AvmMemoryTag::U0, IntermRegister::IA);
        auto point_y1_read = mem_trace_builder.read_and_load_from_memory(
            call_ptr, clk, IntermRegister::IB, point_x1_read.direct_address + 1, AvmMemoryTag::FF, AvmMemoryTag::U0);

        bool tag_match = point_x1_read.tag_match && point_y1_read.tag_match;
        points_coords_vec.insert(points_coords_vec.end(), { point_x1_read.val, point_y1_read.val });
        main_trace.push_back(Row{
            .main_clk = clk,
            .main_ia = point_x1_read.val,
            .main_ib = point_y1_read.val,
            .main_ind_addr_a = FF(point_x1_read.indirect_address),
            .main_internal_return_ptr = FF(internal_return_ptr),
            .main_mem_addr_a = FF(point_x1_read.direct_address),
            .main_mem_addr_b = FF(point_x1_read.direct_address + 1),
            .main_pc = FF(pc),
            .main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
            .main_sel_mem_op_a = FF(1),
            .main_sel_mem_op_b = FF(1),
            .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(point_x1_read.is_indirect)),
            .main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
        });
        clk++;
        // Update the coords offset to read the next point (subsequent points are always direct and separated by 3
        // addresses)
        coords_offset = { AddressingMode::DIRECT, point_x1_read.direct_address + 3 };
    }
    uint32_t inf_direct_address = resolved_points_offset.offset + 2;
    // Read the Infinities flags
    for (uint32_t i = 0; i < num_points; i++) {
        auto point_inf_read = mem_trace_builder.read_and_load_from_memory(
            call_ptr, clk, IntermRegister::IA, inf_direct_address, AvmMemoryTag::U8, AvmMemoryTag::U0);
        points_inf_vec.emplace_back(point_inf_read.val);

        main_trace.push_back(Row{
            .main_clk = clk,
            .main_ia = point_inf_read.val,
            .main_internal_return_ptr = FF(internal_return_ptr),
            .main_mem_addr_a = FF(inf_direct_address),
            .main_pc = FF(pc),
            .main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U8)),
            .main_sel_mem_op_a = FF(1),
            .main_tag_err = FF(static_cast<uint32_t>(!point_inf_read.tag_match)),
        });
        clk++;
        // Update the inf offset to read the next point (subsequent points are always direct and separated by 3
        inf_direct_address += 3;
    }
    // Scalar read length is num_points* 2 since scalars are stored as lo and hi limbs
    uint32_t scalar_read_length = num_points * 2;
    // Scalars are easy to read since they are stored as [lo1, hi1, lo2, hi2, ...] with the types [FF, FF, FF,FF,
    // ...]
    auto num_scalar_rows = read_slice_to_memory(call_ptr,
                                                clk,
                                                resolved_scalars_offset,
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
    auto write_x = constrained_write_to_memory(
        call_ptr, clk, resolved_output_offset, result.x, AvmMemoryTag::U0, AvmMemoryTag::FF, IntermRegister::IA);
    mem_trace_builder.write_into_memory(
        call_ptr, clk, IntermRegister::IB, write_x.direct_address + 1, result.y, AvmMemoryTag::U0, AvmMemoryTag::FF);

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_ia = result.x,
        .main_ib = result.y,
        .main_ind_addr_a = FF(write_x.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(write_x.direct_address),
        .main_mem_addr_b = FF(write_x.direct_address + 1),
        .main_pc = FF(pc),
        .main_rwa = FF(1),
        .main_rwb = FF(1),
        .main_sel_mem_op_a = FF(1),
        .main_sel_mem_op_b = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(write_x.is_indirect)),
        .main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
    });
    clk++;
    // Write the infinity
    mem_trace_builder.write_into_memory(call_ptr,
                                        clk,
                                        IntermRegister::IA,
                                        write_x.direct_address + 2,
                                        result.is_point_at_infinity(),
                                        AvmMemoryTag::U0,
                                        AvmMemoryTag::U8);
    main_trace.push_back(Row{
        .main_clk = clk,
        .main_ia = static_cast<uint8_t>(result.is_point_at_infinity()),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(write_x.direct_address + 2),
        .main_pc = FF(pc),
        .main_rwa = FF(1),
        .main_sel_mem_op_a = FF(1),
        .main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U8)),
    });

    pc++;
}

/**************************************************************************************************
 *                                   CONVERSIONS
 **************************************************************************************************/

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
    auto [resolved_src_offset, resolved_dst_offset] = unpack_indirects<2>(indirect, { src_offset, dst_offset });

    auto read_src = constrained_read_from_memory(
        call_ptr, clk, resolved_src_offset, AvmMemoryTag::FF, AvmMemoryTag::U8, IntermRegister::IA);

    auto read_dst = constrained_read_from_memory(
        call_ptr, clk, resolved_dst_offset, AvmMemoryTag::FF, AvmMemoryTag::U8, IntermRegister::IB);

    FF input = read_src.val;

    // In case of a memory tag error, we do not perform the computation.
    // Therefore, we do not create any entry in gadget table and return a vector of 0
    std::vector<uint8_t> res = read_src.tag_match
                                   ? conversion_trace_builder.op_to_radix_le(input, radix, num_limbs, clk)
                                   : std::vector<uint8_t>(num_limbs, 0);

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::TORADIXLE);

    // This is the row that contains the selector to trigger the sel_op_radix_le
    // In this row, we read the input value and the destination address into register A and B respectively
    main_trace.push_back(Row{
        .main_clk = clk,
        .main_call_ptr = call_ptr,
        .main_ia = input,
        .main_ib = read_dst.val,
        .main_ic = radix,
        .main_id = num_limbs,
        .main_ind_addr_a = read_src.indirect_address,
        .main_ind_addr_b = read_dst.indirect_address,
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = read_src.direct_address,
        .main_mem_addr_b = read_dst.direct_address,
        .main_pc = FF(pc++),
        .main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::FF)),
        .main_sel_mem_op_a = FF(1),
        .main_sel_mem_op_b = FF(1),
        .main_sel_op_radix_le = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(read_src.is_indirect)),
        .main_sel_resolve_ind_addr_b = FF(static_cast<uint32_t>(read_dst.is_indirect)),
        .main_w_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U8)),
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
        call_ptr, clk, resolved_dst_offset, AvmMemoryTag::FF, AvmMemoryTag::U8, FF(internal_return_ptr), ff_res);
}

/**************************************************************************************************
 *                                   FUTURE GADGETS -- pending changes in noir
 **************************************************************************************************/

/**
 * @brief SHA256 Compression with direct or indirect memory access.
 *
 * @param indirect byte encoding information about indirect/direct memory access.
 * @param h_init_offset An index in memory pointing to the first U32 value of the state array to be used in the next
 * instance of sha256 compression.
 * @param input_offset An index in memory pointing to the first U32 value of the input array to be used in the next
 * instance of sha256 compression.
 * @param output_offset An index in memory pointing to where the first U32 value of the output array should be
 * stored.
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
    auto [resolved_h_init_offset, resolved_input_offset, resolved_output_offset] =
        unpack_indirects<3>(indirect, { h_init_offset, input_offset, output_offset });

    auto read_a = constrained_read_from_memory(
        call_ptr, clk, resolved_h_init_offset, AvmMemoryTag::U32, AvmMemoryTag::U0, IntermRegister::IA);
    auto read_b = constrained_read_from_memory(
        call_ptr, clk, resolved_input_offset, AvmMemoryTag::U32, AvmMemoryTag::U0, IntermRegister::IB);
    bool tag_match = read_a.tag_match && read_b.tag_match;

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
        .main_clk = clk,
        .main_ia = read_a.val, // First element of state
        .main_ib = read_b.val, // First element of input
        .main_ind_addr_a = FF(read_a.indirect_address),
        .main_ind_addr_b = FF(read_b.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(read_a.direct_address),
        .main_mem_addr_b = FF(read_b.direct_address),
        .main_pc = FF(pc++),
        .main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U32)),
        .main_sel_mem_op_a = FF(1),
        .main_sel_mem_op_b = FF(1),
        .main_sel_op_sha256 = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(read_a.is_indirect)),
        .main_sel_resolve_ind_addr_b = FF(static_cast<uint32_t>(read_b.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
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
                                   resolved_h_init_offset,
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
                                   resolved_input_offset,
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
    write_slice_to_memory(call_ptr,
                          clk,
                          resolved_output_offset,
                          AvmMemoryTag::U32,
                          AvmMemoryTag::U32,
                          FF(internal_return_ptr),
                          ff_result);
}

/**
 * @brief Keccakf1600  with direct or indirect memory access.
 * This function temporarily has the same interface as the kecccak opcode for compatibility, when the keccak
 * migration is complete (to keccakf1600) We will update this function call as we will not likely need
 * input_size_offset
 * @param indirect byte encoding information about indirect/direct memory access.
 * @param output_offset An index in memory pointing to where the first u64 value of the output array should be
 * stored.
 * @param input_offset An index in memory pointing to the first u64 value of the input array to be used in the next
 * instance of poseidon2 permutation.
 * @param input_size offset An index in memory pointing to the size of the input array. Temporary while we maintain
 * the same interface as keccak (this is fixed to 25)
 */
void AvmTraceBuilder::op_keccakf1600(uint8_t indirect,
                                     uint32_t output_offset,
                                     uint32_t input_offset,
                                     uint32_t input_size_offset)
{
    // What happens if the input_size_offset is > 25 when the state is more that that?
    auto clk = static_cast<uint32_t>(main_trace.size()) + 1;
    auto [resolved_output_offset, resolved_input_offset] =
        unpack_indirects<2>(indirect, { output_offset, input_offset });
    auto input_read = constrained_read_from_memory(
        call_ptr, clk, resolved_input_offset, AvmMemoryTag::U64, AvmMemoryTag::U0, IntermRegister::IA);
    auto output_read = constrained_read_from_memory(
        call_ptr, clk, resolved_output_offset, AvmMemoryTag::U64, AvmMemoryTag::U0, IntermRegister::IC);
    bool tag_match = input_read.tag_match && output_read.tag_match;

    // Constrain gas cost
    gas_trace_builder.constrain_gas_lookup(clk, OpCode::KECCAKF1600);

    main_trace.push_back(Row{
        .main_clk = clk,
        .main_ia = input_read.val,  // First element of input
        .main_ic = output_read.val, // First element of output
        .main_ind_addr_a = FF(input_read.indirect_address),
        .main_ind_addr_c = FF(output_read.indirect_address),
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_a = FF(input_read.direct_address),
        .main_mem_addr_c = FF(output_read.direct_address),
        .main_pc = FF(pc++),
        .main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U64)),
        .main_sel_mem_op_a = FF(1),
        .main_sel_mem_op_c = FF(1),
        .main_sel_op_keccak = FF(1),
        .main_sel_resolve_ind_addr_a = FF(static_cast<uint32_t>(input_read.is_indirect)),
        .main_sel_resolve_ind_addr_c = FF(static_cast<uint32_t>(output_read.is_indirect)),
        .main_tag_err = FF(static_cast<uint32_t>(!tag_match)),
    });
    // We store the current clk this main trace row occurred so that we can line up the keccak gadget operation
    // at the same clk later.
    auto keccak_op_clk = clk;
    // We need to increment the clk
    clk++;
    auto input_length_read = mem_trace_builder.read_and_load_from_memory(
        call_ptr, clk, IntermRegister::IB, input_size_offset, AvmMemoryTag::U32, AvmMemoryTag::U0);
    main_trace.push_back(Row{
        .main_clk = clk,
        .main_ib = input_length_read.val, // Message Length
        .main_internal_return_ptr = FF(internal_return_ptr),
        .main_mem_addr_b = FF(input_size_offset), // length
        .main_pc = FF(pc),
        .main_r_in_tag = FF(static_cast<uint32_t>(AvmMemoryTag::U32)),
        .main_sel_mem_op_b = FF(1),
        .main_tag_err = FF(static_cast<uint32_t>(!input_length_read.tag_match)),
    });
    clk++;
    // Array input is fixed to 1600 bits
    std::vector<uint64_t> input_vec;
    // Read results are written to input array
    uint32_t num_main_rows = read_slice_to_memory<uint64_t>(call_ptr,
                                                            clk,
                                                            resolved_input_offset,
                                                            AvmMemoryTag::U64,
                                                            AvmMemoryTag::U0,
                                                            FF(internal_return_ptr),
                                                            25,
                                                            input_vec);

    std::array<uint64_t, 25> input = vec_to_arr<uint64_t, 25>(input_vec);
    // Increment the clock by 7 since (25 reads / 4 reads per row = 7)
    clk += num_main_rows;

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
        call_ptr, clk, resolved_output_offset, AvmMemoryTag::U0, AvmMemoryTag::U64, FF(internal_return_ptr), ff_result);
}

/**************************************************************************************************
 *                                   FINALIZE
 **************************************************************************************************/

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
    auto slice_trace = slice_trace_builder.finalize();
    const auto& fixed_gas_table = FixedGasTable::get();
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
    size_t slice_trace_size = slice_trace.size();

    // Data structure to collect all lookup counts pertaining to 16-bit/32-bit range checks in memory trace
    std::unordered_map<uint16_t, uint32_t> mem_rng_check_lo_counts;
    std::unordered_map<uint16_t, uint32_t> mem_rng_check_mid_counts;
    std::unordered_map<uint8_t, uint32_t> mem_rng_check_hi_counts;

    // Main Trace needs to be at least as big as the biggest subtrace.
    // If the bin_trace_size has entries, we need the main_trace to be as big as our byte lookup table (3 *
    // 2**16 long)
    size_t const lookup_table_size = (bin_trace_size > 0 && range_check_required) ? 3 * (1 << 16) : 0;
    size_t const range_check_size = range_check_required ? UINT16_MAX + 1 : 0;
    std::vector<size_t> trace_sizes = { mem_trace_size,     main_trace_size,        alu_trace_size,
                                        range_check_size,   conv_trace_size,        lookup_table_size,
                                        sha256_trace_size,  poseidon2_trace_size,   pedersen_trace_size,
                                        gas_trace_size + 1, KERNEL_INPUTS_LENGTH,   KERNEL_OUTPUTS_LENGTH,
                                        min_trace_size,     fixed_gas_table.size(), slice_trace_size,
                                        calldata.size() };
    auto trace_size = std::max_element(trace_sizes.begin(), trace_sizes.end());

    // We only need to pad with zeroes to the size to the largest trace here, pow_2 padding is handled in the
    // subgroup_size check in bb
    // Resize the main_trace to accomodate a potential lookup, filling with default empty rows.
    main_trace_size = *trace_size;
    main_trace.resize(*trace_size, {});

    main_trace.at(*trace_size - 1).main_sel_last = FF(1);

    /**********************************************************************************************
     * MEMORY TRACE INCLUSION
     **********************************************************************************************/

    // We compute in the main loop the timestamp and global address for next row.
    // Perform initialization for index 0 outside of the loop provided that mem trace exists.
    if (mem_trace_size > 0) {
        main_trace.at(0).mem_tsp =
            FF(AvmMemTraceBuilder::NUM_SUB_CLK * mem_trace.at(0).m_clk + mem_trace.at(0).m_sub_clk);
        main_trace.at(0).mem_glob_addr =
            FF(mem_trace.at(0).m_addr + (static_cast<uint64_t>(mem_trace.at(0).m_space_id) << 32));
    }

    for (size_t i = 0; i < mem_trace_size; i++) {
        auto const& src = mem_trace.at(i);
        auto& dest = main_trace.at(i);

        dest.mem_sel_mem = FF(1);
        dest.mem_clk = FF(src.m_clk);
        dest.mem_addr = FF(src.m_addr);
        dest.mem_space_id = FF(src.m_space_id);
        dest.mem_val = src.m_val;
        dest.mem_rw = FF(static_cast<uint32_t>(src.m_rw));
        dest.mem_r_in_tag = FF(static_cast<uint32_t>(src.r_in_tag));
        dest.mem_w_in_tag = FF(static_cast<uint32_t>(src.w_in_tag));
        dest.mem_tag = FF(static_cast<uint32_t>(src.m_tag));
        dest.mem_tag_err = FF(static_cast<uint32_t>(src.m_tag_err));
        dest.mem_one_min_inv = src.m_one_min_inv;
        dest.mem_sel_mov_ia_to_ic = FF(static_cast<uint32_t>(src.m_sel_mov_ia_to_ic));
        dest.mem_sel_mov_ib_to_ic = FF(static_cast<uint32_t>(src.m_sel_mov_ib_to_ic));
        dest.mem_sel_op_cmov = FF(static_cast<uint32_t>(src.m_sel_cmov));
        dest.mem_sel_op_slice = FF(static_cast<uint32_t>(src.m_sel_op_slice));

        dest.incl_mem_tag_err_counts = FF(static_cast<uint32_t>(src.m_tag_err_count_relevant));

        // Calldatacopy/return memory operations are handled differently and are activated by m_sel_op_slice.
        if (!src.m_sel_op_slice) {
            switch (src.m_sub_clk) {
            case AvmMemTraceBuilder::SUB_CLK_LOAD_A:
            case AvmMemTraceBuilder::SUB_CLK_STORE_A:
                dest.mem_sel_op_a = 1;
                break;
            case AvmMemTraceBuilder::SUB_CLK_LOAD_B:
            case AvmMemTraceBuilder::SUB_CLK_STORE_B:
                dest.mem_sel_op_b = 1;
                break;
            case AvmMemTraceBuilder::SUB_CLK_LOAD_C:
            case AvmMemTraceBuilder::SUB_CLK_STORE_C:
                dest.mem_sel_op_c = 1;
                break;
            case AvmMemTraceBuilder::SUB_CLK_LOAD_D:
            case AvmMemTraceBuilder::SUB_CLK_STORE_D:
                dest.mem_sel_op_d = 1;
                break;
            case AvmMemTraceBuilder::SUB_CLK_IND_LOAD_A:
                dest.mem_sel_resolve_ind_addr_a = 1;
                break;
            case AvmMemTraceBuilder::SUB_CLK_IND_LOAD_B:
                dest.mem_sel_resolve_ind_addr_b = 1;
                break;
            case AvmMemTraceBuilder::SUB_CLK_IND_LOAD_C:
                dest.mem_sel_resolve_ind_addr_c = 1;
                break;
            case AvmMemTraceBuilder::SUB_CLK_IND_LOAD_D:
                dest.mem_sel_resolve_ind_addr_d = 1;
                break;
            default:
                break;
            }
        }

        if (src.m_sel_cmov || src.m_sel_op_slice) {
            dest.mem_skip_check_tag =
                dest.mem_sel_op_cmov * (dest.mem_sel_op_d + dest.mem_sel_op_a * (-dest.mem_sel_mov_ia_to_ic + 1) +
                                        dest.mem_sel_op_b * (-dest.mem_sel_mov_ib_to_ic + 1)) +
                dest.mem_sel_op_slice;
        }

        if (i + 1 < mem_trace_size) {
            auto const& next = mem_trace.at(i + 1);
            auto& dest_next = main_trace.at(i + 1);
            dest_next.mem_tsp = FF(AvmMemTraceBuilder::NUM_SUB_CLK * next.m_clk + next.m_sub_clk);
            dest_next.mem_glob_addr = FF(next.m_addr + (static_cast<uint64_t>(next.m_space_id) << 32));

            FF diff{};
            if (dest_next.mem_glob_addr == dest.mem_glob_addr) {
                diff = dest_next.mem_tsp - dest.mem_tsp;
            } else {
                diff = dest_next.mem_glob_addr - dest.mem_glob_addr;
                dest.mem_lastAccess = FF(1);
            }
            dest.mem_sel_rng_chk = FF(1);

            // Decomposition of diff
            auto const diff_64 = uint64_t(diff);
            auto const diff_hi = static_cast<uint8_t>(diff_64 >> 32);
            auto const diff_mid = static_cast<uint16_t>((diff_64 & UINT32_MAX) >> 16);
            auto const diff_lo = static_cast<uint16_t>(diff_64 & UINT16_MAX);
            dest.mem_diff_hi = FF(diff_hi);
            dest.mem_diff_mid = FF(diff_mid);
            dest.mem_diff_lo = FF(diff_lo);

            // Add the range checks counts
            mem_rng_check_hi_counts[diff_hi]++;
            mem_rng_check_mid_counts[diff_mid]++;
            mem_rng_check_lo_counts[diff_lo]++;
        } else {
            dest.mem_lastAccess = FF(1);
            dest.mem_last = FF(1);
        }
    }

    /**********************************************************************************************
     * ALU TRACE INCLUSION
     **********************************************************************************************/

    for (size_t i = 0; i < alu_trace_size; i++) {
        auto const& src = alu_trace.at(i);
        auto& dest = main_trace.at(i);

        dest.alu_clk = FF(static_cast<uint32_t>(src.alu_clk));

        dest.alu_op_add = FF(static_cast<uint32_t>(src.alu_op_add));
        dest.alu_op_sub = FF(static_cast<uint32_t>(src.alu_op_sub));
        dest.alu_op_mul = FF(static_cast<uint32_t>(src.alu_op_mul));
        dest.alu_op_not = FF(static_cast<uint32_t>(src.alu_op_not));
        dest.alu_op_eq = FF(static_cast<uint32_t>(src.alu_op_eq));
        dest.alu_op_lt = FF(static_cast<uint32_t>(src.alu_op_lt));
        dest.alu_op_lte = FF(static_cast<uint32_t>(src.alu_op_lte));
        dest.alu_op_cast = FF(static_cast<uint32_t>(src.alu_op_cast));
        dest.alu_op_cast_prev = FF(static_cast<uint32_t>(src.alu_op_cast_prev));
        dest.alu_sel_cmp = FF(static_cast<uint8_t>(src.alu_op_lt) + static_cast<uint8_t>(src.alu_op_lte));
        dest.alu_sel_rng_chk = FF(static_cast<uint8_t>(src.rng_chk_sel));
        dest.alu_op_shr = FF(static_cast<uint8_t>(src.alu_op_shr));
        dest.alu_op_shl = FF(static_cast<uint8_t>(src.alu_op_shl));
        dest.alu_op_div = FF(static_cast<uint8_t>(src.alu_op_div));

        dest.alu_ff_tag = FF(static_cast<uint32_t>(src.alu_ff_tag));
        dest.alu_u8_tag = FF(static_cast<uint32_t>(src.alu_u8_tag));
        dest.alu_u16_tag = FF(static_cast<uint32_t>(src.alu_u16_tag));
        dest.alu_u32_tag = FF(static_cast<uint32_t>(src.alu_u32_tag));
        dest.alu_u64_tag = FF(static_cast<uint32_t>(src.alu_u64_tag));
        dest.alu_u128_tag = FF(static_cast<uint32_t>(src.alu_u128_tag));

        dest.alu_in_tag = dest.alu_u8_tag + FF(2) * dest.alu_u16_tag + FF(3) * dest.alu_u32_tag +
                          FF(4) * dest.alu_u64_tag + FF(5) * dest.alu_u128_tag + FF(6) * dest.alu_ff_tag;

        dest.alu_ia = src.alu_ia;
        dest.alu_ib = src.alu_ib;
        dest.alu_ic = src.alu_ic;

        dest.alu_cf = FF(static_cast<uint32_t>(src.alu_cf));

        dest.alu_u8_r0 = FF(src.alu_u8_r0);
        dest.alu_u8_r1 = FF(src.alu_u8_r1);

        dest.alu_u16_r0 = FF(src.alu_u16_reg.at(0));
        dest.alu_u16_r1 = FF(src.alu_u16_reg.at(1));
        dest.alu_u16_r2 = FF(src.alu_u16_reg.at(2));
        dest.alu_u16_r3 = FF(src.alu_u16_reg.at(3));
        dest.alu_u16_r4 = FF(src.alu_u16_reg.at(4));
        dest.alu_u16_r5 = FF(src.alu_u16_reg.at(5));
        dest.alu_u16_r6 = FF(src.alu_u16_reg.at(6));
        dest.alu_u16_r7 = FF(src.alu_u16_reg.at(7));
        dest.alu_u16_r8 = FF(src.alu_u16_reg.at(8));
        dest.alu_u16_r9 = FF(src.alu_u16_reg.at(9));
        dest.alu_u16_r10 = FF(src.alu_u16_reg.at(10));
        dest.alu_u16_r11 = FF(src.alu_u16_reg.at(11));
        dest.alu_u16_r12 = FF(src.alu_u16_reg.at(12));
        dest.alu_u16_r13 = FF(src.alu_u16_reg.at(13));
        dest.alu_u16_r14 = FF(src.alu_u16_reg.at(14));

        dest.alu_sel_div_rng_chk = FF(static_cast<uint8_t>(src.div_u64_range_chk_sel));
        dest.alu_div_u16_r0 = FF(src.div_u64_range_chk.at(0));
        dest.alu_div_u16_r1 = FF(src.div_u64_range_chk.at(1));
        dest.alu_div_u16_r2 = FF(src.div_u64_range_chk.at(2));
        dest.alu_div_u16_r3 = FF(src.div_u64_range_chk.at(3));
        dest.alu_div_u16_r4 = FF(src.div_u64_range_chk.at(4));
        dest.alu_div_u16_r5 = FF(src.div_u64_range_chk.at(5));
        dest.alu_div_u16_r6 = FF(src.div_u64_range_chk.at(6));
        dest.alu_div_u16_r7 = FF(src.div_u64_range_chk.at(7));
        dest.alu_op_eq_diff_inv = FF(src.alu_op_eq_diff_inv);

        // Not all rows in ALU are enabled with a selector. For instance,
        // multiplication over u128 and cast is taking two lines.
        if (AvmAluTraceBuilder::is_alu_row_enabled(src)) {
            dest.alu_sel_alu = FF(1);
        }

        if (dest.alu_sel_cmp == FF(1) || dest.alu_sel_rng_chk == FF(1)) {
            dest.alu_a_lo = FF(src.hi_lo_limbs.at(0));
            dest.alu_a_hi = FF(src.hi_lo_limbs.at(1));
            dest.alu_b_lo = FF(src.hi_lo_limbs.at(2));
            dest.alu_b_hi = FF(src.hi_lo_limbs.at(3));
            dest.alu_p_sub_a_lo = FF(src.hi_lo_limbs.at(4));
            dest.alu_p_sub_a_hi = FF(src.hi_lo_limbs.at(5));
            dest.alu_p_sub_b_lo = FF(src.hi_lo_limbs.at(6));
            dest.alu_p_sub_b_hi = FF(src.hi_lo_limbs.at(7));
            dest.alu_res_lo = FF(src.hi_lo_limbs.at(8));
            dest.alu_res_hi = FF(src.hi_lo_limbs.at(9));
            dest.alu_p_a_borrow = FF(static_cast<uint8_t>(src.p_a_borrow));
            dest.alu_p_b_borrow = FF(static_cast<uint8_t>(src.p_b_borrow));
            dest.alu_borrow = FF(static_cast<uint8_t>(src.borrow));
            dest.alu_cmp_rng_ctr = FF(static_cast<uint8_t>(src.cmp_rng_ctr));
            dest.alu_sel_rng_chk_lookup = FF(1);
        }
        if (dest.alu_op_div == FF(1)) {
            dest.alu_op_div_std = uint256_t(src.alu_ia) >= uint256_t(src.alu_ib);
            dest.alu_op_div_a_lt_b = uint256_t(src.alu_ia) < uint256_t(src.alu_ib);
            dest.alu_sel_rng_chk_lookup = FF(1);
            dest.alu_a_lo = FF(src.hi_lo_limbs.at(0));
            dest.alu_a_hi = FF(src.hi_lo_limbs.at(1));
            dest.alu_b_lo = FF(src.hi_lo_limbs.at(2));
            dest.alu_b_hi = FF(src.hi_lo_limbs.at(3));
            dest.alu_p_sub_a_lo = FF(src.hi_lo_limbs.at(4));
            dest.alu_p_sub_a_hi = FF(src.hi_lo_limbs.at(5));
            dest.alu_remainder = src.remainder;
            dest.alu_divisor_lo = src.divisor_lo;
            dest.alu_divisor_hi = src.divisor_hi;
            dest.alu_quotient_lo = src.quotient_lo;
            dest.alu_quotient_hi = src.quotient_hi;
            dest.alu_partial_prod_lo = src.partial_prod_lo;
            dest.alu_partial_prod_hi = src.partial_prod_hi;
        }

        if (dest.alu_op_add == FF(1) || dest.alu_op_sub == FF(1) || dest.alu_op_mul == FF(1)) {
            dest.alu_sel_rng_chk_lookup = FF(1);
        }

        if (dest.alu_op_cast == FF(1)) {
            dest.alu_a_lo = FF(src.hi_lo_limbs.at(0));
            dest.alu_a_hi = FF(src.hi_lo_limbs.at(1));
            dest.alu_p_sub_a_lo = FF(src.hi_lo_limbs.at(2));
            dest.alu_p_sub_a_hi = FF(src.hi_lo_limbs.at(3));
            dest.alu_p_a_borrow = FF(static_cast<uint8_t>(src.p_a_borrow));
            dest.alu_sel_rng_chk_lookup = FF(1);
        }

        if (dest.alu_op_cast_prev == FF(1)) {
            dest.alu_a_lo = FF(src.hi_lo_limbs.at(0));
            dest.alu_a_hi = FF(src.hi_lo_limbs.at(1));
            dest.alu_sel_rng_chk_lookup = FF(1);
        }

        // Multiplication over u128 expands over two rows.
        if (dest.alu_op_mul == FF(1) && dest.alu_u128_tag) {
            main_trace.at(i + 1).alu_sel_rng_chk_lookup = FF(1);
        }
        if (src.alu_op_shr || src.alu_op_shl) {
            dest.alu_a_lo = FF(src.hi_lo_limbs[0]);
            dest.alu_a_hi = FF(src.hi_lo_limbs[1]);
            dest.alu_b_lo = FF(src.hi_lo_limbs[2]);
            dest.alu_b_hi = FF(src.hi_lo_limbs[3]);
            dest.alu_sel_shift_which = FF(1);
            dest.alu_shift_lt_bit_len = FF(static_cast<uint8_t>(src.shift_lt_bit_len));
            dest.alu_t_sub_s_bits = FF(src.mem_tag_sub_shift);
            dest.alu_two_pow_s = FF(uint256_t(1) << dest.alu_ib);
            dest.alu_two_pow_t_sub_s = FF(uint256_t(1) << uint256_t(dest.alu_t_sub_s_bits));
            dest.alu_sel_rng_chk_lookup = FF(1);
        }
    }

    /**********************************************************************************************
     * GADGET TABLES INCLUSION
     **********************************************************************************************/

    // Add Conversion Gadget table
    for (size_t i = 0; i < conv_trace_size; i++) {
        auto const& src = conv_trace.at(i);
        auto& dest = main_trace.at(i);
        dest.conversion_sel_to_radix_le = FF(static_cast<uint8_t>(src.to_radix_le_sel));
        dest.conversion_clk = FF(src.conversion_clk);
        dest.conversion_input = src.input;
        dest.conversion_radix = FF(src.radix);
        dest.conversion_num_limbs = FF(src.num_limbs);
    }

    // Add SHA256 Gadget table
    for (size_t i = 0; i < sha256_trace_size; i++) {
        auto const& src = sha256_trace.at(i);
        auto& dest = main_trace.at(i);
        dest.sha256_clk = FF(src.clk);
        dest.sha256_input = src.input[0];
        // TODO: This will need to be enabled later
        // dest.sha256_output = src.output[0];
        dest.sha256_sel_sha256_compression = FF(1);
        dest.sha256_state = src.state[0];
    }

    // Add Poseidon2 Gadget table
    for (size_t i = 0; i < poseidon2_trace_size; i++) {
        auto const& src = poseidon2_trace.at(i);
        auto& dest = main_trace.at(i);
        dest.poseidon2_clk = FF(src.clk);
        dest.poseidon2_input = src.input[0];
        // TODO: This will need to be enabled later
        // dest.poseidon2_output = src.output[0];
        dest.poseidon2_sel_poseidon_perm = FF(1);
    }

    // Add KeccakF1600 Gadget table
    for (size_t i = 0; i < keccak_trace_size; i++) {
        auto const& src = keccak_trace.at(i);
        auto& dest = main_trace.at(i);
        dest.keccakf1600_clk = FF(src.clk);
        dest.keccakf1600_input = FF(src.input[0]);
        // TODO: This will need to be enabled later
        // dest.keccakf1600_output = src.output[0];
        dest.keccakf1600_sel_keccakf1600 = FF(1);
    }

    // Add Pedersen Gadget table
    for (size_t i = 0; i < pedersen_trace_size; i++) {
        auto const& src = pedersen_trace.at(i);
        auto& dest = main_trace.at(i);
        dest.pedersen_clk = FF(src.clk);
        dest.pedersen_input = FF(src.input[0]);
        dest.pedersen_sel_pedersen = FF(1);
    }

    /**********************************************************************************************
     * SLICE TRACE INCLUSION
     **********************************************************************************************/
    for (size_t i = 0; i < slice_trace_size; i++) {
        merge_into(main_trace.at(i), slice_trace.at(i));
    }

    /**********************************************************************************************
     * BINARY TRACE INCLUSION
     **********************************************************************************************/

    // Add Binary Trace table
    for (size_t i = 0; i < bin_trace_size; i++) {
        auto const& src = bin_trace.at(i);
        auto& dest = main_trace.at(i);
        dest.binary_clk = src.binary_clk;
        dest.binary_sel_bin = static_cast<uint8_t>(src.bin_sel);
        dest.binary_acc_ia = src.acc_ia;
        dest.binary_acc_ib = src.acc_ib;
        dest.binary_acc_ic = src.acc_ic;
        dest.binary_in_tag = src.in_tag;
        dest.binary_op_id = src.op_id;
        dest.binary_ia_bytes = src.bin_ia_bytes;
        dest.binary_ib_bytes = src.bin_ib_bytes;
        dest.binary_ic_bytes = src.bin_ic_bytes;
        dest.binary_start = FF(static_cast<uint8_t>(src.start));
        dest.binary_mem_tag_ctr = src.mem_tag_ctr;
        dest.binary_mem_tag_ctr_inv = src.mem_tag_ctr_inv;
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

                        main_trace.at(main_trace_index).byte_lookup_sel_bin = FF(1);
                        main_trace.at(main_trace_index).byte_lookup_table_op_id = op_id;
                        main_trace.at(main_trace_index).byte_lookup_table_input_a = a;
                        main_trace.at(main_trace_index).byte_lookup_table_input_b = b;
                        // Add the counter value stored throughout the execution
                        main_trace.at(main_trace_index).lookup_byte_operations_counts =
                            bin_trace_builder.byte_operation_counter[main_trace_index];
                        if (op_id == 0) {
                            main_trace.at(main_trace_index).byte_lookup_table_output = a & b;
                        } else if (op_id == 1) {
                            main_trace.at(main_trace_index).byte_lookup_table_output = a | b;
                        } else {
                            main_trace.at(main_trace_index).byte_lookup_table_output = a ^ b;
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
            main_trace.at(avm_in_tag).byte_lookup_sel_bin = FF(1);
            main_trace.at(avm_in_tag).byte_lookup_table_in_tags = avm_in_tag + 1;
            main_trace.at(avm_in_tag).byte_lookup_table_byte_lengths = static_cast<uint8_t>(pow(2, avm_in_tag));
            main_trace.at(avm_in_tag).lookup_byte_lengths_counts =
                bin_trace_builder.byte_length_counter[avm_in_tag + 1];
        }
    }

    /**********************************************************************************************
     * GAS TRACE INCLUSION
     **********************************************************************************************/

    // Add the gas cost table to the main trace
    // TODO: do i need a way to produce an interupt that will stop the execution of the trace when the gas left
    // becomes zero in the gas_trace_builder Does all of the gas trace information need to be added to this main
    // machine?????

    // Add the gas accounting for each row
    // We can assume that the gas trace will never be larger than the main trace
    // We infer that a row is active for gas (.main_gas_cost_active = 1) based on the presence
    // of a gas entry row.
    // Set the initial gas
    auto& first_opcode_row = main_trace.at(0);
    first_opcode_row.main_l2_gas_remaining = gas_trace_builder.initial_l2_gas;
    first_opcode_row.main_da_gas_remaining = gas_trace_builder.initial_da_gas;
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
            next.main_l2_gas_remaining = current_l2_gas_remaining;
            next.main_da_gas_remaining = current_da_gas_remaining;
            current_clk++;
        }

        auto& dest = main_trace.at(gas_entry.clk - 1);
        auto& next = main_trace.at(gas_entry.clk);

        // Write each of the relevant gas accounting values
        dest.main_opcode_val = static_cast<uint8_t>(gas_entry.opcode);
        dest.main_l2_gas_op_cost = gas_entry.l2_gas_cost;
        dest.main_da_gas_op_cost = gas_entry.da_gas_cost;

        // If gas remaining is increasing, it means we underflowed in uint32_t
        bool l2_out_of_gas = current_l2_gas_remaining < gas_entry.remaining_l2_gas;
        bool da_out_of_gas = current_da_gas_remaining < gas_entry.remaining_da_gas;

        uint32_t abs_l2_gas_remaining = l2_out_of_gas ? -gas_entry.remaining_l2_gas : gas_entry.remaining_l2_gas;
        uint32_t abs_da_gas_remaining = da_out_of_gas ? -gas_entry.remaining_da_gas : gas_entry.remaining_da_gas;

        dest.main_abs_l2_rem_gas_hi = abs_l2_gas_remaining >> 16;
        dest.main_abs_da_rem_gas_hi = abs_da_gas_remaining >> 16;
        dest.main_abs_l2_rem_gas_lo = static_cast<uint16_t>(abs_l2_gas_remaining);
        dest.main_abs_da_rem_gas_lo = static_cast<uint16_t>(abs_da_gas_remaining);

        // TODO: gas is not constrained for external call at this time
        if (gas_entry.opcode != OpCode::CALL) {
            dest.main_sel_gas_accounting_active = FF(1);

            // lookups counting
            rem_gas_rng_check_counts[L2_HI_GAS_COUNTS_IDX][static_cast<uint16_t>(dest.main_abs_l2_rem_gas_hi)]++;
            rem_gas_rng_check_counts[L2_LO_GAS_COUNTS_IDX][static_cast<uint16_t>(dest.main_abs_l2_rem_gas_lo)]++;
            rem_gas_rng_check_counts[DA_HI_GAS_COUNTS_IDX][static_cast<uint16_t>(dest.main_abs_da_rem_gas_hi)]++;
            rem_gas_rng_check_counts[DA_LO_GAS_COUNTS_IDX][static_cast<uint16_t>(dest.main_abs_da_rem_gas_lo)]++;
        }

        dest.main_l2_out_of_gas = static_cast<uint32_t>(l2_out_of_gas);
        dest.main_da_out_of_gas = static_cast<uint32_t>(da_out_of_gas);

        current_l2_gas_remaining = gas_entry.remaining_l2_gas;
        current_da_gas_remaining = gas_entry.remaining_da_gas;
        next.main_l2_gas_remaining =
            l2_out_of_gas ? FF::modulus - uint256_t(abs_l2_gas_remaining) : current_l2_gas_remaining;
        next.main_da_gas_remaining =
            da_out_of_gas ? FF::modulus - uint256_t(abs_da_gas_remaining) : current_da_gas_remaining;

        current_clk++;
    }

    // Pad the rest of the trace with the same gas remaining
    for (size_t i = current_clk; i < main_trace_size; i++) {
        auto& dest = main_trace.at(i);
        dest.main_l2_gas_remaining = current_l2_gas_remaining;
        dest.main_da_gas_remaining = current_da_gas_remaining;
    }

    // Adding extra row for the shifted values at the top of the execution trace.
    Row first_row = Row{ .main_sel_first = FF(1), .mem_lastAccess = FF(1) };
    main_trace.insert(main_trace.begin(), first_row);

    /**********************************************************************************************
     * RANGE CHECKS AND SELECTORS INCLUSION
     **********************************************************************************************/

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

        if ((r.main_sel_op_add == FF(1) || r.main_sel_op_sub == FF(1) || r.main_sel_op_mul == FF(1) ||
             r.main_sel_op_eq == FF(1) || r.main_sel_op_not == FF(1) || r.main_sel_op_lt == FF(1) ||
             r.main_sel_op_lte == FF(1) || r.main_sel_op_cast == FF(1) || r.main_sel_op_shr == FF(1) ||
             r.main_sel_op_shl == FF(1) || r.main_sel_op_div == FF(1)) &&
            r.main_tag_err == FF(0) && r.main_op_err == FF(0)) {
            r.main_sel_alu = FF(1);
        }

        if (r.main_sel_op_internal_call == FF(1) || r.main_sel_op_internal_return == FF(1)) {
            r.main_space_id = INTERNAL_CALL_SPACE_ID;
        } else {
            r.main_space_id = r.main_call_ptr;
        };

        r.main_clk = i >= old_trace_size ? r.main_clk : FF(i);
        auto counter = i >= old_trace_size ? static_cast<uint32_t>(r.main_clk) : static_cast<uint32_t>(i);
        r.incl_main_tag_err_counts = mem_trace_builder.m_tag_err_lookup_counts[static_cast<uint32_t>(counter)];

        if (counter <= UINT8_MAX) {
            auto counter_u8 = static_cast<uint8_t>(counter);
            r.lookup_u8_0_counts = alu_trace_builder.u8_range_chk_counters[0][counter_u8];
            r.lookup_u8_1_counts = alu_trace_builder.u8_range_chk_counters[1][counter_u8];
            r.lookup_pow_2_0_counts = alu_trace_builder.u8_pow_2_counters[0][counter_u8];
            r.lookup_pow_2_1_counts = alu_trace_builder.u8_pow_2_counters[1][counter_u8];
            r.lookup_mem_rng_chk_hi_counts = mem_rng_check_hi_counts[counter_u8];
            r.main_sel_rng_8 = FF(1);

            // Also merge the powers of 2 table.
            merge_into(r, FixedPowersTable::get().at(counter));
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

            r.main_sel_rng_16 = FF(1);
        }
    }

    /**********************************************************************************************
     * KERNEL TRACE INCLUSION
     **********************************************************************************************/

    // Write the kernel trace into the main trace
    // 1. The write offsets are constrained to be non changing over the entire trace, so we fill in the values
    // until we
    //    hit an operation that changes one of the write_offsets (a relevant opcode)
    // 2. Upon hitting the clk of each kernel operation we copy the values into the main trace
    // 3. When an increment is required, we increment the value in the next row, then continue the process until
    // the end
    // 4. Whenever we hit the last row, we zero all write_offsets such that the shift relation will succeed
    std::vector<AvmKernelTraceBuilder::KernelTraceEntry> kernel_trace = kernel_trace_builder.finalize();
    size_t kernel_padding_main_trace_bottom = 1;

    // Index 1 corresponds here to the first active row of the main execution trace, as
    // we already prepended the extra row for shifted columns. Therefore, initialization
    // of side_effect_counter occurs occurs on this row.
    main_trace.at(1).kernel_side_effect_counter = initial_side_effect_counter;
    // This index is required to retrieve the right side effect counter after an external call.
    size_t external_call_cnt = 0;

    // External loop iterates over the kernel entries which are sorted by increasing clk.
    // Internal loop iterates to fill the gap in main trace between each kernel entries.
    for (auto const& src : kernel_trace) {
        // Check the clock and iterate through the main trace until we hit the clock
        auto clk = src.clk;

        // Until the next kernel changing instruction is encountered we set all of the values of the offset
        // arrays to be the same as the previous row This satisfies the `offset' - (offset + operation_selector)
        // = 0` constraints
        for (size_t j = kernel_padding_main_trace_bottom; j < clk; j++) {
            auto const& prev = main_trace.at(j);
            auto& dest = main_trace.at(j + 1);

            dest.kernel_note_hash_exist_write_offset = prev.kernel_note_hash_exist_write_offset;
            dest.kernel_emit_note_hash_write_offset = prev.kernel_emit_note_hash_write_offset;
            dest.kernel_nullifier_exists_write_offset = prev.kernel_nullifier_exists_write_offset;
            dest.kernel_nullifier_non_exists_write_offset = prev.kernel_nullifier_non_exists_write_offset;
            dest.kernel_emit_nullifier_write_offset = prev.kernel_emit_nullifier_write_offset;
            dest.kernel_emit_l2_to_l1_msg_write_offset = prev.kernel_emit_l2_to_l1_msg_write_offset;
            dest.kernel_emit_unencrypted_log_write_offset = prev.kernel_emit_unencrypted_log_write_offset;
            dest.kernel_l1_to_l2_msg_exists_write_offset = prev.kernel_l1_to_l2_msg_exists_write_offset;
            dest.kernel_sload_write_offset = prev.kernel_sload_write_offset;
            dest.kernel_sstore_write_offset = prev.kernel_sstore_write_offset;

            // Adjust side effect counter after an external call
            if (prev.main_sel_op_external_call == 1) {
                dest.kernel_side_effect_counter =
                    execution_hints.externalcall_hints.at(external_call_cnt).end_side_effect_counter;
                external_call_cnt++;
            } else {
                dest.kernel_side_effect_counter = prev.kernel_side_effect_counter;
            }
        }

        Row& curr = main_trace.at(clk);

        // Read in values from kernel trace
        // Lookup values
        curr.kernel_kernel_in_offset = src.kernel_in_offset;
        curr.kernel_kernel_out_offset = src.kernel_out_offset;
        curr.main_sel_q_kernel_lookup = static_cast<uint32_t>(src.q_kernel_lookup);
        curr.main_sel_q_kernel_output_lookup = static_cast<uint32_t>(src.q_kernel_output_lookup);

        // Operation selectors
        curr.main_sel_op_note_hash_exists = static_cast<uint32_t>(src.op_note_hash_exists);
        curr.main_sel_op_emit_note_hash = static_cast<uint32_t>(src.op_emit_note_hash);
        curr.main_sel_op_nullifier_exists = static_cast<uint32_t>(src.op_nullifier_exists);
        curr.main_sel_op_emit_nullifier = static_cast<uint32_t>(src.op_emit_nullifier);
        curr.main_sel_op_l1_to_l2_msg_exists = static_cast<uint32_t>(src.op_l1_to_l2_msg_exists);
        curr.main_sel_op_emit_unencrypted_log = static_cast<uint32_t>(src.op_emit_unencrypted_log);
        curr.main_sel_op_emit_l2_to_l1_msg = static_cast<uint32_t>(src.op_emit_l2_to_l1_msg);
        curr.main_sel_op_sload = static_cast<uint32_t>(src.op_sload);
        curr.main_sel_op_sstore = static_cast<uint32_t>(src.op_sstore);

        if (clk < old_trace_size) {
            Row& next = main_trace.at(clk + 1);

            // Increment the write offset counter for the following row
            next.kernel_note_hash_exist_write_offset =
                curr.kernel_note_hash_exist_write_offset + static_cast<FF>(src.op_note_hash_exists);
            next.kernel_emit_note_hash_write_offset =
                curr.kernel_emit_note_hash_write_offset + static_cast<FF>(src.op_emit_note_hash);
            next.kernel_emit_nullifier_write_offset =
                curr.kernel_emit_nullifier_write_offset + static_cast<FF>(src.op_emit_nullifier);
            next.kernel_nullifier_exists_write_offset =
                curr.kernel_nullifier_exists_write_offset + (static_cast<FF>(src.op_nullifier_exists) * curr.main_ib);
            next.kernel_nullifier_non_exists_write_offset =
                curr.kernel_nullifier_non_exists_write_offset +
                (static_cast<FF>(src.op_nullifier_exists) * (FF(1) - curr.main_ib));
            next.kernel_l1_to_l2_msg_exists_write_offset =
                curr.kernel_l1_to_l2_msg_exists_write_offset + static_cast<FF>(src.op_l1_to_l2_msg_exists);
            next.kernel_emit_l2_to_l1_msg_write_offset =
                curr.kernel_emit_l2_to_l1_msg_write_offset + static_cast<FF>(src.op_emit_l2_to_l1_msg);
            next.kernel_emit_unencrypted_log_write_offset =
                curr.kernel_emit_unencrypted_log_write_offset + static_cast<FF>(src.op_emit_unencrypted_log);
            next.kernel_sload_write_offset = curr.kernel_sload_write_offset + static_cast<FF>(src.op_sload);
            next.kernel_sstore_write_offset = curr.kernel_sstore_write_offset + static_cast<FF>(src.op_sstore);

            // The side effect counter will increment regardless of the offset value
            next.kernel_side_effect_counter = curr.kernel_side_effect_counter + 1;
        }

        kernel_padding_main_trace_bottom = clk + 1;
    }

    // Pad out the main trace from the bottom of the main trace until the end
    for (size_t i = kernel_padding_main_trace_bottom + 1; i < old_trace_size; ++i) {

        Row const& prev = main_trace.at(i - 1);
        Row& dest = main_trace.at(i);

        // Setting all of the counters to 0 after the IS_LAST check so we can satisfy the constraints until the
        // end
        if (i == old_trace_size) {
            dest.kernel_note_hash_exist_write_offset = 0;
            dest.kernel_emit_note_hash_write_offset = 0;
            dest.kernel_nullifier_exists_write_offset = 0;
            dest.kernel_nullifier_non_exists_write_offset = 0;
            dest.kernel_emit_nullifier_write_offset = 0;
            dest.kernel_l1_to_l2_msg_exists_write_offset = 0;
            dest.kernel_emit_unencrypted_log_write_offset = 0;
            dest.kernel_emit_l2_to_l1_msg_write_offset = 0;
            dest.kernel_sload_write_offset = 0;
            dest.kernel_sstore_write_offset = 0;
            dest.kernel_side_effect_counter = 0;
        } else {
            dest.kernel_note_hash_exist_write_offset = prev.kernel_note_hash_exist_write_offset;
            dest.kernel_emit_note_hash_write_offset = prev.kernel_emit_note_hash_write_offset;
            dest.kernel_nullifier_exists_write_offset = prev.kernel_nullifier_exists_write_offset;
            dest.kernel_nullifier_non_exists_write_offset = prev.kernel_nullifier_non_exists_write_offset;
            dest.kernel_emit_nullifier_write_offset = prev.kernel_emit_nullifier_write_offset;
            dest.kernel_l1_to_l2_msg_exists_write_offset = prev.kernel_l1_to_l2_msg_exists_write_offset;
            dest.kernel_emit_unencrypted_log_write_offset = prev.kernel_emit_unencrypted_log_write_offset;
            dest.kernel_emit_l2_to_l1_msg_write_offset = prev.kernel_emit_l2_to_l1_msg_write_offset;
            dest.kernel_sload_write_offset = prev.kernel_sload_write_offset;
            dest.kernel_sstore_write_offset = prev.kernel_sstore_write_offset;
            dest.kernel_side_effect_counter = prev.kernel_side_effect_counter;
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
            dest.kernel_q_public_input_kernel_add_to_table = FF(1);
        }
    }

    // Copy the kernel input public inputs
    for (size_t i = 0; i < KERNEL_INPUTS_LENGTH; i++) {
        main_trace.at(i).kernel_kernel_inputs = std::get<KERNEL_INPUTS>(kernel_trace_builder.public_inputs).at(i);
    }

    // Write lookup counts for outputs
    for (uint32_t i = 0; i < KERNEL_OUTPUTS_LENGTH; i++) {
        auto value = kernel_trace_builder.kernel_output_selector_counter.find(i);
        if (value != kernel_trace_builder.kernel_output_selector_counter.end()) {
            auto& dest = main_trace.at(i);
            dest.kernel_output_lookup_counts = FF(value->second);
            dest.kernel_q_public_input_kernel_out_add_to_table = FF(1);
        }
    }

    // Copy the kernel outputs counts into the main trace
    for (size_t i = 0; i < KERNEL_OUTPUTS_LENGTH; i++) {
        main_trace.at(i).kernel_kernel_value_out =
            std::get<KERNEL_OUTPUTS_VALUE>(kernel_trace_builder.public_inputs).at(i);

        main_trace.at(i).kernel_kernel_side_effect_out =
            std::get<KERNEL_OUTPUTS_SIDE_EFFECT_COUNTER>(kernel_trace_builder.public_inputs).at(i);

        main_trace.at(i).kernel_kernel_metadata_out =
            std::get<KERNEL_OUTPUTS_METADATA>(kernel_trace_builder.public_inputs).at(i);
    }

    // calldata column inclusion and selector
    for (size_t i = 0; i < calldata.size(); i++) {
        main_trace.at(i).main_calldata = calldata.at(i);
        main_trace.at(i).main_sel_calldata = 1;
    }

    // calldata loookup counts for calldatacopy operations
    for (auto const& [cd_offset, count] : slice_trace_builder.cd_lookup_counts) {
        main_trace.at(cd_offset).lookup_cd_value_counts = count;
    }

    // returndata column inclusion and selector
    for (size_t i = 0; i < returndata.size(); i++) {
        main_trace.at(i).main_returndata = returndata.at(i);
        main_trace.at(i).main_sel_returndata = 1;
    }

    // returndata loookup counts for return operations
    for (auto const& [cd_offset, count] : slice_trace_builder.ret_lookup_counts) {
        main_trace.at(cd_offset).lookup_ret_value_counts = count;
    }

    // Get tag_err counts from the mem_trace_builder
    if (range_check_required) {
        finalise_mem_trace_lookup_counts();
    }

    // Add the gas costs table to the main trace
    // For each opcode we write its l2 gas cost and da gas cost
    for (size_t i = 0; i < fixed_gas_table.size(); i++) {
        merge_into(main_trace.at(i), fixed_gas_table.at(i));
    }

    // Finalise gas left lookup counts
    for (auto const& [opcode, count] : gas_trace_builder.gas_opcode_lookup_counter) {
        main_trace.at(static_cast<uint8_t>(opcode)).lookup_opcode_gas_counts = count;
    }

    auto trace = std::move(main_trace);
    reset();

    return trace;
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
    slice_trace_builder.reset();

    external_call_counter = 0;
}

} // namespace bb::avm_trace
