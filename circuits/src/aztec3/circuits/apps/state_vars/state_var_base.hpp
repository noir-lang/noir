#pragma once

#include "../function_execution_context.hpp"

#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>

namespace aztec3::circuits::apps::state_vars {

using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename Composer> class StateVar {
  public:
    typedef CircuitTypes<Composer> CT;
    typedef NativeTypes NT;
    typedef typename CT::fr fr;
    typedef typename CT::grumpkin_point grumpkin_point;

    FunctionExecutionContext<Composer>* exec_ctx;

    std::string state_var_name;

    fr start_slot;
    grumpkin_point storage_slot_point;
    // In order to calculate the correct storage_slot_point, we need to know how many containers we're nested inside, so
    // that we can find the correct Pedersen generator.
    size_t level_of_container_nesting = 0;
    bool is_partial_slot = false;

    // native_storage_slot.x => value cache, to prevent creating constraints with each call.
    // V value;

    // StateVar(){};

    StateVar operator=(StateVar const& other)
    {
        this->exec_ctx = other.exec_ctx;
        this->state_var_name = other.state_var_name;
        this->start_slot = other.start_slot;
        this->storage_slot_point = other.storage_slot_point;
        this->level_of_container_nesting = other.level_of_container_nesting;
        this->is_partial_slot = other.is_partial_slot;
        return *this;
    }

    // StateVar(StateVar const& other)
    // {
    //     this->exec_ctx = other.exec_ctx;
    //     this->state_var_name = other.state_var_name;
    //     this->start_slot = other.start_slot;
    //     this->storage_slot_point = other.storage_slot_point;
    //     this->level_of_container_nesting = other.level_of_container_nesting;
    //     this->is_partial_slot = other.is_partial_slot;
    //     // return *this;
    // }

    StateVar(){};

    // Instantiate a top-level state:
    StateVar(FunctionExecutionContext<Composer>* exec_ctx, std::string const& state_var_name, fr const& start_slot)
        : exec_ctx(exec_ctx)
        , state_var_name(state_var_name)
        , start_slot(start_slot)
    {
        storage_slot_point = compute_slot_point();
    };

    // Instantiate a nested state:
    StateVar(
        FunctionExecutionContext<Composer>* exec_ctx,
        std::string const& state_var_name,
        grumpkin_point const& storage_slot_point, // the parent always calculates the storage_slot_point of its child.
        size_t level_of_container_nesting,        // the parent always calculates the level of nesting of its child.
        bool is_partial_slot = false)
        : exec_ctx(exec_ctx)
        , state_var_name(state_var_name)
        , storage_slot_point(storage_slot_point)
        , level_of_container_nesting(level_of_container_nesting)
        , is_partial_slot(is_partial_slot){};

    bool operator==(StateVar<Composer> const&) const = default;

  private:
    grumpkin_point compute_slot_point();
};

} // namespace aztec3::circuits::apps::state_vars

// Importing in this way (rather than explicit instantiation of a template class at the bottom of a .cpp file) preserves
// the following:
// - We retain implicit instantiation of templates, meaning we can pick and choose (with static_assert) which class
// methods support native,
//   circuit or both types.
#include "state_var_base.tpp"
