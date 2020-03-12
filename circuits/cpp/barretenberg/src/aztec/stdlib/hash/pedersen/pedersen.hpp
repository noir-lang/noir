#pragma once
#include <stdlib/types/turbo.hpp>

namespace plonk {
namespace stdlib {
namespace pedersen {

using namespace plonk::stdlib::types::turbo;

field_ct compress_eight(const std::array<field_ct, 8>& inputs);

field_ct compress(const field_ct& left, const field_ct& right, const size_t hash_index = 0);

point compress_to_point(const field_ct& left, const field_ct& right, const size_t hash_index = 0);

} // namespace pedersen
} // namespace stdlib
} // namespace plonk