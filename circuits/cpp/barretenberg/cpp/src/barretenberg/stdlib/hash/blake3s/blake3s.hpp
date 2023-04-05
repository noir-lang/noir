#pragma once
#include "barretenberg/stdlib/primitives/byte_array/byte_array.hpp"

namespace proof_system::plonk {
class TurboComposer;
class UltraComposer;
} // namespace proof_system::plonk

namespace proof_system::plonk {
namespace stdlib {

template <typename Composer> byte_array<Composer> blake3s(const byte_array<Composer>& input);

extern template byte_array<plonk::StandardComposer> blake3s(const byte_array<plonk::StandardComposer>& input);
extern template byte_array<plonk::TurboComposer> blake3s(const byte_array<plonk::TurboComposer>& input);
extern template byte_array<plonk::UltraComposer> blake3s(const byte_array<plonk::UltraComposer>& input);

} // namespace stdlib
} // namespace proof_system::plonk
