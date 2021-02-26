#include "account.hpp"
#include "../notes/circuit/account_note.hpp"
#include "../notes/constants.hpp"
#include <common/log.hpp>
#include <plonk/composer/turbo/compute_verification_key.hpp>
#include <stdlib/primitives/field/pow.hpp>
#include <stdlib/merkle_tree/membership.hpp>
#include <plonk/proof_system/commitment_scheme/kate_commitment_scheme.hpp>

// #pragma GCC diagnostic ignored "-Wunused-variable"
// #pragma GCC diagnostic ignored "-Wunused-parameter"
namespace rollup {
namespace proofs {
namespace account {

using namespace plonk;
using namespace plonk::stdlib::types::turbo;
using namespace notes::circuit;

static std::shared_ptr<waffle::proving_key> proving_key;
static std::shared_ptr<waffle::verification_key> verification_key;

field_ct compute_account_alias_id_nullifier(field_ct const& proof_id,
                                            field_ct const& account_alias_id,
                                            field_ct const& gibberish,
                                            bool_ct migrate)
{
    return pedersen::compress(
        { proof_id, account_alias_id, gibberish * !migrate }, true, notes::ACCOUNT_ALIAS_ID_HASH_INDEX);
}

field_ct compute_gibberish_nullifier(field_ct const& proof_id, field_ct const& gibberish)
{
    return pedersen::compress({ proof_id, gibberish }, true, notes::ACCOUNT_GIBBERISH_HASH_INDEX);
}

void account_circuit(Composer& composer, account_tx const& tx)
{
    const auto proof_id = field_ct(witness_ct(&composer, 1));
    const auto nonce = field_ct(witness_ct(&composer, tx.nonce));
    const auto alias_hash = field_ct(witness_ct(&composer, tx.alias_hash));
    const auto migrate = bool_ct(witness_ct(&composer, tx.migrate));
    const auto gibberish = field_ct(witness_ct(&composer, tx.gibberish));
    const auto signature = stdlib::schnorr::convert_signature(&composer, tx.signature);
    const auto account_public_key = stdlib::create_point_witness(composer, tx.account_public_key);
    const auto new_account_public_key = stdlib::create_point_witness(composer, tx.new_account_public_key);
    const auto spending_public_key_1 = stdlib::create_point_witness(composer, tx.new_signing_pub_key_1);
    const auto spending_public_key_2 = stdlib::create_point_witness(composer, tx.new_signing_pub_key_2);
    const auto account_note_index = field_ct(witness_ct(&composer, tx.account_index));
    const auto account_note_path = merkle_tree::create_witness_hash_path(composer, tx.account_path);
    const auto signing_pub_key = stdlib::create_point_witness(composer, tx.signing_pub_key);
    const auto data_tree_root = field_ct(witness_ct(&composer, tx.merkle_root));

    // alias hash must be 224 bits or fewer
    composer.create_range_constraint(alias_hash.witness_index, 224);
    const auto account_alias_id = alias_hash + nonce * pow(field_ct(2), uint32_ct(224));
    const auto output_nonce = nonce + migrate;
    const auto output_account_alias_id = alias_hash + (output_nonce * pow(field_ct(2), uint32_ct(224)));

    const auto output_note_1 =
        encrypt_account_note(output_account_alias_id, new_account_public_key, spending_public_key_1);
    const auto output_note_2 =
        encrypt_account_note(output_account_alias_id, new_account_public_key, spending_public_key_2);

    const auto nullifier_1 = compute_account_alias_id_nullifier(proof_id, account_alias_id, gibberish, migrate);
    const auto nullifier_2 = compute_gibberish_nullifier(proof_id, gibberish);

    // Check signature.
    const bool_ct zero_nonce = nonce == field_ct(0);

    // Validate that, if nonce == 0 then migrate == 1
    const bool_ct migrate_check = (migrate || !zero_nonce).normalize();
    composer.assert_equal_constant(migrate_check.witness_index, 1, "both nonce and migrate are 0");
    const point_ct signer = { account_public_key.x * zero_nonce + signing_pub_key.x * !zero_nonce,
                              account_public_key.y * zero_nonce + signing_pub_key.y * !zero_nonce };
    std::vector<field_ct> to_compress = { account_alias_id,
                                          account_public_key.x,
                                          new_account_public_key.x,
                                          spending_public_key_1.x,
                                          spending_public_key_2.x };
    const byte_array_ct message = pedersen::compress(to_compress, true);
    stdlib::schnorr::verify_signature(message, signer, signature);
    if (composer.failed) {
        composer.err = "verify signature failed.";
    }

    // Check signing account note exists if nonce != 0.
    const auto assert_account_exists = !zero_nonce;
    const auto account_note_data = encrypt_account_note(account_alias_id, account_public_key, signer);
    const auto leaf_data = byte_array_ct(account_note_data.x).write(account_note_data.y);
    const auto exists = merkle_tree::check_membership(
        composer, data_tree_root, account_note_path, leaf_data, byte_array_ct(account_note_index));
    composer.assert_equal(exists.normalize().witness_index,
                          assert_account_exists.normalize().witness_index,
                          "account check_membership failed");

    // Check account public key does not change unless migrating.
    const auto account_keys_equal_or_migrating =
        (account_public_key.x == new_account_public_key.x && account_public_key.y == new_account_public_key.y) ||
        migrate;
    composer.assert_equal_constant(account_keys_equal_or_migrating.witness_index, 1, "public key should not change");

    field_ct dummy_tx_fee = witness_ct(&composer, 0);
    composer.assert_equal(dummy_tx_fee.witness_index, composer.zero_idx);

    // Expose public inputs.
    composer.set_public_input(proof_id.witness_index);                 // proof_id
    composer.set_public_input(new_account_public_key.x.witness_index); // public_input but using for owner x.
    composer.set_public_input(new_account_public_key.y.witness_index); // public_output but using for owner y.
    composer.set_public_input(output_account_alias_id.witness_index);  // asset_id
    composer.set_public_input(output_note_1.x.witness_index);
    composer.set_public_input(output_note_1.y.witness_index);
    composer.set_public_input(output_note_2.x.witness_index);
    composer.set_public_input(output_note_2.y.witness_index);
    composer.set_public_input(nullifier_1.witness_index);
    composer.set_public_input(nullifier_2.witness_index);
    composer.set_public_input(spending_public_key_1.x.witness_index); // input_owner
    composer.set_public_input(spending_public_key_2.x.witness_index); // output_owner
    composer.set_public_input(data_tree_root.witness_index);
    composer.set_public_input(dummy_tx_fee.witness_index);
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

    std::unique_ptr<waffle::KateCommitmentScheme<waffle::unrolled_turbo_settings>> kate_commitment_scheme =
        std::make_unique<waffle::KateCommitmentScheme<waffle::unrolled_turbo_settings>>();
    verifier.commitment_scheme = std::move(kate_commitment_scheme);

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
