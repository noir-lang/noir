#pragma once

// #include "../state_vars/utxo_state_var.hpp"

#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>

#include <variant>

namespace aztec3::circuits::apps::state_vars {
template <typename Composer> class StateVar;
template <typename Composer, typename Note> class UTXOStateVar;
template <typename Composer, typename Note> class UTXOSetStateVar;
} // namespace aztec3::circuits::apps::state_vars

namespace aztec3::circuits::apps::opcodes {

using aztec3::circuits::apps::state_vars::StateVar;        // Don't #include it!
using aztec3::circuits::apps::state_vars::UTXOSetStateVar; // Don't #include it!
using aztec3::circuits::apps::state_vars::UTXOStateVar;    // Don't #include it!

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename Composer> class Opcodes {
  public:
    typedef CircuitTypes<Composer> CT;
    typedef NativeTypes NT;

    template <typename Note>
    using VariantUTXOStateVar = std::variant<UTXOStateVar<Composer, Note>, UTXOSetStateVar<Composer, Note>>;

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
    template <typename Note>
    static std::vector<Note> UTXO_SLOAD(UTXOSetStateVar<Composer, Note>* utxo_set_state_var,
                                        size_t const& num_notes,
                                        typename Note::NotePreimage const& advice);

    /**
     * @brief Compute and push a new nullifier to the public inputs of this exec_ctx.
     */
    template <typename Note> static void UTXO_NULL(StateVar<Composer>* state_var, Note& note);

    /**
     * @brief Compute and push a new comitment to the public inputs of this exec_ctx.
     */
    template <typename Note>
    static void UTXO_SSTORE(StateVar<Composer>* state_var, typename Note::NotePreimage new_note_preimage);
};

} // namespace aztec3::circuits::apps::opcodes

// - We don't implement method definitions in this file, to avoid a circular dependency with
// utxo_state_var.hpp.
#include "opcodes.tpp"
