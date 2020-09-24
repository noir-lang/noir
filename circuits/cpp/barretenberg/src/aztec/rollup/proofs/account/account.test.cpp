#include "../notes/pedersen_note.hpp"
#include "../../fixtures/user_context.hpp"
#include "account.hpp"
#include "../inner_proof_data.hpp"
#include "../notes/note_types.hpp"
#include <common/streams.hpp>
#include <common/test.hpp>
#include <crypto/schnorr/schnorr.hpp>
#include <crypto/blake2s/blake2s.hpp>
#include <stdlib/merkle_tree/memory_store.hpp>
#include <stdlib/merkle_tree/leveldb_tree.hpp>
#include <plonk/proof_system/proving_key/serialize.hpp>

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::merkle_tree;
using namespace rollup::proofs;
using namespace rollup::proofs::account;

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

    std::vector<uint8_t> create_account_leaf_data(grumpkin::g1::affine_element const& owner_key,
                                                  grumpkin::g1::affine_element const& signing_key)
    {
        std::vector<uint8_t> buf;
        write(buf, owner_key.x);
        write(buf, signing_key.x);
        return buf;
    }

    /**
     * Add two account notes for the user.
     */
    void preload_account_notes()
    {
        tree->update_element(tree->size(),
                             create_account_leaf_data(user.owner.public_key, user.signing_keys[0].public_key));
        tree->update_element(tree->size(),
                             create_account_leaf_data(user.owner.public_key, user.signing_keys[1].public_key));
    }

    uint256_t compute_account_nullifier(grumpkin::g1::affine_element const& owner_key,
                                        grumpkin::g1::affine_element const& signing_key)
    {
        auto result = from_buffer<fr>(blake2::blake2s(create_account_leaf_data(owner_key, signing_key)));
        return uint256_t(result);
    }

    uint256_t compute_alias_nullifier(fr const& alias, bool register_alias)
    {
        std::vector<uint8_t> buf;
        write(buf, (uint8_t)(register_alias ? notes::ALIAS : notes::GIBBERISH));
        write(buf, alias);
        auto result = from_buffer<fr>(blake2::blake2s(buf));
        return uint256_t(result);
    }

    account_tx create_account_tx()
    {
        account_tx tx;
        tx.merkle_root = tree->root();
        tx.owner_pub_key = user.owner.public_key;
        tx.num_new_keys = 2;
        tx.new_signing_pub_key_1 = user.signing_keys[0].public_key;
        tx.new_signing_pub_key_2 = user.signing_keys[1].public_key;
        tx.register_alias = true;
        const std::string alias = "my_alias";
        tx.alias = from_buffer<fr>(blake2::blake2s({ alias.begin(), alias.end() }));
        tx.nullify_key = true;
        tx.nullified_key = user.owner.public_key;
        tx.account_index = 0;
        tx.signing_pub_key = user.owner.public_key;
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

TEST_F(account_tests, test_missing_signing_key_fails)
{
    auto tx = create_account_tx();
    tx.signing_pub_key = user.signing_keys[0].public_key;
    EXPECT_FALSE(verify_logic(tx));
}

TEST_F(account_tests, test_alternative_signing_key_1)
{
    preload_account_notes();
    auto tx = create_account_tx();
    tx.account_index = 0;
    tx.sign(user.signing_keys[0]);
    EXPECT_TRUE(verify_logic(tx));
}

TEST_F(account_tests, test_alternative_signing_key_2)
{
    preload_account_notes();
    auto tx = create_account_tx();
    tx.account_index = 1;
    tx.sign(user.signing_keys[1]);
    EXPECT_TRUE(verify_logic(tx));
}

HEAVY_TEST_F(account_tests, test_correct_alias_nullifier)
{
    auto tx = create_account_tx();
    auto prover = new_account_prover(tx);
    auto proof = prover.construct_proof();
    auto data = inner_proof_data(proof.proof_data);
    EXPECT_EQ(data.nullifier1, compute_alias_nullifier(tx.alias, true));
}

HEAVY_TEST_F(account_tests, test_correct_account_nullifier)
{
    auto tx = create_account_tx();
    auto prover = new_account_prover(tx);
    auto proof = prover.construct_proof();
    auto data = inner_proof_data(proof.proof_data);
    EXPECT_EQ(data.nullifier2, compute_account_nullifier(user.owner.public_key, user.owner.public_key));
}
