#include "../../fixtures/user_context.hpp"
#include "../inner_proof_data.hpp"
#include "join_split.hpp"
#include "join_split_circuit.hpp"
#include "../notes/native/sign_notes.hpp"
#include "../notes/native/encrypt_note.hpp"
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

std::vector<uint8_t> create_account_leaf_data(grumpkin::g1::affine_element const& owner_key,
                                              grumpkin::g1::affine_element const& signing_key)
{
    std::vector<uint8_t> buf;
    write(buf, owner_key.x);
    write(buf, signing_key.x);
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

    /**
     * Add two account notes for the user, and two value notes.
     */
    void preload_value_notes()
    {
        value_note note1 = { user.owner.public_key, 100, user.note_secret, 0 };
        value_note note2 = { user.owner.public_key, 50, user.note_secret, 0 };

        auto enc_note1 = encrypt_note(note1);
        tree->update_element(tree->size(), create_leaf_data(enc_note1));

        auto enc_note2 = encrypt_note(note2);
        tree->update_element(tree->size(), create_leaf_data(enc_note2));
    }

    join_split_tx create_join_split_tx(std::array<uint32_t, 2> const& input_indicies, uint32_t account_index)
    {
        value_note input_note1 = { user.owner.public_key, 100, user.note_secret, 0 };
        value_note input_note2 = { user.owner.public_key, 50, user.note_secret, 0 };
        value_note output_note1 = { user.owner.public_key, 70, user.note_secret, 0 };
        value_note output_note2 = { user.owner.public_key, 80, user.note_secret, 0 };

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
        return tx;
    }

    /**
     * Add account notes and value notes (sum 150).
     * Return a join split tx that spends them.
     */
    join_split_tx simple_setup()
    {
        preload_account_notes();
        preload_value_notes();
        return create_join_split_tx({ 2, 3 }, 0);
    }

    /**
     * Return a join split tx that performs a 100 token transfer.
     */
    join_split_tx public_transfer_setup()
    {
        value_note input_note1 = { user.owner.public_key, 0, user.note_secret, 0 };
        value_note input_note2 = { user.owner.public_key, 0, user.note_secret, 0 };
        value_note output_note1 = { user.owner.public_key, 0, user.note_secret, 0 };
        value_note output_note2 = { user.owner.public_key, 0, user.note_secret, 0 };

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
        return tx;
    }

    bool sign_and_verify(join_split_tx& tx, grumpkin::fr const& signing_private_key)
    {
        tx.signature = sign_notes({ tx.input_note[0], tx.input_note[1], tx.output_note[0], tx.output_note[1] },
                                  tx.output_owner,
                                  { signing_private_key, tx.signing_pub_key });

        auto prover = new_join_split_prover(tx);
        auto proof = prover.construct_proof();
        return verify_proof(proof);
    }

    bool sign_and_verify_logic(join_split_tx& tx, grumpkin::fr const& signing_private_key)
    {
        tx.signature = sign_notes({ tx.input_note[0], tx.input_note[1], tx.output_note[0], tx.output_note[1] },
                                  tx.output_owner,
                                  { signing_private_key, tx.signing_pub_key });

        Composer composer(get_proving_key(), nullptr);
        join_split_circuit(composer, tx);
        return !composer.failed;
    }

    rollup::fixtures::user_context user;
    std::unique_ptr<MemoryStore> store;
    std::unique_ptr<MerkleTree<MemoryStore>> tree;
};

TEST_F(join_split_tests, test_0_input_notes)
{
    value_note gibberish = { user.owner.public_key, 0, user.note_secret, 0 };

    join_split_tx tx = simple_setup();
    tx.public_input = 150;
    tx.num_input_notes = 0;
    tx.input_note = { gibberish, gibberish };

    EXPECT_TRUE(sign_and_verify_logic(tx, user.signing_keys[0].private_key));
}

