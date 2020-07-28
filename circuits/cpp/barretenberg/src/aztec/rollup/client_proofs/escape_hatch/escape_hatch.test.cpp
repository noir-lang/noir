#include "../../pedersen_note/pedersen_note.hpp"
#include "../../fixtures/user_context.hpp"
#include "escape_hatch.hpp"
#include "escape_hatch_tx.hpp"
#include "sign_notes.hpp"
#include <common/streams.hpp>
#include <common/test.hpp>
#include <crypto/schnorr/schnorr.hpp>
#include <stdlib/merkle_tree/memory_store.hpp>
#include <stdlib/merkle_tree/leveldb_tree.hpp>
#include <plonk/proof_system/proving_key/serialize.hpp>

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::merkle_tree;
using namespace rollup::client_proofs::escape_hatch;

class client_proofs_escape_hatch : public ::testing::Test {
  protected:
    static void SetUpTestCase()
    {
        auto null_crs_factory = std::make_unique<waffle::ReferenceStringFactory>();
        init_proving_key(std::move(null_crs_factory));
        auto crs_factory = std::make_unique<waffle::FileReferenceStringFactory>("../srs_db/ignition");
        init_verification_key(std::move(crs_factory));
    }

    virtual void SetUp()
    {
        store = std::make_unique<MemoryStore>();
        tree = std::make_unique<MerkleTree<MemoryStore>>(*store, 32, 0);
        nullifier_tree = std::make_unique<MerkleTree<MemoryStore>>(*store, 128, 1);
        user = rollup::fixtures::create_user_context();
    }

    void preload_value_notes()
    {
        tx_note note1 = { user.owner.public_key, 100, user.note_secret };
        tx_note note2 = { user.owner.public_key, 50, user.note_secret };

        auto enc_note1 = encrypt_note(note1);
        tree->update_element(tree->size(), create_leaf_data(enc_note1));

        auto enc_note2 = encrypt_note(note2);
        tree->update_element(tree->size(), create_leaf_data(enc_note2));
    }

    void preload_account_notes()
    {
        tree->update_element(tree->size(),
                             create_account_leaf_data(user.owner.public_key, user.signing_keys[0].public_key));
        tree->update_element(tree->size(),
                             create_account_leaf_data(user.owner.public_key, user.signing_keys[1].public_key));
    }

    bool sign_and_verify(escape_hatch_tx& tx, grumpkin::fr const& signing_private_key)
    {
        tx.signature = sign_notes({ tx.input_note[0], tx.input_note[1] }, { signing_private_key, tx.signing_pub_key });
        auto prover = new_escape_hatch_prover(tx);
        auto proof = prover.construct_proof();
        return verify_proof(proof);
    }

    std::vector<uint8_t> create_leaf_data(grumpkin::g1::affine_element const& enc_note)
    {
        std::vector<uint8_t> buf;
        write(buf, enc_note.x);
        write(buf, enc_note.y);
        return buf;
    }

    std::vector<uint8_t> create_account_leaf_data(grumpkin::g1::affine_element const& owner_key,
                                                  grumpkin::g1::affine_element const& signing_key)
    {
        std::vector<uint8_t> buf;
        write(buf, owner_key.x);
        write(buf, signing_key.x);
        return buf;
    }

    uint128_t create_nullifier(tx_note note, uint32_t index)
    {
        grumpkin::g1::affine_element enc_note = encrypt_note(note);
        std::vector<uint8_t> buf;
        write(buf, enc_note.x);
        write(buf, index);
        auto vk_buf = to_buffer(note.secret);

        std::array<uint8_t, 28> vk_slice;
        std::copy(vk_buf.begin() + 4, vk_buf.end(), vk_slice.begin());
        write(buf, vk_slice);
        buf[63] |= 1;

        auto result = from_buffer<fr>(blake2::blake2s(buf));
        auto nullifier = uint128_t(result);
        return nullifier;
    }

    escape_hatch_tx simple_setup()
    {
        preload_account_notes();
        preload_value_notes();
        return create_escape_hatch_tx();
    }

    escape_hatch_tx setup_1_input_note()
    {
        preload_account_notes();
        preload_value_notes();
        return create_1_input_note_tx();
    }

