#pragma once
#include "../uint/uint.hpp"

namespace plonk {
namespace stdlib {

template <typename ComposerContext> using uint64 = uint<ComposerContext, uint64_t>;

template <typename ComposerContext> using uint32 = uint<ComposerContext, uint32_t>;

template <typename ComposerContext> using uint16 = uint<ComposerContext, uint16_t>;

template <typename ComposerContext> using uint8 = uint<ComposerContext, uint8_t>;

} // namespace stdlib
} // namespace plonk
