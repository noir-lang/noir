#pragma once

#include "barretenberg/numeric/uint128/uint128.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/avm_trace/aztec_constants.hpp"
#include "constants.hpp"

#include <cstdint>
#include <unordered_map>
#include <vector>

namespace bb::avm_trace {

class AvmKernelTraceBuilder {
  public:
    struct KernelTraceEntry {
        // Clk - to join black onto the main trace
        uint32_t clk = 0;
        uint32_t kernel_in_offset = 0;
        uint32_t kernel_out_offset = 0;
        bool q_kernel_lookup = false;
        bool q_kernel_output_lookup = false;

        // In finalise, the main trace writes the correct write_offset for each operation based appearing selectors
        bool op_note_hash_exists = false;
        bool op_emit_note_hash = false;
        bool op_nullifier_exists = false;
        bool op_emit_nullifier = false;
        bool op_l1_to_l2_msg_exists = false;
        bool op_emit_unencrypted_log = false;
        bool op_emit_l2_to_l1_msg = false;
        bool op_sload = false;
        bool op_sstore = false;
    };

    VmPublicInputs public_inputs;

    // Counts the number of accesses into each SELECTOR for the environment selector lookups;
    std::unordered_map<uint32_t, uint32_t> kernel_input_selector_counter;

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/6484): as outputs are only written to once, we can
    // optimise this to just hardcode the counter to be the same as the lookup selector value!!!
    std::unordered_map<uint32_t, uint32_t> kernel_output_selector_counter;

    // Constructor receives copy of kernel_inputs from the main trace builder
    AvmKernelTraceBuilder(VmPublicInputs public_inputs);

    void reset();
    std::vector<KernelTraceEntry> finalize();

    // Context
    FF op_address();
    FF op_storage_address();
    FF op_sender();
    FF op_function_selector();
    FF op_transaction_fee();

    // Globals
    FF op_chain_id();
    FF op_version();
    FF op_block_number();
    FF op_coinbase();
    FF op_timestamp();
    // Globals - Gas
    FF op_fee_per_da_gas();
    FF op_fee_per_l2_gas();

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
