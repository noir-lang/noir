#include "../../fixtures/user_context.hpp"
#include "escape_hatch.hpp"
#include "escape_hatch_circuit.hpp"
#include "escape_hatch_tx.hpp"
#include "../notes/native/sign_notes.hpp"
#include "../notes/native/encrypt_note.hpp"
#include "../notes/native/account_note.hpp"
#include "../notes/native/compute_nullifier.hpp"
#include "../../constants.hpp"
#include <common/streams.hpp>
#include <common/test.hpp>
#include <crypto/schnorr/schnorr.hpp>
#include <stdlib/merkle_tree/memory_store.hpp>
#include <stdlib/merkle_tree/merkle_tree.hpp>
#include <plonk/proof_system/proving_key/serialize.hpp>

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::merkle_tree;
using namespace rollup::proofs::join_split;
using namespace rollup::proofs::escape_hatch;
using namespace rollup::proofs::notes::native;

class escape_hatch_tests : public ::testing::Test {
  protected:
    static void SetUpTestCase()
    {
        auto null_crs_factory = std::make_unique<waffle::ReferenceStringFactory>();
        init_proving_key(std::move(null_crs_factory));
        auto crs_factory = std::make_unique<waffle::FileReferenceStringFactory>("../srs_db");
        init_verification_key(std::move(crs_factory));
    }

    escape_hatch_tests()
        : data_tree(store, rollup::DATA_TREE_DEPTH, 0)
        , null_tree(store, rollup::NULL_TREE_DEPTH, 1)
        , root_tree(store, rollup::ROOT_TREE_DEPTH, 2)
    {
        update_root_tree_with_data_root(0);
        user = rollup::fixtures::create_user_context();
    }

    void preload_value_notes(uint32_t nonce = 0)
    {
        value_note note1 = { user.owner.public_key, 100, user.note_secret, 0, nonce };
        value_note note2 = { user.owner.public_key, 50, user.note_secret, 0, nonce };

        auto enc_note1 = encrypt_note(note1);
        data_tree.update_element(data_tree.size(), create_leaf_data(enc_note1));

        auto enc_note2 = encrypt_note(note2);
        data_tree.update_element(data_tree.size(), create_leaf_data(enc_note2));
    }

    void preload_account_notes()
    {
        auto account_id = rollup::fixtures::generate_account_id(user.alias_hash, 1);
        data_tree.update_element(
            data_tree.size(),
            create_account_leaf_data(account_id, user.owner.public_key, user.signing_keys[0].public_key));
        data_tree.update_element(
            data_tree.size(),
            create_account_leaf_data(account_id, user.owner.public_key, user.signing_keys[1].public_key));
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
            tx.js_tx.output_owner,
            { signing_private_key, tx.js_tx.signing_pub_key });
        auto prover = new_escape_hatch_prover(tx);
        auto proof = prover.construct_proof();
        return verify_proof(proof);
    }

    bool verify_logic(escape_hatch_tx& tx)
    {
        Composer composer(get_proving_key(), nullptr);
        escape_hatch_circuit(composer, tx);
        if (composer.failed) {
            std::cout << "Logic failed: " << composer.err << std::endl;
        }
        return !composer.failed;
    }

    bool sign_and_verify_logic(escape_hatch_tx& tx, grumpkin::fr const& signing_private_key)
    {
        tx.js_tx.signature = sign_notes(
            { tx.js_tx.input_note[0], tx.js_tx.input_note[1], tx.js_tx.output_note[0], tx.js_tx.output_note[1] },
            tx.js_tx.output_owner,
            { signing_private_key, tx.js_tx.signing_pub_key });
        return verify_logic(tx);
    }

    std::vector<uint8_t> create_leaf_data(grumpkin::g1::affine_element const& enc_note)
    {
        std::vector<uint8_t> buf;
        write(buf, enc_note.x);
        write(buf, enc_note.y);
        return buf;
    }

    std::vector<uint8_t> create_account_leaf_data(fr const& account_id,
                                                  grumpkin::g1::affine_element const& owner_key,
                                                  grumpkin::g1::affine_element const& signing_key)
    {
        auto enc_note = encrypt_account_note({ account_id, owner_key, signing_key });
        std::vector<uint8_t> buf;
        write(buf, enc_note.x);
        write(buf, enc_note.y);
        return buf;
    }

    escape_hatch_tx simple_setup()
    {
        preload_account_notes();
        preload_value_notes(1);
        update_root_tree_with_data_root(1);
        return create_escape_hatch_tx({ 2, 3 }, 0, 1);
    }

    join_split_tx create_join_split_tx(std::array<uint32_t, 2> const& input_indicies,
                                       uint32_t account_index,
                                       uint32_t nonce)
    {
        value_note input_note1 = { user.owner.public_key, 100, user.note_secret, 0, nonce };
        value_note input_note2 = { user.owner.public_key, 50, user.note_secret, 0, nonce };
        value_note output_note1 = { user.owner.public_key, 70, user.note_secret, 0, nonce };
        value_note output_note2 = { user.owner.public_key, 80, user.note_secret, 0, nonce };

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
        tx.asset_id = 0;
        tx.account_private_key = user.owner.private_key;
        tx.alias_hash = user.alias_hash;
        tx.nonce = nonce;
        return tx;
    }

    escape_hatch_tx create_escape_hatch_tx(std::array<uint32_t, 2> const& input_indicies,
                                           uint32_t account_index,
                                           uint32_t nonce = 0)
    {
        escape_hatch_tx tx;
        tx.js_tx = create_join_split_tx(input_indicies, account_index, nonce);

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

        auto nullifier1 = uint256_t(compute_nullifier(
            encrypt_note(tx.js_tx.input_note[0]), tx.js_tx.input_index[0], user.owner.private_key, true));
        auto nullifier2 = uint256_t(compute_nullifier(
            encrypt_note(tx.js_tx.input_note[1]), tx.js_tx.input_index[1], user.owner.private_key, true));

        auto nullifier_value = std::vector<uint8_t>(64, 0);
        nullifier_value[63] = 1;

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
    EXPECT_TRUE(sign_and_verify_logic(tx, user.signing_keys[0].private_key));
}

