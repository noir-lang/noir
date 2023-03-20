#pragma once

#include "../opcodes/opcodes.hpp"

#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/circuit_types.hpp>

namespace {
using aztec3::circuits::apps::opcodes::Opcodes;
} // namespace

namespace aztec3::circuits::apps::state_vars {

template <typename Composer, typename Note>
std::vector<Note> UTXOSetStateVar<Composer, Note>::get(size_t const& num_notes,
                                                       typename Note::NotePreimage const& advice)
{
    return Opcodes<Composer>::template UTXO_SLOAD<Note>(this, num_notes, advice);
};

template <typename Composer, typename Note>
void UTXOSetStateVar<Composer, Note>::insert(typename Note::NotePreimage new_note_preimage)
{
    return Opcodes<Composer>::template UTXO_SSTORE<Note>(this, new_note_preimage);
};

} // namespace aztec3::circuits::apps::state_vars