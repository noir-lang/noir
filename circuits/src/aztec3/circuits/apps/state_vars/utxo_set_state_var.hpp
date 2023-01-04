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

// template <typename Composer> class FunctionExecutionContext;

using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename Composer, typename Note> class UTXOSetStateVar : public StateVar<Composer> {
  public:
    typedef CircuitTypes<Composer> CT;
    typedef NativeTypes NT;
    typedef typename CT::fr fr;
    typedef typename CT::grumpkin_point grumpkin_point;

    typedef typename Note::NotePreimage NotePreimage;

    UTXOSetStateVar(){};

    // Instantiate a top-level var:
    UTXOSetStateVar(FunctionExecutionContext<Composer>* exec_ctx, std::string const& state_var_name)
        : StateVar<Composer>(exec_ctx, state_var_name){};

    // Instantiate a var nested within a container:
    UTXOSetStateVar(FunctionExecutionContext<Composer>* exec_ctx,
                    std::string const& state_var_name,
                    grumpkin_point const& storage_slot_point,
                    size_t level_of_container_nesting,
                    bool is_partial_slot)
        : StateVar<Composer>(
              exec_ctx, state_var_name, storage_slot_point, level_of_container_nesting, is_partial_slot){};

    // bool operator==(UTXOSetStateVar<Composer, V> const&) const = default;

    /**
     * @param advice - For NotePreimages, we allow 'advice' to be given, so that the correct DB entry is
     * chosen.
     * E.g. so that the `owner` can be specified.
     */
    std::vector<Note> get(size_t const& num_notes, NotePreimage const& advice);

    void insert(NotePreimage new_note_preimage);
};

} // namespace aztec3::circuits::apps::state_vars

#include "utxo_set_state_var.tpp"