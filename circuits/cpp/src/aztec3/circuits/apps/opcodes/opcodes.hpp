#pragma once

#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::apps::state_vars {
template <typename Composer> class StateVar;
template <typename Composer, typename Note> class UTXOStateVar;
template <typename Composer, typename Note> class UTXOSetStateVar;
}  // namespace aztec3::circuits::apps::state_vars

namespace aztec3::circuits::apps::opcodes {

using aztec3::circuits::apps::state_vars::StateVar;         // Don't #include it!
using aztec3::circuits::apps::state_vars::UTXOSetStateVar;  // Don't #include it!
using aztec3::circuits::apps::state_vars::UTXOStateVar;     // Don't #include it!

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using plonk::stdlib::witness_t;

/**
 * @brief - These static methods are a suggestion for what ACIR++ 'Opcodes' might do. They can get
 * data from an oracle. They can apply constraints to that data. And they are the only class allowed to push data to the
 * execution context.
 * Separating out this functionality into a separate `Opcodes` class, like this, was trickier than
 * just writing this stuff directly in the `Note` or `FunctionExecutionContext` classes, but hopefully the separation is
 * sensible.
 *
 * TODO: Any oracle access or exec_ctx access should go through this class?
 */
template <typename Composer> class Opcodes {
  public:
    using CT = CircuitTypes<Composer>;
    using address = typename CT::address;

    /**
     * @brief
     * - Load a singleton UTXOSLoadDatum from the Private Client's DB
     * - Generate constraints to prove its existence in the tree
     * - Validate the data
     */
    template <typename Note>
    static Note UTXO_SLOAD(UTXOStateVar<Composer, Note>* utxo_state_var, typename Note::NotePreimage const& advice);

    /**
     * @brief
     * - Load a subset of `UTXOSLoadDatum`s (which belong to a particular UTXOSetStateVar), from the Private Client's DB
     * - Generate constraints to prove each datum's existence in the tree
     * - Validate the data
     */
    template <typename Note> static std::vector<Note> UTXO_SLOAD(UTXOSetStateVar<Composer, Note>* utxo_set_state_var,
                                                                 size_t const& num_notes,
                                                                 typename Note::NotePreimage const& advice);

    /**
     * @brief Compute and push a new nullifier to the public inputs of this exec_ctx.
     */
    template <typename Note> static void UTXO_NULL(StateVar<Composer>* state_var, Note& note_to_nullify);

    /**
     * @brief Compute and push a new commitment to the public inputs of this exec_ctx, BUT ALSO compute and produce an
     * initialisation nullifier, to prevent this note from being initialised again in the future.
     */
    template <typename Note> static void UTXO_INIT(StateVar<Composer>* state_var, Note& note_to_initialise);

    /**
     * @brief Compute and push a new commitment to the public inputs of this exec_ctx.
     */
    template <typename Note>
    static void UTXO_SSTORE(StateVar<Composer>* state_var, typename Note::NotePreimage new_note_preimage);
};

}  // namespace aztec3::circuits::apps::opcodes

// Importing in this way (rather than explicit instantiation of a template class at the bottom of a .cpp file) preserves
// the following:
// - We retain implicit instantiation of templates.
// - We don't implement method definitions in this file, to avoid a circular dependency with the state_var files (which
//   are forward-declared in this file).
#include "opcodes.tpp"
