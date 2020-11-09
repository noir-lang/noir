#pragma once
#include <uchar.h>

namespace rollup {
namespace proofs {
namespace notes {

constexpr size_t NOTE_VALUE_BIT_LENGTH = 252;

constexpr size_t TX_NOTE_HASH_INDEX = 0;
constexpr size_t TX_NOTE_ACCOUNT_PRIVATE_KEY_INDEX = 4;
constexpr size_t TX_NOTE_NULLIFIER_INDEX = 5;
constexpr size_t ACCOUNT_HASH_INDEX = 10;
constexpr size_t ACCOUNT_NULLIFIER_INDEX = 12;
constexpr size_t ALIAS_HASH_INDEX = 14;
constexpr size_t ALIAS_NULLIFIER_INDEX = 16;

constexpr size_t GIBBERISH_NULLIFIER_PREFIX = 0;
constexpr size_t ALIAS_NULLIFIER_PREFIX = 3;

} // namespace notes
} // namespace proofs
} // namespace rollup