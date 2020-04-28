#include "../../pedersen_note/pedersen_note.hpp"
#include "../../tx/user_context.hpp"
#include "join_split.hpp"
#include "sign_notes.hpp"
#include <common/streams.hpp>
#include <crypto/schnorr/schnorr.hpp>
#include <stdlib/merkle_tree/leveldb_store.hpp>
#include <stdlib/merkle_tree/leveldb_tree.hpp>
#include <common/test.hpp>

using namespace barretenberg;
using namespace plonk::stdlib;
using namespace plonk::stdlib::types::turbo;
using namespace rollup::client_proofs::join_split;

std::vector<uint8_t> create_leaf_data(grumpkin::g1::affine_element const& enc_note)
{
    std::vector<uint8_t> buf;
    write(buf, enc_note.x);
    write(buf, enc_note.y);
    return buf;
}

class client_proofs_join_split : public ::testing::Test {
  protected:
    static void SetUpTestCase()
    {
        auto null_crs_factory = std::make_unique<waffle::ReferenceStringFactory>();
        init_proving_key(std::move(null_crs_factory));
        auto crs_factory = std::make_unique<waffle::FileReferenceStringFactory>("../srs_db");
        init_verification_key(std::move(crs_factory));
    }

    virtual void SetUp()
    {
        merkle_tree::LevelDbStore::destroy("/tmp/client_proofs_join_split_db");
        store = std::make_unique<merkle_tree::LevelDbStore>("/tmp/client_proofs_join_split_db");
        tree = std::make_unique<merkle_tree::LevelDbTree>(*store, 32);
        user = rollup::tx::create_user_context();
    }

    void preload_two_notes() {
        tx_note note1 = { user.public_key, 100, user.note_secret };
        tx_note note2 = { user.public_key, 50, user.note_secret };

        auto enc_note1 = encrypt_note(note1);
        tree->update_element(0, create_leaf_data(enc_note1));

        auto enc_note2 = encrypt_note(note2);
        tree->update_element(1, create_leaf_data(enc_note2));
    }

    bool sign_and_verify(join_split_tx& tx)
    {
        tx.signature = sign_notes({ tx.input_note[0], tx.input_note[1], tx.output_note[0], tx.output_note[1] },
                                  { user.private_key, user.public_key });

        auto prover = new_join_split_prover(tx);
        auto proof = prover.construct_proof();
        return verify_proof(proof);
    }

    rollup::tx::user_context user;
    std::unique_ptr<merkle_tree::LevelDbStore> store;
    std::unique_ptr<merkle_tree::LevelDbTree> tree;
};

HEAVY_TEST_F(client_proofs_join_split, test_0_input_notes)
{
    tx_note gibberish = { user.public_key, 0, fr::random_element() };
    tx_note output_note1 = { user.public_key, 100, user.note_secret };
    tx_note output_note2 = { user.public_key, 0, user.note_secret };

    join_split_tx tx;
    tx.owner_pub_key = user.public_key;
    tx.public_input = 100;
    tx.public_output = 0;
    tx.num_input_notes = 0;
    tx.input_index = { 0, 1 };
    tx.merkle_root = tree->root();
    // We can't have zero field elements in our hash paths or it breaks. Why?
    tx.input_path = { tree->get_hash_path(0), tree->get_hash_path(1) };
    tx.input_note = { gibberish, gibberish };
    tx.output_note = { output_note1, output_note2 };

    EXPECT_TRUE(sign_and_verify(tx));
}

HEAVY_TEST_F(client_proofs_join_split, test_2_input_notes)
{
    preload_two_notes();

    tx_note input_note1 = { user.public_key, 100, user.note_secret };
    tx_note input_note2 = { user.public_key, 50, user.note_secret };
    tx_note output_note1 = { user.public_key, 70, user.note_secret };
    tx_note output_note2 = { user.public_key, 80, user.note_secret };

    join_split_tx tx;
    tx.owner_pub_key = user.public_key;
    tx.public_input = 0;
    tx.public_output = 0;
    tx.num_input_notes = 2;
    tx.input_index = { 1, 0 };
    tx.merkle_root = tree->root();
    tx.input_path = { tree->get_hash_path(1), tree->get_hash_path(0) };
    tx.input_note = { input_note2, input_note1 };
    tx.output_note = { output_note1, output_note2 };

    EXPECT_TRUE(sign_and_verify(tx));
}

