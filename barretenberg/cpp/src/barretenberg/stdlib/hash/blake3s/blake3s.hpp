#pragma once
#include "barretenberg/stdlib/primitives/byte_array/byte_array.hpp"
#include "barretenberg/stdlib/primitives/circuit_builders/circuit_builders_fwd.hpp"

namespace proof_system::plonk {
namespace stdlib {

template <typename Builder> byte_array<Builder> blake3s(const byte_array<Builder>& input);

} // namespace stdlib
} // namespace proof_system::plonk
