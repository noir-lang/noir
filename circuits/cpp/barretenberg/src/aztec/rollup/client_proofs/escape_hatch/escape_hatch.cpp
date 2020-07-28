#include "escape_hatch.hpp"
#include "../../pedersen_note/pedersen_note.hpp"
#include "note_pair.hpp"
#include "../notes/account_note.hpp"
#include "verify_signature.hpp"
#include <common/log.hpp>
#include <plonk/composer/turbo/compute_verification_key.hpp>
#include <stdlib/encryption/schnorr/schnorr.hpp>
#include <stdlib/merkle_tree/membership.hpp>
#include <plonk/proof_system/proving_key/serialize.hpp>

#pragma GCC diagnostic ignored "-Wunused-variable"
#pragma GCC diagnostic ignored "-Wunused-parameter"
namespace rollup {
namespace client_proofs {
namespace escape_hatch {

using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::merkle_tree;

typedef std::pair<private_note, public_note> note_pair;

static std::shared_ptr<waffle::proving_key> proving_key;
static std::shared_ptr<waffle::verification_key> verification_key;

field_ct process_input_note(Composer& composer,
                            field_ct merkle_root,
                            merkle_tree::fr_hash_path hash_path,
                            field_ct index,
                            note_pair const& note,
                            bool_ct is_real)
{
    // Check that the input note data, follows the given hash paths, to the publically given merkle root.
    auto witness_hash_path = merkle_tree::create_witness_hash_path(composer, hash_path);

    byte_array_ct leaf(&composer);
    leaf.write(note.second.ciphertext.x).write(note.second.ciphertext.y);

    bool_ct exists =
        merkle_tree::check_membership(composer, merkle_root, witness_hash_path, leaf, byte_array_ct(index));
    composer.assert_equal(is_real.witness_index, exists.witness_index);

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
                              notes::account_note const& account_note,
                              bool_ct must_exist)
{
    // Check that the input note data, follows the given hash paths, to the publically given merkle root.
    auto witness_hash_path = merkle_tree::create_witness_hash_path(composer, hash_path);

    byte_array_ct leaf = account_note.leaf_data();

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

void check_value_note_nullifiers(Composer& composer,
                                 escape_hatch_tx const& tx,
                                 std::array<field_ct, 2> const& nullifiers,
                                 uint32_t num_input_notes)
{
    auto old_null_root = field_ct(witness_ct(&composer, tx.old_nullifier_merkle_root));
    auto old_nullifier_value = byte_array_ct(&composer, 64);
    auto new_nullifier_value = byte_array_ct(&composer, 64);
    new_nullifier_value.set_bit(511, true);

    for (size_t i = 0; i < nullifiers.size(); ++i) {
        auto new_null_root = field_ct(witness_ct(&composer, tx.new_null_roots[i]));

        auto new_null_path = create_witness_hash_path(composer, tx.new_nullifier_paths[i]);
        auto old_null_path = create_witness_hash_path(composer, tx.current_nullifier_paths[i]);
        merkle_tree::update_membership(composer,
                                       new_null_root,
                                       new_null_path,
                                       new_nullifier_value,
                                       old_null_root,
                                       old_null_path,
                                       old_nullifier_value,
                                       byte_array_ct(nullifiers[i]));
        if (composer.failed && composer.err.empty()) {
            composer.err = "failed value note nullifier";
        }
        old_null_root = new_null_root;
    }
}

void check_account_not_nullified(Composer& composer,
                                 field_ct const& new_null_root,
                                 field_ct const& account_nullifier_index,
                                 fr_hash_path const& account_null_path,
                                 bool can_throw)
{
    auto hashes = create_witness_hash_path(composer, account_null_path);
    auto exists = merkle_tree::check_membership(
        composer, new_null_root, hashes, byte_array_ct(&composer, 64), byte_array_ct(account_nullifier_index));
    composer.assert_equal_constant(exists.witness_index, 1, "Failed account not nullified.");
}

void escape_hatch_circuit(Composer& composer, escape_hatch_tx const& tx)
{
    uint32_ct public_output = witness_ct(&composer, tx.public_output);
    uint32_ct num_input_notes = witness_ct(&composer, tx.num_input_notes);

    field_ct input_note1_index = witness_ct(&composer, tx.input_index[0]);
    field_ct input_note2_index = witness_ct(&composer, tx.input_index[1]);

    // Check we're not joining the same input note.
    bool_ct indicies_equal = input_note1_index == input_note2_index;
    composer.assert_equal_constant(indicies_equal.witness_index, 0, "joining same note");

    note_pair input_note1_data = create_note_pair(composer, tx.input_note[0]);
    note_pair input_note2_data = create_note_pair(composer, tx.input_note[1]);

    // Verify input and output notes balance. Use field_ct to prevent overflow.
    field_ct total_in_value = field_ct(input_note1_data.first.value) + field_ct(input_note2_data.first.value);
    field_ct total_out_value = field_ct(public_output);
    composer.assert_equal(total_in_value.witness_index, total_out_value.witness_index);

    // Verify input notes have the same owner.
    auto note1_owner = input_note1_data.first.owner;
    auto note2_owner = input_note2_data.first.owner;

    composer.assert_equal(note1_owner.x.witness_index, note2_owner.x.witness_index, "input note owners don't match");
    composer.assert_equal(note1_owner.y.witness_index, note2_owner.y.witness_index, "input note owners don't match");

    // Verify that the given signature was signed over all 2 notes using the input note owners private key.
    std::array<public_note, 2> notes = {
        input_note1_data.second,
        input_note2_data.second,
    };

    verify_signature(composer, notes, tx.signing_pub_key, tx.signature);

    field_ct merkle_root = witness_ct(&composer, tx.old_data_root);
    field_ct nullifier1 = process_input_note(
        composer, merkle_root, tx.input_path[0], input_note1_index, input_note1_data, num_input_notes >= 1);
    field_ct nullifier2 = process_input_note(
        composer, merkle_root, tx.input_path[1], input_note2_index, input_note2_data, num_input_notes >= 2);

    // Verify that the signing key is owned by the owner of the notes.
    field_ct account_index = witness_ct(&composer, tx.account_index);
    auto account_note = notes::account_note(note1_owner, tx.signing_pub_key, true);
    // The first condition means we can spend notes with only an account key (e.g. if there are no account notes).
    bool_ct must_exist = account_note.owner_pub_key().x != account_note.signing_pub_key().x && num_input_notes >= 1;
    auto account_note_nullifier =
        process_account_note(composer, merkle_root, tx.account_path, account_index, account_note, must_exist);

    auto old_null_root = field_ct(witness_ct(&composer, tx.old_nullifier_merkle_root));
    check_account_not_nullified(composer, old_null_root, account_note_nullifier, tx.account_nullifier_path, true);
    check_value_note_nullifiers(composer, tx, { nullifier1, nullifier2 }, tx.num_input_notes);

    // The following make up the public inputs to the circuit
    field_ct(public_witness_ct(&composer, tx.old_data_root));
    field_ct(public_witness_ct(&composer, tx.new_data_root));
    field_ct(public_witness_ct(&composer, tx.old_nullifier_merkle_root));
    field_ct(public_witness_ct(&composer, tx.new_null_roots[1])); // new_null_root
    field_ct(public_witness_ct(&composer, tx.old_data_roots_root));
    field_ct(public_witness_ct(&composer, tx.new_data_roots_root));
    uint32_ct(public_witness_ct(&composer, 2)); // proof_id
    composer.set_public_input(public_output.get_witness_index());
    public_witness_ct(&composer, tx.public_owner);
}

void init_proving_key(std::unique_ptr<waffle::ReferenceStringFactory>&& crs_factory)
{
    // Junk data required just to create proving key.
    escape_hatch_tx tx;
    tx.input_path[0].resize(32);
    tx.input_path[1].resize(32);
    tx.account_path.resize(32);
    tx.current_nullifier_paths[0].resize(128);
    tx.current_nullifier_paths[1].resize(128);
    tx.new_nullifier_paths[0].resize(128);
    tx.new_nullifier_paths[1].resize(128);
    tx.account_nullifier_path.resize(128);

    Composer composer(std::move(crs_factory), 512 * 1024);

    escape_hatch_circuit(composer, tx);

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
    info("vk: ", blake2::blake2s(to_buffer(*verification_key)));
}

void init_verification_key(std::shared_ptr<waffle::VerifierMemReferenceString> const& crs,
                           waffle::verification_key_data&& vk_data)
{
    verification_key = std::make_shared<waffle::verification_key>(std::move(vk_data), crs);
}

UnrolledProver new_escape_hatch_prover(escape_hatch_tx const& tx)
{
    Composer composer(proving_key, nullptr);
    escape_hatch_circuit(composer, tx);

    if (composer.failed) {
        error("composer logic failed: ", composer.err);
    }

    info("composer gates: ", composer.get_num_gates());
    info("public inputs: ", composer.public_inputs.size());

    return composer.create_unrolled_prover();
}

std::vector<uint8_t> create_escape_hatch_proof(escape_hatch_tx const& tx)
{
    Composer composer(proving_key, nullptr);
    escape_hatch_circuit(composer, tx);

    if (composer.failed) {
        error("composer logic failed: ", composer.err);
    }

    info("composer gates: ", composer.get_num_gates());
    info("public inputs: ", composer.public_inputs.size());

    auto prover = composer.create_unrolled_prover();
    auto proof = prover.construct_proof();
    auto proof_data = proof.proof_data;
    return proof_data;
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

} // namespace escape_hatch
} // namespace client_proofs
} // namespace rollup
