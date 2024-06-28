#pragma once

#include <stack>

#include "barretenberg/vm/avm_trace/avm_alu_trace.hpp"
#include "barretenberg/vm/avm_trace/avm_binary_trace.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/avm_trace/avm_gas_trace.hpp"
#include "barretenberg/vm/avm_trace/avm_kernel_trace.hpp"
#include "barretenberg/vm/avm_trace/avm_mem_trace.hpp"
#include "barretenberg/vm/avm_trace/avm_opcode.hpp"
#include "barretenberg/vm/avm_trace/constants.hpp"
#include "barretenberg/vm/avm_trace/gadgets/avm_conversion_trace.hpp"
#include "barretenberg/vm/avm_trace/gadgets/avm_ecc.hpp"
#include "barretenberg/vm/avm_trace/gadgets/avm_keccak.hpp"
#include "barretenberg/vm/avm_trace/gadgets/avm_pedersen.hpp"
#include "barretenberg/vm/avm_trace/gadgets/avm_poseidon2.hpp"
#include "barretenberg/vm/avm_trace/gadgets/avm_sha256.hpp"
#include "barretenberg/vm/generated/avm_circuit_builder.hpp"

namespace bb::avm_trace {

using Row = bb::AvmFullRow<bb::fr>;
enum class AddressingMode {
    DIRECT,
    INDIRECT,
};
struct AddressWithMode {
    AddressingMode mode;
    uint32_t offset;
};

// This is the internal context that we keep along the lifecycle of bytecode execution
// to iteratively build the whole trace. This is effectively performing witness generation.
// At the end of circuit building, mainTrace can be moved to AvmCircuitBuilder by calling
// AvmCircuitBuilder::set_trace(rows).
class AvmTraceBuilder {

  public:
    AvmTraceBuilder(VmPublicInputs public_inputs = {},
                    ExecutionHints execution_hints = {},
                    uint32_t side_effect_counter = 0,
                    std::vector<FF> calldata = {});

    std::vector<Row> finalize(uint32_t min_trace_size = 0, bool range_check_required = ENABLE_PROVING);
    void reset();

    uint32_t getPc() const { return pc; }

    // Addition with direct or indirect memory access.
    void op_add(uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag);

    // Subtraction with direct or indirect memory access.
    void op_sub(uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag);

    // Multiplication with direct or indirect memory access.
    void op_mul(uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag);

    // Finite field division with direct or indirect memory access.
    void op_fdiv(uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset);

    // Bitwise not with direct or indirect memory access.
    void op_not(uint8_t indirect, uint32_t a_offset, uint32_t dst_offset, AvmMemoryTag in_tag);

    // Equality with direct or indirect memory access.
    void op_eq(uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag);

    // Bitwise and with direct or indirect memory access.
    void op_and(uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag);

    // Bitwise or with direct or indirect memory access.
    void op_or(uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag);

    // Bitwise xor with direct or indirect memory access.
    void op_xor(uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag);

    // Less Than with direct or indirect memory access.
    void op_lt(uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag);

    // Less Than or Equal to with direct or indirect memory access.
    void op_lte(uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag);

    // Shift Right with direct or indirect memory access.
    void op_shr(uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag);

    // Shift Left with direct or indirect memory access.
    void op_shl(uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag);

    // Set a constant from bytecode with direct or indirect memory access.
    void op_set(uint8_t indirect, uint128_t val, uint32_t dst_offset, AvmMemoryTag in_tag);

    // Move (copy) the value and tag of a memory cell to another one.
    void op_mov(uint8_t indirect, uint32_t src_offset, uint32_t dst_offset);

    // Move (copy) the value and tag of a memory cell to another one whereby the source
    // is determined conditionally based on a conditional value determined by cond_offset.
    void op_cmov(uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t cond_offset, uint32_t dst_offset);

    // Call Context
    void op_storage_address(uint8_t indirect, uint32_t dst_offset);
    void op_sender(uint8_t indirect, uint32_t dst_offset);
    void op_address(uint8_t indirect, uint32_t dst_offset);

    // Fees
    void op_fee_per_da_gas(uint8_t indirect, uint32_t dst_offset);
    void op_fee_per_l2_gas(uint8_t indirect, uint32_t dst_offset);
    void op_transaction_fee(uint8_t indirect, uint32_t dst_offset);

    // Globals
    void op_chain_id(uint8_t indirect, uint32_t dst_offset);
    void op_version(uint8_t indirect, uint32_t dst_offset);
    void op_block_number(uint8_t indirect, uint32_t dst_offset);
    void op_coinbase(uint8_t indirect, uint32_t dst_offset);
    void op_timestamp(uint8_t indirect, uint32_t dst_offset);

