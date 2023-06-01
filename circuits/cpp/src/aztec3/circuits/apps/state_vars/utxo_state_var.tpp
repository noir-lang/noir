#pragma once

#include "../opcodes/opcodes.hpp"

#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/native_types.hpp"

namespace {
using aztec3::circuits::apps::opcodes::Opcodes;
}  // namespace

namespace aztec3::circuits::apps::state_vars {

template <typename Composer, typename Note>
Note UTXOStateVar<Composer, Note>::get(typename Note::NotePreimage const& advice)
{
    return Opcodes<Composer>::template UTXO_SLOAD<Note>(this, advice);
};

template <typename Composer, typename Note>
void UTXOStateVar<Composer, Note>::initialise(typename Note::NotePreimage new_note_preimage)
{
    Note new_note{ this, new_note_preimage };
    Opcodes<Composer>::template UTXO_INIT<Note>(this, new_note);
};

template <typename Composer, typename Note>
void UTXOStateVar<Composer, Note>::insert(typename Note::NotePreimage new_note_preimage)
{
    Opcodes<Composer>::template UTXO_SSTORE<Note>(this, new_note_preimage);
};

}  // namespace aztec3::circuits::apps::state_vars