#pragma once

#include "avm_common.hpp"
#include "barretenberg/numeric/uint128/uint128.hpp"
#include "constants.hpp"
#include <cstdint>
#include <unordered_map>

inline const uint32_t SENDER_SELECTOR = 0;
inline const uint32_t ADDRESS_SELECTOR = 1;
inline const uint32_t PORTAL_SELECTOR = 2;

inline const uint32_t START_GLOBAL_VARIABLES = CALL_CONTEXT_LENGTH + HEADER_LENGTH;

inline const uint32_t CHAIN_ID_SELECTOR = START_GLOBAL_VARIABLES;
inline const uint32_t VERSION_SELECTOR = START_GLOBAL_VARIABLES + 1;
inline const uint32_t BLOCK_NUMBER_SELECTOR = START_GLOBAL_VARIABLES + 2;
inline const uint32_t TIMESTAMP_SELECTOR = START_GLOBAL_VARIABLES + 3;
inline const uint32_t COINBASE_SELECTOR = START_GLOBAL_VARIABLES + 4;

inline const uint32_t END_GLOBAL_VARIABLES = START_GLOBAL_VARIABLES + GLOBAL_VARIABLES_LENGTH;
inline const uint32_t START_SIDE_EFFECT_COUNTER = END_GLOBAL_VARIABLES;

inline const uint32_t FEE_PER_DA_GAS_SELECTOR = START_SIDE_EFFECT_COUNTER + 1;
inline const uint32_t FEE_PER_L2_GAS_SELECTOR = FEE_PER_DA_GAS_SELECTOR + 1;
inline const uint32_t TRANSACTION_FEE_SELECTOR = FEE_PER_L2_GAS_SELECTOR + 1;

const std::array<uint32_t, 11> KERNEL_INPUTS_SELECTORS = {
    SENDER_SELECTOR,         ADDRESS_SELECTOR,         PORTAL_SELECTOR,   FEE_PER_DA_GAS_SELECTOR,
    FEE_PER_L2_GAS_SELECTOR, TRANSACTION_FEE_SELECTOR, CHAIN_ID_SELECTOR, VERSION_SELECTOR,
    BLOCK_NUMBER_SELECTOR,   COINBASE_SELECTOR,        TIMESTAMP_SELECTOR
};

namespace bb::avm_trace {

class AvmKernelTraceBuilder {
  public:
    struct KernelTraceEntry {
        uint32_t kernel_selector = 0;
        bool q_kernel_lookup = false;
    };

    std::array<FF, KERNEL_INPUTS_LENGTH> kernel_inputs{};

    // Counts the number of accesses into each SELECTOR for the environment selector lookups;
    std::unordered_map<uint32_t, uint32_t> kernel_selector_counter;

    // Constructor receives copy of kernel_inputs from the main trace builder
    AvmKernelTraceBuilder(std::array<FF, KERNEL_INPUTS_LENGTH> kernel_inputs);

    void reset();

    // Context
    FF op_sender();
    FF op_address();
    FF op_portal();

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

  private:
    std::vector<KernelTraceEntry> kernel_trace;

    FF perform_kernel_lookup(uint32_t selector);
};
} // namespace bb::avm_trace