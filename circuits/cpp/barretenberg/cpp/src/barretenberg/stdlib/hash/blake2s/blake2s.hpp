#pragma once
#include "barretenberg/stdlib/primitives/byte_array/byte_array.hpp"

namespace proof_system::plonk {
class TurboPlonkComposer;
class UltraPlonkComposer;
} // namespace proof_system::plonk

namespace proof_system::plonk {
namespace stdlib {

template <typename Composer> byte_array<Composer> blake2s(const byte_array<Composer>& input);

#define BLAKE2S(COMPOSER_TYPE) byte_array<COMPOSER_TYPE> blake2s(const byte_array<COMPOSER_TYPE>& input)

EXTERN_STDLIB_METHOD(BLAKE2S)

} // namespace stdlib
} // namespace proof_system::plonk
