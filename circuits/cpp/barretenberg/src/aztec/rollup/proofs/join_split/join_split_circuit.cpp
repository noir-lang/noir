#include "join_split_circuit.hpp"
#include "../../constants.hpp"
#include "../notes/circuit/value/value_note.hpp"
#include "../notes/circuit/account/account_note.hpp"
#include "../notes/circuit/claim/claim_note.hpp"
#include "../notes/circuit/compute_nullifier.hpp"
#include "verify_signature.hpp"
#include <stdlib/merkle_tree/membership.hpp>
#include <stdlib/primitives/field/pow.hpp>

// #pragma GCC diagnostic ignored "-Wunused-variable"
// #pragma GCC diagnostic ignored "-Wunused-parameter"
namespace rollup {
namespace proofs {
namespace join_split {

using namespace plonk;
using namespace notes::circuit;
using namespace plonk::stdlib::merkle_tree;

/**
 * Check that the input note data, follows the given hash paths, to the publically given merkle root.
 * Return the nullifier for the input note.
 */
field_ct process_input_note(field_ct const& account_private_key,
                            field_ct const& merkle_root,
                            merkle_tree::hash_path const& hash_path,
                            field_ct const& index,
                            value::value_note const& note,
                            bool_ct is_real)
{
    auto exists = merkle_tree::check_membership(merkle_root, hash_path, note.commitment, byte_array_ct(index));
    (exists || !is_real).assert_equal(true, "input note not a member");

    bool_ct valid_value = note.value == 0 || is_real;
    valid_value.assert_equal(true, "padding note non zero");

    return compute_nullifier(note.commitment, index, account_private_key, is_real);
}

join_split_outputs join_split_circuit_component(Composer& composer, join_split_inputs const& inputs)
{
    auto not_defi_bridge = inputs.claim_note.deposit_value.is_zero();
    auto is_defi_bridge = (!not_defi_bridge).normalize();
    auto proof_id = (field_ct(2) * is_defi_bridge).normalize();
    auto public_output = inputs.claim_note.deposit_value + (inputs.public_output * not_defi_bridge);
    auto input_note1 = value::value_note(inputs.input_note1);
    auto input_note2 = value::value_note(inputs.input_note2);
    auto output_note1 = value::value_note(inputs.output_note1);
    auto output_note2 = value::value_note(inputs.output_note2);
    auto claim_note = claim::partial_claim_note(inputs.claim_note);
    auto asset_id = inputs.input_note1.asset_id * not_defi_bridge + claim_note.bridge_id * is_defi_bridge;
    auto input_owner = inputs.input_owner * not_defi_bridge + claim_note.value_note_partial_commitment * is_defi_bridge;
    auto output_note1_commitment =
        output_note1.commitment * not_defi_bridge + claim_note.partial_commitment * is_defi_bridge;

    // Check public input/output are zero when in defi mode.
    (not_defi_bridge || inputs.public_input == 0).assert_equal(true, "public input not zero");
    (not_defi_bridge || inputs.public_output == 0).assert_equal(true, "public output not zero");

    // Verify all notes have a consistent asset id
    inputs.input_note1.asset_id.assert_equal(inputs.input_note2.asset_id, "input note asset ids don't match");
    inputs.output_note1.asset_id.assert_equal(inputs.output_note2.asset_id, "output note asset ids don't match");
    inputs.input_note1.asset_id.assert_equal(inputs.output_note1.asset_id, "input/output note asset ids don't match");
    inputs.input_note1.asset_id.assert_equal(inputs.asset_id, "note asset ids not equal to tx asset id");

    auto valid_claim_note_asset_id =
        inputs.claim_note.bridge_id_data.input_asset_id == inputs.input_note1.asset_id || not_defi_bridge;
    valid_claim_note_asset_id.assert_equal(true, "input note and claim note asset ids don't match");

    // Check the claim note interaction nonce is 0 (will be added by the rollup circuit).
    claim_note.defi_interaction_nonce.assert_is_zero("interaction nonce must be 0");

    // Verify the asset id is less than the total number of assets.
    composer.create_range_constraint(inputs.asset_id.witness_index, NUM_ASSETS_BIT_LENGTH, "asset id too large");

    // Check we're not joining the same input note.
    (inputs.input_note1_index == inputs.input_note2_index).assert_equal(false, "joining same note");

    // Check public values.
    inputs.public_input.range_constraint(NOTE_VALUE_BIT_LENGTH, "public input too large");
    inputs.public_output.range_constraint(NOTE_VALUE_BIT_LENGTH, "public output too large");
    inputs.claim_note.deposit_value.range_constraint(DEFI_DEPOSIT_VALUE_BIT_LENGTH, "defi deposit too large");

    // Derive tx_fee.
    field_ct total_in_value = inputs.input_note1.value + inputs.input_note2.value + inputs.public_input;
    field_ct total_out_value = inputs.output_note1.value + inputs.output_note2.value + public_output;
    field_ct tx_fee = total_in_value - total_out_value;
    composer.create_range_constraint(tx_fee.witness_index, TX_FEE_BIT_LENGTH, "tx fee too large");

    // Verify input notes have the same account value id.
    auto note1 = inputs.input_note1;
    auto note2 = inputs.input_note2;
    note1.owner.x.assert_equal(note2.owner.x, "input note owners don't match");
    note1.owner.y.assert_equal(note2.owner.y, "input note owners don't match");
    note1.nonce.assert_equal(note2.nonce, "input note nonce don't match");

    // Verify input notes are owned by account private key and nonce.
    auto account_public_key = group_ct::fixed_base_scalar_mul<254>(inputs.account_private_key);
    account_public_key.x.assert_equal(note1.owner.x, "account_private_key incorrect");
    account_public_key.y.assert_equal(note1.owner.y, "account_private_key incorrect");
    inputs.nonce.assert_equal(note1.nonce, "nonce incorrect");

    // Verify that the given signature was signed over all 4 notes and output owner using
    // -> the account public key if nonce == 0
    // -> the given signing key if nonce > 0
    bool_ct zero_nonce = inputs.nonce == field_ct(0);
    point_ct signer = { account_public_key.x * zero_nonce + inputs.signing_pub_key.x * !zero_nonce,
                        account_public_key.y * zero_nonce + inputs.signing_pub_key.y * !zero_nonce };

    // alias hash must be 224 bits or fewer
    composer.create_range_constraint(inputs.alias_hash.witness_index, 224, "alias hash too large");
    // Verify that the account exists if nonce > 0
    auto account_alias_id = inputs.alias_hash + (inputs.nonce * pow(field_ct(2), uint32_ct(224)));
    auto account_note_data = account::account_note(account_alias_id, account_public_key, signer);
    auto signing_key_exists = merkle_tree::check_membership(
        inputs.merkle_root, inputs.account_path, account_note_data.commitment, byte_array_ct(inputs.account_index));
    (signing_key_exists || zero_nonce).assert_equal(true, "account check_membership failed");

    // Verify each input note exists in the tree, and compute nullifiers.
    bool_ct note_1_valid = !inputs.num_input_notes.is_zero();
    bool_ct note_2_valid = inputs.num_input_notes == 2;
    field_ct nullifier1 = process_input_note(inputs.account_private_key,
                                             inputs.merkle_root,
                                             inputs.input_path1,
                                             inputs.input_note1_index,
                                             input_note1,
                                             note_1_valid);
    field_ct nullifier2 = process_input_note(inputs.account_private_key,
                                             inputs.merkle_root,
                                             inputs.input_path2,
                                             inputs.input_note2_index,
                                             input_note2,
                                             note_2_valid);

    verify_signature(inputs.public_input,
                     public_output,
                     asset_id,
                     output_note1_commitment,
                     output_note2.commitment,
                     nullifier1,
                     nullifier2,
                     tx_fee,
                     signer,
                     inputs.input_owner,
                     inputs.output_owner,
                     inputs.signature);

    return { proof_id, nullifier1, nullifier2, tx_fee, public_output, output_note1_commitment, output_note2.commitment,
             asset_id, input_owner };
}

void join_split_circuit(Composer& composer, join_split_tx const& tx)
{
    join_split_inputs inputs = {
        witness_ct(&composer, tx.public_input),
        witness_ct(&composer, tx.public_output),
        witness_ct(&composer, tx.asset_id),
        witness_ct(&composer, tx.num_input_notes),
        witness_ct(&composer, tx.input_index[0]),
        witness_ct(&composer, tx.input_index[1]),
        value::witness_data(composer, tx.input_note[0]),
        value::witness_data(composer, tx.input_note[1]),
        value::witness_data(composer, tx.output_note[0]),
        value::witness_data(composer, tx.output_note[1]),
        claim::claim_note_tx_witness_data(composer, tx.claim_note),
        { witness_ct(&composer, tx.signing_pub_key.x), witness_ct(&composer, tx.signing_pub_key.y) },
        stdlib::schnorr::convert_signature(&composer, tx.signature),
        witness_ct(&composer, tx.old_data_root),
        merkle_tree::create_witness_hash_path(composer, tx.input_path[0]),
        merkle_tree::create_witness_hash_path(composer, tx.input_path[1]),
        witness_ct(&composer, tx.account_index),
        merkle_tree::create_witness_hash_path(composer, tx.account_path),
        witness_ct(&composer, tx.input_owner),
        witness_ct(&composer, tx.output_owner),
        witness_ct(&composer, static_cast<fr>(tx.account_private_key)),
        witness_ct(&composer, tx.alias_hash),
        witness_ct(&composer, tx.nonce),
    };

    auto outputs = join_split_circuit_component(composer, inputs);

    // The following make up the public inputs to the circuit.
    outputs.proof_id.set_public();
    inputs.public_input.set_public();
    outputs.public_output.set_public();
    outputs.asset_id.set_public();
    outputs.output_note1.set_public();
    outputs.output_note2.set_public();
    outputs.nullifier1.set_public();
    outputs.nullifier2.set_public();
    outputs.input_owner.set_public();
    inputs.output_owner.set_public();

    // Any public witnesses exposed from here on, will not be exposed by the rollup, and thus will
    // not be part of the calldata on chain, and will also not be part of tx id generation, or be signed over.
    inputs.merkle_root.set_public();
    outputs.tx_fee.set_public();
}

} // namespace join_split
} // namespace proofs
} // namespace rollup