TEST_F(escape_hatch_tests, test_1_false_new_null_path_fails)
{
    escape_hatch_tx tx = simple_setup();
    auto gibberish_path = fr_hash_path(256, std::make_pair(fr::random_element(), fr::random_element()));
    tx.old_null_paths[1] = gibberish_path;
    EXPECT_FALSE((sign_and_verify_logic(tx, user.signing_keys[0].private_key)));
}

TEST_F(escape_hatch_tests, test_switched_nullifier_paths_order_fails)
{
    escape_hatch_tx tx = simple_setup();
    merkle_tree::fr_hash_path null_path_copy = tx.new_null_paths[1];

    tx.new_null_paths[1] = tx.new_null_paths[0];
    tx.new_null_paths[0] = null_path_copy;
    EXPECT_FALSE((sign_and_verify_logic(tx, user.signing_keys[0].private_key)));
}

TEST_F(escape_hatch_tests, test_1_false_old_nullifier_path_fails)
{
    escape_hatch_tx tx = simple_setup();
    auto gibberish_path = fr_hash_path(256, std::make_pair(fr::random_element(), fr::random_element()));
    tx.old_null_paths[1] = gibberish_path;
    EXPECT_FALSE((sign_and_verify_logic(tx, user.signing_keys[0].private_key)));
}

TEST_F(escape_hatch_tests, test_incorrect_new_null_root_fails)
{
    escape_hatch_tx tx = simple_setup();
    tx.new_null_roots[1] = fr::random_element();
    EXPECT_FALSE((sign_and_verify_logic(tx, user.signing_keys[0].private_key)));
}

TEST_F(escape_hatch_tests, switched_around_new_null_root_fails)
{
    escape_hatch_tx tx = simple_setup();
    barretenberg::fr null_root_copy = tx.new_null_roots[1];

    tx.new_null_roots[1] = tx.new_null_roots[0];
    tx.new_null_roots[0] = null_root_copy;

    EXPECT_FALSE((sign_and_verify_logic(tx, user.signing_keys[0].private_key)));
}

TEST_F(escape_hatch_tests, wrong_null_merkle_root_fails)
{
    escape_hatch_tx tx = simple_setup();
    tx.old_null_root = fr::random_element();
    EXPECT_FALSE((sign_and_verify_logic(tx, user.signing_keys[0].private_key)));
}

TEST_F(escape_hatch_tests, switch_current_new_null_paths_fails)
{
    escape_hatch_tx tx = simple_setup();
    auto new_paths_copy = tx.new_null_paths;

    // n.b. simply swapping the hash paths will now produce a valid proof. The
    // circuit only extracts one item out of each hash path index - the partner hash.
    // This partner hash does not change when performing a state update, so both old/new paths are valid.
    tx.new_null_paths = tx.old_null_paths;
    tx.old_null_paths = new_paths_copy;

    std::swap(tx.old_null_paths[0], tx.old_null_paths[1]);
    EXPECT_FALSE((sign_and_verify_logic(tx, user.signing_keys[0].private_key)));
}

TEST_F(escape_hatch_tests, test_joining_same_note_fails)
{
    escape_hatch_tx tx = simple_setup();
    tx.js_tx.input_note[0].value = 75;
    tx.js_tx.input_note[1].value = 75;
    tx.js_tx.input_index = { 1, 1 };

    EXPECT_FALSE(sign_and_verify_logic(tx, user.signing_keys[0].private_key));
}

TEST_F(escape_hatch_tests, test_unbalanced_notes_fails)
{
    escape_hatch_tx tx = simple_setup();
    tx.js_tx.input_note[0].value = 99;
    tx.js_tx.input_note[1].value = 50;
    EXPECT_FALSE(sign_and_verify_logic(tx, user.signing_keys[0].private_key));
}

TEST_F(escape_hatch_tests, test_wrong_input_note_owner_fails)
{
    escape_hatch_tx tx = simple_setup();
    tx.js_tx.input_note[1].owner = grumpkin::g1::element::random_element();
    EXPECT_FALSE(sign_and_verify_logic(tx, user.signing_keys[0].private_key));
}

TEST_F(escape_hatch_tests, test_wrong_hash_path_fails)
{
    escape_hatch_tx tx = simple_setup();
    tx.js_tx.input_path[1] = data_tree.get_hash_path(0);

    EXPECT_FALSE(sign_and_verify_logic(tx, user.signing_keys[0].private_key));
}

TEST_F(escape_hatch_tests, test_wrong_merkle_root_fails)
{
    escape_hatch_tx tx = simple_setup();
    tx.js_tx.old_data_root = fr::random_element();

    EXPECT_FALSE(sign_and_verify_logic(tx, user.signing_keys[0].private_key));
}

TEST_F(escape_hatch_tests, test_wrong_signature_fails)
{
    escape_hatch_tx tx = simple_setup();
    EXPECT_FALSE((sign_and_verify_logic(tx, user.signing_keys[1].private_key)));
}

HEAVY_TEST_F(escape_hatch_tests, test_2_input_notes_full_test)
{
    escape_hatch_tx tx = simple_setup();
    EXPECT_TRUE(sign_and_verify(tx, user.signing_keys[0].private_key));
}