TEST_F(join_split_tests, test_large_output_note)
{
    auto deposit_value = (uint256_t(1) << 252) - 1;

    value_note gibberish = { user.owner.public_key, 0, user.note_secret, 0 };
    value_note output_note1 = { user.owner.public_key, deposit_value, user.note_secret, 0 };
    value_note output_note2 = { user.owner.public_key, 0, user.note_secret, 0 };

    join_split_tx tx = simple_setup();
    tx.public_input = deposit_value;
    tx.public_output = 0;
    tx.num_input_notes = 0;
    tx.input_note = { gibberish, gibberish };
    tx.output_note = { output_note1, output_note2 };

    EXPECT_TRUE(sign_and_verify_logic(tx, user.signing_keys[0].private_key));
}

TEST_F(join_split_tests, test_1_input_notes_with_account_key_signer)
{
    preload_value_notes();
    join_split_tx tx = create_join_split_tx({ 0, 1 }, 0);
    tx.num_input_notes = 1;
    tx.input_note[1].value = 0;
    tx.output_note[1].value = 30;
    tx.signing_pub_key = tx.input_note[0].owner;

    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner.private_key));
}

TEST_F(join_split_tests, test_2_input_notes)
{
    join_split_tx tx = simple_setup();
    EXPECT_TRUE(sign_and_verify_logic(tx, user.signing_keys[0].private_key));
}

TEST_F(join_split_tests, test_wrong_output_owner_sig_fail)
{
    join_split_tx tx = simple_setup();

    // sign with correct output owner
    tx.signature = sign_notes({ tx.input_note[0], tx.input_note[1], tx.output_note[0], tx.output_note[1] },
                              tx.output_owner,
                              { user.signing_keys[0].private_key, tx.signing_pub_key });

    // set a fake output owner
    auto fake_owner = fr::random_element();
    tx.output_owner = fake_owner;

    Composer composer(get_proving_key(), nullptr);
    join_split_circuit(composer, tx);

    bool verified = !composer.failed;
    EXPECT_FALSE(verified);
}

TEST_F(join_split_tests, test_zero_output_owner)
{
    join_split_tx tx = simple_setup();

    tx.output_owner = fr::zero();

    tx.signature = sign_notes({ tx.input_note[0], tx.input_note[1], tx.output_note[0], tx.output_note[1] },
                              tx.output_owner,
                              { user.signing_keys[0].private_key, tx.signing_pub_key });

    Composer composer(get_proving_key(), nullptr);
    join_split_circuit(composer, tx);

    bool verified = !composer.failed;
    EXPECT_TRUE(verified);
}

HEAVY_TEST_F(join_split_tests, test_2_input_notes_full_proof)
{
    join_split_tx tx = simple_setup();
    EXPECT_TRUE(sign_and_verify(tx, user.signing_keys[0].private_key));
}

TEST_F(join_split_tests, test_0_output_notes)
{
    join_split_tx tx = simple_setup();
    tx.output_note[0].value = 0;
    tx.output_note[1].value = 0;
    tx.public_output = 150;

    EXPECT_TRUE(sign_and_verify_logic(tx, user.signing_keys[0].private_key));
}

TEST_F(join_split_tests, test_0_notes_with_balanced_public_values)
{
    join_split_tx tx = public_transfer_setup();
    EXPECT_TRUE(sign_and_verify_logic(tx, user.signing_keys[0].private_key));
}

TEST_F(join_split_tests, test_0_input_notes_with_unbalanced_public_values)
{
    join_split_tx tx = public_transfer_setup();
    tx.public_input = 120;
    tx.output_note[0].value = 20;

    EXPECT_TRUE(sign_and_verify_logic(tx, user.signing_keys[0].private_key));
}

TEST_F(join_split_tests, test_2_input_notes_with_unbalanced_public_values)
{
    join_split_tx tx = simple_setup();
    tx.output_note[0].value = 80;
    tx.output_note[1].value = 0;
    tx.public_input = 100;
    tx.public_output = 170;

    EXPECT_TRUE(sign_and_verify_logic(tx, user.signing_keys[0].private_key));
}

