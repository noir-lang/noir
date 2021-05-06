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
field_ct process_input_note(Composer& composer,
                            field_ct const& account_private_key,
                            field_ct const& merkle_root,
                            merkle_tree::hash_path const& hash_path,
                            field_ct const& index,
                            value::value_note const& note,
                            bool_ct is_real)
{
    auto exists =
        merkle_tree::check_membership(composer, merkle_root, hash_path, byte_array_ct(note), byte_array_ct(index));
    auto good = exists || !is_real;
    composer.assert_equal_constant(good.witness_index, 1, "input note not a member");

    bool_ct valid_value = note.value == field_ct(&composer, 0) || is_real;
    composer.assert_equal_constant(valid_value.witness_index, 1, "padding note non zero");

    return compute_nullifier(note.encrypted, index, account_private_key, is_real);
}

join_split_outputs join_split_circuit_component(Composer& composer, join_split_inputs const& inputs)
{
    auto not_defi_bridge = inputs.claim_note.deposit_value.is_zero();
    auto is_defi_bridge = (!not_defi_bridge).normalize();
    auto public_input = inputs.public_input * not_defi_bridge;
    auto public_output = inputs.claim_note.deposit_value + (inputs.public_output * not_defi_bridge);

    auto input_note1 = value::value_note(inputs.input_note1);
    auto input_note2 = value::value_note(inputs.input_note2);
    auto output_note1 = value::value_note(inputs.output_note1);
    auto output_note2 = value::value_note(inputs.output_note2);
    auto claim_note = claim::claim_note(inputs.claim_note, inputs.input_note1.owner, inputs.input_note1.nonce);

    // Verify all notes have a consistent asset id
    composer.assert_equal(inputs.input_note1.asset_id.witness_index,
                          inputs.input_note2.asset_id.witness_index,
                          "input note asset ids don't match");
    composer.assert_equal(inputs.output_note1.asset_id.witness_index,
                          inputs.output_note2.asset_id.witness_index,
                          "output note asset ids don't match");
    composer.assert_equal(inputs.input_note1.asset_id.witness_index,
                          inputs.output_note1.asset_id.witness_index,
                          "input/output note asset ids don't match");
    composer.assert_equal(inputs.input_note1.asset_id.witness_index,
                          inputs.asset_id.witness_index,
                          "note asset ids not equal to tx asset id");
    auto valid_claim_note_asset_id =
        inputs.claim_note.bridge_id.input_asset_id == inputs.input_note1.asset_id || not_defi_bridge;
    composer.assert_equal_constant(valid_claim_note_asset_id.witness_index, 1, "bridge asset ids don't match");

    // Verify the asset id is less than the total number of assets.
    composer.create_range_constraint(inputs.asset_id.witness_index, NUM_ASSETS_BIT_LENGTH);

    // Check we're not joining the same input note.
    bool_ct indicies_equal = inputs.input_note1_index == inputs.input_note2_index;
    composer.assert_equal_constant(indicies_equal.witness_index, 0, "joining same note");

    // Check public values.
    composer.create_range_constraint(inputs.public_input.witness_index, NOTE_VALUE_BIT_LENGTH);
    composer.create_range_constraint(inputs.public_output.witness_index, NOTE_VALUE_BIT_LENGTH);

    // Derive tx_fee.
    field_ct total_in_value = inputs.input_note1.value + inputs.input_note2.value + public_input;
    field_ct total_out_value = inputs.output_note1.value + inputs.output_note2.value + public_output;
    field_ct tx_fee = (total_in_value - total_out_value).normalize();
    composer.create_range_constraint(tx_fee.witness_index, TX_FEE_BIT_LENGTH);

    // Verify input notes have the same account value id.
    auto note1 = inputs.input_note1;
    auto note2 = inputs.input_note2;
    composer.assert_equal(note1.owner.x.witness_index, note2.owner.x.witness_index, "input note owners don't match");
    composer.assert_equal(note1.owner.y.witness_index, note2.owner.y.witness_index, "input note owners don't match");
    composer.assert_equal(note1.nonce.witness_index, note2.nonce.witness_index, "input note nonce don't match");

    // Verify input notes are owned by account private key and nonce.
    auto account_public_key = group_ct::fixed_base_scalar_mul<254>(inputs.account_private_key);
    composer.assert_equal(
        account_public_key.x.witness_index, note1.owner.x.witness_index, "account_private_key incorrect");
    composer.assert_equal(
        account_public_key.y.witness_index, note1.owner.y.witness_index, "account_private_key incorrect");
    composer.assert_equal(inputs.nonce.witness_index, note1.nonce.witness_index, "nonce incorrect");

    // Verify that the given signature was signed over all 4 notes and output owner using
    // -> the account public key if nonce == 0
    // -> the given signing key if nonce > 0
    bool_ct zero_nonce = inputs.nonce == field_ct(0);
    point_ct signer = { account_public_key.x * zero_nonce + inputs.signing_pub_key.x * !zero_nonce,
                        account_public_key.y * zero_nonce + inputs.signing_pub_key.y * !zero_nonce };

    // alias hash must be 224 bits or fewer
    composer.create_range_constraint(inputs.alias_hash.witness_index, 224);
    // Verify that the account exists if nonce > 0
    auto account_alias_id = inputs.alias_hash + (inputs.nonce * pow(field_ct(2), uint32_ct(224)));
    auto account_note_data = account::account_note(account_alias_id, account_public_key, signer);
    auto exists = merkle_tree::check_membership(composer,
                                                inputs.merkle_root,
                                                inputs.account_path,
                                                byte_array_ct(account_note_data),
                                                byte_array_ct(inputs.account_index));
    auto signing_key_registered_or_zero_nonce = exists || zero_nonce;
    composer.assert_equal_constant(
        signing_key_registered_or_zero_nonce.witness_index, 1, "account check_membership failed");

    // Verify each input note exists in the tree, and compute nullifiers.
    bool_ct note_1_valid = (!field_ct(inputs.num_input_notes).is_zero()).normalize();
    bool_ct note_2_valid = field_ct(inputs.num_input_notes) == field_ct(&composer, 2);
    field_ct nullifier1 = process_input_note(composer,
                                             inputs.account_private_key,
                                             inputs.merkle_root,
                                             inputs.input_path1,
                                             inputs.input_note1_index,
                                             input_note1,
                                             note_1_valid);
    field_ct nullifier2 = process_input_note(composer,
                                             inputs.account_private_key,
                                             inputs.merkle_root,
                                             inputs.input_path2,
                                             inputs.input_note2_index,
                                             input_note2,
                                             note_2_valid);

    verify_signature(inputs,
                     output_note1.encrypted,
                     output_note2.encrypted,
                     nullifier1,
                     nullifier2,
                     tx_fee,
                     signer,
                     inputs.signature);

    // Compute circuit public outputs.
    auto proof_id = field_ct(2) * is_defi_bridge;
    point_ct out_note1 = { output_note1.encrypted.x * not_defi_bridge + claim_note.encrypted.x * is_defi_bridge,
                           output_note1.encrypted.y * not_defi_bridge + claim_note.encrypted.y * is_defi_bridge };
    auto asset_id = inputs.input_note1.asset_id * not_defi_bridge + claim_note.bridge_id * is_defi_bridge;

    return { proof_id, nullifier1, nullifier2, tx_fee, public_input, public_output, out_note1, output_note2.encrypted,
             asset_id };
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
        value::witness_data::from_tx_data(composer, tx.input_note[0]),
        value::witness_data::from_tx_data(composer, tx.input_note[1]),
        value::witness_data::from_tx_data(composer, tx.output_note[0]),
        value::witness_data::from_tx_data(composer, tx.output_note[1]),
        claim::witness_data::from_tx_data(composer, tx.claim_note),
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
    composer.set_public_input(outputs.proof_id.witness_index);
    composer.set_public_input(inputs.public_input.get_witness_index());
    composer.set_public_input(outputs.public_output.witness_index);
    composer.set_public_input(outputs.asset_id.witness_index);
    outputs.output_note1.set_public();
    outputs.output_note2.set_public();
    composer.set_public_input(outputs.nullifier1.witness_index);
    composer.set_public_input(outputs.nullifier2.witness_index);
    composer.set_public_input(inputs.input_owner.witness_index);
    composer.set_public_input(inputs.output_owner.witness_index);

    // Any public witnesses exposed from here on, will not be exposed by the rollup, and thus will
    // not be part of the calldata on chain, and will also not be part of tx id generation, or be signed over.
    composer.set_public_input(inputs.merkle_root.witness_index);
    composer.set_public_input(outputs.tx_fee.witness_index);
} // namespace join_split

} // namespace join_split
} // namespace proofs
} // namespace rollup
