#include "claim_circuit.hpp"
#include "ratio_check.hpp"
#include "../add_zero_public_inputs.hpp"
#include "../notes/circuit/index.hpp"
#include <stdlib/merkle_tree/membership.hpp>

// #pragma GCC diagnostic ignored "-Wunused-variable"
// #pragma GCC diagnostic ignored "-Wunused-parameter"
namespace rollup {
namespace proofs {
namespace claim {

using namespace plonk::stdlib::merkle_tree;
using namespace notes;

void claim_circuit(Composer& composer, claim_tx const& tx)
{
    // Create witnesses.
    const auto proof_id = field_ct(witness_ct(&composer, ProofIds::DEFI_CLAIM));
    const auto data_root = field_ct(witness_ct(&composer, tx.data_root));
    const auto defi_root = field_ct(witness_ct(&composer, tx.defi_root));
    const auto claim_note_index = witness_ct(&composer, tx.claim_note_index);
    const auto claim_note_path = create_witness_hash_path(composer, tx.claim_note_path);
    const auto claim_note_data = circuit::claim::claim_note_witness_data(composer, tx.claim_note);
    const auto claim_note = circuit::claim::claim_note(claim_note_data);
    const auto defi_interaction_note_path = create_witness_hash_path(composer, tx.defi_interaction_note_path);
    const auto defi_interaction_note = circuit::defi_interaction::note({ composer, tx.defi_interaction_note });
    const auto defi_interaction_note_dummy_nullifier_nonce =
        field_ct(witness_ct(&composer, tx.defi_interaction_note_dummy_nullifier_nonce));
    const auto output_value_a = field_ct(witness_ct(&composer, tx.output_value_a));
    const auto output_value_b = field_ct(witness_ct(&composer, tx.output_value_b));
    const auto two_output_notes = claim_note_data.bridge_id_data.num_output_notes == field_ct(2);

    // Ratio checks. Guarantees:
    // defi_interaction_note.total_input_value != 0
    // claim_note.deposit_value != 0
    const auto in_out_diff = defi_interaction_note.total_input_value - claim_note.deposit_value;
    in_out_diff.create_range_constraint(NOTE_VALUE_BIT_LENGTH);

    auto rc1 = ratio_check(composer,
                           { .a1 = claim_note.deposit_value,
                             .a2 = defi_interaction_note.total_input_value,
                             .b1 = output_value_a,
                             .b2 = defi_interaction_note.total_output_a_value });
    auto valid1 = (output_value_a == 0 && defi_interaction_note.total_output_a_value == 0) || rc1;
    valid1.assert_equal(true, "ratio check 1 failed");

    auto rc2 = ratio_check(composer,
                           { .a1 = claim_note.deposit_value,
                             .a2 = defi_interaction_note.total_input_value,
                             .b1 = output_value_b,
                             .b2 = defi_interaction_note.total_output_b_value });
    auto valid2 = (output_value_b == 0 && defi_interaction_note.total_output_b_value == 0) || rc2;
    valid2.assert_equal(true, "ratio check 2 failed");

    // Value notes must be completed with input_nullifiers' known unique values.
    // The second nullifier is is a 'dummy' - generated from randomness provided by the user.
    const auto nullifier1 = circuit::claim::compute_nullifier(claim_note.commitment);

    // TODO: Ask Ariel about this nullifier.
    const auto nullifier2 = circuit::defi_interaction::compute_dummy_nullifier(
        defi_interaction_note.commitment, defi_interaction_note_dummy_nullifier_nonce);

    // Compute output notes. Second note is zeroed if not used.
    // If defi interaction result is 0, refund original value.
    auto output_note1 = circuit::value::complete_partial_commitment(claim_note.value_note_partial_commitment,
                                                                    output_value_a,
                                                                    claim_note_data.bridge_id_data.output_asset_id_a,
                                                                    nullifier1);
    auto output_note2 = circuit::value::complete_partial_commitment(claim_note.value_note_partial_commitment,
                                                                    output_value_b,
                                                                    claim_note_data.bridge_id_data.output_asset_id_b,
                                                                    nullifier2);
    auto refund_note = circuit::value::complete_partial_commitment(claim_note.value_note_partial_commitment,
                                                                   claim_note_data.deposit_value,
                                                                   claim_note_data.bridge_id_data.input_asset_id,
                                                                   nullifier1);
    auto interaction_success = defi_interaction_note.interaction_result;
    output_note1 = output_note1 * interaction_success + refund_note * !interaction_success;
    output_note2 = output_note2 * two_output_notes * interaction_success;

    // Check claim note and interaction note are related.
    claim_note.bridge_id.assert_equal(defi_interaction_note.bridge_id, "note bridge ids don't match");
    claim_note.defi_interaction_nonce.assert_equal(defi_interaction_note.interaction_nonce, "note nonces don't match");

    // Check claim note exists and compute nullifier.
    auto claim_exists =
        check_membership(data_root, claim_note_path, claim_note.commitment, byte_array_ct(claim_note_index));
    claim_exists.assert_equal(true, "claim note not a member");

    // Check defi interaction note exists.
    const auto din_exists = check_membership(defi_root,
                                             defi_interaction_note_path,
                                             defi_interaction_note.commitment,
                                             byte_array_ct(defi_interaction_note.interaction_nonce));
    din_exists.assert_equal(true, "defi interaction note not a member");

    // Force unused public inputs to 0.
    const field_ct public_value = witness_ct(&composer, 0);
    const field_ct public_owner = witness_ct(&composer, 0);
    const field_ct asset_id = witness_ct(&composer, 0);
    const field_ct defi_deposit_value = witness_ct(&composer, 0);
    const field_ct propagated_input_index = witness_ct(&composer, 0);
    const field_ct backward_link = witness_ct(&composer, 0);
    const field_ct allow_claim = witness_ct(&composer, 0);
    public_value.assert_is_zero();
    public_owner.assert_is_zero();
    asset_id.assert_is_zero();
    defi_deposit_value.assert_is_zero();
    propagated_input_index.assert_is_zero();
    backward_link.assert_is_zero();
    allow_claim.assert_is_zero();

    // The following make up the public inputs to the circuit.
    proof_id.set_public();
    output_note1.set_public();
    output_note2.set_public();
    nullifier1.set_public();
    nullifier2.set_public();
    public_value.set_public();
    public_owner.set_public();
    asset_id.set_public();
    data_root.set_public();
    claim_note.fee.set_public();
    claim_note_data.bridge_id_data.input_asset_id.set_public();
    claim_note.bridge_id.set_public();
    defi_deposit_value.set_public();
    defi_root.set_public();
    propagated_input_index.set_public();
    backward_link.set_public();
    allow_claim.set_public();
}

} // namespace claim
} // namespace proofs
} // namespace rollup
