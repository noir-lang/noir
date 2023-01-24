#pragma once
#include <stdlib/primitives/byte_array/byte_array.hpp>

namespace waffle {
class StandardComposer;
class TurboComposer;
class PlookupComposer;
} // namespace waffle

namespace plonk {
namespace stdlib {

template <typename Composer> byte_array<Composer> blake2s(const byte_array<Composer>& input);

extern template byte_array<waffle::StandardComposer> blake2s(const byte_array<waffle::StandardComposer>& input);
extern template byte_array<waffle::TurboComposer> blake2s(const byte_array<waffle::TurboComposer>& input);
extern template byte_array<waffle::PlookupComposer> blake2s(const byte_array<waffle::PlookupComposer>& input);

} // namespace stdlib
} // namespace plonk
