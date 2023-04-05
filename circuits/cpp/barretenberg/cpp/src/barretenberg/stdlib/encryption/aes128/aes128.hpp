#pragma once

#include <array>
#include <vector>

#include "../../primitives/composers/composers_fwd.hpp"

#include "../../primitives/field/field.hpp"
#include "../../primitives/witness/witness.hpp"

namespace proof_system::plonk {
namespace stdlib {

namespace aes128 {

std::vector<field_t<plonk::UltraComposer>> encrypt_buffer_cbc(const std::vector<field_t<plonk::UltraComposer>>& input,
                                                              const field_t<plonk::UltraComposer>& iv,
                                                              const field_t<plonk::UltraComposer>& key);

} // namespace aes128
} // namespace stdlib
} // namespace proof_system::plonk
