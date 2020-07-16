#include "../../pedersen_note/pedersen_note.hpp"
#include "../../tx/user_context.hpp"
#include "join_split.hpp"
#include "sign_notes.hpp"
#include <common/streams.hpp>
#include <common/test.hpp>
#include <crypto/schnorr/schnorr.hpp>
#include <stdlib/merkle_tree/memory_store.hpp>
#include <stdlib/merkle_tree/leveldb_tree.hpp>

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::merkle_tree;
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
        store = std::make_unique<MemoryStore>();
        tree = std::make_unique<MerkleTree<MemoryStore>>(*store, 32);
        user = rollup::tx::create_user_context();
    }

    void preload_two_notes()
    {
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
    std::unique_ptr<MemoryStore> store;
    std::unique_ptr<MerkleTree<MemoryStore>> tree;
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
    tx.input_owner = fr::random_element();
    tx.output_owner = fr::random_element();

    EXPECT_TRUE(sign_and_verify(tx));
}

HEAVY_TEST_F(client_proofs_join_split, test_noop)
{
    tx_note gibberish_note = { user.public_key, 0, fr::random_element() };
    auto gibberish_path =
        plonk::stdlib::merkle_tree::fr_hash_path(32, std::make_pair(fr::random_element(), fr::random_element()));

    join_split_tx tx;
    tx.owner_pub_key = user.public_key;
    tx.public_input = 0;
    tx.public_output = 0;
    tx.num_input_notes = 0;
    tx.input_index = { 0, 1 };
    tx.merkle_root = fr::random_element();
    tx.input_path = { gibberish_path, gibberish_path };
    tx.input_note = { gibberish_note, gibberish_note };
    tx.output_note = { gibberish_note, gibberish_note };

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
    tx.input_owner = fr::random_element();
    tx.output_owner = fr::random_element();

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
    tx.input_owner = fr::random_element();
    tx.output_owner = fr::random_element();

    EXPECT_TRUE(sign_and_verify(tx));
}

HEAVY_TEST_F(client_proofs_join_split, test_0_notes_with_balanced_public_values)
{
    tx_note input_note1 = { user.public_key, 0, user.note_secret };
    tx_note input_note2 = { user.public_key, 0, user.note_secret };
    tx_note output_note1 = { user.public_key, 0, user.note_secret };
    tx_note output_note2 = { user.public_key, 0, user.note_secret };

    join_split_tx tx;
    tx.owner_pub_key = user.public_key;
    tx.public_input = 100;
    tx.public_output = 100;
    tx.num_input_notes = 0;
    tx.input_index = { 1, 0 };
    tx.merkle_root = tree->root();
    tx.input_path = { tree->get_hash_path(1), tree->get_hash_path(0) };
    tx.input_note = { input_note2, input_note1 };
    tx.output_note = { output_note1, output_note2 };
    tx.input_owner = fr::random_element();
    tx.output_owner = fr::random_element();

    EXPECT_TRUE(sign_and_verify(tx));
}

HEAVY_TEST_F(client_proofs_join_split, test_0_input_notes_with_unbalanced_public_values)
{
    tx_note input_note1 = { user.public_key, 0, user.note_secret };
    tx_note input_note2 = { user.public_key, 0, user.note_secret };
    tx_note output_note1 = { user.public_key, 20, user.note_secret };
    tx_note output_note2 = { user.public_key, 0, user.note_secret };

    join_split_tx tx;
    tx.owner_pub_key = user.public_key;
    tx.public_input = 120;
    tx.public_output = 100;
    tx.num_input_notes = 0;
    tx.input_index = { 1, 0 };
    tx.merkle_root = tree->root();
    tx.input_path = { tree->get_hash_path(1), tree->get_hash_path(0) };
    tx.input_note = { input_note2, input_note1 };
    tx.output_note = { output_note1, output_note2 };
    tx.input_owner = fr::random_element();
    tx.output_owner = fr::random_element();

    EXPECT_TRUE(sign_and_verify(tx));
}

