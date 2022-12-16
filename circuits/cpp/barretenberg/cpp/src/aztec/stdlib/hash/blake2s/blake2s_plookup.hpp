#pragma once
#include <array>
#include <plonk/composer/plookup_tables/plookup_tables.hpp>
#include <stdlib/primitives/uint/uint.hpp>

#include <numeric/bitop/sparse_form.hpp>

#include "../../primitives/field/field.hpp"
#include "../../primitives/composers/composers_fwd.hpp"
#include "../../primitives/packed_byte_array/packed_byte_array.hpp"

namespace plonk {
namespace stdlib {

namespace blake2s_plookup {

template <typename Composer> byte_array<Composer> blake2s(const byte_array<Composer>& input);

extern template byte_array<waffle::UltraComposer> blake2s(const byte_array<waffle::UltraComposer>& input);

} // namespace blake2s_plookup

} // namespace stdlib
} // namespace plonk
