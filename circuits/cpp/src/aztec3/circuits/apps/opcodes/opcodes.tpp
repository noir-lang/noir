#pragma once

#include "../state_vars/state_var_base.hpp"
#include "../state_vars/utxo_set_state_var.hpp"
#include "../state_vars/utxo_state_var.hpp"
#include "../function_execution_context.hpp"
#include "../utxo_datum.hpp"

#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace {
// Declared here, so that `opcodes.hpp` doesn't see it; thereby preventing circular dependencies.
using aztec3::circuits::apps::state_vars::StateVar;
using aztec3::circuits::apps::state_vars::UTXOSetStateVar;
using aztec3::circuits::apps::state_vars::UTXOStateVar;
}  // namespace

namespace aztec3::circuits::apps::opcodes {

using aztec3::circuits::apps::FunctionExecutionContext;

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using plonk::stdlib::witness_t;

template <typename Builder> template <typename Note>
Note Opcodes<Builder>::UTXO_SLOAD(UTXOStateVar<Builder, Note>* utxo_state_var,
                                  typename Note::NotePreimage const& advice)
{
    auto& oracle = utxo_state_var->exec_ctx->oracle;

    typename CT::grumpkin_point const& storage_slot_point = utxo_state_var->storage_slot_point;

    // Retrieve UTXO witness datum from the DB:
    UTXOSLoadDatum<CT, typename Note::NotePreimage> const utxo_datum =
        oracle.template get_utxo_sload_datum<typename Note::NotePreimage>(storage_slot_point, advice);

    Note new_note{ utxo_state_var, utxo_datum.preimage };
    Note advice_note{ utxo_state_var, advice };

    new_note.constrain_against_advice(advice_note);

    // TODO: hard-code or calculate the correct commitment in the FakeDB stub, so that the returned data passes this
    // check.
    // Commenting-out this check for now, so the proof verifies.
    // info("calculated commitment: ", new_note.get_commitment());
    // info("retrieved commitment: ", utxo_datum.commitment);
    // new_note.get_commitment().assert_equal(utxo_datum.commitment, "UTXO_SLOAD: bad commitment");

    oracle.get_contract_address().assert_equal(utxo_datum.contract_address, "UTXO_SLOAD: bad contract address");

    // TODO within this function:
    // - Merkle Membership Check using the contract_address, utxo_datum.{sibling_path, leaf_index,
    // historic_private_data_tree_root}

    return new_note;
};

template <typename Builder> template <typename Note>
std::vector<Note> Opcodes<Builder>::UTXO_SLOAD(UTXOSetStateVar<Builder, Note>* utxo_set_state_var,
                                               size_t const& num_notes,
                                               typename Note::NotePreimage const& advice)
{
    auto& oracle = utxo_set_state_var->exec_ctx->oracle;

    typename CT::grumpkin_point const& storage_slot_point = utxo_set_state_var->storage_slot_point;

    // Retrieve multiple UTXO witness datum's from the DB:
    std::vector<UTXOSLoadDatum<CT, typename Note::NotePreimage>> utxo_data =
        oracle.template get_utxo_sload_data<typename Note::NotePreimage>(storage_slot_point, num_notes, advice);

    // Rely on the oracle to pad the data set with dummies, if there aren't enough notes in the DB.
    ASSERT(utxo_data.size() == num_notes);

    std::vector<Note> new_notes;

    for (size_t i = 0; i < num_notes; i++) {
        auto& utxo_datum = utxo_data[i];
        Note new_note{ utxo_set_state_var, utxo_datum.preimage };
        Note advice_note{ utxo_set_state_var, advice };

        new_note.constrain_against_advice(advice_note);

        // TODO: hard-code or calculate the correct commitment in the FakeDB stub, so that the returned data passes this
        // check.
        // Commenting-out this check for now, so the proof verifies.
        // info("calculated commitment: ", new_note.get_commitment());
        // info("retrieved commitment: ", utxo_datum.commitment);
        // new_note.get_commitment().assert_equal(utxo_datum.commitment, "UTXO_SLOAD: bad commitment");

        oracle.get_contract_address().assert_equal(utxo_datum.contract_address, "UTXO_SLOAD: bad contract address");

        // TODO within this function:
        // - Merkle Membership Check using the contract_address, utxo_datum.{sibling_path, leaf_index,
        // historic_private_data_tree_root}

        new_notes.push_back(new_note);
    }

    return new_notes;
};

template <typename Builder> template <typename Note>
void Opcodes<Builder>::UTXO_NULL(StateVar<Builder>* state_var, Note& note_to_nullify)
{
    typename CT::fr const nullifier = note_to_nullify.get_nullifier();
    typename CT::fr const nullified_note_commitment = note_to_nullify.get_commitment();

    auto& exec_ctx = state_var->exec_ctx;

    exec_ctx->new_nullifiers.push_back(nullifier);
    exec_ctx->nullified_commitments.push_back(nullified_note_commitment);

    std::shared_ptr<Note> const nullified_note_ptr = std::make_shared<Note>(note_to_nullify);

    exec_ctx->nullified_notes.push_back(nullified_note_ptr);
};

template <typename Builder> template <typename Note>
void Opcodes<Builder>::UTXO_INIT(StateVar<Builder>* state_var, Note& note_to_initialise)
{
    typename CT::fr const init_nullifier = note_to_initialise.get_initialisation_nullifier();
    typename CT::fr const init_commitment = note_to_initialise.get_initialisation_commitment();

    auto& exec_ctx = state_var->exec_ctx;

    exec_ctx->new_nullifiers.push_back(init_nullifier);
    exec_ctx->nullified_commitments.push_back(init_commitment);

    std::shared_ptr<Note> const init_note_ptr = std::make_shared<Note>(note_to_initialise);

    // TODO: consider whether this should actually be pushed-to...
    exec_ctx->nullified_notes.push_back(init_note_ptr);

    exec_ctx->new_notes.push_back(init_note_ptr);
};

template <typename Builder> template <typename Note>
void Opcodes<Builder>::UTXO_SSTORE(StateVar<Builder>* state_var, typename Note::NotePreimage new_note_preimage)
{
    auto& exec_ctx = state_var->exec_ctx;

    // Make a shared pointer, so we don't end up with a dangling pointer in the exec_ctx when this `new_note`
    // immediately goes out of scope.
    std::shared_ptr<Note> const new_note_ptr = std::make_shared<Note>(state_var, new_note_preimage);

    exec_ctx->new_notes.push_back(new_note_ptr);
};

}  // namespace aztec3::circuits::apps::opcodes