HEAVY_TEST_F(client_proofs_join_split, test_2_input_notes_with_unbalanced_public_values)
{
    preload_two_notes();

    tx_note input_note1 = { user.public_key, 100, user.note_secret };
    tx_note input_note2 = { user.public_key, 50, user.note_secret };
    tx_note output_note1 = { user.public_key, 80, user.note_secret };
    tx_note output_note2 = { user.public_key, 0, user.note_secret };

    join_split_tx tx;
    tx.owner_pub_key = user.public_key;
    tx.public_input = 100;
    tx.public_output = 170;
    tx.num_input_notes = 2;
    tx.input_index = { 1, 0 };
    tx.merkle_root = tree->root();
    tx.input_path = { tree->get_hash_path(1), tree->get_hash_path(0) };
    tx.input_note = { input_note2, input_note1 };
    tx.output_note = { output_note1, output_note2 };
    tx.input_owner = fr::random_element();
    tx.output_owner = fr::random_element();

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
    tx.input_owner = fr::random_element();
    tx.output_owner = fr::random_element();

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
    tx.input_owner = fr::random_element();
    tx.output_owner = fr::random_element();

    EXPECT_FALSE(sign_and_verify(tx));
}

HEAVY_TEST_F(client_proofs_join_split, test_0_notes_with_unbalanced_public_values_fails)
{
    tx_note input_note1 = { user.public_key, 0, user.note_secret };
    tx_note input_note2 = { user.public_key, 0, user.note_secret };
    tx_note output_note1 = { user.public_key, 0, user.note_secret };
    tx_note output_note2 = { user.public_key, 0, user.note_secret };

    join_split_tx tx;
    tx.owner_pub_key = user.public_key;
    tx.public_input = 120;
    tx.public_output = 100;
    tx.num_input_notes = 0;
    tx.input_index = { 1, 0 };
    tx.merkle_root = tree->root();
    tx.input_path = { tree->get_hash_path(1), tree->get_hash_path(0) };
    tx.input_note = { input_note2, input_note1 };
    tx.output_note = { output_note1, output_note2 };
    tx.input_owner = fr::random_element();
    tx.output_owner = fr::random_element();

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
    tx.input_owner = fr::random_element();
    tx.output_owner = fr::random_element();

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
    tx.input_owner = fr::random_element();
    tx.output_owner = fr::random_element();

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
    tx.input_owner = fr::random_element();
    tx.output_owner = fr::random_element();

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
    tx.input_owner = fr::random_element();
    tx.output_owner = fr::random_element();

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
    tx.input_index = { 0, 1 };
    tx.merkle_root = tree->root();
    tx.input_path = { tree->get_hash_path(0), tree->get_hash_path(1) };
    tx.input_note = { input_note1, input_note2 };
    tx.output_note = { output_note1, output_note2 };
    tx.signature = sign_notes({ tx.input_note[0], tx.input_note[1], tx.output_note[0], tx.output_note[1] },
                              { pk, grumpkin::g1::one * pk });
    tx.input_owner = fr::random_element();
    tx.output_owner = fr::random_element();

    auto prover = new_join_split_prover(tx);
    auto proof = prover.construct_proof();

    EXPECT_FALSE(verify_proof(proof));
}

HEAVY_TEST_F(client_proofs_join_split, test_tainted_output_owner_fails)
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
    tx.signature = sign_notes({ tx.input_note[0], tx.input_note[1], tx.output_note[0], tx.output_note[1] },
                              { user.private_key, user.public_key });
    tx.input_owner = fr::random_element();
    uint8_t output_owner[] = { 0x45, 0xaa, 0x42, 0xd4, 0x72, 0x88, 0x8e, 0xae, 0xa5, 0x56, 0x39,
                               0x46, 0xeb, 0x5c, 0xf5, 0x6c, 0x81, 0x6,  0x4d, 0x80, 0xc6, 0xf5,
                               0xa5, 0x38, 0xcc, 0x87, 0xae, 0x54, 0xae, 0xdb, 0x75, 0xd9 };
    tx.output_owner = barretenberg::fr::serialize_from_buffer(output_owner);

    auto prover = new_join_split_prover(tx);
    auto proof = prover.construct_proof();
    EXPECT_EQ(proof.proof_data[10 * 32 + 1], 0x45);
    proof.proof_data[10 * 32 + 1] = 0x55;

    EXPECT_FALSE(verify_proof(proof));
}
