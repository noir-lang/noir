#pragma once
#include <stddef.h>
#include <stdint.h>
#include <numeric/uint256/uint256.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>

namespace join_split_example {

constexpr size_t DATA_TREE_DEPTH = 32;

constexpr size_t MAX_NO_WRAP_INTEGER_BIT_LENGTH = grumpkin::MAX_NO_WRAP_INTEGER_BIT_LENGTH;
constexpr size_t MAX_TXS_BIT_LENGTH = 10;
constexpr size_t TX_FEE_BIT_LENGTH = MAX_NO_WRAP_INTEGER_BIT_LENGTH - MAX_TXS_BIT_LENGTH;

constexpr size_t NUM_ASSETS_BIT_LENGTH = 4;
constexpr size_t NUM_ASSETS = 1 << NUM_ASSETS_BIT_LENGTH;
constexpr size_t ASSET_ID_BIT_LENGTH = 30;
constexpr size_t MAX_NUM_ASSETS_BIT_LENGTH = 30;
constexpr size_t MAX_NUM_ASSETS = 1 << MAX_NUM_ASSETS_BIT_LENGTH;
constexpr size_t ALIAS_HASH_BIT_LENGTH = 224;

namespace ProofIds {
enum { PADDING = 0, DEPOSIT = 1, WITHDRAW = 2, SEND = 3, ACCOUNT = 4, DEFI_DEPOSIT = 5, DEFI_CLAIM = 6 };
};

} // namespace join_split_example