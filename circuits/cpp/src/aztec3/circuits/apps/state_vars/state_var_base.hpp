#pragma once

#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>

// Forward-declare from this namespace in particular:
namespace aztec3::circuits::apps {
template <typename Composer> class FunctionExecutionContext;
}

namespace aztec3::circuits::apps::state_vars {

using aztec3::circuits::apps::FunctionExecutionContext; // Don't #include it!

using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

/**
 * @brief StateVar is a base class from which contract state variables are derived. Its main purpose is deriving storage
 * slots, and generating constraints for those slot derivations, in a protocol-consistent way, regardless of the app
 * being written.
 */
template <typename Composer> class StateVar {
  public:
    typedef CircuitTypes<Composer> CT;
    typedef NativeTypes NT;
    typedef typename CT::fr fr;
    typedef typename CT::grumpkin_point grumpkin_point;

    // The execution context of the function currently being executed.
    FunctionExecutionContext<Composer>* exec_ctx;

    // Must match the name of a state which has been declared to the `Contract`.
    std::string state_var_name;

    // The `start slot` of the state variable is the slot which is assigned to this particular state by the `Contract`,
    // based on the ordering of declarations of the _names_ of states. For container types (mappings/arrays/structs),
    // the state variable might be able to access multiple storage slots. The start slot is the 'starting point' for
    // deriving such slots.
    fr start_slot;

    // The 'storage slot point' of the state variable. Having a _point_ for every storage slot allows for
    // partial-commitment functionality.
    // I.e. we can generate placeholder storage slots, which can be partially-committed to in one function, and then
    // completed in some future function, once the mapping keys or array indices at which we'd like to store the data
    // are known in future. Aztec Connect does something similar (the `asset_id` of the output value note isn't known
    // until later, so is partially committed-to).
    grumpkin_point storage_slot_point;

    // In order to calculate the correct storage_slot_point, we need to know how many containers
    // we're nested inside, so that we can find the correct Pedersen generator.
    size_t level_of_container_nesting = 0;

    // Optionally informs custom notes whether they should commit or partially-commit to this state.
    bool is_partial_slot = false;

    StateVar(){};

    // Instantiate a top-level state:
    StateVar(FunctionExecutionContext<Composer>* exec_ctx, std::string const& state_var_name);

    // Instantiate a state nested within a container:
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

  private:
    grumpkin_point compute_slot_point();
};

} // namespace aztec3::circuits::apps::state_vars

// Importing in this way (rather than explicit instantiation of a template class at the bottom of a .cpp file) preserves
// the following:
// - We retain implicit instantiation of templates.
// - We avoid circular dependencies with function_execution_context.hpp
#include "state_var_base.tpp"
