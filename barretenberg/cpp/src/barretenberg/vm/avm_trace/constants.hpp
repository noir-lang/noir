#pragma once
#include "aztec_constants.hpp"
#include <cstdint>

// NOTE(MD): for now we will only include the public inputs that are included in call_context
// With more being added in subsequent prs
// KERNEL_INPUTS_LENGTH = CALL_CONTEXT_LENGTH +
inline const std::size_t KERNEL_INPUTS_LENGTH = PUBLIC_CONTEXT_INPUTS_LENGTH;

inline const std::size_t KERNEL_OUTPUTS_LENGTH =
    MAX_NOTE_HASH_READ_REQUESTS_PER_CALL + MAX_NEW_NOTE_HASHES_PER_CALL + MAX_NULLIFIER_READ_REQUESTS_PER_CALL +
    MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL + MAX_NEW_NULLIFIERS_PER_CALL + MAX_NEW_L2_TO_L1_MSGS_PER_CALL +
    MAX_UNENCRYPTED_LOGS_PER_CALL + MAX_PUBLIC_DATA_READS_PER_CALL + MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL;

// START INDEXES in the PUBLIC_CIRCUIT_PUBLIC_INPUTS
// These line up with indexes found in
// https://github.com/AztecProtocol/aztec-packages/blob/master/yarn-project/circuits.js/src/structs/public_circuit_public_inputs.ts
inline const uint32_t PCPI_GLOBALS_START = PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH - 7 - GLOBAL_VARIABLES_LENGTH;

inline const uint32_t CHAIN_ID_OFFSET = PCPI_GLOBALS_START;
inline const uint32_t VERSION_OFFSET = PCPI_GLOBALS_START + 1;
inline const uint32_t BLOCK_NUMBER_OFFSET = PCPI_GLOBALS_START + 2;
inline const uint32_t TIMESTAMP_OFFSET = PCPI_GLOBALS_START + 3;
inline const uint32_t COINBASE_OFFSET = PCPI_GLOBALS_START + 4;

inline const uint32_t FEE_PER_DA_GAS_OFFSET = PCPI_GLOBALS_START + 6;
inline const uint32_t FEE_PER_L2_GAS_OFFSET = PCPI_GLOBALS_START + 7;

inline const uint32_t TRANSACTION_FEE_OFFSET = PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH - 1;
inline const uint32_t DA_GAS_LEFT_PCPI_OFFSET = PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH - 3 - GAS_LENGTH;
inline const uint32_t L2_GAS_LEFT_PCPI_OFFSET = PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH - 2 - GAS_LENGTH;

// END INDEXES in the PUBLIC_CIRCUIT_PUBLIC_INPUTS

// L2 and Da gas left are the 3rd last and 2nd last items in the context kernel inputs respectively
inline const std::size_t DA_GAS_LEFT_CONTEXT_INPUTS_OFFSET = KERNEL_INPUTS_LENGTH - 3;
inline const std::size_t L2_GAS_LEFT_CONTEXT_INPUTS_OFFSET = KERNEL_INPUTS_LENGTH - 2;