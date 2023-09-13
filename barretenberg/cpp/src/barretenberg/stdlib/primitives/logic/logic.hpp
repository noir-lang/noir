#pragma once
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/stdlib/primitives/circuit_builders/circuit_builders_fwd.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "barretenberg/stdlib/primitives/witness/witness.hpp"
#include <cstdint>
#include <functional>
#include <utility>

namespace proof_system::plonk::stdlib {

template <typename Composer> class logic {
  public:
    using field_pt = field_t<Composer>;
    using witness_pt = witness_t<Composer>;

  public:
    static field_pt create_logic_constraint(
        field_pt& a,
        field_pt& b,
        size_t num_bits,
        bool is_xor_gate,
        const std::function<std::pair<uint256_t, uint256_t>(uint256_t, uint256_t, size_t)>& get_chunk =
            [](uint256_t left, uint256_t right, size_t chunk_size) {
                uint256_t left_chunk = left & ((uint256_t(1) << chunk_size) - 1);
                uint256_t right_chunk = right & ((uint256_t(1) << chunk_size) - 1);
                return std::make_pair(left_chunk, right_chunk);
            });
};

EXTERN_STDLIB_TYPE(logic);

} // namespace proof_system::plonk::stdlib