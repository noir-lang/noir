#pragma once

#include "state_var_base.hpp"

#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/circuit_types.hpp>

// Forward-declare from this namespace in particular:
namespace aztec3::circuits::apps {
template <typename Composer> class FunctionExecutionContext;
}

namespace aztec3::circuits::apps::state_vars {

using aztec3::circuits::apps::FunctionExecutionContext; // Don't #include it!

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;

/**
 * @brief - A derived StateVar which represents an unordered set of UTXOs which live in the UTXO tree.
 * Notice the `get` and `insert` methods for this StateVar, which interact with the UTXO tree opcodes.
 *
 * @tparam Note - A UTXO state variable always acts on notes and note preimages. We allow for custom Note types to be
 * designed. The Note type must implement the NoteInterface. TODO: maybe explicitly have this class act on the
 * NoteInterface type, rather than a template type.
 */
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