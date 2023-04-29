#pragma once

#include "state_var_base.hpp"
#include "../function_execution_context.hpp"

#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/circuit_types.hpp>

namespace aztec3::circuits::apps::state_vars {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;

// TODO: we can probably generalise this to be a PrimitiveStateVar for any stdlib primitive.
template <typename Composer> class FieldStateVar : public StateVar<Composer> {
  public:
    typedef CircuitTypes<Composer> CT;
    typedef NativeTypes NT;
    typedef typename CT::fr fr;
    typedef typename CT::grumpkin_point grumpkin_point;

    fr value = 0;

    FieldStateVar& operator=(fr&& other)
    {
        value = other;
        return *this;
    }

    FieldStateVar() {}

    // Instantiate a top-level var:
    FieldStateVar(FunctionExecutionContext<Composer>* exec_ctx, std::string const& state_var_name, fr const& start_slot)
        : StateVar<Composer>(exec_ctx, state_var_name, start_slot){};

    // Instantiate a var nested within a container:
    FieldStateVar(FunctionExecutionContext<Composer>* exec_ctx,
                  std::string const& state_var_name,
                  grumpkin_point const& storage_slot_point,
                  size_t level_of_container_nesting,
                  bool is_partial_slot)
        : StateVar<Composer>(
              exec_ctx, state_var_name, storage_slot_point, level_of_container_nesting, is_partial_slot){};

    bool operator==(FieldStateVar<Composer> const&) const = default;
};

template <typename Composer> inline std::ostream& operator<<(std::ostream& os, FieldStateVar<Composer> const& v)
{
    return os << v.value;
}

} // namespace aztec3::circuits::apps::state_vars
