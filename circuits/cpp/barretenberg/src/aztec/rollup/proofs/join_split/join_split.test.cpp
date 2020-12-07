#include "../../fixtures/user_context.hpp"
#include "../inner_proof_data.hpp"
#include "join_split.hpp"
#include "join_split_circuit.hpp"
#include "../notes/native/sign_notes.hpp"
#include "../notes/native/encrypt_note.hpp"
#include "../notes/native/account_note.hpp"
#include <common/streams.hpp>
#include <common/test.hpp>
#include <crypto/schnorr/schnorr.hpp>
#include <stdlib/merkle_tree/memory_store.hpp>
#include <stdlib/merkle_tree/merkle_tree.hpp>

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::merkle_tree;
using namespace rollup::proofs;
using namespace rollup::proofs::notes::native;
using namespace rollup::proofs::join_split;

std::vector<uint8_t> create_leaf_data(grumpkin::g1::affine_element const& enc_note)
{
    std::vector<uint8_t> buf;
    write(buf, enc_note.x);
    write(buf, enc_note.y);
    return buf;
}

std::vector<uint8_t> create_account_leaf_data(fr const& account_alias_id,
                                              grumpkin::g1::affine_element const& owner_key,
                                              grumpkin::g1::affine_element const& signing_key)
{
    auto enc_note = encrypt_account_note({ account_alias_id, owner_key, signing_key });
    std::vector<uint8_t> buf;
    write(buf, enc_note.x);
    write(buf, enc_note.y);
    return buf;
}

class join_split_tests : public ::testing::Test {
  protected:
#ifndef DISABLE_HEAVY_TESTS
    static void SetUpTestCase()
    {
        auto null_crs_factory = std::make_unique<waffle::ReferenceStringFactory>();
        init_proving_key(std::move(null_crs_factory));
        auto crs_factory = std::make_unique<waffle::FileReferenceStringFactory>("../srs_db");
        init_verification_key(std::move(crs_factory));
    }
#endif