    escape_hatch_tx create_1_input_note_tx()
    {
        tx_note input_note1 = { user.owner.public_key, 100, user.note_secret };
        tx_note input_note2 = { user.owner.public_key, 0, user.note_secret };

        escape_hatch_tx tx;
        tx.public_output = 100;
        tx.num_input_notes = 1;
        tx.input_index = { 2, 3 };
        tx.old_data_root = tree->root();
        tx.input_path = { tree->get_hash_path(tx.input_index[0]), tree->get_hash_path(tx.input_index[1]) };

        uint128_t nullifier1 = create_nullifier(input_note1, uint32_t(tx.input_index[0]));
        uint128_t nullifier2 = create_nullifier(input_note2, uint32_t(tx.input_index[1]));

        auto nullifier_value = std::vector<uint8_t>(64, 0);
        nullifier_value[63] = 1;

        tx.current_nullifier_paths[0] = nullifier_tree->get_hash_path(nullifier1);

        tx.old_nullifier_merkle_root = nullifier_tree->root();

        nullifier_tree->update_element(nullifier1, nullifier_value);
        tx.new_null_roots[0] = nullifier_tree->root();
        tx.new_nullifier_paths[0] = nullifier_tree->get_hash_path(nullifier1);
        tx.current_nullifier_paths[1] = nullifier_tree->get_hash_path(nullifier2);

        nullifier_tree->update_element(nullifier2, nullifier_value);
        tx.new_null_roots[1] = nullifier_tree->root();
        tx.new_nullifier_paths[1] = nullifier_tree->get_hash_path(nullifier2);

        tx.input_note = { input_note1, input_note2 };
        tx.public_owner = fr::random_element();
        tx.account_index = 0;
        tx.account_path = tree->get_hash_path(0);
        tx.signing_pub_key = user.signing_keys[0].public_key;

        uint128_t account_nullifier = create_account_nullifier(user.owner.public_key, user.signing_keys[0].public_key);
        tx.account_nullifier_path = nullifier_tree->get_hash_path(account_nullifier);
        return tx;
    }

    uint128_t create_account_nullifier(grumpkin::g1::affine_element const& owner_key,
                                       grumpkin::g1::affine_element const& signing_key)
    {
        auto data = create_account_leaf_data(owner_key, signing_key);
        auto nullifier = merkle_tree::hash_value_native(data);
        return uint128_t(nullifier);
    }

    escape_hatch_tx create_escape_hatch_tx()
    {
        tx_note input_note1 = { user.owner.public_key, 100, user.note_secret };
        tx_note input_note2 = { user.owner.public_key, 50, user.note_secret };

        escape_hatch_tx tx;
        tx.public_output = 150;
        tx.num_input_notes = 2;
        tx.input_index = { 2, 3 };
        tx.old_data_root = tree->root(); // old_data_root
        tx.input_path = { tree->get_hash_path(tx.input_index[0]), tree->get_hash_path(tx.input_index[1]) };

        uint128_t nullifier1 = create_nullifier(input_note1, uint32_t(tx.input_index[0]));
        uint128_t nullifier2 = create_nullifier(input_note2, uint32_t(tx.input_index[1]));

        auto nullifier_value = std::vector<uint8_t>(64, 0);
        nullifier_value[63] = 1;

        uint128_t account_nullifier = create_account_nullifier(user.owner.public_key, user.signing_keys[0].public_key);
        tx.account_nullifier_path = nullifier_tree->get_hash_path(account_nullifier);

        tx.current_nullifier_paths[0] = nullifier_tree->get_hash_path(nullifier1);
        tx.old_nullifier_merkle_root = nullifier_tree->root();

        nullifier_tree->update_element(nullifier1, nullifier_value);
        tx.new_null_roots[0] = nullifier_tree->root();
        tx.new_nullifier_paths[0] = nullifier_tree->get_hash_path(nullifier1);
        tx.current_nullifier_paths[1] = nullifier_tree->get_hash_path(nullifier2);

        nullifier_tree->update_element(nullifier2, nullifier_value);
        tx.new_null_roots[1] = nullifier_tree->root(); // new nullifier root
        tx.new_nullifier_paths[1] = nullifier_tree->get_hash_path(nullifier2);

        tx.input_note = { input_note1, input_note2 };
        tx.public_owner = fr::random_element();
        tx.account_index = 0;
        tx.account_path = tree->get_hash_path(0);
        tx.signing_pub_key = user.signing_keys[0].public_key;

        // todo: where to get these roots from?
        tx.new_data_root = fr::random_element();
        tx.old_data_roots_root = fr::random_element();
        tx.new_data_roots_root = fr::random_element();
        return tx;
    }

    rollup::fixtures::user_context user;
    std::unique_ptr<MemoryStore> store;
    std::unique_ptr<MerkleTree<MemoryStore>> tree;
    std::unique_ptr<MerkleTree<MemoryStore>> nullifier_tree;
};

HEAVY_TEST_F(client_proofs_escape_hatch, test_2_input_notes)
{
    escape_hatch_tx tx = simple_setup();
    auto buf = to_buffer(tx);
    EXPECT_TRUE(sign_and_verify(tx, user.signing_keys[0].private_key));
}

HEAVY_TEST_F(client_proofs_escape_hatch, test_1_input_notes)
{
    escape_hatch_tx tx_1_input_note = setup_1_input_note();
    auto buf = to_buffer(tx_1_input_note);
    EXPECT_TRUE(sign_and_verify(tx_1_input_note, user.signing_keys[0].private_key));
}

