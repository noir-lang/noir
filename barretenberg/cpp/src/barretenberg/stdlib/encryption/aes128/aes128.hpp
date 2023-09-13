#pragma once

#include <array>
#include <vector>

#include "../../primitives/field/field.hpp"
#include "../../primitives/witness/witness.hpp"
#include "barretenberg/stdlib/primitives/circuit_builders/circuit_builders_fwd.hpp"

namespace proof_system::plonk {
namespace stdlib {

namespace aes128 {

template <typename Composer>
std::vector<stdlib::field_t<Composer>> encrypt_buffer_cbc(const std::vector<stdlib::field_t<Composer>>& input,
                                                          const stdlib::field_t<Composer>& iv,
                                                          const stdlib::field_t<Composer>& key);

} // namespace aes128
} // namespace stdlib
} // namespace proof_system::plonk
