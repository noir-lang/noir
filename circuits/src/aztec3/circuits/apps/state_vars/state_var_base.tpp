#pragma once
// #include <common/container.hpp>
// #include "oracle_wrapper.hpp"
// #include "private_state_note.hpp"
// #include "private_state_note_preimage.hpp"
// #include "private_state_operand.hpp"
#include "plonk/composer/turbo_composer.hpp"

#include <common/streams.hpp>
#include <common/map.hpp>
#include <crypto/pedersen/generator_data.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>

namespace aztec3::circuits::apps::state_vars {

using crypto::pedersen::generator_index_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

// Fr: start_slot
//
// mapping(fr => V):
//      level of nesting = 1
//      start_slot_point = start_slot * A
//      at(k1).slot = start_slot_point + k1 * B
//
// mapping(fr => mapping(fr => T)):
//      level_of_nesting = 2
//      start_slot_point becomes: prev_start_slot_point + k1 * B
//      at(k2).slot = new_start_slot_point + k2 * C

template <typename Composer> typename CircuitTypes<Composer>::grumpkin_point StateVar<Composer>::compute_slot_point()
{
    ASSERT(level_of_container_nesting == 0);
    return CT::commit({ start_slot }, { StorageSlotGeneratorIndex::BASE_SLOT });
}

// template class PrivateStateVar<waffle::TurboComposer>;

}; // namespace aztec3::circuits::apps::state_vars
