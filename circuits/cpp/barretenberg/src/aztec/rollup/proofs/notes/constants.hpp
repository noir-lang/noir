#pragma once
#include <stddef.h>

namespace rollup {
namespace proofs {
namespace notes {

constexpr size_t NOTE_VALUE_BIT_LENGTH = 252;

enum GeneratorIndex {
    JOIN_SPLIT_NOTE_VALUE,
    JOIN_SPLIT_NOTE_SECRET,
    JOIN_SPLIT_NOTE_ASSET_ID,
    JOIN_SPLIT_NOTE_OWNER,
    JOIN_SPLIT_NOTE_NONCE = 5,
    JOIN_SPLIT_NULLIFIER_ACCOUNT_PRIVATE_KEY,
    JOIN_SPLIT_NULLIFIER_HASH_INPUTS,
    ACCOUNT_NOTE_HASH_INPUTS = 20,
    ACCOUNT_ALIAS_ID_NULLIFIER,
    ACCOUNT_GIBBERISH_NULLIFIER,
};

constexpr uint32_t ADDRESS_BIT_LENGTH = 160;
constexpr uint32_t NUM_OUTPUT_NOTES_LEN = 2;
constexpr uint32_t INPUT_ASSET_ID_LEN = 32;
constexpr uint32_t OUTPUT_A_ASSET_ID_LEN = 32;
constexpr uint32_t OUTPUT_B_ASSET_ID_LEN = 26;

} // namespace notes
} // namespace proofs
} // namespace rollup