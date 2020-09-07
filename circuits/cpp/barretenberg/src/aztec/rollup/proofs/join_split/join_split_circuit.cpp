#include "join_split_circuit.hpp"
#include "../notes/account_note.hpp"
#include "note_pair.hpp"
#include "verify_signature.hpp"
#include <stdlib/merkle_tree/membership.hpp>

// #pragma GCC diagnostic ignored "-Wunused-variable"
// #pragma GCC diagnostic ignored "-Wunused-parameter"
namespace rollup {
namespace proofs {
namespace join_split {

using namespace plonk;
using namespace pedersen_note;
using namespace plonk::stdlib::merkle_tree;

/**
 * Check that the input note data, follows the given hash paths, to the publically given merkle root.
 * Return the nullifier for the input note.
 */
field_ct process_input_note(Composer& composer,
                            field_ct const& merkle_root,
                            merkle_tree::hash_path const& hash_path,
                            field_ct const& index,
                            note_pair const& note,
                            bool_ct is_real)
{
    byte_array_ct leaf(&composer);
    leaf.write(note.second.ciphertext.x).write(note.second.ciphertext.y);

    bool_ct good =
        merkle_tree::check_membership(composer, merkle_root, hash_path, leaf, byte_array_ct(index)) || !is_real;
    composer.assert_equal_constant(good.witness_index, 1, "input note not a member");

    bool_ct validValue = note.first.value == field_ct(&composer, 0) || is_real;
    composer.assert_equal_constant(validValue.witness_index, 1, "padding note non zero");

    // Compute input notes nullifier index. We mix in the index and notes secret as part of the value we hash into the
    // tree to ensure notes will always have unique entries. The is_real flag protects against nullifing a real
    // note when the number of input notes < 2.
    // [256 bits of encrypted note x coord][32 least sig bits of index][223 bits of note secret][1 bit is_real]
    byte_array_ct note_hash_data = byte_array_ct(&composer);
    note_hash_data.write(note.second.ciphertext.x)
        .write(byte_array_ct(index).slice(28, 4))
        .write(byte_array_ct(note.first.secret).slice(4, 28));
    note_hash_data.set_bit(511, is_real);

    // We have to convert the byte_array_ct into a field_ct to get the montgomery form. Can we avoid this?
    field_ct nullifier_index = stdlib::merkle_tree::hash_value(note_hash_data);

    return nullifier_index;
}

field_ct process_account_note(Composer& composer,
                              field_ct const& merkle_root,
                              merkle_tree::hash_path const& hash_path,
                              field_ct const& index,
                              notes::account_note const& account_note,
                              bool_ct must_exist)
{
    byte_array_ct leaf = account_note.leaf_data();

    field_ct hashed = stdlib::merkle_tree::hash_value(leaf);

    // To avoid hashing the data twice (for nullifier and leaf value calculation), we use check_subtree_membership
    // at a height of 0, instead of the simpler check_membership function.
    bool_ct exists =
        merkle_tree::check_subtree_membership(composer, merkle_root, hash_path, hashed, byte_array_ct(index), 0);
    bool_ct good = exists || !must_exist;

    // No input notes means we're not spending anything, in which case must_exist will be false.
    composer.assert_equal_constant(good.witness_index, 1, "account note not a member");

    return hashed;
}

join_split_outputs join_split_circuit_component(Composer& composer, join_split_inputs const& inputs)
{
    // Check we're not joining the same input note.
    bool_ct indicies_equal = inputs.input_note1_index == inputs.input_note2_index;
    composer.assert_equal_constant(indicies_equal.witness_index, 0, "joining same note");

    // Verify input and output notes balance. Use field_ct to prevent overflow.
    field_ct total_in_value = inputs.input_note1.first.value + inputs.input_note2.first.value + inputs.public_input;
    field_ct total_out_value = inputs.output_note1.first.value + inputs.output_note2.first.value + inputs.public_output;
    composer.assert_equal(total_in_value.witness_index, total_out_value.witness_index, "values don't balance");

    // Verify input notes have the same owner.
    auto note1_owner = inputs.input_note1.first.owner;
    auto note2_owner = inputs.input_note2.first.owner;
    composer.assert_equal(note1_owner.x.witness_index, note2_owner.x.witness_index, "input note owners don't match");
    composer.assert_equal(note1_owner.y.witness_index, note2_owner.y.witness_index, "input note owners don't match");

    // Verify that the given signature was signed over all 4 notes using the given signing key.
    std::array<public_note, 4> notes = {
        inputs.input_note1.second, inputs.input_note2.second, inputs.output_note1.second, inputs.output_note2.second
    };
    verify_signature(notes, inputs.signing_pub_key, inputs.signature);

    // Verify each input note exists in the tree, and compute nullifiers.
    field_ct nullifier1 = process_input_note(composer,
                                             inputs.merkle_root,
                                             inputs.input_path1,
                                             inputs.input_note1_index,
                                             inputs.input_note1,
                                             inputs.num_input_notes >= 1);
    field_ct nullifier2 = process_input_note(composer,
                                             inputs.merkle_root,
                                             inputs.input_path2,
                                             inputs.input_note2_index,
                                             inputs.input_note2,
                                             inputs.num_input_notes >= 2);

    // Verify that the signing key is owned by the owner of the notes.
    auto account_note = notes::account_note(note1_owner, inputs.signing_pub_key, true);
    // The first condition means we can spend notes with only an account key (e.g. if there are no account notes).
    bool_ct must_exist =
        account_note.owner_pub_key().x != account_note.signing_pub_key().x && inputs.num_input_notes >= 1;
    field_ct account_nullifier = process_account_note(
        composer, inputs.merkle_root, inputs.account_path, inputs.account_index, account_note, must_exist);

    return { nullifier1, nullifier2, account_nullifier };
}

void join_split_circuit(Composer& composer, join_split_tx const& tx)
{
    join_split_inputs inputs = {
        witness_ct(&composer, tx.public_input),
        witness_ct(&composer, tx.public_output),
        witness_ct(&composer, tx.num_input_notes),
        witness_ct(&composer, tx.input_index[0]),
        witness_ct(&composer, tx.input_index[1]),
        create_note_pair(composer, tx.input_note[0]),
        create_note_pair(composer, tx.input_note[1]),
        create_note_pair(composer, tx.output_note[0]),
        create_note_pair(composer, tx.output_note[1]),
        { witness_ct(&composer, tx.signing_pub_key.x), witness_ct(&composer, tx.signing_pub_key.y) },
        stdlib::schnorr::convert_signature(&composer, tx.signature),
        witness_ct(&composer, tx.old_data_root),
        merkle_tree::create_witness_hash_path(composer, tx.input_path[0]),
        merkle_tree::create_witness_hash_path(composer, tx.input_path[1]),
        witness_ct(&composer, tx.account_index),
        merkle_tree::create_witness_hash_path(composer, tx.account_path),
    };

    auto outputs = join_split_circuit_component(composer, inputs);

    // The following make up the public inputs to the circuit.
    public_witness_ct(&composer, 0); // proof_id
    composer.set_public_input(inputs.public_input.get_witness_index());
    composer.set_public_input(inputs.public_output.get_witness_index());
    set_note_public(composer, inputs.output_note1.second);
    set_note_public(composer, inputs.output_note2.second);
    composer.set_public_input(outputs.nullifier1.witness_index);
    composer.set_public_input(outputs.nullifier2.witness_index);
    public_witness_ct(&composer, tx.input_owner);
    public_witness_ct(&composer, tx.output_owner);

    // Any public witnesses exposed from here on, will not be exposed by the rollup, and thus will
    // not be part of the calldata on chain, and will also not be part of tx id generation, or be signed over.
    composer.set_public_input(inputs.merkle_root.witness_index);
    composer.set_public_input(outputs.account_nullifier.witness_index);
} // namespace join_split

} // namespace join_split
} // namespace proofs
} // namespace rollup
