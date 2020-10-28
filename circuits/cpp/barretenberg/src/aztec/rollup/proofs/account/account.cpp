#include "account.hpp"
#include "../notes/account_note.hpp"
#include "../notes/note_generator_indices.hpp"
#include <common/log.hpp>
#include <plonk/composer/turbo/compute_verification_key.hpp>
#include <stdlib/merkle_tree/membership.hpp>

#pragma GCC diagnostic ignored "-Wunused-variable"
#pragma GCC diagnostic ignored "-Wunused-parameter"
namespace rollup {
namespace proofs {
namespace account {

using namespace plonk;
using namespace plonk::stdlib::types::turbo;

static std::shared_ptr<waffle::proving_key> proving_key;
static std::shared_ptr<waffle::verification_key> verification_key;

field_ct process_account_note(Composer& composer,
                              field_ct const& merkle_root,
                              merkle_tree::fr_hash_path const& hash_path,
                              field_ct const& index,
                              notes::account_note const& account_note,
                              bool_ct const& must_exist)
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

    // Account not nullifier leaks info. Returning 0 for now.
    // return hashed;
    return field_ct(witness_ct(&composer, 0));
}

field_ct compute_alias_nullifier(Composer& composer, field_ct const& alias, bool register_alias_)
{
    const bool_ct register_alias = bool_ct(witness_ct(&composer, register_alias_));
    const field_ct prefix = (field_ct(witness_ct(&composer, (uint8_t)notes::ALIAS)) * register_alias) +
                            (field_ct(witness_ct(&composer, (uint8_t)notes::GIBBERISH)) * !register_alias);
    const std::vector<field_ct> hash_elements{
        prefix,
        alias,  
    };
    return pedersen::compress(hash_elements, true, notes::ALIAS_NULLIFIER_INDEX);
}

void account_circuit(Composer& composer, account_tx const& tx)
{
    const field_ct merkle_root = witness_ct(&composer, tx.merkle_root);
    const point_ct owner_pub_key = stdlib::create_point_witness(composer, tx.owner_pub_key);
    const point_ct new_signing_pub_key_1 = stdlib::create_point_witness(composer, tx.new_signing_pub_key_1);
    const point_ct new_signing_pub_key_2 = stdlib::create_point_witness(composer, tx.new_signing_pub_key_2);
    const point_ct signing_pub_key = stdlib::create_point_witness(composer, tx.signing_pub_key);
    const point_ct nullified_key = stdlib::create_point_witness(composer, tx.nullified_key);
    const uint32_ct num_new_keys = witness_ct(&composer, tx.num_new_keys);
    const auto new_account_note_1 = notes::account_note(owner_pub_key, new_signing_pub_key_1, num_new_keys >= 1);
    const auto new_account_note_2 = notes::account_note(owner_pub_key, new_signing_pub_key_2, num_new_keys >= 2);
    const auto remove_account = notes::account_note(owner_pub_key, nullified_key, bool_ct(&composer, tx.nullify_key));
    const field_ct alias = witness_ct(&composer, tx.alias);
    const auto signing_account_note = notes::account_note(owner_pub_key, signing_pub_key, true);

    const auto alias_nullifier = compute_alias_nullifier(composer, alias, tx.register_alias);
    const auto remove_account_nullifier = remove_account.nullifier();

    std::vector<field_ct> to_compress = {
        owner_pub_key.x, new_account_note_1.signing_pub_key().x, new_account_note_2.signing_pub_key().x,
        alias,           remove_account.signing_pub_key().x,
    };
    const byte_array_ct message = pedersen::compress(to_compress, true);
    stdlib::schnorr::signature_bits<Composer> signature = stdlib::schnorr::convert_signature(&composer, tx.signature);
    stdlib::schnorr::verify_signature(message, signing_account_note.signing_pub_key(), signature);
    if (composer.failed) {
        composer.err = "verify signature failed.";
    }

    // Verify that the signing key is either the owner key, or another existing account key.
    field_ct account_index = witness_ct(&composer, tx.account_index);
    bool_ct must_exist = signing_account_note.owner_pub_key().x != signing_account_note.signing_pub_key().x;
    field_ct account_nullifier =
        process_account_note(composer, merkle_root, tx.account_path, account_index, signing_account_note, must_exist);

    // Expose public inputs.
    public_witness_ct(&composer, 1);                          // proof_id
    composer.set_public_input(owner_pub_key.x.witness_index); // public_input but using for owner x.
    composer.set_public_input(owner_pub_key.y.witness_index); // public_output but using for owner y.
    public_witness_ct(&composer, 0);                          // asset_id
    new_account_note_1.set_public();
    new_account_note_2.set_public();
    composer.set_public_input(alias_nullifier.witness_index);
    composer.set_public_input(remove_account_nullifier.witness_index);
    public_witness_ct(&composer, 0); // input_owner
    public_witness_ct(&composer, 0); // output_owner
    composer.set_public_input(merkle_root.witness_index);
    composer.set_public_input(account_nullifier.witness_index);
}

void init_proving_key(std::shared_ptr<waffle::ReferenceStringFactory> const& crs_factory)
{
    // Junk data required just to create proving key.
    account_tx tx;
    tx.account_path.resize(32);

    Composer composer(crs_factory);
    account_circuit(composer, tx);
    proving_key = composer.compute_proving_key();
}

void init_proving_key(std::shared_ptr<waffle::ProverReferenceString> const& crs, waffle::proving_key_data&& pk_data)
{
    proving_key = std::make_shared<waffle::proving_key>(std::move(pk_data), crs);
}

void init_verification_key(std::shared_ptr<waffle::ReferenceStringFactory> const& crs_factory)
{
    if (!proving_key) {
        init_proving_key(crs_factory);
    } else {
        // Patch the 'nothing' reference string fed to init_proving_key.
        proving_key->reference_string = crs_factory->get_prover_crs(proving_key->n);
    }
    verification_key = waffle::turbo_composer::compute_verification_key(proving_key, crs_factory->get_verifier_crs());
}

void init_verification_key(std::shared_ptr<waffle::VerifierMemReferenceString> const& crs,
                           waffle::verification_key_data&& vk_data)
{
    verification_key = std::make_shared<waffle::verification_key>(std::move(vk_data), crs);
}

UnrolledProver new_account_prover(account_tx const& tx)
{
    Composer composer(proving_key, nullptr);
    account_circuit(composer, tx);

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

} // namespace account
} // namespace proofs
} // namespace rollup