TEST_F(join_split_tests, test_joining_same_note_fails)
{
    join_split_tx tx = simple_setup();
    tx.input_note[0].value = 100;
    tx.input_note[1].value = 100;
    tx.output_note[0].value = 200;
    tx.output_note[1].value = 0;
    tx.input_index = { 2, 2 };

    EXPECT_FALSE(sign_and_verify_logic(tx, user.signing_keys[0].private_key));
}

TEST_F(join_split_tests, test_spending_1_note_with_non_0_value_second_note_fails)
{
    join_split_tx tx = simple_setup();
    tx.num_input_notes = 1;
    tx.input_note[0].value = 100;
    tx.input_note[1].value = 100;
    tx.output_note[0].value = 100;
    tx.output_note[1].value = 100;

    EXPECT_FALSE(sign_and_verify_logic(tx, user.signing_keys[0].private_key));
}

TEST_F(join_split_tests, test_unbalanced_notes_fails)
{
    join_split_tx tx = simple_setup();
    tx.input_note[1].value = 51;

    EXPECT_FALSE(sign_and_verify_logic(tx, user.signing_keys[0].private_key));
}

TEST_F(join_split_tests, test_0_notes_with_unbalanced_public_values_fails)
{
    join_split_tx tx = public_transfer_setup();
    tx.public_input = 120;

    EXPECT_FALSE(sign_and_verify_logic(tx, user.signing_keys[0].private_key));
}

TEST_F(join_split_tests, test_wrong_input_note_owner_fails)
{
    join_split_tx tx = simple_setup();
    tx.input_note[1].owner = grumpkin::g1::element::random_element();

    EXPECT_FALSE(sign_and_verify_logic(tx, user.signing_keys[0].private_key));
}

TEST_F(join_split_tests, test_random_output_owners_succeeds)
{
    join_split_tx tx = simple_setup();
    tx.output_note[0].owner = grumpkin::g1::element::random_element();
    tx.output_note[1].owner = grumpkin::g1::element::random_element();

    EXPECT_TRUE(sign_and_verify_logic(tx, user.signing_keys[0].private_key));
}

TEST_F(join_split_tests, test_wrong_hash_path_fails)
{
    join_split_tx tx = simple_setup();
    tx.input_path[1] = tree->get_hash_path(0);

    EXPECT_FALSE(sign_and_verify_logic(tx, user.signing_keys[0].private_key));
}

TEST_F(join_split_tests, test_wrong_merkle_root_fails)
{
    join_split_tx tx = simple_setup();
    tx.old_data_root = fr::random_element();

    EXPECT_FALSE(sign_and_verify_logic(tx, user.signing_keys[0].private_key));
}

TEST_F(join_split_tests, test_alternative_signing_key)
{
    join_split_tx tx = simple_setup();
    tx.account_index = 1;
    tx.signing_pub_key = user.signing_keys[1].public_key;
    EXPECT_TRUE(sign_and_verify_logic(tx, user.signing_keys[1].private_key));
}

TEST_F(join_split_tests, test_signing_key_equal_account_key_disables_account_check)
{
    preload_value_notes();
    auto tx = create_join_split_tx({ 0, 1 }, 0);
    tx.signing_pub_key = user.owner.public_key;
    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner.private_key));
}

TEST_F(join_split_tests, test_wrong_account_private_key_fails)
{
    join_split_tx tx = simple_setup();
    tx.account_private_key = grumpkin::fr::random_element();
    EXPECT_FALSE(sign_and_verify_logic(tx, user.signing_keys[0].private_key));
}

TEST_F(join_split_tests, test_wrong_signature_fails)
{
    join_split_tx tx = simple_setup();
    EXPECT_FALSE(sign_and_verify_logic(tx, user.signing_keys[1].private_key));
}

HEAVY_TEST_F(join_split_tests, test_tainted_output_owner_fails)
{
    join_split_tx tx = simple_setup();
    tx.signing_pub_key = user.owner.public_key;
    tx.signature = sign_notes({ tx.input_note[0], tx.input_note[1], tx.output_note[0], tx.output_note[1] },
                              tx.output_owner,
                              { user.owner.private_key, user.owner.public_key });
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
