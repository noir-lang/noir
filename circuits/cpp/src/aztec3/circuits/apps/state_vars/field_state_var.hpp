#pragma once

#include "state_var_base.hpp"

#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/native_types.hpp"

namespace aztec3::circuits::apps::state_vars {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;

// TODO: we can probably generalise this to be a PrimitiveStateVar for any stdlib primitive.
template <typename Builder> class FieldStateVar : public StateVar<Builder> {
  public:
    using CT = CircuitTypes<Builder>;
    using NT = NativeTypes;
    using fr = typename CT::fr;
    using grumpkin_point = typename CT::grumpkin_point;

    fr value = 0;

    FieldStateVar& operator=(fr&& other)
    {
        value = other;
        return *this;
    }

    FieldStateVar() = default;

    // Instantiate a top-level var:
    FieldStateVar(FunctionExecutionContext<Builder>* exec_ctx, std::string const& state_var_name, fr const& start_slot)
        : StateVar<Builder>(exec_ctx, state_var_name, start_slot){};

    // Instantiate a var nested within a container:
    FieldStateVar(FunctionExecutionContext<Builder>* exec_ctx,
                  std::string const& state_var_name,
                  grumpkin_point const& storage_slot_point,
                  size_t level_of_container_nesting,
                  bool is_partial_slot)
        : StateVar<Builder>(
              exec_ctx, state_var_name, storage_slot_point, level_of_container_nesting, is_partial_slot){};

    bool operator==(FieldStateVar<Builder> const&) const = default;
};

template <typename Builder> inline std::ostream& operator<<(std::ostream& os, FieldStateVar<Builder> const& v)
{
    return os << v.value;
}

}  // namespace aztec3::circuits::apps::state_vars
