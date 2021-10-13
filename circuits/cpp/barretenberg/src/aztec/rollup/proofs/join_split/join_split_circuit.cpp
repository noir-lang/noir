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
 * The note does not need to exist in the tree if it's not real, or if it's consumed (i.e. propagated = input).
 * Return the nullifier for the input note. If the input note is consumed, the nullifier becomes 0.
 */
field_ct process_input_note(field_ct const& account_private_key,
                            field_ct const& merkle_root,
                            merkle_tree::hash_path const& hash_path,
                            field_ct const& index,
                            value::value_note const& note,
                            bool_ct is_propagated,
                            bool_ct is_real)
{
    auto exists = merkle_tree::check_membership(merkle_root, hash_path, note.commitment, byte_array_ct(index));
    auto valid = exists || is_propagated || !is_real;
    valid.assert_equal(true, "input note not a member");

    bool_ct valid_value = note.value == 0 || is_real;
    valid_value.assert_equal(true, "padding note non zero");

    return compute_nullifier(note.commitment, account_private_key, is_real);
}

join_split_outputs join_split_circuit_component(join_split_inputs const& inputs)
{
    auto is_deposit = inputs.proof_id == field_ct(ProofIds::DEPOSIT);
    auto is_withdraw = inputs.proof_id == field_ct(ProofIds::WITHDRAW);
    auto is_public_tx = is_deposit || is_withdraw;
    auto is_defi_deposit = inputs.proof_id == field_ct(ProofIds::DEFI_DEPOSIT);
    auto not_defi_deposit = !is_defi_deposit;
    auto input_note1 = value::value_note(inputs.input_note1);
    auto input_note2 = value::value_note(inputs.input_note2);
    auto output_note1 = value::value_note(inputs.output_note1);
    auto output_note2 = value::value_note(inputs.output_note2);
    auto claim_note = claim::partial_claim_note(inputs.claim_note, inputs.input_note1.owner, inputs.input_note1.nonce);
    auto output_note1_commitment =
        field_ct::conditional_assign(is_defi_deposit, claim_note.partial_commitment, output_note1.commitment);
    auto public_asset_id = inputs.asset_id * is_public_tx;
    auto public_input = inputs.public_value * is_deposit;
    auto public_output = inputs.public_value * is_withdraw;
    auto defi_deposit_value = inputs.claim_note.deposit_value * is_defi_deposit;
    auto bridge_id = claim_note.bridge_id * is_defi_deposit;

    // Check public value and owner are not zero for deposit and withdraw.
    // Otherwise, they must be zero.
    (is_public_tx == inputs.public_value.is_zero()).assert_equal(false, "public value incorrect");
    (is_public_tx == inputs.public_owner.is_zero()).assert_equal(false, "public owner incorrect");

    // Verify all notes have a consistent asset id
    inputs.input_note1.asset_id.assert_equal(inputs.input_note2.asset_id, "input note asset ids don't match");
    inputs.output_note1.asset_id.assert_equal(inputs.output_note2.asset_id, "output note asset ids don't match");
    inputs.input_note1.asset_id.assert_equal(inputs.output_note1.asset_id, "input/output note asset ids don't match");
    inputs.input_note1.asset_id.assert_equal(inputs.asset_id, "note asset ids not equal to tx asset id");

    auto valid_claim_note_asset_id =
        inputs.claim_note.bridge_id_data.input_asset_id == inputs.input_note1.asset_id || not_defi_deposit;
    valid_claim_note_asset_id.assert_equal(true, "input note and claim note asset ids don't match");

    // Verify the asset id is less than the total number of assets.
    inputs.asset_id.create_range_constraint(MAX_NUM_ASSETS_BIT_LENGTH, "asset id too large");

    // Check we're not joining the same input note.
    (input_note1.commitment).assert_not_equal(input_note2.commitment, "joining same note");

    // Check public values.
    inputs.public_value.create_range_constraint(NOTE_VALUE_BIT_LENGTH, "public value too large");
    inputs.claim_note.deposit_value.create_range_constraint(DEFI_DEPOSIT_VALUE_BIT_LENGTH, "defi deposit too large");

    // Check chaining values:
    // Propagated_input_index must be in {0, 1, 2}
    bool_ct no_note_propagated = inputs.propagated_input_index == 0;
    bool_ct note1_propagated = inputs.propagated_input_index == 1;
    bool_ct note2_propagated = inputs.propagated_input_index == 2;
    (no_note_propagated || note1_propagated || note2_propagated)
        .assert_equal(true, "propagated_input_index out of range");

    // allow_chain must be in {0, 1, 2}
    bool_ct allow_chain_1 = inputs.allow_chain == 1;
    bool_ct allow_chain_2 = inputs.allow_chain == 2;
    (inputs.allow_chain == 0 || allow_chain_1 || allow_chain_2).assert_equal(true, "allow_chain out of range");

    // prevent chaining from a partial claim note:
    (is_defi_deposit).must_imply(!allow_chain_1, "cannot chain from a partial claim note");

    bool_ct note1_linked = inputs.backward_link == input_note1.commitment;
    bool_ct note2_linked = inputs.backward_link == input_note2.commitment;
    // These implications are forward-compatible with the more complex linking spec.
    (note1_propagated).must_imply(note1_linked, "inconsistent backward_link & propagated_input_index");
    (note2_propagated).must_imply(note2_linked, "inconsistent backward_link & propagated_input_index");
    (!note1_linked && !note2_linked)
        .must_imply(no_note_propagated, "inconsistent backward_link & propagated_input_index");

    // when allowing chaining, ensure propagation is to one's self (and not to some other user):
    point_ct self = input_note1.owner;
    (allow_chain_1).must_imply(output_note1.owner == self, "inter-user chaining disallowed");
    (allow_chain_2).must_imply(output_note2.owner == self, "inter-user chaining disallowed");

    // Derive tx_fee.
    field_ct total_in_value = public_input.add_two(inputs.input_note1.value, inputs.input_note2.value);
    field_ct total_out_value = inputs.output_note1.value.madd(not_defi_deposit, defi_deposit_value)
                                   .add_two(public_output, inputs.output_note2.value);
    field_ct tx_fee = total_in_value - total_out_value;
    tx_fee.create_range_constraint(TX_FEE_BIT_LENGTH, "tx fee too large");

    // Verify input notes have the same account value id.
    auto note1 = inputs.input_note1;
    auto note2 = inputs.input_note2;
    // These checks ensure we only need to compute one signature, for efficiency.
    note1.owner.assert_equal(note2.owner, "input note owners don't match");
    note1.nonce.assert_equal(note2.nonce, "input note nonces don't match");

    auto account_public_key = group_ct::fixed_base_scalar_mul<254>(inputs.account_private_key);
    account_public_key.assert_equal(note1.owner, "account_private_key incorrect");
    inputs.nonce.assert_equal(note1.nonce, "nonce incorrect");

    // Verify that the given signature was signed using
    // -> the account public key if nonce == 0
    // -> the given signing key if nonce > 0
    bool_ct zero_nonce = inputs.nonce == field_ct(0);
    point_ct signer = { field_ct::conditional_assign(zero_nonce, account_public_key.x, inputs.signing_pub_key.x),
                        field_ct::conditional_assign(zero_nonce, account_public_key.y, inputs.signing_pub_key.y) };

    // alias hash must be 224 bits or fewer
    inputs.alias_hash.create_range_constraint(224, "alias hash too large");
    // Verify that the account exists if nonce > 0
    auto account_alias_id = inputs.alias_hash + (inputs.nonce * field_ct(uint256_t(1) << 224));

    // Verify creator_pubkey is EITHER account_public_key.x OR 0 for both output notes
    account_public_key.x.assert_equal(
        account_public_key.x.madd(inputs.output_note1.creator_pubkey.is_zero(), inputs.output_note1.creator_pubkey),
        "output note 1 sender_pubkey mismatch");
    account_public_key.x.assert_equal(
        account_public_key.x.madd(inputs.output_note2.creator_pubkey.is_zero(), inputs.output_note2.creator_pubkey),
        "output note 2 sender_pubkey id mismatch");

    auto account_note_data = account::account_note(account_alias_id, account_public_key, signer);
    auto signing_key_exists = merkle_tree::check_membership(
        inputs.merkle_root, inputs.account_path, account_note_data.commitment, byte_array_ct(inputs.account_index));
    (signing_key_exists || zero_nonce).assert_equal(true, "account check_membership failed");

    // Verify each input note exists in the tree, and compute nullifiers.
    bool_ct note1_valid = inputs.num_input_notes == 1 || inputs.num_input_notes == 2;
    bool_ct note2_valid = inputs.num_input_notes == 2;
    field_ct nullifier1 = process_input_note(inputs.account_private_key,
                                             inputs.merkle_root,
                                             inputs.input_path1,
                                             inputs.input_note1_index,
                                             input_note1,
                                             note1_propagated,
                                             note1_valid);
    field_ct nullifier2 = process_input_note(inputs.account_private_key,
                                             inputs.merkle_root,
                                             inputs.input_path2,
                                             inputs.input_note2_index,
                                             input_note2,
                                             note2_propagated,
                                             note2_valid);

    // Assert that input nullifiers in the output note commitments equal the input note nullifiers.
    output_note1.input_nullifier.assert_equal(nullifier1, "output note 1 has incorrect input nullifier");
    output_note2.input_nullifier.assert_equal(nullifier2, "output note 2 has incorrect input nullifier");
    claim_note.input_nullifier.assert_equal(nullifier1 * is_defi_deposit, "claim note has incorrect input nullifier");

    verify_signature(inputs.public_value,
                     inputs.public_owner,
                     public_asset_id,
                     output_note1_commitment,
                     output_note2.commitment,
                     nullifier1,
                     nullifier2,
                     signer,
                     inputs.propagated_input_index,
                     inputs.backward_link,
                     inputs.allow_chain,
                     inputs.signature);

    return { nullifier1, nullifier2, output_note1_commitment, output_note2.commitment, public_asset_id,
             tx_fee,     bridge_id,  defi_deposit_value };
}