HEAVY_TEST_F(client_proofs_join_split, test_0_output_notes)
{
    preload_two_notes();

    tx_note input_note1 = { user.public_key, 100, user.note_secret };
    tx_note input_note2 = { user.public_key, 50, user.note_secret };
    tx_note output_note1 = { user.public_key, 0, user.note_secret };
    tx_note output_note2 = { user.public_key, 0, user.note_secret };

    join_split_tx tx;
    tx.owner_pub_key = user.public_key;
    tx.public_input = 0;
    tx.public_output = 150;
    tx.num_input_notes = 2;
    tx.input_index = { 0, 1 };
    tx.merkle_root = tree->root();
    tx.input_path = { tree->get_hash_path(0), tree->get_hash_path(1) };
    tx.input_note = { input_note1, input_note2 };
    tx.output_note = { output_note1, output_note2 };

    EXPECT_TRUE(sign_and_verify(tx));
}

HEAVY_TEST_F(client_proofs_join_split, test_joining_same_note_fails)
{
    preload_two_notes();

    tx_note input_note1 = { user.public_key, 100, user.note_secret };
    tx_note input_note2 = { user.public_key, 100, user.note_secret };
    tx_note output_note1 = { user.public_key, 200, user.note_secret };
    tx_note output_note2 = { user.public_key, 0, user.note_secret };

    join_split_tx tx;
    tx.owner_pub_key = user.public_key;
    tx.public_input = 0;
    tx.public_output = 0;
    tx.num_input_notes = 2;
    tx.input_index = { 0, 0 };
    tx.merkle_root = tree->root();
    tx.input_path = { tree->get_hash_path(0), tree->get_hash_path(0) };
    tx.input_note = { input_note1, input_note2 };
    tx.output_note = { output_note1, output_note2 };

    EXPECT_FALSE(sign_and_verify(tx));
}

HEAVY_TEST_F(client_proofs_join_split, test_unbalanced_notes_fails)
{
    preload_two_notes();

    tx_note input_note1 = { user.public_key, 100, user.note_secret };
    tx_note input_note2 = { user.public_key, 51, user.note_secret };
    tx_note output_note1 = { user.public_key, 70, user.note_secret };
    tx_note output_note2 = { user.public_key, 80, user.note_secret };

    join_split_tx tx;
    tx.owner_pub_key = user.public_key;
    tx.public_input = 0;
    tx.public_output = 0;
    tx.num_input_notes = 2;
    tx.input_index = { 0, 1 };
    tx.merkle_root = tree->root();
    tx.input_path = { tree->get_hash_path(0), tree->get_hash_path(1) };
    tx.input_note = { input_note1, input_note2 };
    tx.output_note = { output_note1, output_note2 };

    EXPECT_FALSE(sign_and_verify(tx));
}

HEAVY_TEST_F(client_proofs_join_split, test_wrong_input_note_owner_fails)
{
    preload_two_notes();

    tx_note input_note1 = { user.public_key, 100, user.note_secret };
    tx_note input_note2 = { grumpkin::g1::element::random_element(), 50, user.note_secret };
    tx_note output_note1 = { user.public_key, 70, user.note_secret };
    tx_note output_note2 = { user.public_key, 80, user.note_secret };

    join_split_tx tx;
    tx.owner_pub_key = user.public_key;
    tx.public_input = 0;
    tx.public_output = 0;
    tx.num_input_notes = 2;
    tx.input_index = { 0, 1 };
    tx.merkle_root = tree->root();
    tx.input_path = { tree->get_hash_path(0), tree->get_hash_path(1) };
    tx.input_note = { input_note1, input_note2 };
    tx.output_note = { output_note1, output_note2 };

    EXPECT_FALSE(sign_and_verify(tx));
}

