#pragma once

#include "../utxo_datum.hpp"

#include "../state_vars/state_var_base.hpp"
#include "../state_vars/utxo_state_var.hpp"

#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>

namespace aztec3::circuits::apps::opcodes {

using aztec3::circuits::apps::state_vars::UTXOStateVar;

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename Composer>
template <typename Note>
Note Opcodes<Composer>::UTXO_SLOAD(UTXOStateVar<Composer, Note>* utxo_state_var,
                                   typename Note::NotePreimage const& advice)
{
    auto& oracle = utxo_state_var->exec_ctx->oracle;

    typename CT::grumpkin_point& storage_slot_point = utxo_state_var->storage_slot_point;

    UTXOSLoadDatum<CT, typename Note::NotePreimage> utxo_datum =
        oracle.template get_utxo_sload_datum<typename Note::NotePreimage>(storage_slot_point, advice);

    Note new_note{ utxo_state_var, utxo_datum.preimage };

    oracle.get_contract_address().assert_equal(utxo_datum.contract_address, "UTXO_SLOAD: bad contract address");

    new_note.get_commitment().assert_equal(utxo_datum.commitment, "UTXO_SLOAD: bad commitment");

    // TODO within this function:
    // - Merkle Membership Check using utxo_datum.{sibling_path, leaf_index, old_private_data_tree_root}

    return new_note;
};

} // namespace aztec3::circuits::apps::opcodes