void join_split_circuit(Composer& composer, join_split_tx const& tx)
{
    join_split_inputs inputs = {
        witness_ct(&composer, tx.proof_id()),
        witness_ct(&composer, tx.public_value()),
        witness_ct(&composer, tx.public_owner),
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
        witness_ct(&composer, static_cast<fr>(tx.account_private_key)),
        witness_ct(&composer, tx.alias_hash),
        witness_ct(&composer, tx.nonce),
        witness_ct(&composer, tx.propagated_input_index),
        witness_ct(&composer, tx.backward_link),
        witness_ct(&composer, tx.allow_chain),
    };

    auto outputs = join_split_circuit_component(inputs);

    const field_ct defi_root = witness_ct(&composer, 0);
    defi_root.assert_is_zero();

    // The following make up the public inputs to the circuit.
    inputs.proof_id.set_public();
    outputs.output_note1.set_public();
    outputs.output_note2.set_public();
    outputs.nullifier1.set_public();
    outputs.nullifier2.set_public();
    inputs.public_value.set_public();
    inputs.public_owner.set_public();
    outputs.public_asset_id.set_public();

    // Any public witnesses exposed from here on, will not be exposed by the rollup, and thus will
    // not be part of the calldata on chain, and will also not be part of tx id generation, or be signed over.
    inputs.merkle_root.set_public();
    outputs.tx_fee.set_public();
    inputs.asset_id.set_public();
    outputs.bridge_id.set_public();
    outputs.defi_deposit_value.set_public();
    defi_root.set_public();
    inputs.propagated_input_index.set_public();
    inputs.backward_link.set_public();
    inputs.allow_chain.set_public();
}

} // namespace join_split
} // namespace proofs
} // namespace rollup
