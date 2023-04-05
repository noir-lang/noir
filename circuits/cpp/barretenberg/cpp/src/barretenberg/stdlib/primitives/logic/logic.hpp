#pragma once
#include "barretenberg/stdlib/primitives/composers/composers_fwd.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "barretenberg/stdlib/primitives/witness/witness.hpp"

namespace proof_system::plonk::stdlib {

template <typename Composer> class logic {
  private:
    using field_pt = field_t<Composer>;
    using witness_pt = witness_t<Composer>;

  public:
    static field_pt create_logic_constraint(field_pt& a, field_pt& b, size_t num_bits, bool is_xor_gate);
};

EXTERN_STDLIB_TYPE(logic);

} // namespace proof_system::plonk::stdlib