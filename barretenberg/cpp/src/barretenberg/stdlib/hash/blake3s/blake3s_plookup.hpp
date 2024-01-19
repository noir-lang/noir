#pragma once
#include "barretenberg/proof_system/plookup_tables/plookup_tables.hpp"
#include "barretenberg/stdlib/primitives/uint/uint.hpp"
#include <array>

#include "barretenberg/numeric/bitop/sparse_form.hpp"

#include "../../primitives/circuit_builders/circuit_builders_fwd.hpp"
#include "../../primitives/field/field.hpp"
#include "../../primitives/packed_byte_array/packed_byte_array.hpp"

namespace bb::plonk {
namespace stdlib {

namespace blake3s_plookup {

template <typename Builder> byte_array<Builder> blake3s(const byte_array<Builder>& input);

} // namespace blake3s_plookup

} // namespace stdlib
} // namespace bb::plonk
