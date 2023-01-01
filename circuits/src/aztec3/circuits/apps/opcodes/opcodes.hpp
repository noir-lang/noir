#pragma once

#include "../state_vars/utxo_state_var.hpp"

#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>

namespace aztec3::circuits::apps::opcodes {

using aztec3::circuits::apps::state_vars::UTXOStateVar;

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename Composer> class Opcodes {
  public:
    typedef CircuitTypes<Composer> CT;
    typedef NativeTypes NT;

    /**
     * @brief Load a singleton UTXOSLoadDatum from the Private Client's DB.
     */
    template <typename Note>
    static Note UTXO_SLOAD(UTXOStateVar<Composer, Note>* utxo_state_var, typename Note::NotePreimage const& advice);
};

} // namespace aztec3::circuits::apps::opcodes

// - We don't implement method definitions in this file, to avoid a circular dependency with
// utxo_state_var.hpp.
#include "opcodes.tpp"
