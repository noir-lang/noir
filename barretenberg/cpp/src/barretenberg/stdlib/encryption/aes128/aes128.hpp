#pragma once

#include <array>
#include <vector>

#include "../../primitives/field/field.hpp"
#include "../../primitives/witness/witness.hpp"
#include "barretenberg/stdlib/primitives/circuit_builders/circuit_builders_fwd.hpp"

namespace proof_system::plonk {
namespace stdlib {

namespace aes128 {

template <typename Builder>
std::vector<stdlib::field_t<Builder>> encrypt_buffer_cbc(const std::vector<stdlib::field_t<Builder>>& input,
                                                         const stdlib::field_t<Builder>& iv,
                                                         const stdlib::field_t<Builder>& key);

} // namespace aes128
} // namespace stdlib
} // namespace proof_system::plonk
