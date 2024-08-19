#pragma once

#include "barretenberg/numeric/uint128/uint128.hpp"
#include "barretenberg/vm/avm/trace/common.hpp"
#include "barretenberg/vm/avm/trace/execution_hints.hpp"

#include "barretenberg/vm/constants.hpp"

#include <cstdint>
#include <unordered_map>
#include <vector>

namespace bb::avm_trace {

class AvmKernelTraceBuilder {
  public:
    enum class KernelTraceOpType {
        // IN
        ADDRESS,
        STORAGE_ADDRESS,
        SENDER,
        FUNCTION_SELECTOR,
        TRANSACTION_FEE,
        CHAIN_ID,
        VERSION,
        BLOCK_NUMBER,
        COINBASE,
        TIMESTAMP,
        FEE_PER_DA_GAS,
        FEE_PER_L2_GAS,
        // OUT
        SLOAD,
        SSTORE,
        NOTE_HASH_EXISTS,
        EMIT_NOTE_HASH,
        NULLIFIER_EXISTS,
        EMIT_NULLIFIER,
        L1_TO_L2_MSG_EXISTS,
        EMIT_UNENCRYPTED_LOG,
        EMIT_L2_TO_L1_MSG
    };

    // While the kernel trace is expected to be 1-1 with the main trace,
    // we store it in "compressed form". That is, only actual operations are stored.
    // Then, in finalize things are padded for in between clks.
    struct KernelTraceEntry {
        uint32_t clk = 0;
        uint32_t kernel_out_offset = 0;
        // In finalise, the main trace writes the correct write_offset for each operation based appearing selectors
        KernelTraceOpType operation;
    };

    // Counts the number of accesses into each SELECTOR for the environment selector lookups;
    std::unordered_map<uint32_t, uint32_t> kernel_input_selector_counter;

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/6484): as outputs are only written to once, we can
    // optimise this to just hardcode the counter to be the same as the lookup selector value!!!
    std::unordered_map<uint32_t, uint32_t> kernel_output_selector_counter;

    AvmKernelTraceBuilder(uint32_t initial_side_effect_counter, VmPublicInputs public_inputs, ExecutionHints hints)
        : initial_side_effect_counter(initial_side_effect_counter)
        , public_inputs(std::move(public_inputs))
        , hints(std::move(hints))
    {}

    void reset();
    void finalize(std::vector<AvmFullRow<FF>>& main_trace);
    void finalize_columns(std::vector<AvmFullRow<FF>>& main_trace) const;

    // Context
    FF op_address(uint32_t clk);
    FF op_storage_address(uint32_t clk);
    FF op_sender(uint32_t clk);
    FF op_function_selector(uint32_t clk);
    FF op_transaction_fee(uint32_t clk);

    // Globals
    FF op_chain_id(uint32_t clk);
    FF op_version(uint32_t clk);
    FF op_block_number(uint32_t clk);
    FF op_coinbase(uint32_t clk);
    FF op_timestamp(uint32_t clk);
    // Globals - Gas
    FF op_fee_per_da_gas(uint32_t clk);
    FF op_fee_per_l2_gas(uint32_t clk);

    // Outputs
    // Each returns the selector that was used
    void op_sload(uint32_t clk, uint32_t side_effect_counter, const FF& slot, const FF& value);
    void op_sstore(uint32_t clk, uint32_t side_effect_counter, const FF& slot, const FF& value);
    void op_note_hash_exists(uint32_t clk, uint32_t side_effect_counter, const FF& note_hash, uint32_t result);
    void op_emit_note_hash(uint32_t clk, uint32_t side_effect_counter, const FF& note_hash);
    void op_nullifier_exists(uint32_t clk, uint32_t side_effect_counter, const FF& nullifier, uint32_t result);
    void op_emit_nullifier(uint32_t clk, uint32_t side_effect_counter, const FF& nullifier);
    void op_l1_to_l2_msg_exists(uint32_t clk, uint32_t side_effect_counter, const FF& message, uint32_t result);
    void op_emit_unencrypted_log(uint32_t clk, uint32_t side_effect_counter, const FF& log_hash);
    void op_emit_l2_to_l1_msg(uint32_t clk, uint32_t side_effect_counter, const FF& l2_to_l1_msg, const FF& recipient);

  private:
    std::vector<KernelTraceEntry> kernel_trace;

    uint32_t initial_side_effect_counter;
    VmPublicInputs public_inputs;
    ExecutionHints hints;

    // Output index counters
    uint32_t note_hash_exists_offset = 0;
    uint32_t emit_note_hash_offset = 0;
    uint32_t nullifier_exists_offset = 0;
    uint32_t nullifier_non_exists_offset = 0;
    uint32_t emit_nullifier_offset = 0;
    uint32_t l1_to_l2_msg_exists_offset = 0;
    uint32_t emit_unencrypted_log_offset = 0;
    uint32_t emit_l2_to_l1_msg_offset = 0;
    uint32_t sload_write_offset = 0;
    uint32_t sstore_write_offset = 0;

    FF perform_kernel_input_lookup(uint32_t selector);
    void perform_kernel_output_lookup(uint32_t write_offset,
                                      uint32_t side_effect_counter,
                                      const FF& value,
                                      const FF& metadata);
};
} // namespace bb::avm_trace