HEAVY_TEST_F(client_proofs_escape_hatch, test_1_false_new_nullifier_path_fails)
{
    escape_hatch_tx tx = simple_setup();
    tx.new_nullifier_paths[1] = nullifier_tree->get_hash_path(3);
    EXPECT_FALSE((sign_and_verify(tx, user.signing_keys[0].private_key)));
}

HEAVY_TEST_F(client_proofs_escape_hatch, test_switched_nullifier_paths_order_fails)
{
    escape_hatch_tx tx = simple_setup();
    merkle_tree::fr_hash_path null_path_copy = tx.new_nullifier_paths[1];

    tx.new_nullifier_paths[1] = tx.new_nullifier_paths[0];
    tx.new_nullifier_paths[0] = null_path_copy;
    EXPECT_FALSE((sign_and_verify(tx, user.signing_keys[0].private_key)));
}

HEAVY_TEST_F(client_proofs_escape_hatch, test_1_false_old_nullifier_path_fails)
{
    escape_hatch_tx tx = simple_setup();
    tx.current_nullifier_paths[1] = nullifier_tree->get_hash_path(3);
    EXPECT_FALSE((sign_and_verify(tx, user.signing_keys[0].private_key)));
}

HEAVY_TEST_F(client_proofs_escape_hatch, test_incorrect_new_null_root_fails)
{
    escape_hatch_tx tx = simple_setup();
    tx.new_null_roots[1] = fr::random_element();
    EXPECT_FALSE((sign_and_verify(tx, user.signing_keys[0].private_key)));
}

HEAVY_TEST_F(client_proofs_escape_hatch, switched_around_new_null_root_fails)
{
    escape_hatch_tx tx = simple_setup();
    barretenberg::fr null_root_copy = tx.new_null_roots[1];

    tx.new_null_roots[1] = tx.new_null_roots[0];
    tx.new_null_roots[0] = null_root_copy;

    EXPECT_FALSE((sign_and_verify(tx, user.signing_keys[0].private_key)));
}

HEAVY_TEST_F(client_proofs_escape_hatch, wrong_null_merkle_root_fails)
{
    escape_hatch_tx tx = simple_setup();
    tx.old_nullifier_merkle_root = fr::random_element();
    EXPECT_FALSE((sign_and_verify(tx, user.signing_keys[0].private_key)));
}

HEAVY_TEST_F(client_proofs_escape_hatch, switch_current_new_nullifier_paths_fails)
{
    escape_hatch_tx tx = simple_setup();
    std::array<merkle_tree::fr_hash_path, 2> new_paths_copy = tx.new_nullifier_paths;

    tx.new_nullifier_paths = tx.current_nullifier_paths;
    tx.current_nullifier_paths = new_paths_copy;

    EXPECT_FALSE((sign_and_verify(tx, user.signing_keys[0].private_key)));
}

HEAVY_TEST_F(client_proofs_escape_hatch, test_joining_same_note_fails)
{
    escape_hatch_tx tx = simple_setup();
    tx.input_note[0].value = 75;
    tx.input_note[1].value = 75;
    tx.input_index = { 1, 1 };

    EXPECT_FALSE(sign_and_verify(tx, user.signing_keys[0].private_key));
}

HEAVY_TEST_F(client_proofs_escape_hatch, test_unbalanced_notes_fails)
{
    escape_hatch_tx tx = simple_setup();
    tx.input_note[0].value = 99;
    tx.input_note[1].value = 50;
    EXPECT_FALSE(sign_and_verify(tx, user.signing_keys[0].private_key));
}

HEAVY_TEST_F(client_proofs_escape_hatch, test_wrong_input_note_owner_fails)
{
    escape_hatch_tx tx = simple_setup();
    tx.input_note[1].owner = grumpkin::g1::element::random_element();
    EXPECT_FALSE(sign_and_verify(tx, user.signing_keys[0].private_key));
}

HEAVY_TEST_F(client_proofs_escape_hatch, test_wrong_hash_path_fails)
{
    escape_hatch_tx tx = simple_setup();
    tx.input_path[1] = tree->get_hash_path(0);

    EXPECT_FALSE(sign_and_verify(tx, user.signing_keys[0].private_key));
}

HEAVY_TEST_F(client_proofs_escape_hatch, test_wrong_merkle_root_fails)
{
    escape_hatch_tx tx = simple_setup();
    tx.old_data_root = fr::random_element();

    EXPECT_FALSE(sign_and_verify(tx, user.signing_keys[0].private_key));
}

HEAVY_TEST_F(client_proofs_escape_hatch, test_wrong_signature_fails)
{
    escape_hatch_tx tx = simple_setup();
    EXPECT_FALSE((sign_and_verify(tx, user.signing_keys[1].private_key)));
}
