#include "../../fixtures/user_context.hpp"
#include "account.hpp"
#include "../inner_proof_data.hpp"
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
using namespace rollup::proofs;
using namespace rollup::proofs::account;
using namespace rollup::proofs::notes::native::account;

class account_tests : public ::testing::Test {
  protected:
#ifndef DISABLE_HEAVY_TESTS
    static void SetUpTestCase()
    {
        auto crs_factory =
            std::shared_ptr<waffle::ReferenceStringFactory>(new waffle::FileReferenceStringFactory("../srs_db"));
        init_verification_key(crs_factory);
    }
#endif

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

    uint256_t compute_account_alias_id_nullifier(fr const& account_alias_id, fr const& gibberish, bool migrate_account)
    {
        const std::vector<fr> hash_elements{
            fr(1),
            account_alias_id,
            gibberish * !migrate_account,
        };
        auto result =
            crypto::pedersen::compress_native(hash_elements, notes::GeneratorIndex::ACCOUNT_ALIAS_ID_NULLIFIER);
        return uint256_t(result);
    }

    uint256_t compute_gibberish_nullifier(fr const& gibberish)
    {
        const std::vector<fr> hash_elements{
            fr(1),
            gibberish,
        };
        auto result =
            crypto::pedersen::compress_native(hash_elements, notes::GeneratorIndex::ACCOUNT_GIBBERISH_NULLIFIER);
        return uint256_t(result);
    }

    account_tx create_account_tx(uint32_t nonce = 0)
    {
        account_tx tx;
        tx.merkle_root = tree->root();
        tx.account_public_key = user.owner.public_key;
        tx.new_account_public_key = user.owner.public_key;
        tx.num_new_keys = 2;
        tx.new_signing_pub_key_1 = user.signing_keys[0].public_key;
        tx.new_signing_pub_key_2 = user.signing_keys[1].public_key;
        tx.alias_hash = user.alias_hash;
        tx.nonce = nonce;
        tx.migrate = true;
        tx.gibberish = fr::random_element();
        tx.account_index = 0;
        tx.signing_pub_key = user.signing_keys[0].public_key;
        tx.account_path = tree->get_hash_path(0);
        tx.sign(user.owner);

        return tx;
    }

    bool verify(account_tx& tx)
    {
        auto prover = new_account_prover(tx);
        auto proof = prover.construct_proof();
        return verify_proof(proof);
    }

    bool verify_logic(account_tx& tx)
    {
        Composer composer(get_proving_key(), nullptr);
        account_circuit(composer, tx);
        if (composer.failed) {
            std::cout << "Logic failed: " << composer.err << std::endl;
        }
        return !composer.failed;
    }

    rollup::fixtures::user_context user;
    std::unique_ptr<MemoryStore> store;
    std::unique_ptr<MerkleTree<MemoryStore>> tree;
};

TEST_F(account_tests, test_create_account)
{
    auto tx = create_account_tx();
    EXPECT_TRUE(verify_logic(tx));
}

HEAVY_TEST_F(account_tests, test_create_account_full_proof)
{
    auto tx = create_account_tx();
    EXPECT_TRUE(verify(tx));
}

TEST_F(account_tests, test_migrate_account)
{
    preload_account_notes();
    auto tx = create_account_tx(1);
    tx.account_index = 0;
    tx.sign(user.signing_keys[0]);

    EXPECT_TRUE(verify_logic(tx));
}

// Signature

TEST_F(account_tests, test_wrong_account_key_pair_fails)
{
    auto tx = create_account_tx();
    auto keys = rollup::fixtures::create_key_pair(nullptr);
    tx.sign(keys);

    EXPECT_FALSE(tx.account_public_key == keys.public_key);
    EXPECT_FALSE(verify_logic(tx));
}

TEST_F(account_tests, test_migrate_account_with_account_key_fails)
{
    preload_account_notes();
    auto tx = create_account_tx(1);
    tx.signing_pub_key = user.signing_keys[0].public_key;

    EXPECT_FALSE(verify_logic(tx));
}

// Account membership

TEST_F(account_tests, test_alternative_signing_key_1)
{
    preload_account_notes();
    auto tx = create_account_tx(1);
    tx.migrate = false;
    tx.account_index = 0;
    tx.sign(user.signing_keys[0]);

    EXPECT_TRUE(verify_logic(tx));
}

TEST_F(account_tests, test_alternative_signing_key_2)
{
    preload_account_notes();
    auto tx = create_account_tx(1);
    tx.migrate = false;
    tx.account_index = 1;
    tx.account_path = tree->get_hash_path(1);
    tx.sign(user.signing_keys[1]);

    EXPECT_TRUE(verify_logic(tx));
}

TEST_F(account_tests, test_wrong_alias_hash_fails)
{
    preload_account_notes();
    auto tx = create_account_tx(1);
    tx.alias_hash = rollup::fixtures::generate_alias_hash("penguin");
    tx.sign(user.signing_keys[0]);

    EXPECT_FALSE(verify_logic(tx));
}

// Account public key

TEST_F(account_tests, test_migrate_to_new_account_public_key)
{
    preload_account_notes();
    auto tx = create_account_tx(1);
    auto new_keys = rollup::fixtures::create_key_pair(nullptr);
    tx.new_account_public_key = new_keys.public_key;
    tx.account_index = 0;
    tx.sign(user.signing_keys[0]);

    EXPECT_TRUE(verify_logic(tx));
}

TEST_F(account_tests, test_change_account_public_key_fails)
{
    preload_account_notes();
    auto tx = create_account_tx(1);
    auto new_keys = rollup::fixtures::create_key_pair(nullptr);
    tx.migrate = false;
    tx.new_account_public_key = new_keys.public_key;
    tx.account_index = 0;
    tx.sign(user.signing_keys[0]);

    EXPECT_FALSE(verify_logic(tx));
}

// Nullifier

HEAVY_TEST_F(account_tests, test_correct_account_alias_id_nullifier)
{
    auto tx = create_account_tx();
    auto prover = new_account_prover(tx);
    auto proof = prover.construct_proof();
    auto data = inner_proof_data(proof.proof_data);

    EXPECT_EQ(data.nullifier1, compute_account_alias_id_nullifier(tx.account_alias_id(), tx.gibberish, true));
    EXPECT_EQ(data.nullifier2, compute_gibberish_nullifier(tx.gibberish));
}

HEAVY_TEST_F(account_tests, test_gibberish_account_alias_id_nullifier)
{
    preload_account_notes();
    auto tx = create_account_tx(1);
    tx.migrate = false;
    auto prover = new_account_prover(tx);
    auto proof = prover.construct_proof();
    auto data = inner_proof_data(proof.proof_data);

    EXPECT_EQ(data.nullifier1, compute_account_alias_id_nullifier(tx.account_alias_id(), tx.gibberish, false));
    EXPECT_EQ(data.nullifier2, compute_gibberish_nullifier(tx.gibberish));
}
