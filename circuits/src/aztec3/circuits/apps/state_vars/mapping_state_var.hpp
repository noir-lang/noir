#pragma once

#include "state_var_base.hpp"
// #include "../function_execution_context.hpp"

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
 * Note: we restrict mapping keys to always be something which is convertible into a `field` type. This is to allow
 * storage_slot_points to be computed more easily. You'll notice, therefore, that there's no Key template type.
 */
template <typename Composer, typename V> class MappingStateVar : public StateVar<Composer> {
  public:
    typedef CircuitTypes<Composer> CT;
    typedef NativeTypes NT;
    typedef typename CT::fr fr;
    typedef typename CT::grumpkin_point grumpkin_point;

    // native_storage_slot.x => value cache, to prevent creating constraints with each `at()` call.
    std::map<NT::fr, V> value_cache;

    MappingStateVar(){};

    // Instantiate a top-level mapping:
    MappingStateVar(FunctionExecutionContext<Composer>* exec_ctx, std::string const& state_var_name)
        : StateVar<Composer>(exec_ctx, state_var_name){};

    // Instantiate a nested mapping:
    MappingStateVar(FunctionExecutionContext<Composer>* exec_ctx,
                    std::string const& state_var_name,
                    grumpkin_point const& storage_slot_point,
                    size_t level_of_container_nesting,
                    bool is_partial_slot)
        : StateVar<Composer>(
              exec_ctx, state_var_name, storage_slot_point, level_of_container_nesting, is_partial_slot){};

    bool operator==(MappingStateVar<Composer, V> const&) const = default;

    V& operator[](std::optional<fr> const& key) { return this->at(key); };
    V& operator[](std::string const& question_mark)
    {
        ASSERT(question_mark != "?");
        return this->at(std::nullopt);
    };

    V& at(std::optional<fr> const& key);

    static std::tuple<NT::grumpkin_point, bool> compute_slot_point_at_mapping_key(NT::fr const& start_slot,
                                                                                  size_t level_of_container_nesting,
                                                                                  std::optional<NT::fr> const& key);

    std::tuple<grumpkin_point, bool> compute_slot_point_at_mapping_key(std::optional<fr> const& key);
};

} // namespace aztec3::circuits::apps::state_vars

// Importing in this way (rather than explicit instantiation of a template class at the bottom of a .cpp file) preserves
// the following:
// - We retain implicit instantiation of templates.
// - We don't implement method definitions in this file, to avoid a circular dependency with
// function_execution_context.hpp.
#include "mapping_state_var.tpp"
