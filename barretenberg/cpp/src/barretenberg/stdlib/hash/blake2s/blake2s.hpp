#pragma once
#include "barretenberg/stdlib/primitives/byte_array/byte_array.hpp"
#include "barretenberg/stdlib/primitives/circuit_builders/circuit_builders_fwd.hpp"

namespace proof_system::plonk {
namespace stdlib {

template <typename Builder> byte_array<Builder> blake2s(const byte_array<Builder>& input);

#define BLAKE2S(circuit_type) byte_array<circuit_type> blake2s(const byte_array<circuit_type>& input)

EXTERN_STDLIB_METHOD(BLAKE2S)

} // namespace stdlib
} // namespace proof_system::plonk
