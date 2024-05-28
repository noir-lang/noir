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
