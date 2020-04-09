#pragma once
#include <stdlib/types/turbo.hpp>

namespace plonk {
namespace stdlib {
namespace pedersen {

using namespace plonk::stdlib::types::turbo;

field_ct compress_eight(std::array<field_ct, 8>& inputs, bool handle_edge_cases = false);

// TODO: use unique generators for each range
field_ct compress(std::vector<field_ct>& inputs, bool handle_edge_cases = false);

field_ct compress(const field_ct& left,
                  const field_ct& right,
                  const size_t hash_index = 0,
                  bool handle_edge_cases = false);

byte_array_ct compress(const byte_array_ct& inputs);

point compress_to_point(const field_ct& left, const field_ct& right, const size_t hash_index = 0);

} // namespace pedersen
} // namespace stdlib
} // namespace plonk