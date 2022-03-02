#include "../../constants.hpp"
#include "../../fixtures/user_context.hpp"
#include "account.hpp"
#include "../inner_proof_data/inner_proof_data.hpp"
#include "../notes/constants.hpp"
#include "../notes/native/index.hpp"
#include <common/streams.hpp>
#include <common/test.hpp>
#include <stdlib/merkle_tree/memory_store.hpp>
#include <stdlib/merkle_tree/merkle_tree.hpp>
#include <plonk/proof_system/proving_key/serialize.hpp>

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::merkle_tree;
using namespace rollup;
using namespace rollup::proofs;
using namespace rollup::proofs::account;
using namespace rollup::proofs::notes::native::account;

class account_tests : public ::testing::Test {
  protected:
    static void SetUpTestCase()
    {
        auto crs_factory =
            std::shared_ptr<waffle::ReferenceStringFactory>(new waffle::FileReferenceStringFactory("../srs_db"));
        init_proving_key(crs_factory, false);
        init_verification_key(crs_factory);
    }

    virtual void SetUp()
    {
        store = std::make_unique<MemoryStore>();
        tree = std::make_unique<MerkleTree<MemoryStore>>(*store, 32);
        user = rollup::fixtures::create_user_context();
    }

    /**
     * Add two account notes for the user.
     */
    void preload_account_notes()
    {
        auto account_alias_id = rollup::fixtures::generate_account_alias_id(user.alias_hash, 1);
        tree->update_element(
            tree->size(),
            create_account_leaf_data(account_alias_id, user.owner.public_key, user.signing_keys[0].public_key));
        tree->update_element(
            tree->size(),
            create_account_leaf_data(account_alias_id, user.owner.public_key, user.signing_keys[1].public_key));
    }

    fr create_account_leaf_data(fr const& account_alias_id,
                                grumpkin::g1::affine_element const& owner_key,
                                grumpkin::g1::affine_element const& signing_key)
    {
        return account_note{ account_alias_id, owner_key, signing_key }.commit();
    }

    uint256_t compute_account_alias_id_nullifier(fr const& account_alias_id)
    {
        const std::vector<fr> hash_elements{ account_alias_id };
        auto result =
            crypto::pedersen::compress_native(hash_elements, notes::GeneratorIndex::ACCOUNT_ALIAS_ID_NULLIFIER);
        return uint256_t(result);
    }

    fr compute_account_alias_id(barretenberg::fr alias_hash, uint32_t account_nonce)
    {
        return alias_hash + (fr{ (uint64_t)account_nonce } * fr(2).pow(224));
    }

    account_tx create_account_tx(uint32_t account_nonce = 0)
    {
        account_tx tx;
        tx.merkle_root = tree->root();
        tx.account_public_key = user.owner.public_key;
        tx.new_account_public_key = user.owner.public_key;
        tx.new_signing_pub_key_1 = user.signing_keys[0].public_key;
        tx.new_signing_pub_key_2 = user.signing_keys[1].public_key;
        tx.alias_hash = user.alias_hash;
        tx.account_nonce = account_nonce;
        tx.migrate = true;
        tx.account_note_index = 0;
        tx.signing_pub_key = user.signing_keys[0].public_key;
        tx.account_note_path = tree->get_hash_path(0);
        tx.sign(user.owner);

        return tx;
    }

    bool verify(account_tx& tx)
    {
        auto prover = new_account_prover(tx, false);
        auto proof = prover.construct_proof();
        return verify_proof(proof);
    }

    struct verify_logic_result {
        bool valid;
        std::string err;
    };

    verify_logic_result verify_logic(account_tx& tx)
    {
        Composer composer(get_proving_key(), nullptr);
        account_circuit(composer, tx);
        if (composer.failed) {
            info("Circuit logic failed: " + composer.err);
        }
        return { !composer.failed, composer.err };
    }

    rollup::fixtures::user_context user;
    std::unique_ptr<MemoryStore> store;
    std::unique_ptr<MerkleTree<MemoryStore>> tree;
};

TEST_F(account_tests, test_create_account)
{
    auto tx = create_account_tx();
    EXPECT_TRUE(verify_logic(tx).valid);
}

TEST_F(account_tests, test_create_account_full_proof)
{
    auto tx = create_account_tx();
    EXPECT_TRUE(verify(tx));
}

TEST_F(account_tests, test_migrate_account)
{
    preload_account_notes();
    auto tx = create_account_tx(1);
    tx.account_note_index = 0;
    tx.sign(user.signing_keys[0]);

    EXPECT_TRUE(verify_logic(tx).valid);
}

// Initial migration (account_nonce = 0)

TEST_F(account_tests, test_initial_account_not_migrated_fails)
{
    auto tx = create_account_tx();
    tx.migrate = false;
    tx.sign(user.owner);

    auto result = verify_logic(tx);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "account must be migrated");
}

// Signature

TEST_F(account_tests, test_wrong_account_key_pair_fails)
{
    auto tx = create_account_tx();
    auto keys = rollup::fixtures::create_key_pair(nullptr);
    tx.sign(keys); // sign the tx with the wrong signing private key

    EXPECT_FALSE(tx.account_public_key == keys.public_key);
    auto result = verify_logic(tx);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "verify signature failed");
}

TEST_F(account_tests, test_migrate_account_with_account_key_fails)
{
    preload_account_notes();
    auto tx = create_account_tx(1);
    // The `tx.signature`, by default, gets signed by the original account private key.
    // So if we change the public key with which to verify this signature, it should fail.
    // (Note, even without this change the tx would fail, because the `tx` we got back attests to an account note which
    // doesn't exist in the tree).
    tx.signing_pub_key = user.signing_keys[0].public_key;

    auto result = verify_logic(tx);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "verify signature failed");
}

