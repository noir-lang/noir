#pragma once
#include <stddef.h>

namespace rollup {

constexpr size_t DATA_TREE_DEPTH = 32;
constexpr size_t NULL_TREE_DEPTH = 256;
constexpr size_t ROOT_TREE_DEPTH = 28;
constexpr size_t DEFI_TREE_DEPTH = 30;

constexpr size_t MAX_TXS_BIT_LENGTH = 10;
constexpr size_t TX_FEE_BIT_LENGTH = 254 - MAX_TXS_BIT_LENGTH;

constexpr size_t NUM_ASSETS_BIT_LENGTH = 2;
constexpr size_t NUM_ASSETS = 1 << NUM_ASSETS_BIT_LENGTH;

constexpr size_t NUM_BRIDGE_CALLS_PER_BLOCK = 4;

namespace ProofIds {
enum { JOIN_SPLIT = 0, ACCOUNT = 1, DEFI_DEPOSIT = 2, DEFI_CLAIM = 3 };
};

} // namespace rollup