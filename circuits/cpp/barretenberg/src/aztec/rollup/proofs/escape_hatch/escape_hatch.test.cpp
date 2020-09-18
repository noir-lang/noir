#include "../notes/pedersen_note.hpp"
#include "../../fixtures/user_context.hpp"
#include "escape_hatch.hpp"
#include "escape_hatch_circuit.hpp"
#include "escape_hatch_tx.hpp"
#include "../notes/sign_notes.hpp"
#include <common/streams.hpp>
#include <common/test.hpp>
#include <crypto/schnorr/schnorr.hpp>
#include <stdlib/merkle_tree/memory_store.hpp>
#include <stdlib/merkle_tree/leveldb_tree.hpp>
#include <plonk/proof_system/proving_key/serialize.hpp>

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::merkle_tree;
using namespace rollup::proofs::join_split;
using namespace rollup::proofs::escape_hatch;

class escape_hatch_tests : public ::testing::Test {
  protected:
    static void SetUpTestCase()
    {
        auto null_crs_factory = std::make_unique<waffle::ReferenceStringFactory>();
        init_proving_key(std::move(null_crs_factory));
        auto crs_factory = std::make_unique<waffle::FileReferenceStringFactory>("../srs_db/ignition");
        init_verification_key(std::move(crs_factory));
    }

    escape_hatch_tests()
        : data_tree(store, 32, 0)
        , null_tree(store, 128, 1)
        , root_tree(store, 28, 2)
    {
        update_root_tree_with_data_root(0);
        user = rollup::fixtures::create_user_context();
    }

    void preload_value_notes()
    {
        tx_note note1 = { user.owner.public_key, 100, user.note_secret };
        tx_note note2 = { user.owner.public_key, 50, user.note_secret };

        auto enc_note1 = encrypt_note(note1);
        data_tree.update_element(data_tree.size(), create_leaf_data(enc_note1));

        auto enc_note2 = encrypt_note(note2);
        data_tree.update_element(data_tree.size(), create_leaf_data(enc_note2));
    }

    void preload_account_notes()
    {
        data_tree.update_element(data_tree.size(),
                                 create_account_leaf_data(user.owner.public_key, user.signing_keys[0].public_key));
        data_tree.update_element(data_tree.size(),
                                 create_account_leaf_data(user.owner.public_key, user.signing_keys[1].public_key));
    }

    void update_root_tree_with_data_root(size_t index)
    {
        auto data_root = to_buffer(data_tree.root());
        root_tree.update_element(index, data_root);
    }

    bool sign_and_verify(escape_hatch_tx& tx, grumpkin::fr const& signing_private_key)
    {
        tx.js_tx.signature = sign_notes(
            { tx.js_tx.input_note[0], tx.js_tx.input_note[1], tx.js_tx.output_note[0], tx.js_tx.output_note[1] },
            { signing_private_key, tx.js_tx.signing_pub_key });
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
        update_root_tree_with_data_root(1);
        return create_escape_hatch_tx({ 2, 3 }, 0);
    }

    uint128_t create_account_nullifier(grumpkin::g1::affine_element const& owner_key,
                                       grumpkin::g1::affine_element const& signing_key)
    {
        auto data = create_account_leaf_data(owner_key, signing_key);
        auto nullifier = merkle_tree::hash_value_native(data);
        return uint128_t(nullifier);
    }

    join_split_tx create_join_split_tx(std::array<uint32_t, 2> const& input_indicies, uint32_t account_index)
    {
        tx_note input_note1 = { user.owner.public_key, 100, user.note_secret };
        tx_note input_note2 = { user.owner.public_key, 50, user.note_secret };
        tx_note output_note1 = { user.owner.public_key, 70, user.note_secret };
        tx_note output_note2 = { user.owner.public_key, 80, user.note_secret };

        join_split_tx tx;
        tx.public_input = 0;
        tx.public_output = 0;
        tx.num_input_notes = 2;
        tx.input_index = input_indicies;
        tx.old_data_root = data_tree.root();
        tx.input_path = { data_tree.get_hash_path(input_indicies[0]), data_tree.get_hash_path(input_indicies[1]) };
        tx.input_note = { input_note1, input_note2 };
        tx.output_note = { output_note1, output_note2 };
        tx.input_owner = fr::random_element();
        tx.output_owner = fr::random_element();
        tx.account_index = account_index;
        tx.account_path = data_tree.get_hash_path(account_index);
        tx.signing_pub_key = user.signing_keys[0].public_key;

        return tx;
    }

