#pragma once
#include "barretenberg/stdlib/primitives/uint/uint.hpp"
#include "barretenberg/stdlib_circuit_builders/plookup_tables/plookup_tables.hpp"
#include <array>

#include "barretenberg/numeric/bitop/sparse_form.hpp"

#include "../../primitives/circuit_builders/circuit_builders_fwd.hpp"
#include "../../primitives/field/field.hpp"
#include "../../primitives/packed_byte_array/packed_byte_array.hpp"

namespace bb::stdlib::blake2s_plookup {

template <typename Builder> byte_array<Builder> blake2s(const byte_array<Builder>& input);

} // namespace bb::stdlib::blake2s_plookup