// Account membership

TEST_F(account_tests, test_alternative_signing_key_1)
{
    preload_account_notes();
    auto tx = create_account_tx(1);
    tx.migrate = false;
    tx.account_note_index = 0;
    tx.sign(user.signing_keys[0]);

    EXPECT_TRUE(verify_logic(tx).valid);
}

TEST_F(account_tests, test_alternative_signing_key_2)
{
    preload_account_notes();
    auto tx = create_account_tx(1);
    tx.migrate = false;
    tx.account_note_index = 1;
    tx.account_note_path = tree->get_hash_path(1);
    tx.sign(user.signing_keys[1]);

    EXPECT_TRUE(verify_logic(tx).valid);
}

TEST_F(account_tests, test_wrong_alias_hash_fails)
{
    preload_account_notes();
    auto tx = create_account_tx(1);
    // The circuit will calculate an 'old' account note with the wrong alias, so the membership check should fail.
    tx.alias_hash = rollup::fixtures::generate_alias_hash("penguin"); // it's actually "pebble"
    tx.sign(user.signing_keys[0]);

    auto result = verify_logic(tx);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "account check_membership failed");
}

// Account public key

TEST_F(account_tests, test_migrate_to_new_account_public_key)
{
    preload_account_notes();
    auto tx = create_account_tx(1);
    auto new_keys = rollup::fixtures::create_key_pair(nullptr);
    tx.new_account_public_key = new_keys.public_key;
    tx.sign(user.signing_keys[0]);

    EXPECT_TRUE(verify_logic(tx).valid);
}

TEST_F(account_tests, test_change_account_public_key_without_migrating_fails)
{
    preload_account_notes();
    auto tx = create_account_tx(1);
    auto new_keys = rollup::fixtures::create_key_pair(nullptr);
    tx.migrate = false;
    tx.new_account_public_key = new_keys.public_key;
    tx.sign(user.signing_keys[0]);

    auto result = verify_logic(tx);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "cannot change account keys unless migrating");
}

TEST_F(account_tests, test_migrate_account_full_proof)
{
    auto tx = create_account_tx();
    auto prover = new_account_prover(tx, false);
    auto proof = prover.construct_proof();
    auto data = inner_proof_data(proof.proof_data);

    auto new_account_alias_id = compute_account_alias_id(tx.alias_hash, tx.account_nonce + 1);
    auto note1_commitment =
        account_note{ new_account_alias_id, tx.account_public_key, tx.new_signing_pub_key_1 }.commit();
    auto note2_commitment =
        account_note{ new_account_alias_id, tx.account_public_key, tx.new_signing_pub_key_2 }.commit();

    EXPECT_EQ(data.proof_id, ProofIds::ACCOUNT);
    EXPECT_EQ(data.note_commitment1, note1_commitment);
    EXPECT_EQ(data.note_commitment2, note2_commitment);
    EXPECT_EQ(data.nullifier1, compute_account_alias_id_nullifier(tx.account_alias_id()));
    EXPECT_EQ(data.nullifier2, uint256_t(0));
    EXPECT_EQ(data.public_value, uint256_t(0));
    EXPECT_EQ(data.public_owner, fr(0));
    EXPECT_EQ(data.asset_id, uint256_t(0));
    EXPECT_EQ(data.merkle_root, tx.merkle_root);
    EXPECT_EQ(data.tx_fee, uint256_t(0));
    EXPECT_EQ(data.tx_fee_asset_id, uint256_t(0));
    EXPECT_EQ(data.bridge_id, uint256_t(0));
    EXPECT_EQ(data.defi_deposit_value, uint256_t(0));
    EXPECT_EQ(data.defi_root, fr(0));
    EXPECT_EQ(data.backward_link, fr(0));
    EXPECT_EQ(data.allow_chain, uint256_t(0));
}

TEST_F(account_tests, test_non_migrate_account_full_proof)
{
    preload_account_notes();
    auto tx = create_account_tx(1);
    tx.migrate = false;
    auto prover = new_account_prover(tx, false);
    auto proof = prover.construct_proof();
    auto data = inner_proof_data(proof.proof_data);

    auto note1_commitment =
        account_note{ tx.account_alias_id(), tx.account_public_key, tx.new_signing_pub_key_1 }.commit();
    auto note2_commitment =
        account_note{ tx.account_alias_id(), tx.account_public_key, tx.new_signing_pub_key_2 }.commit();

    EXPECT_EQ(data.proof_id, ProofIds::ACCOUNT);
    EXPECT_EQ(data.note_commitment1, note1_commitment);
    EXPECT_EQ(data.note_commitment2, note2_commitment);
    EXPECT_EQ(data.nullifier1, uint256_t(0));
    EXPECT_EQ(data.nullifier2, uint256_t(0));
    EXPECT_EQ(data.public_value, uint256_t(0));
    EXPECT_EQ(data.public_owner, fr(0));
    EXPECT_EQ(data.asset_id, uint256_t(0));
    EXPECT_EQ(data.merkle_root, tx.merkle_root);
    EXPECT_EQ(data.tx_fee, uint256_t(0));
    EXPECT_EQ(data.tx_fee_asset_id, uint256_t(0));
    EXPECT_EQ(data.bridge_id, uint256_t(0));
    EXPECT_EQ(data.defi_deposit_value, uint256_t(0));
    EXPECT_EQ(data.defi_root, fr(0));
    EXPECT_EQ(data.backward_link, fr(0));
    EXPECT_EQ(data.allow_chain, uint256_t(0));
}