    escape_hatch_tx create_escape_hatch_tx(std::array<uint32_t, 2> const& input_indicies, uint32_t account_index)
    {
        escape_hatch_tx tx;
        tx.js_tx = create_join_split_tx(input_indicies, account_index);

        tx.rollup_id = static_cast<uint32_t>(data_tree.size() / 2 - 1);
        tx.data_start_index = static_cast<uint32_t>(data_tree.size());
        tx.old_data_path = data_tree.get_hash_path(tx.data_start_index);
        auto enc_note1 = encrypt_note(tx.js_tx.output_note[0]);
        auto enc_note2 = encrypt_note(tx.js_tx.output_note[1]);
        data_tree.update_element(data_tree.size(), create_leaf_data(enc_note1));
        data_tree.update_element(data_tree.size(), create_leaf_data(enc_note2));
        tx.new_data_root = data_tree.root();
        tx.new_data_path = data_tree.get_hash_path(tx.data_start_index);

        auto root_tree_index = root_tree.size();
        tx.old_data_roots_root = root_tree.root();
        tx.old_data_roots_path = root_tree.get_hash_path(root_tree_index);
        auto data_root = to_buffer(data_tree.root());
        root_tree.update_element(root_tree_index, data_root);
        tx.new_data_roots_root = root_tree.root();
        tx.new_data_roots_path = root_tree.get_hash_path(root_tree_index);

        uint128_t nullifier1 = create_nullifier(tx.js_tx.input_note[0], uint32_t(tx.js_tx.input_index[0]));
        uint128_t nullifier2 = create_nullifier(tx.js_tx.input_note[1], uint32_t(tx.js_tx.input_index[1]));

        auto nullifier_value = std::vector<uint8_t>(64, 0);
        nullifier_value[63] = 1;

        uint128_t account_nullifier = create_account_nullifier(user.owner.public_key, user.signing_keys[0].public_key);
        tx.account_null_path = null_tree.get_hash_path(account_nullifier);

        tx.old_null_root = null_tree.root();
        tx.old_null_paths.resize(2);
        tx.new_null_paths.resize(2);
        tx.new_null_roots.resize(2);

        tx.old_null_paths[0] = null_tree.get_hash_path(nullifier1);
        null_tree.update_element(nullifier1, nullifier_value);
        tx.new_null_roots[0] = null_tree.root();
        tx.new_null_paths[0] = null_tree.get_hash_path(nullifier1);

        tx.old_null_paths[1] = null_tree.get_hash_path(nullifier2);
        null_tree.update_element(nullifier2, nullifier_value);
        tx.new_null_roots[1] = null_tree.root();
        tx.new_null_paths[1] = null_tree.get_hash_path(nullifier2);

        return tx;
    }

    MemoryStore store;
    MerkleTree<MemoryStore> data_tree;
    MerkleTree<MemoryStore> null_tree;
    MerkleTree<MemoryStore> root_tree;
    rollup::fixtures::user_context user;
};

TEST_F(escape_hatch_tests, test_2_input_notes)
{
    escape_hatch_tx tx = simple_setup();
    auto buf = to_buffer(tx);
    EXPECT_TRUE(sign_and_verify(tx, user.signing_keys[0].private_key));
}

HEAVY_TEST_F(escape_hatch_tests, test_1_false_new_null_path_fails)
{
    escape_hatch_tx tx = simple_setup();
    tx.new_null_paths[1] = null_tree.get_hash_path(3);
    EXPECT_FALSE((sign_and_verify(tx, user.signing_keys[0].private_key)));
}

