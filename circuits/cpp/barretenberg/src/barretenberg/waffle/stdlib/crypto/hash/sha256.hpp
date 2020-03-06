#pragma once

#include "../../uint32/uint32.hpp"
#include <array>

namespace waffle {
class StandardComposer;
class MiMCComposer;
class TurboComposer;
} // namespace waffle

namespace plonk {
namespace stdlib {
template <typename Composer> class bitarray;

template <typename Composer> void prepare_constants(std::array<uint<Composer, uint32_t>, 8>& input);

template <typename Composer>
std::array<uint<Composer, uint32_t>, 8> sha256_block(const std::array<uint<Composer, uint32_t>, 8>& h_init,
                                                     const std::array<uint<Composer, uint32_t>, 16>& input);

template <typename Composer> byte_array<Composer> sha256_block(const byte_array<Composer>& input);

template <typename Composer> bitarray<Composer> sha256(const bitarray<Composer>& input);

extern template byte_array<waffle::TurboComposer> sha256_block(const byte_array<waffle::TurboComposer>& input);

extern template bitarray<waffle::StandardComposer> sha256(const bitarray<waffle::StandardComposer>& input);
extern template bitarray<waffle::MiMCComposer> sha256(const bitarray<waffle::MiMCComposer>& input);
extern template bitarray<waffle::TurboComposer> sha256(const bitarray<waffle::TurboComposer>& input);

} // namespace stdlib
} // namespace plonk
