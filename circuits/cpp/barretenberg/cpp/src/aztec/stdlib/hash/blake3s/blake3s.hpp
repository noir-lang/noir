#pragma once
#include <stdlib/primitives/byte_array/byte_array.hpp>

namespace waffle {
class TurboComposer;
class UltraComposer;
} // namespace waffle

namespace plonk {
namespace stdlib {

template <typename Composer> byte_array<Composer> blake3s(const byte_array<Composer>& input);

extern template byte_array<waffle::StandardComposer> blake3s(const byte_array<waffle::StandardComposer>& input);
extern template byte_array<waffle::TurboComposer> blake3s(const byte_array<waffle::TurboComposer>& input);
extern template byte_array<waffle::UltraComposer> blake3s(const byte_array<waffle::UltraComposer>& input);

} // namespace stdlib
} // namespace plonk
