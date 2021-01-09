#pragma once
#include <stddef.h>

namespace rollup {

constexpr size_t DATA_TREE_DEPTH = 32;
constexpr size_t NULL_TREE_DEPTH = 256;
constexpr size_t ROOT_TREE_DEPTH = 28;

constexpr size_t MAX_TXS_BIT_LENGTH = 10;
constexpr size_t TX_FEE_BIT_LENGTH = 254 - MAX_TXS_BIT_LENGTH;

} // namespace rollup