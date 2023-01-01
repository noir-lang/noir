#pragma once

#include "../opcodes/opcodes.hpp"

#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>

namespace aztec3::circuits::apps::state_vars {

using aztec3::circuits::apps::opcodes::Opcodes;

template <typename Composer, typename Note>
Note UTXOStateVar<Composer, Note>::get(typename Note::NotePreimage const& advice)
{
    return Opcodes<Composer>::template UTXO_SLOAD<Note>(this, advice);
};

// void insert(NotePreimage new_value);

// template <typename Composer> inline std::ostream& operator<<(std::ostream& os, UTXOStateVar<Composer> const& v)
// {
//     return os << v.value;
// }

} // namespace aztec3::circuits::apps::state_vars
