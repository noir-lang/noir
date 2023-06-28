#pragma once

#include "../opcodes/opcodes.hpp"

#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/native_types.hpp"

namespace {
using aztec3::circuits::apps::opcodes::Opcodes;
}  // namespace

namespace aztec3::circuits::apps::state_vars {

template <typename Builder, typename Note>
Note UTXOStateVar<Builder, Note>::get(typename Note::NotePreimage const& advice)
{
    return Opcodes<Builder>::template UTXO_SLOAD<Note>(this, advice);
};

template <typename Builder, typename Note>
void UTXOStateVar<Builder, Note>::initialise(typename Note::NotePreimage new_note_preimage)
{
    Note new_note{ this, new_note_preimage };
    Opcodes<Builder>::template UTXO_INIT<Note>(this, new_note);
};

template <typename Builder, typename Note>
void UTXOStateVar<Builder, Note>::insert(typename Note::NotePreimage new_note_preimage)
{
    Opcodes<Builder>::template UTXO_SSTORE<Note>(this, new_note_preimage);
};

}  // namespace aztec3::circuits::apps::state_vars