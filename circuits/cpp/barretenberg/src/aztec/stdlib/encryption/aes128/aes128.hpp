#pragma once

#include <array>
#include <vector>

#include "../../primitives/composers/composers_fwd.hpp"

#include "../../primitives/field/field.hpp"
#include "../../primitives/witness/witness.hpp"

namespace plonk {
namespace stdlib {
namespace aes128 {

std::vector<field_t<waffle::PLookupComposer>> encrypt_buffer_cbc(
    const std::vector<field_t<waffle::PLookupComposer>>& input,
    const field_t<waffle::PLookupComposer>& iv,
    const field_t<waffle::PLookupComposer>& key);

} // namespace aes128
} // namespace stdlib
} // namespace plonk
