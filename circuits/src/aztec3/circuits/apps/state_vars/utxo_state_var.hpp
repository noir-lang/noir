#pragma once

#include "state_var_base.hpp"

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
 * @brief - A derived StateVar which represents a singleton UTXO type. I.e. a state which can only ever have at-most ONE
 * non-nullified UTXO in the tree at any time.  Notice the `get` and `insert` methods for this StateVar, which interact
 * with the UTXO tree opcodes.
 *
 * @tparam Note - A UTXO state variable always acts on notes and note preimages. We allow for custom Note types to be
 * designed. The Note type must implement the NoteInterface. TODO: maybe explicitly have this class act on the
 * NoteInterface type, rather than a template type.
 */
template <typename Composer, typename Note> class UTXOStateVar : public StateVar<Composer> {
  public:
    typedef CircuitTypes<Composer> CT;
    typedef NativeTypes NT;
    typedef typename CT::fr fr;
    typedef typename CT::grumpkin_point grumpkin_point;

    typedef typename Note::NotePreimage NotePreimage;

    UTXOStateVar(){};

    // Instantiate a top-level var:
    UTXOStateVar(FunctionExecutionContext<Composer>* exec_ctx, std::string const& state_var_name)
        : StateVar<Composer>(exec_ctx, state_var_name){};

    // Instantiate a var nested within a container:
    UTXOStateVar(FunctionExecutionContext<Composer>* exec_ctx,
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
    Note get(NotePreimage const& advice);

    /**
     * @brief For singleton UTXOs, there's a distinction between initialising and modifying in future. See here:
     * https://discourse.aztec.network/t/utxo-syntax-2-initialising-singleton-utxos/47. So we include this method for
     * singleton UTXO types.
     */
    void initialise(NotePreimage new_note_preimage);

    void insert(NotePreimage new_note_preimage);
};

} // namespace aztec3::circuits::apps::state_vars

#include "utxo_state_var.tpp"