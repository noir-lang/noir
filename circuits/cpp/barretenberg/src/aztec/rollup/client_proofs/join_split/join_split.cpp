#include "join_split.hpp"
#include "../../pedersen_note/pedersen_note.hpp"
#include "account_note.hpp"
#include "note_pair.hpp"
#include "verify_signature.hpp"
#include <common/log.hpp>
#include <plonk/composer/turbo/compute_verification_key.hpp>
#include <stdlib/encryption/schnorr/schnorr.hpp>
#include <stdlib/merkle_tree/membership.hpp>

// #pragma GCC diagnostic ignored "-Wunused-variable"
// #pragma GCC diagnostic ignored "-Wunused-parameter"
namespace rollup {
namespace client_proofs {
namespace join_split {

using namespace plonk;
using namespace pedersen_note;

typedef std::pair<private_note, public_note> note_pair;

static std::shared_ptr<waffle::proving_key> proving_key;
static std::shared_ptr<waffle::verification_key> verification_key;

field_ct process_input_note(Composer& composer,
                            field_ct const& merkle_root,
                            merkle_tree::fr_hash_path const& hash_path,
                            field_ct const& index,
                            note_pair const& note,
                            bool_ct is_real)
{
    // Check that the input note data, follows the given hash paths, to the publically given merkle root.
    auto witness_hash_path = merkle_tree::create_witness_hash_path(composer, hash_path);

    byte_array_ct leaf(&composer);
    leaf.write(note.second.ciphertext.x).write(note.second.ciphertext.y);

    bool_ct good =
        merkle_tree::check_membership(composer, merkle_root, witness_hash_path, leaf, byte_array_ct(index)) || !is_real;
    composer.assert_equal_constant(good.witness_index, 1, "input note not a member");

    bool_ct validValue = note.first.value == uint32_ct(&composer, 0) || is_real;
    composer.assert_equal_constant(validValue.witness_index, 1, "padding note non zero");

    // Compute input notes nullifier index. We mix in the index and notes secret as part of the value we hash into the
    // tree to ensure notes will always have unique entries. The is_real flag protects against nullifing a real
    // note when the number of input notes < 2.
    // [256 bits of encrypted note x coord][32 least sig bits of index][223 bits of note viewing key][1 bit is_real]
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
                              merkle_tree::fr_hash_path const& hash_path,
                              field_ct const& index,
                              account_note const& account_note,
                              bool_ct must_exist)
{
    // Check that the input note data, follows the given hash paths, to the publically given merkle root.
    auto witness_hash_path = merkle_tree::create_witness_hash_path(composer, hash_path);

    // TODO: Is it ok to just use the x coord?
    byte_array_ct leaf(&composer);
    leaf.write(account_note.account_key.x).write(account_note.signing_key.x);

    field_ct hashed = stdlib::merkle_tree::hash_value(leaf);

    // To avoid hashing the data twice (for nullifier and leaf value calculation), we use check_subtree_membership
    // at a height of 0, instead of the simpler check_membership function.
    bool_ct exists = merkle_tree::check_subtree_membership(
        composer, merkle_root, witness_hash_path, hashed, byte_array_ct(index), 0);
    bool_ct good = exists || !must_exist;

    // No input notes means we're not spending anything, in which case must_exist will be false.
    composer.assert_equal_constant(good.witness_index, 1, "account note not a member");

    return hashed;
}

void join_split_circuit(Composer& composer, join_split_tx const& tx)
{
    uint32_ct public_input = witness_ct(&composer, tx.public_input);
    uint32_ct public_output = witness_ct(&composer, tx.public_output);
    uint32_ct num_input_notes = witness_ct(&composer, tx.num_input_notes);

    field_ct input_note1_index = witness_ct(&composer, tx.input_index[0]);
    field_ct input_note2_index = witness_ct(&composer, tx.input_index[1]);

    // Check we're not joining the same input note.
    bool_ct indicies_equal = input_note1_index == input_note2_index;
    composer.assert_equal_constant(indicies_equal.witness_index, 0, "joining same note");

    note_pair input_note1_data = create_note_pair(composer, tx.input_note[0]);
    note_pair input_note2_data = create_note_pair(composer, tx.input_note[1]);

    note_pair output_note1_data = create_note_pair(composer, tx.output_note[0]);
    note_pair output_note2_data = create_note_pair(composer, tx.output_note[1]);

    // Verify input and output notes balance. Use field_ct to prevent overflow.
    field_ct total_in_value =
        field_ct(input_note1_data.first.value) + field_ct(input_note2_data.first.value) + field_ct(public_input);
    field_ct total_out_value =
        field_ct(output_note1_data.first.value) + field_ct(output_note2_data.first.value) + field_ct(public_output);
    composer.assert_equal(total_in_value.witness_index, total_out_value.witness_index, "values don't balance");

    // Verify input notes have the same owner.
    auto note1_owner = input_note1_data.first.owner;
    auto note2_owner = input_note2_data.first.owner;
    composer.assert_equal(note1_owner.x.witness_index, note2_owner.x.witness_index, "input note owners don't match");
    composer.assert_equal(note1_owner.y.witness_index, note2_owner.y.witness_index, "input note owners don't match");

    // Verify that the given signature was signed over all 4 notes using the given signing key.
    std::array<public_note, 4> notes = {
        input_note1_data.second, input_note2_data.second, output_note1_data.second, output_note2_data.second
    };
    verify_signature(composer, notes, tx.signing_pub_key, tx.signature);

    field_ct merkle_root = witness_ct(&composer, tx.merkle_root);

    // Verify each input note exists in the tree, and compute nullifiers.
    field_ct nullifier1 = process_input_note(
        composer, merkle_root, tx.input_path[0], input_note1_index, input_note1_data, num_input_notes >= 1);
    field_ct nullifier2 = process_input_note(
        composer, merkle_root, tx.input_path[1], input_note2_index, input_note2_data, num_input_notes >= 2);

    // Verify that the signing key is owned by the owner of the notes.
    field_ct account_index = witness_ct(&composer, tx.account_index);
    auto account_note = create_account_note(composer, tx.input_note[0].owner, tx.signing_pub_key);
    // The first condition means we can spend notes with only an account key (e.g. if there are no account notes).
    bool_ct must_exist = account_note.account_key.x != account_note.signing_key.x && num_input_notes >= 1;
    field_ct account_nullifier =
        process_account_note(composer, merkle_root, tx.account_path, account_index, account_note, must_exist);

    // The following make up the public inputs to the circuit.
    composer.set_public_input(public_input.get_witness_index());
    composer.set_public_input(public_output.get_witness_index());
    set_note_public(composer, output_note1_data.second);
    set_note_public(composer, output_note2_data.second);
    composer.set_public_input(nullifier1.witness_index);
    composer.set_public_input(nullifier2.witness_index);
    public_witness_ct(&composer, tx.input_owner);
    public_witness_ct(&composer, tx.output_owner);

    // Any public witnesses exposed from here on, will not be exposed by the rollup, and thus will
    // not be part of the calldata on chain, and will also not be part of tx id generation, or be signed over.
    composer.set_public_input(merkle_root.witness_index);
    composer.set_public_input(account_nullifier.witness_index);
}

void init_proving_key(std::unique_ptr<waffle::ReferenceStringFactory>&& crs_factory)
{
    // Junk data required just to create proving key.
    join_split_tx tx;
    tx.input_path[0].resize(32);
    tx.input_path[1].resize(32);
    tx.account_path.resize(32);

    Composer composer(std::move(crs_factory));
    join_split_circuit(composer, tx);
    proving_key = composer.compute_proving_key();
}

void init_proving_key(std::shared_ptr<waffle::ProverReferenceString> const& crs, waffle::proving_key_data&& pk_data)
{
    proving_key = std::make_shared<waffle::proving_key>(std::move(pk_data), crs);
}

void init_verification_key(std::unique_ptr<waffle::ReferenceStringFactory>&& crs_factory)
{
    if (!proving_key) {
        std::abort();
    }
    // Patch the 'nothing' reference string fed to init_proving_key.
    proving_key->reference_string = crs_factory->get_prover_crs(proving_key->n);
    verification_key = waffle::turbo_composer::compute_verification_key(proving_key, crs_factory->get_verifier_crs());
}

void init_verification_key(std::shared_ptr<waffle::VerifierMemReferenceString> const& crs,
                           waffle::verification_key_data&& vk_data)
{
    verification_key = std::make_shared<waffle::verification_key>(std::move(vk_data), crs);
}

UnrolledProver new_join_split_prover(join_split_tx const& tx)
{
    Composer composer(proving_key, nullptr);
    join_split_circuit(composer, tx);

    if (composer.failed) {
        error("composer logic failed: ", composer.err);
    }

    info("composer gates: ", composer.get_num_gates());
    info("public inputs: ", composer.public_inputs.size());

    return composer.create_unrolled_prover();
}

bool verify_proof(waffle::plonk_proof const& proof)
{
    UnrolledVerifier verifier(verification_key,
                              Composer::create_unrolled_manifest(verification_key->num_public_inputs));
    return verifier.verify_proof(proof);
}

std::shared_ptr<waffle::proving_key> get_proving_key()
{
    return proving_key;
}

std::shared_ptr<waffle::verification_key> get_verification_key()
{
    return verification_key;
}

} // namespace join_split
} // namespace client_proofs
} // namespace rollup