    virtual void SetUp()
    {
        store = std::make_unique<MemoryStore>();
        tree = std::make_unique<MerkleTree<MemoryStore>>(*store, 32);
        user = rollup::fixtures::create_user_context();
        value_notes[0] = { user.owner.public_key, 100, user.note_secret, 0, 0 };
        value_notes[1] = { user.owner.public_key, 50, user.note_secret, 0, 0 };
        value_notes[2] = { user.owner.public_key, 90, user.note_secret, 0, 1 };
        value_notes[3] = { user.owner.public_key, 40, user.note_secret, 0, 1 };
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

    /**
     * Add two account notes for the user, and two value notes.
     */
    void preload_value_notes()
    {
        for (value_note note : value_notes) {
            auto enc_note = encrypt_note(note);
            tree->update_element(tree->size(), create_leaf_data(enc_note));
        }
    }

    join_split_tx create_join_split_tx(std::array<uint32_t, 2> const& input_indicies,
                                       uint32_t account_index,
                                       uint32_t nonce)
    {
        value_note input_note1 = value_notes[input_indicies[0]];
        value_note input_note2 = value_notes[input_indicies[1]];
        value_note output_note1 = {
            user.owner.public_key, input_note1.value + input_note2.value, user.note_secret, 0, nonce
        };
        value_note output_note2 = { user.owner.public_key, 0, user.note_secret, 0, nonce };

        join_split_tx tx;
        tx.public_input = 0;
        tx.public_output = 0;
        tx.num_input_notes = 2;
        tx.input_index = input_indicies;
        tx.old_data_root = tree->root();
        tx.input_path = { tree->get_hash_path(input_indicies[0]), tree->get_hash_path(input_indicies[1]) };
        tx.input_note = { input_note1, input_note2 };
        tx.output_note = { output_note1, output_note2 };
        tx.input_owner = fr::random_element();
        tx.output_owner = fr::random_element();
        tx.account_index = account_index;
        tx.account_path = tree->get_hash_path(account_index);
        tx.signing_pub_key = user.signing_keys[0].public_key;
        tx.asset_id = 1; // asset_id can take any value if there are no public input/output values
        tx.account_private_key = user.owner.private_key;
        tx.alias_hash = !nonce ? fr::random_element() : user.alias_hash;
        tx.nonce = nonce;
        return tx;
    }

    /**
     * Add account notes and value notes (sum 150).
     * Return a join split tx that spends them.
     */
    join_split_tx simple_setup(std::array<uint32_t, 2> const& input_indicies = { 0, 1 },
                               uint32_t account_index = 0,
                               uint32_t nonce = 0)
    {
        preload_value_notes();   // indicies: [0, 1](nonce 0), [2, 3](nonce 1)
        preload_account_notes(); // indicies: [4, 5]
        return create_join_split_tx(input_indicies, account_index, nonce);
    }

    /**
     * Return a join split tx that performs a 100 token transfer.
     */
    join_split_tx public_transfer_setup()
    {
        value_note input_note1 = { user.owner.public_key, 0, user.note_secret, 0, 0 };
        value_note input_note2 = { user.owner.public_key, 0, user.note_secret, 0, 0 };
        value_note output_note1 = { user.owner.public_key, 0, user.note_secret, 0, 0 };
        value_note output_note2 = { user.owner.public_key, 0, user.note_secret, 0, 0 };

        join_split_tx tx;
        tx.public_input = 100;
        tx.public_output = 100;
        tx.num_input_notes = 0;
        tx.input_index = { 1, 0 };
        tx.old_data_root = tree->root();
        tx.input_path = { tree->get_hash_path(1), tree->get_hash_path(0) };
        tx.input_note = { input_note2, input_note1 };
        tx.output_note = { output_note1, output_note2 };
        tx.input_owner = fr::random_element();
        tx.output_owner = fr::random_element();
        tx.account_index = 0;
        tx.account_path = tree->get_hash_path(0);
        tx.signing_pub_key = user.signing_keys[0].public_key;
        tx.asset_id = 0;
        tx.account_private_key = user.owner.private_key;
        tx.alias_hash = fr::random_element();
        tx.nonce = 0;
        return tx;
    }

    bool sign_and_verify(join_split_tx& tx, grumpkin::fr const& signing_private_key)
    {
        tx.signature = sign_notes(tx, { signing_private_key, tx.signing_pub_key });

        auto prover = new_join_split_prover(tx);
        auto proof = prover.construct_proof();
        return verify_proof(proof);
    }

    bool verify_logic(join_split_tx& tx)
    {
        Composer composer(get_proving_key(), nullptr);
        join_split_circuit(composer, tx);
        if (composer.failed) {
            std::cout << "Logic failed: " << composer.err << std::endl;
        }
        return !composer.failed;
    }

    bool sign_and_verify_logic(join_split_tx& tx, grumpkin::fr const& signing_private_key)
    {
        tx.signature = sign_notes(tx, { signing_private_key, tx.signing_pub_key });

        return verify_logic(tx);
    }

    rollup::fixtures::user_context user;
    std::unique_ptr<MemoryStore> store;
    std::unique_ptr<MerkleTree<MemoryStore>> tree;
    value_note value_notes[4];
};

TEST_F(join_split_tests, test_0_input_notes)
{
    value_note gibberish = { user.owner.public_key, 0, user.note_secret, 0, 0 };

    join_split_tx tx = simple_setup();
    tx.asset_id = 0;
    tx.num_input_notes = 0;
    tx.input_note = { gibberish, gibberish };
    tx.public_input = 30;
    tx.output_note[0].value = 30;

    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner.private_key));
}

TEST_F(join_split_tests, test_1_input_note)
{
    join_split_tx tx = simple_setup();
    tx.num_input_notes = 1;
    tx.input_note[1].value = 0;
    tx.output_note[0].value = tx.input_note[0].value;

    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner.private_key));
}

TEST_F(join_split_tests, test_2_input_notes)
{
    join_split_tx tx = simple_setup();
    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner.private_key));
}

HEAVY_TEST_F(join_split_tests, test_2_input_notes_full_proof)
{
    join_split_tx tx = simple_setup();
    EXPECT_TRUE(sign_and_verify(tx, user.owner.private_key));
}

