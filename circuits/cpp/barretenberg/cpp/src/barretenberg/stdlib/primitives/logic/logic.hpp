#pragma once
#include "../composers/composers_fwd.hpp"
#include "../field/field.hpp"

namespace plonk {
namespace stdlib {

template <typename Composer> class logic {
  private:
    typedef field_t<Composer> field_pt;
    typedef witness_t<Composer> witness_pt;

  public:
    static field_pt create_logic_constraint(field_pt& a, field_pt& b, size_t num_bits, bool is_xor_gate);
};

EXTERN_STDLIB_TYPE(logic);

} // namespace stdlib
} // namespace plonk