HEAVY_TEST_F(escape_hatch_tests, test_switched_nullifier_paths_order_fails)
{
    escape_hatch_tx tx = simple_setup();
    merkle_tree::fr_hash_path null_path_copy = tx.new_null_paths[1];

    tx.new_null_paths[1] = tx.new_null_paths[0];
    tx.new_null_paths[0] = null_path_copy;
    EXPECT_FALSE((sign_and_verify(tx, user.signing_keys[0].private_key)));
}

HEAVY_TEST_F(escape_hatch_tests, test_1_false_old_nullifier_path_fails)
{
    escape_hatch_tx tx = simple_setup();
    tx.old_null_paths[1] = null_tree.get_hash_path(3);
    EXPECT_FALSE((sign_and_verify(tx, user.signing_keys[0].private_key)));
}

HEAVY_TEST_F(escape_hatch_tests, test_incorrect_new_null_root_fails)
{
    escape_hatch_tx tx = simple_setup();
    tx.new_null_roots[1] = fr::random_element();
    EXPECT_FALSE((sign_and_verify(tx, user.signing_keys[0].private_key)));
}

HEAVY_TEST_F(escape_hatch_tests, switched_around_new_null_root_fails)
{
    escape_hatch_tx tx = simple_setup();
    barretenberg::fr null_root_copy = tx.new_null_roots[1];

    tx.new_null_roots[1] = tx.new_null_roots[0];
    tx.new_null_roots[0] = null_root_copy;

    EXPECT_FALSE((sign_and_verify(tx, user.signing_keys[0].private_key)));
}

HEAVY_TEST_F(escape_hatch_tests, wrong_null_merkle_root_fails)
{
    escape_hatch_tx tx = simple_setup();
    tx.old_null_root = fr::random_element();
    EXPECT_FALSE((sign_and_verify(tx, user.signing_keys[0].private_key)));
}

HEAVY_TEST_F(escape_hatch_tests, switch_current_new_null_paths_fails)
{
    escape_hatch_tx tx = simple_setup();
    auto new_paths_copy = tx.new_null_paths;

    tx.new_null_paths = tx.old_null_paths;
    tx.old_null_paths = new_paths_copy;

    EXPECT_FALSE((sign_and_verify(tx, user.signing_keys[0].private_key)));
}

HEAVY_TEST_F(escape_hatch_tests, test_joining_same_note_fails)
{
    escape_hatch_tx tx = simple_setup();
    tx.js_tx.input_note[0].value = 75;
    tx.js_tx.input_note[1].value = 75;
    tx.js_tx.input_index = { 1, 1 };

    EXPECT_FALSE(sign_and_verify(tx, user.signing_keys[0].private_key));
}

HEAVY_TEST_F(escape_hatch_tests, test_unbalanced_notes_fails)
{
    escape_hatch_tx tx = simple_setup();
    tx.js_tx.input_note[0].value = 99;
    tx.js_tx.input_note[1].value = 50;
    EXPECT_FALSE(sign_and_verify(tx, user.signing_keys[0].private_key));
}

HEAVY_TEST_F(escape_hatch_tests, test_wrong_input_note_owner_fails)
{
    escape_hatch_tx tx = simple_setup();
    tx.js_tx.input_note[1].owner = grumpkin::g1::element::random_element();
    EXPECT_FALSE(sign_and_verify(tx, user.signing_keys[0].private_key));
}

HEAVY_TEST_F(escape_hatch_tests, test_wrong_hash_path_fails)
{
    escape_hatch_tx tx = simple_setup();
    tx.js_tx.input_path[1] = data_tree.get_hash_path(0);

    EXPECT_FALSE(sign_and_verify(tx, user.signing_keys[0].private_key));
}

HEAVY_TEST_F(escape_hatch_tests, test_wrong_merkle_root_fails)
{
    escape_hatch_tx tx = simple_setup();
    tx.js_tx.old_data_root = fr::random_element();

    EXPECT_FALSE(sign_and_verify(tx, user.signing_keys[0].private_key));
}

HEAVY_TEST_F(escape_hatch_tests, test_wrong_signature_fails)
{
    escape_hatch_tx tx = simple_setup();
    EXPECT_FALSE((sign_and_verify(tx, user.signing_keys[1].private_key)));
}