    // Outputs
    // With single output values
    void op_emit_note_hash(uint8_t indirect, uint32_t note_hash_offset);
    void op_emit_nullifier(uint8_t indirect, uint32_t nullifier_offset);
    void op_emit_unencrypted_log(uint8_t indirect, uint32_t log_offset, uint32_t log_size_offset);
    void op_emit_l2_to_l1_msg(uint8_t indirect, uint32_t recipient_offset, uint32_t content_offset);
    void op_get_contract_instance(uint8_t indirect, uint32_t address_offset, uint32_t dst_offset);

    // With additional metadata output
    void op_l1_to_l2_msg_exists(uint8_t indirect, uint32_t log_offset, uint32_t dest_offset);
    void op_note_hash_exists(uint8_t indirect, uint32_t note_hash_offset, uint32_t dest_offset);
    void op_nullifier_exists(uint8_t indirect, uint32_t nullifier_offset, uint32_t dest_offset);

    void op_sload(uint8_t indirect, uint32_t slot_offset, uint32_t size, uint32_t dest_offset);
    void op_sstore(uint8_t indirect, uint32_t src_offset, uint32_t size, uint32_t slot_offset);

    // Cast an element pointed by the address a_offset into type specified by dst_tag and
    // store the result in address given by dst_offset.
    void op_cast(uint8_t indirect, uint32_t a_offset, uint32_t dst_offset, AvmMemoryTag dst_tag);

    // Integer Division with direct or indirect memory access.
    void op_div(uint8_t indirect, uint32_t a_offset, uint32_t b_offset, uint32_t dst_offset, AvmMemoryTag in_tag);

    // Machine State - Gas
    void op_l2gasleft(uint8_t indirect, uint32_t dst_offset);
    void op_dagasleft(uint8_t indirect, uint32_t dst_offset);

    // Jump to a given program counter.
    void op_jump(uint32_t jmp_dest);

    // Jump conditionally to a given program counter.
    void op_jumpi(uint8_t indirect, uint32_t jmp_dest, uint32_t cond_offset);

    // Jump to a given program counter; storing the return location on a call stack.
    // TODO(md): this program counter MUST be an operand to the OPCODE.
    void op_internal_call(uint32_t jmp_dest);

    // Return from a jump.
    void op_internal_return();

    // Halt -> stop program execution.
    void halt();

    // CALLDATACOPY opcode with direct/indirect memory access, i.e.,
    // direct: M[dst_offset:dst_offset+copy_size] = calldata[cd_offset:cd_offset+copy_size]
    // indirect: M[M[dst_offset]:M[dst_offset]+copy_size] = calldata[cd_offset:cd_offset+copy_size]
    void op_calldata_copy(uint8_t indirect, uint32_t cd_offset, uint32_t copy_size, uint32_t dst_offset);

    // REVERT Opcode (that just call return under the hood for now)
    std::vector<FF> op_revert(uint8_t indirect, uint32_t ret_offset, uint32_t ret_size);
    // RETURN opcode with direct and indirect memory access, i.e.,
    // direct:   return(M[ret_offset:ret_offset+ret_size])
    // indirect: return(M[M[ret_offset]:M[ret_offset]+ret_size])
    std::vector<FF> op_return(uint8_t indirect, uint32_t ret_offset, uint32_t ret_size);

    // Calls
    void op_call(uint8_t indirect,
                 uint32_t gas_offset,
                 uint32_t addr_offset,
                 uint32_t args_offset,
                 uint32_t args_size,
                 uint32_t ret_offset,
                 uint32_t ret_size,
                 uint32_t success_offset,
                 uint32_t function_selector_offset);

    // Gadgets
    // --- Conversions
    // To Radix LE conversion operation.
    void op_to_radix_le(uint8_t indirect, uint32_t src_offset, uint32_t dst_offset, uint32_t radix, uint32_t num_limbs);
    // --- Hashing
    // Sha256 compression operation
    void op_sha256_compression(uint8_t indirect, uint32_t output_offset, uint32_t h_init_offset, uint32_t input_offset);
    // Poseidon2 Permutation operation
    void op_poseidon2_permutation(uint8_t indirect, uint32_t input_offset, uint32_t output_offset);
    // Keccakf1600 operation - interface will likely change (e..g no input size offset)
    void op_keccakf1600(uint8_t indirect, uint32_t output_offset, uint32_t input_offset, uint32_t input_size_offset);
    // Keccak operation - temporary while we transition to keccakf1600
    void op_keccak(uint8_t indirect, uint32_t output_offset, uint32_t input_offset, uint32_t input_size_offset);
    // SHA256 operation - temporary while we transition to sha256_compression
    void op_sha256(uint8_t indirect, uint32_t output_offset, uint32_t input_offset, uint32_t input_size_offset);
    // Pedersen Hash operation
    void op_pedersen_hash(uint8_t indirect,
                          uint32_t gen_ctx_offset,
                          uint32_t output_offset,
                          uint32_t input_offset,
                          uint32_t input_size_offset);
    // Embedded EC Add - the offsets are temporary
    void op_ec_add(uint8_t indirect,
                   uint32_t lhs_x_offset,
                   uint32_t lhs_y_offset,
                   uint32_t lhs_is_inf_offset,
                   uint32_t rhs_x_offset,
                   uint32_t rhs_y_offset,
                   uint32_t rhs_is_inf_offset,
                   uint32_t output_offset);
    void op_variable_msm(uint8_t indirect,
                         uint32_t points_offset,
                         uint32_t scalars_offset,
                         uint32_t output_offset,
                         uint32_t point_length_offset);

