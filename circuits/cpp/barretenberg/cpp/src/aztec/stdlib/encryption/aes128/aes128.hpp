#pragma once

#include <array>
#include <vector>

#include "../../primitives/composers/composers_fwd.hpp"

#include "../../primitives/field/field.hpp"
#include "../../primitives/witness/witness.hpp"

namespace plonk {
namespace stdlib {

namespace aes128 {

std::vector<field_t<waffle::UltraComposer>> encrypt_buffer_cbc(const std::vector<field_t<waffle::UltraComposer>>& input,
                                                               const field_t<waffle::UltraComposer>& iv,
                                                               const field_t<waffle::UltraComposer>& key);

} // namespace aes128
} // namespace stdlib
} // namespace plonk
