#pragma once
#include <stddef.h>
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace notes {

constexpr size_t NOTE_VALUE_BIT_LENGTH = 252;
constexpr size_t DEFI_DEPOSIT_VALUE_BIT_LENGTH = MAX_NO_WRAP_INTEGER_BIT_LENGTH - MAX_TXS_BIT_LENGTH;

// Start from 1 to avoid the default generators.
enum GeneratorIndex {
    VALUE_NOTE_PARTIAL_COMMITMENT = 1,
    VALUE_NOTE_COMMITMENT,
    CLAIM_NOTE_PARTIAL_COMMITMENT,
    CLAIM_NOTE_COMMITMENT,
    ACCOUNT_NOTE_COMMITMENT,
    DEFI_INTERACTION_NOTE_COMMITMENT,

    JOIN_SPLIT_NULLIFIER,
    JOIN_SPLIT_NULLIFIER_ACCOUNT_PRIVATE_KEY,
    CLAIM_NOTE_NULLIFIER,
    ACCOUNT_ALIAS_ID_NULLIFIER,
};

constexpr uint32_t DEFI_BRIDGE_ADDRESS_BIT_LENGTH = 160;
constexpr uint32_t DEFI_BRIDGE_NUM_OUTPUT_NOTES_LEN = 2;
constexpr uint32_t DEFI_BRIDGE_INPUT_ASSET_ID_LEN = MAX_NUM_ASSETS_BIT_LENGTH;
constexpr uint32_t DEFI_BRIDGE_OUTPUT_A_ASSET_ID_LEN = MAX_NUM_ASSETS_BIT_LENGTH;
constexpr uint32_t DEFI_BRIDGE_OUTPUT_B_ASSET_ID_LEN = MAX_NUM_ASSETS_BIT_LENGTH;

} // namespace notes
} // namespace proofs
} // namespace rollup