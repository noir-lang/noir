#pragma once
#include <stdlib/types/turbo.hpp>

namespace plonk {
namespace stdlib {
namespace pedersen {

using namespace plonk::stdlib::types::turbo;

field_ct compress_eight(const std::array<field_ct, 8>& inputs);

// TODO: use unique generators for each range
field_ct compress(const std::vector<field_ct>& inputs);

field_ct compress(const field_ct& left, const field_ct& right, const size_t hash_index = 0);

byte_array_ct compress(const byte_array_ct& inputs);

point compress_to_point(const field_ct& left, const field_ct& right, const size_t hash_index = 0);

} // namespace pedersen
} // namespace stdlib
} // namespace plonk