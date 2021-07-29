#pragma once
#include <stddef.h>
#include <stdint.h>

namespace rollup {

constexpr size_t DATA_TREE_DEPTH = 32;
constexpr size_t NULL_TREE_DEPTH = 256;
constexpr size_t ROOT_TREE_DEPTH = 28;
constexpr size_t DEFI_TREE_DEPTH = 30;
// the maximal bit length of integer we can use and range check (cause range check works only for even nums) before
// modulus wrap
constexpr size_t MAX_NO_WRAP_INTEGER_BIT_LENGTH = 252;
constexpr size_t MAX_TXS_BIT_LENGTH = 10;
constexpr size_t TX_FEE_BIT_LENGTH = MAX_NO_WRAP_INTEGER_BIT_LENGTH - MAX_TXS_BIT_LENGTH;

constexpr size_t NUM_ASSETS_BIT_LENGTH = 2;
constexpr size_t NUM_ASSETS = 1 << NUM_ASSETS_BIT_LENGTH;
constexpr size_t MAX_NUM_ASSETS_BIT_LENGTH = 30;
constexpr size_t MAX_NUM_ASSETS = 1 << MAX_NUM_ASSETS_BIT_LENGTH;

constexpr uint32_t DEFI_BRIDGE_ID_BIT_LENGTH = 252;
constexpr uint32_t NUM_BRIDGE_CALLS_PER_BLOCK = 4;

namespace ProofIds {
enum { JOIN_SPLIT = 0, ACCOUNT = 1, DEFI_DEPOSIT = 2, DEFI_CLAIM = 3 };
};

} // namespace rollup