HEAVY_TEST_F(client_proofs_join_split, test_random_output_owners_succeeds)
{
    preload_two_notes();

    tx_note input_note1 = { user.public_key, 100, user.note_secret };
    tx_note input_note2 = { user.public_key, 50, user.note_secret };
    tx_note output_note1 = { grumpkin::g1::element::random_element(), 70, user.note_secret };
    tx_note output_note2 = { grumpkin::g1::element::random_element(), 80, user.note_secret };

    join_split_tx tx;
    tx.owner_pub_key = user.public_key;
    tx.public_input = 0;
    tx.public_output = 0;
    tx.num_input_notes = 2;
    tx.input_index = { 0, 1 };
    tx.merkle_root = tree->root();
    tx.input_path = { tree->get_hash_path(0), tree->get_hash_path(1) };
    tx.input_note = { input_note1, input_note2 };
    tx.output_note = { output_note1, output_note2 };

    EXPECT_TRUE(sign_and_verify(tx));
}

HEAVY_TEST_F(client_proofs_join_split, test_wrong_hash_path_fails)
{
    preload_two_notes();

    tx_note input_note1 = { user.public_key, 100, user.note_secret };
    tx_note input_note2 = { user.public_key, 50, user.note_secret };
    tx_note output_note1 = { user.public_key, 70, user.note_secret };
    tx_note output_note2 = { user.public_key, 80, user.note_secret };

    join_split_tx tx;
    tx.owner_pub_key = user.public_key;
    tx.public_input = 0;
    tx.public_output = 0;
    tx.num_input_notes = 2;
    tx.input_index = { 0, 2 };
    tx.merkle_root = tree->root();
    tx.input_path = { tree->get_hash_path(0), tree->get_hash_path(1) };
    tx.input_note = { input_note1, input_note2 };
    tx.output_note = { output_note1, output_note2 };

    EXPECT_FALSE(sign_and_verify(tx));
}

HEAVY_TEST_F(client_proofs_join_split, test_wrong_merkle_root_fails)
{
    preload_two_notes();

    tx_note input_note1 = { user.public_key, 100, user.note_secret };
    tx_note input_note2 = { user.public_key, 50, user.note_secret };
    tx_note output_note1 = { user.public_key, 70, user.note_secret };
    tx_note output_note2 = { user.public_key, 80, user.note_secret };

    join_split_tx tx;
    tx.owner_pub_key = user.public_key;
    tx.public_input = 0;
    tx.public_output = 0;
    tx.num_input_notes = 2;
    tx.input_index = { 0, 1 };
    tx.merkle_root = fr::random_element();
    tx.input_path = { tree->get_hash_path(0), tree->get_hash_path(1) };
    tx.input_note = { input_note1, input_note2 };
    tx.output_note = { output_note1, output_note2 };

    EXPECT_FALSE(sign_and_verify(tx));
}

HEAVY_TEST_F(client_proofs_join_split, test_wrong_signature_fails)
{
    preload_two_notes();

    tx_note input_note1 = { user.public_key, 100, user.note_secret };
    tx_note input_note2 = { user.public_key, 50, user.note_secret };
    tx_note output_note1 = { user.public_key, 70, user.note_secret };
    tx_note output_note2 = { user.public_key, 80, user.note_secret };

    // Going to sign with this incorrect key.
    auto pk = grumpkin::fr::random_element();

    join_split_tx tx;
    tx.owner_pub_key = user.public_key;
    tx.public_input = 0;
    tx.public_output = 0;
    tx.num_input_notes = 2;
    tx.input_index = { 0, 2 };
    tx.merkle_root = tree->root();
    tx.input_path = { tree->get_hash_path(0), tree->get_hash_path(1) };
    tx.input_note = { input_note1, input_note2 };
    tx.output_note = { output_note1, output_note2 };
    tx.signature = sign_notes({ tx.input_note[0], tx.input_note[1], tx.output_note[0], tx.output_note[1] },
                                { pk, grumpkin::g1::one * pk });

    auto prover = new_join_split_prover(tx);
    auto proof = prover.construct_proof();

    EXPECT_FALSE(verify_proof(proof));
}