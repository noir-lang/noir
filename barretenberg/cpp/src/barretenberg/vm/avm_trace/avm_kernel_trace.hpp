#pragma once

#include "barretenberg/numeric/uint128/uint128.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/avm_trace/aztec_constants.hpp"
#include "constants.hpp"

#include <cstdint>
#include <unordered_map>
#include <vector>

inline const uint32_t SENDER_SELECTOR = 0;
inline const uint32_t ADDRESS_SELECTOR = 1;
inline const uint32_t STORAGE_ADDRESS_SELECTOR = 2;

inline const uint32_t START_GLOBAL_VARIABLES = CALL_CONTEXT_LENGTH + HEADER_LENGTH;

inline const uint32_t CHAIN_ID_SELECTOR = START_GLOBAL_VARIABLES;
inline const uint32_t VERSION_SELECTOR = START_GLOBAL_VARIABLES + 1;
inline const uint32_t BLOCK_NUMBER_SELECTOR = START_GLOBAL_VARIABLES + 2;
inline const uint32_t TIMESTAMP_SELECTOR = START_GLOBAL_VARIABLES + 3;
inline const uint32_t COINBASE_SELECTOR = START_GLOBAL_VARIABLES + 4;

inline const uint32_t END_GLOBAL_VARIABLES = START_GLOBAL_VARIABLES + GLOBAL_VARIABLES_LENGTH;
inline const uint32_t START_SIDE_EFFECT_COUNTER = END_GLOBAL_VARIABLES;

// TODO(https://github.com/AztecProtocol/aztec-packages/issues/6715): update these to come from the global inputs
inline const uint32_t FEE_PER_DA_GAS_SELECTOR = START_GLOBAL_VARIABLES + 6;
inline const uint32_t FEE_PER_L2_GAS_SELECTOR = START_GLOBAL_VARIABLES + 7;
inline const uint32_t TRANSACTION_FEE_SELECTOR = KERNEL_INPUTS_LENGTH - 1;

const std::array<uint32_t, 11> KERNEL_INPUTS_SELECTORS = {
    SENDER_SELECTOR,         ADDRESS_SELECTOR,         STORAGE_ADDRESS_SELECTOR, FEE_PER_DA_GAS_SELECTOR,
    FEE_PER_L2_GAS_SELECTOR, TRANSACTION_FEE_SELECTOR, CHAIN_ID_SELECTOR,        VERSION_SELECTOR,
    BLOCK_NUMBER_SELECTOR,   COINBASE_SELECTOR,        TIMESTAMP_SELECTOR
};

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
    FF op_sender();
    FF op_address();
    FF op_storage_address();

    // Fees
    FF op_fee_per_da_gas();
    FF op_fee_per_l2_gas();
    FF op_transaction_fee();

    // Globals
    FF op_chain_id();
    FF op_version();
    FF op_block_number();
    FF op_coinbase();
    FF op_timestamp();

    // Outputs
    // Each returns the selector that was used
    void op_note_hash_exists(uint32_t clk, uint32_t side_effect_counter, const FF& note_hash, uint32_t result);
    void op_emit_note_hash(uint32_t clk, uint32_t side_effect_counter, const FF& note_hash);
    void op_nullifier_exists(uint32_t clk, uint32_t side_effect_counter, const FF& nullifier, uint32_t result);
    void op_emit_nullifier(uint32_t clk, uint32_t side_effect_counter, const FF& nullifier);
    void op_l1_to_l2_msg_exists(uint32_t clk, uint32_t side_effect_counter, const FF& message, uint32_t result);
    void op_emit_unencrypted_log(uint32_t clk, uint32_t side_effect_counter, const FF& log_hash);
    void op_emit_l2_to_l1_msg(uint32_t clk, uint32_t side_effect_counter, const FF& message, const FF& recipient);

    void op_sload(uint32_t clk, uint32_t side_effect_counter, const FF& slot, const FF& value);
    void op_sstore(uint32_t clk, uint32_t side_effect_counter, const FF& slot, const FF& value);

    // TODO: Move into constants.hpp?
    static const uint32_t START_NOTE_HASH_EXISTS_WRITE_OFFSET = 0;
    static const uint32_t START_NULLIFIER_EXISTS_OFFSET =
        START_NOTE_HASH_EXISTS_WRITE_OFFSET + MAX_NOTE_HASH_READ_REQUESTS_PER_CALL;
    static const uint32_t START_NULLIFIER_NON_EXISTS_OFFSET =
        START_NULLIFIER_EXISTS_OFFSET + MAX_NULLIFIER_READ_REQUESTS_PER_CALL;
    static const uint32_t START_L1_TO_L2_MSG_EXISTS_WRITE_OFFSET =
        START_NULLIFIER_NON_EXISTS_OFFSET + MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL;

    static const uint32_t START_SSTORE_WRITE_OFFSET =
        START_L1_TO_L2_MSG_EXISTS_WRITE_OFFSET + MAX_L1_TO_L2_MSG_READ_REQUESTS_PER_CALL;
    static const uint32_t START_SLOAD_WRITE_OFFSET =
        START_SSTORE_WRITE_OFFSET + MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL;

    static const uint32_t START_EMIT_NOTE_HASH_WRITE_OFFSET = START_SLOAD_WRITE_OFFSET + MAX_PUBLIC_DATA_READS_PER_CALL;
    static const uint32_t START_EMIT_NULLIFIER_WRITE_OFFSET =
        START_EMIT_NOTE_HASH_WRITE_OFFSET + MAX_NEW_NOTE_HASHES_PER_CALL;
    static const uint32_t START_L2_TO_L1_MSG_WRITE_OFFSET =
        START_EMIT_NULLIFIER_WRITE_OFFSET + MAX_NEW_NULLIFIERS_PER_CALL;
    static const uint32_t START_EMIT_UNENCRYPTED_LOG_WRITE_OFFSET =
        START_L2_TO_L1_MSG_WRITE_OFFSET + MAX_NEW_L2_TO_L1_MSGS_PER_CALL;

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