TEST_F(join_split_tests, test_0_output_notes)
{
    join_split_tx tx = simple_setup();
    tx.asset_id = 0;
    tx.output_note[0].value = 0;
    tx.output_note[1].value = 0;
    tx.public_output = tx.input_note[0].value + tx.input_note[1].value;

    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner.private_key));
}

TEST_F(join_split_tests, test_large_output_note)
{
    auto deposit_value = (uint256_t(1) << 252) - 1;

    value_note gibberish = { user.owner.public_key, 0, user.note_secret, 0, 0 };
    value_note output_note1 = { user.owner.public_key, deposit_value, user.note_secret, 0, 0 };
    value_note output_note2 = { user.owner.public_key, 0, user.note_secret, 0, 0 };

    join_split_tx tx = simple_setup();
    tx.public_input = deposit_value;
    tx.public_output = 0;
    tx.num_input_notes = 0;
    tx.asset_id = 0;
    tx.input_note = { gibberish, gibberish };
    tx.output_note = { output_note1, output_note2 };

    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner.private_key));
}

TEST_F(join_split_tests, test_0_input_notes_with_unbalanced_public_values)
{
    join_split_tx tx = public_transfer_setup();
    tx.public_input = 120;
    tx.output_note[0].value = 20;

    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner.private_key));
}

TEST_F(join_split_tests, test_2_input_notes_with_unbalanced_public_values)
{
    join_split_tx tx = simple_setup();
    tx.asset_id = 0;
    tx.output_note[0].value = 80;
    tx.output_note[1].value = 0;
    tx.public_input = 100;
    tx.public_output = 170;

    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner.private_key));
}

TEST_F(join_split_tests, test_spending_1_note_with_non_0_value_second_note_fails)
{
    join_split_tx tx = simple_setup();
    tx.num_input_notes = 1;
    tx.input_note[0].value = 100;
    tx.input_note[1].value = 100;
    tx.output_note[0].value = 100;
    tx.output_note[1].value = 100;

    EXPECT_FALSE(sign_and_verify_logic(tx, user.owner.private_key));
}

TEST_F(join_split_tests, test_unbalanced_notes_fails)
{
    join_split_tx tx = simple_setup();
    tx.input_note[1].value = 51;

    EXPECT_FALSE(sign_and_verify_logic(tx, user.owner.private_key));
}

TEST_F(join_split_tests, test_0_notes_with_unbalanced_public_values_fails)
{
    join_split_tx tx = public_transfer_setup();
    tx.public_input = 120;

    EXPECT_FALSE(sign_and_verify_logic(tx, user.owner.private_key));
}

// Input note

TEST_F(join_split_tests, test_joining_same_note_fails)
{
    join_split_tx tx = simple_setup({ 1, 1 });
    EXPECT_FALSE(sign_and_verify_logic(tx, user.owner.private_key));
}

TEST_F(join_split_tests, test_different_input_note_nonces_fails)
{
    join_split_tx tx = simple_setup({ 1, 2 });

    EXPECT_NE(tx.input_note[0].nonce, tx.input_note[1].nonce);
    EXPECT_FALSE(sign_and_verify_logic(tx, user.owner.private_key));
}

// Input note account value id

TEST_F(join_split_tests, test_spend_notes_with_registered_account)
{
    join_split_tx tx = simple_setup({ 2, 3 }, 4, 1);
    EXPECT_TRUE(sign_and_verify_logic(tx, user.signing_keys[0].private_key));
}

TEST_F(join_split_tests, test_different_note_nonce_vs_account_nonce_fails)
{
    join_split_tx tx = simple_setup({ 2, 3 }, 4, 0);
    EXPECT_FALSE(sign_and_verify_logic(tx, user.owner.private_key));
}

TEST_F(join_split_tests, test_wrong_input_note_owner_fails)
{
    join_split_tx tx = simple_setup();
    tx.input_note[0].owner = grumpkin::g1::element::random_element();
    tx.input_note[1].owner = tx.input_note[0].owner;

    EXPECT_FALSE(sign_and_verify_logic(tx, user.owner.private_key));
}

// Output note owner

TEST_F(join_split_tests, test_random_output_note_owners)
{
    join_split_tx tx = simple_setup();
    tx.output_note[0].owner = grumpkin::g1::element::random_element();
    tx.output_note[1].owner = grumpkin::g1::element::random_element();

    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner.private_key));
}

