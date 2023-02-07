#pragma once
#include <stddef.h>
#include <numeric/uint256/uint256.hpp>
#include "../../constants.hpp"

namespace join_split_example {
namespace proofs {
namespace notes {

constexpr size_t ASSET_ID_BIT_LENGTH = 30;
constexpr size_t NONCE_BIT_LENGTH = 32;
constexpr size_t DEFI_INTERACTION_NONCE_BIT_LENGTH = 30; // must match the bit length of asset ids
constexpr size_t NOTE_VALUE_BIT_LENGTH = 252;
constexpr uint256_t NOTE_VALUE_MAX = (uint256_t(1) << NOTE_VALUE_BIT_LENGTH) - 1;
constexpr size_t DEFI_DEPOSIT_VALUE_BIT_LENGTH = MAX_NO_WRAP_INTEGER_BIT_LENGTH - MAX_TXS_BIT_LENGTH;
constexpr size_t DEFI_BRIDGE_CALL_DATA_BIT_LENGTH = 250;

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
    ACCOUNT_ALIAS_HASH_NULLIFIER,
    DEFI_INTERACTION_NULLIFIER,
    ACCOUNT_PUBLIC_KEY_NULLIFIER,
};

constexpr uint32_t DEFI_BRIDGE_ADDRESS_ID_LEN = 32;
constexpr uint32_t DEFI_BRIDGE_INPUT_A_ASSET_ID_LEN = MAX_NUM_ASSETS_BIT_LENGTH;
constexpr uint32_t DEFI_BRIDGE_INPUT_B_ASSET_ID_LEN = MAX_NUM_ASSETS_BIT_LENGTH;
constexpr uint32_t DEFI_BRIDGE_OUTPUT_A_ASSET_ID_LEN = MAX_NUM_ASSETS_BIT_LENGTH;
constexpr uint32_t DEFI_BRIDGE_OUTPUT_B_ASSET_ID_LEN = MAX_NUM_ASSETS_BIT_LENGTH;
constexpr uint32_t DEFI_BRIDGE_BITCONFIG_LEN = 32;
constexpr uint32_t DEFI_BRIDGE_AUX_DATA = 64;

} // namespace notes
} // namespace proofs
} // namespace join_split_example