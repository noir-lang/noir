#pragma once
#include <stddef.h>

namespace rollup {
namespace proofs {
namespace notes {

constexpr size_t NOTE_VALUE_BIT_LENGTH = 252;

constexpr size_t TX_NOTE_HASH_INDEX = 0;
constexpr size_t TX_NOTE_ACCOUNT_PRIVATE_KEY_INDEX = 6;
constexpr size_t TX_NOTE_NULLIFIER_INDEX = 7;

constexpr size_t ACCOUNT_NOTE_HASH_INDEX = 20;
constexpr size_t ACCOUNT_ALIAS_ID_HASH_INDEX = 21;
constexpr size_t ACCOUNT_GIBBERISH_HASH_INDEX = 22;

constexpr size_t GIBBERISH_NULLIFIER_PREFIX = 0;
constexpr size_t ALIAS_NULLIFIER_PREFIX = 3;

} // namespace notes
} // namespace proofs
} // namespace rollup