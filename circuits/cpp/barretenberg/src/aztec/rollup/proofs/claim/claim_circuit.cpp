#include "claim_circuit.hpp"
#include "ratio_check.hpp"
#include "../notes/circuit/index.hpp"
#include <stdlib/merkle_tree/membership.hpp>

// #pragma GCC diagnostic ignored "-Wunused-variable"
// #pragma GCC diagnostic ignored "-Wunused-parameter"
namespace rollup {
namespace proofs {
namespace claim {

using namespace plonk::stdlib::merkle_tree;
using namespace notes;

field_ct compute_nullifier(point_ct const& note_commitment, field_ct const& tree_index)
{
    auto blake_input = byte_array_ct(note_commitment.x).write(byte_array_ct(tree_index));
    auto blake_result = plonk::stdlib::blake2s(blake_input);
    return field_ct(blake_result);
}

void claim_circuit(Composer& composer, claim_tx const& tx)
{
    // Create witnesses.
    const auto proof_id = field_ct(witness_ct(&composer, 3));
    const auto data_root = field_ct(witness_ct(&composer, tx.data_root));
    const auto defi_root = field_ct(witness_ct(&composer, tx.defi_root));
    const auto claim_note_index = witness_ct(&composer, tx.claim_note_index);
    const auto claim_note_path = create_witness_hash_path(composer, tx.claim_note_path);
    const auto claim_note_data = circuit::claim::claim_note_witness_data(composer, tx.claim_note);
    const auto claim_note = circuit::claim::claim_note(claim_note_data);
    const auto defi_interaction_note_path = create_witness_hash_path(composer, tx.defi_interaction_note_path);
    const auto defi_interaction_note = circuit::defi_interaction::note({ composer, tx.defi_interaction_note });
    const auto output_value_a = field_ct(witness_ct(&composer, tx.output_value_a));
    const auto output_value_b = field_ct(witness_ct(&composer, tx.output_value_b));
    const auto two_output_notes = claim_note_data.bridge_id_data.num_output_notes == field_ct(2);

    // Ratio checks.
    const auto in_out_diff = defi_interaction_note.total_input_value - claim_note.deposit_value;
    composer.create_range_constraint(in_out_diff.witness_index, NOTE_VALUE_BIT_LENGTH);
    ratio_check(composer,
                { .total_in = defi_interaction_note.total_input_value,
                  .total_out = defi_interaction_note.total_output_a_value,
                  .user_in = claim_note.deposit_value,
                  .user_out = output_value_a });
    ratio_check(composer,
                { .total_in = defi_interaction_note.total_input_value,
                  .total_out = defi_interaction_note.total_output_b_value,
                  .user_in = claim_note.deposit_value,
                  .user_out = output_value_b });

    // Compute output notes. Second note is zeroed if not used.
    // If defi interaction result is 0, refund original value.
    auto output_note1 = circuit::claim::complete_partial_value_note(
        claim_note.partial_state, output_value_a, claim_note_data.bridge_id_data.output_asset_id_a);
    auto output_note2 = circuit::claim::complete_partial_value_note(
        claim_note.partial_state, output_value_b, claim_note_data.bridge_id_data.output_asset_id_b);
    auto refund_note = circuit::claim::complete_partial_value_note(
        claim_note.partial_state, claim_note_data.deposit_value, claim_note_data.bridge_id_data.input_asset_id);
    auto interaction_success = defi_interaction_note.interaction_result;
    output_note1.x = output_note1.x * interaction_success + refund_note.x * !interaction_success;
    output_note1.y = output_note1.y * interaction_success + refund_note.y * !interaction_success;
    output_note2.x = output_note2.x * two_output_notes * interaction_success;
    output_note2.y = output_note2.y * two_output_notes * interaction_success;

    // Check claim note and interaction note are related.
    claim_note.bridge_id.assert_equal(defi_interaction_note.bridge_id, "note bridge ids don't match");
    claim_note.defi_interaction_nonce.assert_equal(defi_interaction_note.interaction_nonce, "note nonces don't match");

    // Check claim note exists and compute nullifier.
    auto claim_exists = check_membership(
        composer, data_root, claim_note_path, byte_array_ct(claim_note), byte_array_ct(claim_note_index));
    claim_exists.assert_equal(true, "claim note not a member");
    const auto nullifier1 = compute_nullifier(claim_note.commitment, claim_note_index);

    // Check defi interaction note exists.
    const auto din_exists = check_membership(composer,
                                             defi_root,
                                             defi_interaction_note_path,
                                             byte_array_ct(defi_interaction_note),
                                             byte_array_ct(defi_interaction_note.interaction_nonce));
    din_exists.assert_equal(true, "defi interaction note not a member");

    // Force unused public inputs to 0.
    const field_ct public_input = witness_ct(&composer, 0);
    const field_ct public_output = witness_ct(&composer, 0);
    const field_ct nullifier2 = witness_ct(&composer, 0);
    const field_ct output_owner = witness_ct(&composer, 0);
    const field_ct tx_fee = witness_ct(&composer, 0);
    public_input.assert_is_zero();
    public_output.assert_is_zero();
    nullifier2.assert_is_zero();
    output_owner.assert_is_zero();
    tx_fee.assert_is_zero();

    // The following make up the public inputs to the circuit.
    composer.set_public_input(proof_id.witness_index);
    composer.set_public_input(public_input.witness_index);
    composer.set_public_input(public_output.witness_index);
    composer.set_public_input(claim_note.bridge_id.witness_index);
    output_note1.set_public();
    output_note2.set_public();
    composer.set_public_input(nullifier1.witness_index);
    composer.set_public_input(nullifier2.witness_index);
    composer.set_public_input(defi_root.witness_index);
    composer.set_public_input(output_owner.witness_index);
    composer.set_public_input(data_root.witness_index);
    composer.set_public_input(tx_fee.witness_index);
}

} // namespace claim
} // namespace proofs
} // namespace rollup