    struct MemOp {
        bool is_indirect;
        uint32_t indirect_address;
        uint32_t direct_address;
        AvmMemoryTag tag;
        bool tag_match;
        FF val;
    };

  private:
    std::vector<Row> main_trace;
    AvmMemTraceBuilder mem_trace_builder;
    AvmAluTraceBuilder alu_trace_builder;
    AvmBinaryTraceBuilder bin_trace_builder;
    AvmKernelTraceBuilder kernel_trace_builder;
    AvmGasTraceBuilder gas_trace_builder;
    AvmConversionTraceBuilder conversion_trace_builder;
    AvmSha256TraceBuilder sha256_trace_builder;
    AvmPoseidon2TraceBuilder poseidon2_trace_builder;
    AvmKeccakTraceBuilder keccak_trace_builder;
    AvmPedersenTraceBuilder pedersen_trace_builder;
    AvmEccTraceBuilder ecc_trace_builder;

    std::vector<FF> calldata{};

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
    Row create_kernel_lookup_opcode(
        uint8_t indirect, uint32_t dst_offset, uint32_t selector, FF value, AvmMemoryTag w_tag);

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
    Row create_kernel_output_opcode(uint8_t indirect, uint32_t clk, uint32_t data_offset);

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
    Row create_kernel_output_opcode_with_metadata(uint8_t indirect,
                                                  uint32_t clk,
                                                  uint32_t data_offset,
                                                  AvmMemoryTag data_r_tag,
                                                  uint32_t metadata_offset,
                                                  AvmMemoryTag metadata_r_tag);

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
    Row create_kernel_output_opcode_with_set_metadata_output_from_hint(uint8_t indirect,
                                                                       uint32_t clk,
                                                                       uint32_t data_offset,
                                                                       uint32_t metadata_offset);

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
    Row create_kernel_output_opcode_with_set_value_from_hint(uint8_t indirect,
                                                             uint32_t clk,
                                                             uint32_t data_offset,
                                                             uint32_t metadata_offset);

    void execute_gasleft(OpCode opcode, uint8_t indirect, uint32_t dst_offset);

    void finalise_mem_trace_lookup_counts();

    uint32_t pc = 0;
    uint32_t internal_return_ptr =
        0; // After a nested call, it should be initialized with MAX_SIZE_INTERNAL_STACK * call_ptr
    uint8_t call_ptr = 0;

    // Side effect counter will increment when any state writing values are
    // encountered
    uint32_t side_effect_counter = 0;
    uint32_t initial_side_effect_counter; // This one is constant.
    uint32_t external_call_counter = 0;

    // Execution hints aid witness solving for instructions that require auxiliary information to construct
    // Mapping of side effect counter -> value
    ExecutionHints execution_hints;

    MemOp constrained_read_from_memory(uint8_t space_id,
                                       uint32_t clk,
                                       AddressWithMode addr,
                                       AvmMemoryTag read_tag,
                                       AvmMemoryTag write_tag,
                                       IntermRegister reg);
    MemOp constrained_write_to_memory(uint8_t space_id,
                                      uint32_t clk,
                                      AddressWithMode addr,
                                      FF const& value,
                                      AvmMemoryTag read_tag,
                                      AvmMemoryTag write_tag,
                                      IntermRegister reg);

    // TODO(ilyas: #6383): Temporary way to bulk read slices
    template <typename MEM>
    uint32_t read_slice_to_memory(uint8_t space_id,
                                  uint32_t clk,
                                  AddressWithMode addr,
                                  AvmMemoryTag r_tag,
                                  AvmMemoryTag w_tag,
                                  FF internal_return_ptr,
                                  size_t slice_len,
                                  std::vector<MEM>& slice);
    uint32_t write_slice_to_memory(uint8_t space_id,
                                   uint32_t clk,
                                   AddressWithMode addr,
                                   AvmMemoryTag r_tag,
                                   AvmMemoryTag w_tag,
                                   FF internal_return_ptr,
                                   std::vector<FF> const& slice);
};

} // namespace bb::avm_trace