// Signature

TEST_F(join_split_tests, test_wrong_account_private_key_fails)
{
    join_split_tx tx = simple_setup();
    tx.account_private_key = grumpkin::fr::random_element();

    EXPECT_FALSE(sign_and_verify_logic(tx, user.owner.private_key));
}

TEST_F(join_split_tests, test_wrong_output_owner_sig_fail)
{
    join_split_tx tx = simple_setup();

    // sign with correct output owner
    tx.signature = sign_notes(tx, { user.owner.private_key, tx.signing_pub_key });

    // set a fake output owner
    auto fake_owner = fr::random_element();
    tx.output_owner = fake_owner;

    EXPECT_FALSE(verify_logic(tx));
}

TEST_F(join_split_tests, test_spend_zero_nonce_notes_with_signing_key_fails)
{
    join_split_tx tx = simple_setup();
    EXPECT_FALSE(sign_and_verify_logic(tx, user.signing_keys[0].private_key));
}

TEST_F(join_split_tests, test_spend_registered_notes_with_owner_key_fails)
{
    auto tx = simple_setup({ 2, 3 }, 4, 1);
    EXPECT_FALSE(sign_and_verify_logic(tx, user.owner.private_key));
}

// Account membership

TEST_F(join_split_tests, test_wrong_merkle_root_fails)
{
    join_split_tx tx = simple_setup();
    tx.old_data_root = fr::random_element();

    EXPECT_FALSE(sign_and_verify_logic(tx, user.owner.private_key));
}

TEST_F(join_split_tests, test_wrong_alias_hash_fails)
{
    join_split_tx tx = simple_setup({ 2, 3 }, 4, 1);
    tx.alias_hash = fr::random_element();
    EXPECT_FALSE(sign_and_verify_logic(tx, user.signing_keys[0].private_key));
}

TEST_F(join_split_tests, test_nonregistered_signing_key_fails)
{
    join_split_tx tx = simple_setup({ 2, 3 }, 4, 1);
    auto keys = rollup::fixtures::create_key_pair(nullptr);
    tx.signing_pub_key = keys.public_key;

    EXPECT_FALSE(sign_and_verify_logic(tx, keys.private_key));
}

// Note membership

TEST_F(join_split_tests, test_wrong_note_hash_path_fails)
{
    join_split_tx tx = simple_setup();
    auto gibberish_path = fr_hash_path(32, std::make_pair(fr::random_element(), fr::random_element()));
    tx.input_path[0] = gibberish_path;

    EXPECT_FALSE(sign_and_verify_logic(tx, user.owner.private_key));
}

// Output Owner

TEST_F(join_split_tests, test_zero_output_owner)
{
    join_split_tx tx = simple_setup();

    tx.output_owner = fr::zero();

    tx.signature = sign_notes(tx, { user.owner.private_key, tx.signing_pub_key });

    EXPECT_TRUE(verify_logic(tx));
}

HEAVY_TEST_F(join_split_tests, test_tainted_output_owner_fails)
{
    join_split_tx tx = simple_setup();
    tx.signing_pub_key = user.owner.public_key;
    tx.signature = sign_notes(tx, { user.owner.private_key, user.owner.public_key });
    uint8_t output_owner[32] = { 0x01, 0xaa, 0x42, 0xd4, 0x72, 0x88, 0x8e, 0xae, 0xa5, 0x56, 0x39,
                                 0x46, 0xeb, 0x5c, 0xf5, 0x6c, 0x81, 0x6,  0x4d, 0x80, 0xc6, 0xf5,
                                 0xa5, 0x38, 0xcc, 0x87, 0xae, 0x54, 0xae, 0xdb, 0x75, 0xd9 };
    tx.output_owner = from_buffer<fr>(output_owner);

    auto prover = new_join_split_prover(tx);
    auto proof = prover.construct_proof();

    EXPECT_EQ(proof.proof_data[InnerProofOffsets::OUTPUT_OWNER], 0x01);
    proof.proof_data[InnerProofFields::OUTPUT_OWNER] = 0x02;

    EXPECT_FALSE(verify_proof(proof));
}
