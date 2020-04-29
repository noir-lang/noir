#pragma once
#include <array>
#include <stdlib/primitives/uint/uint.hpp>
#include "sha256_plookup.hpp"

namespace waffle {
class StandardComposer;
class TurboComposer;
} // namespace waffle

namespace plonk {
namespace stdlib {
template <typename Composer> class bit_array;

template <typename Composer> void prepare_constants(std::array<uint32<Composer>, 8>& input);

template <typename Composer>
std::array<uint32<Composer>, 8> sha256_block(const std::array<uint32<Composer>, 8>& h_init,
                                             const std::array<uint32<Composer>, 16>& input);

template <typename Composer> byte_array<Composer> sha256_block(const byte_array<Composer>& input);

template <typename Composer> bit_array<Composer> sha256(const bit_array<Composer>& input);

extern template byte_array<waffle::TurboComposer> sha256_block(const byte_array<waffle::TurboComposer>& input);
extern template bit_array<waffle::StandardComposer> sha256(const bit_array<waffle::StandardComposer>& input);
extern template bit_array<waffle::TurboComposer> sha256(const bit_array<waffle::TurboComposer>& input);

} // namespace stdlib
} // namespace plonk
