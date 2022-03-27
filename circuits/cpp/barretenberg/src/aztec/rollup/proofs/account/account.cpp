#include "account.hpp"
#include "../notes/circuit/account/account_note.hpp"
#include "../mock/mock_circuit.hpp"
#include "../notes/constants.hpp"
#include "../add_zero_public_inputs.hpp"
#include <common/log.hpp>
#include <plonk/composer/turbo/compute_verification_key.hpp>
#include <stdlib/primitives/field/pow.hpp>
#include <stdlib/merkle_tree/membership.hpp>
#include <plonk/proof_system/commitment_scheme/kate_commitment_scheme.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>

// #pragma GCC diagnostic ignored "-Wunused-variable"
// #pragma GCC diagnostic ignored "-Wunused-parameter"

namespace rollup {
namespace proofs {
namespace account {

using namespace plonk;
using namespace plonk::stdlib::types::turbo;
using namespace notes::circuit::account;

static std::shared_ptr<waffle::proving_key> proving_key;
static std::shared_ptr<waffle::verification_key> verification_key;

field_ct compute_account_alias_id_nullifier(suint_ct const& account_alias_id)
{
    return pedersen::compress(std::vector<field_ct>{ account_alias_id.value },
                              notes::GeneratorIndex::ACCOUNT_ALIAS_ID_NULLIFIER);
}

void account_circuit(Composer& composer, account_tx const& tx)
{
    // @dev This has to be a witness because we want to set it as a public input (see set_public() later). However, we
    // don't want provers to have freedom to change this value.
    const auto proof_id = field_ct(witness_ct(&composer, ProofIds::ACCOUNT));
    proof_id.assert_equal(field_ct(ProofIds::ACCOUNT));

    // Extract witnesses
    const auto data_tree_root = field_ct(witness_ct(&composer, tx.merkle_root));
    const auto account_public_key = stdlib::create_point_witness(composer, tx.account_public_key);
    const auto new_account_public_key = stdlib::create_point_witness(composer, tx.new_account_public_key);
    const auto spending_public_key_1 = stdlib::create_point_witness(composer, tx.new_signing_pub_key_1, false);
    const auto spending_public_key_2 = stdlib::create_point_witness(composer, tx.new_signing_pub_key_2, false);
    const auto alias_hash = suint_ct(witness_ct(&composer, tx.alias_hash), ALIAS_HASH_BIT_LENGTH, "alias_hash");
    const auto account_nonce =
        suint_ct(witness_ct(&composer, tx.account_nonce), ACCOUNT_NONCE_BIT_LENGTH, "account_nonce");
    const auto migrate = bool_ct(witness_ct(&composer, tx.migrate));

    const auto account_note_index =
        suint_ct(witness_ct(&composer, tx.account_note_index), DATA_TREE_DEPTH, "account_note_index");
    const auto account_note_path = merkle_tree::create_witness_hash_path(composer, tx.account_note_path);
    const auto signing_pub_key = stdlib::create_point_witness(composer, tx.signing_pub_key);
    const auto signature = stdlib::schnorr::convert_signature(&composer, tx.signature);

    // Calculations begin:
    const auto account_alias_id = alias_hash + account_nonce * suint_ct(uint256_t(1) << 224);
    const auto output_account_nonce = account_nonce + migrate;
    const auto output_account_alias_id = alias_hash + output_account_nonce * suint_ct(uint256_t(1) << 224);

    const auto output_note_1 =
        account_note(output_account_alias_id.value, new_account_public_key, spending_public_key_1);
    const auto output_note_2 =
        account_note(output_account_alias_id.value, new_account_public_key, spending_public_key_2);

    // @dev unlimited zero-valued nullifiers are permitted by the rollup circuit (e.g. if migrate == 0).
    const auto nullifier_1 = compute_account_alias_id_nullifier(account_alias_id) * migrate;

    const bool_ct zero_account_nonce = account_nonce == suint_ct(0);
    const point_ct signer = { account_public_key.x * zero_account_nonce + signing_pub_key.x * !zero_account_nonce,
                              account_public_key.y * zero_account_nonce + signing_pub_key.y * !zero_account_nonce };

    // Validate that, if account_nonce == 0 then migrate == 1
    zero_account_nonce.must_imply(migrate, "account must be migrated");

    // Check signature.
    {
        bool composerAlreadyFailed = composer.failed;
        std::vector<field_ct> to_compress = { account_alias_id.value,
                                              account_public_key.x,
                                              new_account_public_key.x,
                                              spending_public_key_1.x,
                                              spending_public_key_2.x };
        const byte_array_ct message = pedersen::compress(to_compress);
        stdlib::schnorr::verify_signature(message, signer, signature);
        if (composer.failed && !composerAlreadyFailed) {
            // only assign this error if an error hasn't already been assigned.
            composer.err = "verify signature failed";
        }
    }

    // Check signing account note exists if account_nonce != 0.
    {
        const auto account_note_data = account_note(account_alias_id.value, account_public_key, signer);
        const auto account_note_exists =
            merkle_tree::check_membership(data_tree_root,
                                          account_note_path,
                                          account_note_data.commitment,
                                          account_note_index.value.decompose_into_bits(DATA_TREE_DEPTH));
        (!zero_account_nonce).must_imply(account_note_exists, "account check_membership failed");
    }

    // Check account public key does not change unless migrating.
    {
        const auto account_key_change =
            account_public_key.x != new_account_public_key.x || account_public_key.y != new_account_public_key.y;
        account_key_change.must_imply(migrate, "cannot change account keys unless migrating");
    }

    const field_ct nullifier_2 = witness_ct(&composer, 0);
    const field_ct public_value = witness_ct(&composer, 0);
    const field_ct public_owner = witness_ct(&composer, 0);
    const field_ct asset_id = witness_ct(&composer, 0);
    const field_ct tx_fee = witness_ct(&composer, 0);
    const field_ct tx_fee_asset_id = witness_ct(&composer, 0);
    const field_ct bridge_id = witness_ct(&composer, 0);
    const field_ct defi_deposit_value = witness_ct(&composer, 0);
    const field_ct defi_root = witness_ct(&composer, 0);
    const field_ct backward_link = witness_ct(&composer, 0);
    const field_ct allow_chain = witness_ct(&composer, 0);
    nullifier_2.assert_is_zero();
    public_value.assert_is_zero();
    public_owner.assert_is_zero();
    asset_id.assert_is_zero();
    tx_fee.assert_is_zero();
    tx_fee_asset_id.assert_is_zero();
    bridge_id.assert_is_zero();
    defi_deposit_value.assert_is_zero();
    defi_root.assert_is_zero();
    backward_link.assert_is_zero();
    allow_chain.assert_is_zero();

    // Expose public inputs:
    proof_id.set_public();
    output_note_1.commitment.set_public();
    output_note_2.commitment.set_public();
    nullifier_1.set_public();

    // Also expose zero-valued public inputs:
    nullifier_2.set_public();
    public_value.set_public();
    public_owner.set_public();
    asset_id.set_public();
    data_tree_root.set_public();
    tx_fee.set_public();
    tx_fee_asset_id.set_public();
    bridge_id.set_public();
    defi_deposit_value.set_public();
    defi_root.set_public();
    backward_link.set_public();
    allow_chain.set_public();
}

void init_proving_key(std::shared_ptr<waffle::ReferenceStringFactory> const& crs_factory, bool mock)
{
    // Junk data required just to create proving key.
    account_tx tx;
    tx.account_public_key = grumpkin::g1::affine_one;
    tx.new_account_public_key = grumpkin::g1::affine_one;
    tx.new_signing_pub_key_1 = grumpkin::g1::affine_one;
    tx.new_signing_pub_key_2 = grumpkin::g1::affine_one;
    tx.signing_pub_key = grumpkin::g1::affine_one;
    tx.account_note_path.resize(32);

    Composer composer(crs_factory);
    account_circuit(composer, tx);
    if (!mock) {
        proving_key = composer.compute_proving_key();
    } else {
        Composer mock_proof_composer(crs_factory);
        rollup::proofs::mock::mock_circuit(mock_proof_composer, composer.get_public_inputs());
        proving_key = mock_proof_composer.compute_proving_key();
    }
}

void init_proving_key(std::shared_ptr<waffle::ProverReferenceString> const& crs, waffle::proving_key_data&& pk_data)
{
    proving_key = std::make_shared<waffle::proving_key>(std::move(pk_data), crs);
}

void init_verification_key(std::shared_ptr<waffle::ReferenceStringFactory> const& crs_factory)
{
    if (!proving_key) {
        throw_or_abort("Compute proving key first.");
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

UnrolledProver new_account_prover(account_tx const& tx, bool mock)
{
    Composer composer(proving_key, nullptr);
    account_circuit(composer, tx);

    if (composer.failed) {
        info("composer logic failed: ", composer.err);
    }

    info("composer gates: ", composer.get_num_gates());
    info("public inputs: ", composer.public_inputs.size());

    if (!mock) {
        return composer.create_unrolled_prover();
    } else {
        Composer mock_proof_composer(proving_key, nullptr);
        rollup::proofs::mock::mock_circuit(mock_proof_composer, composer.get_public_inputs());
        return mock_proof_composer.create_unrolled_prover();
    }
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
