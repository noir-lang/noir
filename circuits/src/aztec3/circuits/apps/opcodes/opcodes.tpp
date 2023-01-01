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

    // Retrieve UTXO witness datum from the DB:
    UTXOSLoadDatum<CT, typename Note::NotePreimage> utxo_datum =
        oracle.template get_utxo_sload_datum<typename Note::NotePreimage>(storage_slot_point, advice);

    Note new_note{ utxo_state_var, utxo_datum.preimage };

    new_note.get_commitment().assert_equal(utxo_datum.commitment, "UTXO_SLOAD: bad commitment");

    oracle.get_contract_address().assert_equal(utxo_datum.contract_address, "UTXO_SLOAD: bad contract address");

    // TODO within this function:
    // - constrain any of the `advice` fields which aren't std::nullopt (call upon a method in the note itself to
    // `constrain_from_advice()`).
    // - Merkle Membership Check using the contract_address, utxo_datum.{sibling_path, leaf_index,
    // old_private_data_tree_root}

    return new_note;
};

template <typename Composer>
template <typename Note>
void Opcodes<Composer>::UTXO_NULL(UTXOStateVar<Composer, Note>* utxo_state_var, Note& note)
{
    auto [nullifier, nullifier_preimage] = note.compute_nullifier();

    auto exec_ctx = utxo_state_var->exec_ctx;

    (void)exec_ctx; // TODO: finish function.
    (void)nullifier;
    (void)nullifier_preimage;

    // TODO:
    // - Push the nullifier data to the exec_ctx
};

template <typename Composer>
template <typename Note>
void Opcodes<Composer>::UTXO_SSTORE(UTXOStateVar<Composer, Note>* utxo_state_var,
                                    typename Note::NotePreimage new_note_preimage)
{

    (void)utxo_state_var;
    (void)new_note_preimage;

    // TODO: a salt (randomness for hiding) might not be needed for some custom Note types. Leave this to the
    // `compute_commitment()` function of the Note instead (which will be called at FINALISATION of the notes, once
    // enough nonces (nullifiers) are available).

    // auto& oracle = utxo_state_var->exec_ctx->oracle;

    // CT::fr salt = oracle.get_random_element();
    // new_note_preimage.salt = salt;

    // TODO within this function:
    // - Push the commitment data to the exec_ctx, and maybe to the public inputs of the exec_ctx (although we might
    // need to complete the commitments with a nonce using a UTXO_FINALISE() opcode!)

    // Note new_note{ utxo_state_var, new_note_preimage };
    // TODO: the code rightly complains when we try to commit, because we haven't chosen a nonce yet! Hence why we might
    // need to defer committing until a FINALISE opcode at the end.
    // auto [new_note_commitment, _] = new_note.compute_commitment();

    // auto exec_ctx = utxo_state_var->exec_ctx;

    // (void)exec_ctx; // TODO: finish function.
    // (void)new_note_commitment;
};

} // namespace aztec3::circuits::apps::opcodes
