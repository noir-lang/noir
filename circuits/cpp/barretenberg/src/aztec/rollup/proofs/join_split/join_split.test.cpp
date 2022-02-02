#include "../../constants.hpp"
#include "../inner_proof_data/inner_proof_data.hpp"
#include "index.hpp"
#include "../notes/native/index.hpp"
#include <common/streams.hpp>
#include <common/test.hpp>
#include <stdlib/merkle_tree/index.hpp>

namespace rollup {
namespace proofs {
namespace join_split {

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::merkle_tree;
using namespace rollup::proofs::notes::native;

auto create_account_leaf_data(fr const& account_alias_id,
                              grumpkin::g1::affine_element const& owner_key,
                              grumpkin::g1::affine_element const& signing_key)
{
    return notes::native::account::account_note{ account_alias_id, owner_key, signing_key }.commit();
}

class join_split_tests : public ::testing::Test {
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
        user = rollup::fixtures::create_user_context();
        value_notes[0] = {
            100, asset_id, 0, user.owner.public_key, user.note_secret, user.owner.public_key.x, fr::random_element()
        };
        value_notes[1] = { 50,
                           asset_id,
                           0,
                           user.owner.public_key,
                           user.note_secret,
                           rollup::fixtures::create_key_pair(nullptr).public_key.x,
                           fr::random_element() };
        value_notes[2] = { 90, asset_id, 1, user.owner.public_key, user.note_secret, 0, fr::random_element() };
        value_notes[3] = { 40, asset_id, 1, user.owner.public_key, user.note_secret, 0, fr::random_element() };

        const uint32_t virtual_asset_id = (uint32_t(1) << (MAX_NUM_ASSETS_BIT_LENGTH - 1)) + defi_interaction_nonce;
        value_notes[4] = { 100, virtual_asset_id, 0, user.owner.public_key, user.note_secret, 0, fr::random_element() };
        value_notes[5] = { 30, virtual_asset_id, 0, user.owner.public_key, user.note_secret, 0, fr::random_element() };
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
     * Add two value notes with nonce 0, and two value notes with nonce 1.
     */
    void preload_value_notes()
    {
        for (auto note : value_notes) {
            tree->update_element(tree->size(), note.commit());
        }
    }

    void append_notes(std::vector<value::value_note> const& notes)
    {
        for (auto note : notes) {
            tree->update_element(tree->size(), note.commit());
        }
    }

    /**
     * Given 2 input notes, outputs a single output note that is the sum of both.
     */
    join_split_tx create_join_split_tx(std::array<uint32_t, 2> const& input_indicies,
                                       std::array<value::value_note, 2> const& input_notes,
                                       uint32_t tx_asset_id,
                                       uint32_t account_index = 0,
                                       uint32_t nonce = 0)
    {
        auto input_nullifier1 = compute_nullifier(input_notes[0].commit(), user.owner.private_key, true);
        auto input_nullifier2 = compute_nullifier(input_notes[1].commit(), user.owner.private_key, true);
        value::value_note output_note1 = { input_notes[0].value + input_notes[1].value,
                                           tx_asset_id,
                                           nonce,
                                           user.owner.public_key,
                                           user.note_secret,
                                           0,
                                           input_nullifier1 };
        value::value_note output_note2 = { 0, tx_asset_id,     nonce, user.owner.public_key, user.note_secret,
                                           0, input_nullifier2 };

        join_split_tx tx;
        tx.proof_id = ProofIds::SEND;
        tx.public_value = 0;
        tx.num_input_notes = 2;
        tx.input_index = input_indicies;
        tx.old_data_root = tree->root();
        tx.input_path = { tree->get_hash_path(input_indicies[0]), tree->get_hash_path(input_indicies[1]) };
        tx.input_note = input_notes;
        tx.output_note = { output_note1, output_note2 };
        tx.public_owner = fr(0);
        tx.account_index = account_index;
        tx.account_path = tree->get_hash_path(account_index);
        tx.signing_pub_key = user.signing_keys[0].public_key;
        tx.asset_id = tx_asset_id;
        tx.account_private_key = user.owner.private_key;
        tx.partial_claim_note.input_nullifier = 0;
        tx.alias_hash = !nonce ? rollup::fixtures::generate_alias_hash("penguin") : user.alias_hash;
        tx.nonce = nonce;
        // default to no chaining:
        tx.backward_link = 0;
        tx.allow_chain = 0;
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
        // The tree, user and {value, virtual}_notes are initialised in SetUp().
        preload_value_notes();   // indicies: [0, 1](nonce 0), [2, 3](nonce 1), [4, 5](nonce 0, virtual_notes)
        preload_account_notes(); // indicies: [6, 7]
        return create_join_split_tx(input_indicies,
                                    { value_notes[input_indicies[0]], value_notes[input_indicies[1]] },
                                    asset_id,
                                    account_index,
                                    nonce);
    }

    join_split_tx setup_two_virtual_input_notes()
    {
        join_split_tx tx = simple_setup({ 4, 5 });
        tx.output_note[0].asset_id = tx.input_note[0].asset_id;
        tx.output_note[1].asset_id = tx.input_note[1].asset_id;
        tx.asset_id = tx.input_note[0].asset_id;
        return tx;
    }

    join_split_tx setup_defi_case_5()
    {
        join_split_tx tx = simple_setup({ 0, 4 });
        tx.proof_id = ProofIds::DEFI_DEPOSIT;
        tx.partial_claim_note.deposit_value = 90;
        tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

        bridge_id bridge_id = { 0,
                                tx.asset_id,
                                0,
                                0,
                                defi_interaction_nonce,
                                { .first_input_virtual = false,
                                  .second_input_virtual = true,
                                  .first_output_virtual = false,
                                  .second_output_virtual = false,
                                  .second_input_real = false,
                                  .second_output_real = false } };
        tx.partial_claim_note.bridge_id = bridge_id.to_uint256_t();

        return tx;
    }

    /**
     * Return a join split tx with 0-valued input notes.
     */
    join_split_tx zero_input_setup()
    {
        value::value_note input_note1 = { 0, 0, 0, user.owner.public_key, user.note_secret, 0, fr::random_element() };
        value::value_note input_note2 = { 0, 0, 0, user.owner.public_key, user.note_secret, 0, fr::random_element() };
        auto input_nullifier1 = compute_nullifier(input_note1.commit(), user.owner.private_key, false);
        auto input_nullifier2 = compute_nullifier(input_note2.commit(), user.owner.private_key, false);
        value::value_note output_note1 = { 0, 0, 0, user.owner.public_key, user.note_secret, 0, input_nullifier1 };
        value::value_note output_note2 = { 0, 0, 0, user.owner.public_key, user.note_secret, 0, input_nullifier2 };

        join_split_tx tx;
        tx.proof_id = ProofIds::SEND;
        tx.public_value = 0;
        tx.public_owner = 0;
        tx.asset_id = 0;
        tx.num_input_notes = 0;
        tx.input_index = { 0, 1 };
        tx.old_data_root = tree->root();
        tx.input_path = { tree->get_hash_path(0), tree->get_hash_path(1) };
        tx.input_note = { input_note1, input_note2 };
        tx.output_note = { output_note1, output_note2 };
        tx.partial_claim_note.input_nullifier = 0;
        tx.account_private_key = user.owner.private_key;
        tx.alias_hash = rollup::fixtures::generate_alias_hash("penguin");
        tx.nonce = 0;
        tx.account_index = 0;
        tx.account_path = tree->get_hash_path(0);
        tx.signing_pub_key = user.signing_keys[0].public_key;
        tx.backward_link = 0;
        tx.allow_chain = 0;
        return tx;
    }

    waffle::plonk_proof sign_and_create_proof(join_split_tx& tx, grumpkin::fr const& signing_private_key)
    {
        tx.signature = sign_join_split_tx(tx, { signing_private_key, tx.signing_pub_key });

        auto composer = new_join_split_composer(tx);
        auto prover = composer.create_unrolled_prover();
        return prover.construct_proof();
    }

    bool sign_and_verify(join_split_tx& tx, grumpkin::fr const& signing_private_key)
    {
        return verify_proof(sign_and_create_proof(tx, signing_private_key));
    }

    struct verify_result {
        bool valid;
        std::string err;
        std::vector<fr> public_inputs;
    };

    verify_result verify_logic(join_split_tx& tx)
    {
        Composer composer(get_proving_key(), nullptr);
        join_split_circuit(composer, tx);
        if (composer.failed) {
            std::cout << "Logic failed: " << composer.err << std::endl;
        }
        return { !composer.failed, composer.err, composer.get_public_inputs() };
    }

    verify_result sign_and_verify_logic(join_split_tx& tx, grumpkin::fr const& signing_private_key)
    {
        tx.signature = sign_join_split_tx(tx, { signing_private_key, tx.signing_pub_key });
        return verify_logic(tx);
    }

    rollup::fixtures::user_context user;
    std::unique_ptr<MemoryStore> store;
    std::unique_ptr<MerkleTree<MemoryStore>> tree;
    value::value_note value_notes[6];
    value::value_note dummy_value_notes[2];
    const uint32_t asset_id = 1;
    const uint32_t defi_interaction_nonce = 7;
    const uint256_t max_value = notes::NOTE_VALUE_MAX;
};

TEST_F(join_split_tests, test_0_input_notes)
{
    join_split_tx tx = zero_input_setup();
    tx.proof_id = ProofIds::DEPOSIT;
    tx.public_value = 30;
    tx.public_owner = fr::random_element();
    tx.output_note[0].value = 30;

    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner.private_key).valid);
}

TEST_F(join_split_tests, test_0_input_notes_no_dupicate_output_notes)
{
    join_split_tx tx = zero_input_setup();

    tx.proof_id = ProofIds::DEPOSIT;
    tx.public_value = 30;
    tx.output_note[0].value = 15;
    tx.output_note[1] = tx.output_note[0];
    tx.input_note[1] = tx.input_note[0];

    EXPECT_FALSE(sign_and_verify_logic(tx, user.owner.private_key).valid);
}

TEST_F(join_split_tests, test_padding_input_note_non_0_value_fails)
{
    join_split_tx tx = zero_input_setup();
    tx.proof_id = ProofIds::DEPOSIT;
    tx.public_value = 10;
    tx.public_owner = fr::random_element();
    tx.input_note[0].value = 10;
    tx.output_note[0].value = 20;

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "padding note non zero");
}

TEST_F(join_split_tests, test_1_input_note)
{
    join_split_tx tx = simple_setup();
    tx.num_input_notes = 1;
    tx.input_note[1].value = 0;
    tx.output_note[0].value = tx.input_note[0].value;
    tx.output_note[1].input_nullifier = compute_nullifier(tx.input_note[1].commit(), user.owner.private_key, false);

    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner.private_key).valid);
}

TEST_F(join_split_tests, test_1_input_note_with_num_inputs_as_100)
{
    // A pedantic test. It's a bit weird that setting num_inputs > 2 is treated as num_inputs == 1 by the circuit.
    join_split_tx tx = simple_setup();
    tx.num_input_notes = 100;
    tx.input_note[1].value = 0;
    tx.output_note[0].value = tx.input_note[0].value;
    tx.output_note[1].input_nullifier = compute_nullifier(tx.input_note[1].commit(), user.owner.private_key, false);

    EXPECT_FALSE(sign_and_verify_logic(tx, user.owner.private_key).valid);
}

TEST_F(join_split_tests, test_1_input_note_with_num_input_notes_as_0)
{
    join_split_tx tx = simple_setup();
    tx.num_input_notes = 0; // try to trick the circuit into creating a second option for a nullifier where is_real = 0.
    // tx.input_note[0] is nonzero - we're going to try to spend it with a cheeky nullifier
    tx.input_note[1].value = 0;
    tx.output_note[0].value = tx.input_note[0].value;
    // create a cheeky nullifier for tx.input_note[0] where is_real = false
    tx.output_note[0].input_nullifier = compute_nullifier(tx.input_note[0].commit(), user.owner.private_key, false);
    tx.output_note[1].input_nullifier = compute_nullifier(tx.input_note[1].commit(), user.owner.private_key, false);

    EXPECT_FALSE(sign_and_verify_logic(tx, user.owner.private_key).valid);
}

TEST_F(join_split_tests, test_2_input_notes)
{
    join_split_tx tx = simple_setup();
    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_TRUE(result.valid);
    EXPECT_EQ(result.public_inputs.size(), InnerProofFields::NUM_FIELDS);
}

TEST_F(join_split_tests, test_0_output_notes)
{
    join_split_tx tx = simple_setup();
    tx.proof_id = ProofIds::WITHDRAW;
    tx.output_note[0].value = 0;
    tx.output_note[1].value = 0;
    tx.public_value = tx.input_note[0].value + tx.input_note[1].value;
    tx.public_owner = fr::random_element();

    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner.private_key).valid);
}

// Chaining
class test_valid_allow_chain_permutations : public join_split_tests, public ::testing::WithParamInterface<uint32_t> {};
TEST_P(test_valid_allow_chain_permutations, )
{
    join_split_tx tx = simple_setup();
    // sending to self is implied here, by the fixture's default values
    tx.allow_chain = GetParam();
    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_TRUE(result.valid);
    EXPECT_EQ(result.public_inputs[InnerProofFields::ALLOW_CHAIN], GetParam());
}
INSTANTIATE_TEST_SUITE_P(join_split_tests, test_valid_allow_chain_permutations, ::testing::Values(0, 1, 2, 3));

TEST_F(join_split_tests, test_allow_chain_out_of_range_fails)
{
    join_split_tx tx = simple_setup();
    tx.backward_link = fr::random_element(); // choose a value unrelated to the inputs being spent
    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "backward_link unrelated to inputs");
}

TEST_F(join_split_tests, test_unrelated_backward_link_fails)
{
    join_split_tx tx = simple_setup();
    tx.allow_chain = 4;
    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "allow_chain out of range");
}

class test_allow_chain_to_other_users_fail : public join_split_tests, public ::testing::WithParamInterface<uint32_t> {};
TEST_P(test_allow_chain_to_other_users_fail, )
{
    join_split_tx tx = simple_setup();
    tx.allow_chain = GetParam();
    tx.output_note[tx.allow_chain - 1].owner = grumpkin::g1::element::random_element(); // i.e. not owned by self.
    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "inter-user chaining disallowed");
}
INSTANTIATE_TEST_SUITE_P(join_split_tests, test_allow_chain_to_other_users_fail, ::testing::Values(1, 2));

void assign_backward_link(join_split_tx& tx, size_t& indicator)
{
    switch (indicator) {
    case 0:
        tx.backward_link = 0;
        break;
    case 1:
        tx.backward_link = tx.input_note[0].commit();
        break;
    case 2:
        tx.backward_link = tx.input_note[1].commit();
        break;
    default:
        tx.backward_link = barretenberg::fr::random_element();
    }
}

class test_propagated_notes_skip_membership_check : public join_split_tests,
                                                    public ::testing::WithParamInterface<size_t> {};
TEST_P(test_propagated_notes_skip_membership_check, )
{
    join_split_tx tx = simple_setup();
    size_t indicator = GetParam();
    assign_backward_link(tx, indicator);
    tx.input_path[indicator - 1] =
        tree->get_hash_path(99); // select a clearly incorrect path for the input note being propagated.
    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_TRUE(result.valid);
}
INSTANTIATE_TEST_SUITE_P(join_split_tests, test_propagated_notes_skip_membership_check, ::testing::Values(1, 2));

TEST_F(join_split_tests, test_propagated_input_note1_no_double_spend)
{
    join_split_tx tx = simple_setup();
    tx.backward_link = value_notes[0].commit();

    // Let's try to double-spend input_note[0]:
    tx.input_note[1] = tx.input_note[0];

    tx.output_note[0] = { tx.input_note[0].value,           tx.input_note[0].asset_id, tx.input_note[0].nonce,
                          tx.input_note[0].owner,           tx.input_note[0].secret,   tx.input_note[0].creator_pubkey,
                          tx.output_note[0].input_nullifier };
    tx.output_note[1] = { tx.input_note[0].value,           tx.input_note[0].asset_id,
                          tx.input_note[0].nonce,           tx.input_note[0].owner,
                          tx.input_note[0].secret + 1,      tx.input_note[0].creator_pubkey,
                          tx.output_note[0].input_nullifier };

    auto result = sign_and_verify_logic(tx, user.owner.private_key);

    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "joining same note");
}

// Public values
TEST_F(join_split_tests, test_max_public_input)
{
    join_split_tx tx = zero_input_setup();
    tx.proof_id = ProofIds::DEPOSIT;
    tx.public_value = max_value;
    tx.output_note[0].value = max_value;
    tx.public_owner = fr::random_element();

    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner.private_key).valid);
}

TEST_F(join_split_tests, test_overflow_public_value_fails)
{
    join_split_tx tx = zero_input_setup();
    tx.proof_id = ProofIds::DEPOSIT;
    tx.public_value = max_value + 1;
    tx.public_owner = fr::random_element();

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "safe_uint_t range constraint failure: public_value");
}

// Tx fee
TEST_F(join_split_tests, test_non_zero_tx_fee)
{
    join_split_tx tx = zero_input_setup();
    tx.proof_id = ProofIds::DEPOSIT;
    tx.public_value += 10;
    tx.public_owner = fr::random_element();

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_TRUE(result.valid);
    EXPECT_EQ(result.public_inputs[InnerProofFields::TX_FEE], 10);
}

TEST_F(join_split_tests, test_non_zero_tx_fee_zero_public_values)
{
    join_split_tx tx = simple_setup();
    tx.output_note[0].value -= 10;

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_TRUE(result.valid);
    EXPECT_EQ(result.public_inputs[InnerProofFields::TX_FEE], 10);
}

TEST_F(join_split_tests, test_max_tx_fee)
{
    join_split_tx tx = simple_setup();
    auto tx_fee = (uint256_t(1) << rollup::TX_FEE_BIT_LENGTH) - 1;
    tx.proof_id = ProofIds::DEPOSIT;
    tx.public_value += tx_fee;
    tx.public_owner = fr::random_element();

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_TRUE(result.valid);
    EXPECT_EQ(result.public_inputs[InnerProofFields::TX_FEE], fr(tx_fee));
}

TEST_F(join_split_tests, test_overflow_tx_fee_fails)
{
    join_split_tx tx = simple_setup();
    auto tx_fee = uint256_t(1) << rollup::TX_FEE_BIT_LENGTH;
    tx.proof_id = ProofIds::DEPOSIT;
    tx.public_value += tx_fee;
    tx.public_owner = fr::random_element();

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "safe_uint_t range constraint failure: subtract: total_in_value < total_out_value");
}

TEST_F(join_split_tests, test_total_output_value_larger_than_total_input_value_fails)
{
    join_split_tx tx = simple_setup();
    tx.output_note[0].value += 1;

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "safe_uint_t range constraint failure: subtract: total_in_value < total_out_value");
}

// Asset id
TEST_F(join_split_tests, test_wrong_asset_id_fails)
{
    join_split_tx tx = simple_setup();
    tx.asset_id = 3;

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "asset ids don't match");
}

TEST_F(join_split_tests, test_different_input_note_asset_id_fails)
{
    join_split_tx tx = simple_setup();
    tx.input_note[0].asset_id = 3;

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "asset ids don't match");
}

TEST_F(join_split_tests, test_different_output_note_asset_id_fails)
{
    join_split_tx tx = simple_setup();
    tx.output_note[0].asset_id = 3;

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "asset ids don't match");
}

TEST_F(join_split_tests, test_different_input_output_asset_id_fails)
{
    join_split_tx tx = simple_setup();
    tx.output_note[0].asset_id = 3;
    tx.output_note[1].asset_id = 3;

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "asset ids don't match");
}

// Input note
TEST_F(join_split_tests, test_joining_same_note_fails)
{
    join_split_tx tx = simple_setup({ 1, 1 });
    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "joining same note");
}

TEST_F(join_split_tests, test_different_input_note_nonces_fails)
{
    join_split_tx tx = simple_setup({ 1, 2 });

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "input note nonces don't match");
}

// Input note account value id
TEST_F(join_split_tests, test_spend_notes_with_registered_account)
{
    join_split_tx tx = simple_setup({ 2, 3 }, 6, 1);
    EXPECT_TRUE(sign_and_verify_logic(tx, user.signing_keys[0].private_key).valid);
}

TEST_F(join_split_tests, test_different_note_nonce_vs_account_nonce_fails)
{
    join_split_tx tx = simple_setup({ 2, 3 }, 6, 0);
    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "nonce incorrect");
}

TEST_F(join_split_tests, test_wrong_input_note_owner_fails)
{
    join_split_tx tx = simple_setup();
    tx.input_note[0].owner = grumpkin::g1::element::random_element();
    tx.input_note[1].owner = tx.input_note[0].owner;

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "account_private_key incorrect");
}

// Output note owner
TEST_F(join_split_tests, test_random_output_note_owners)
{
    join_split_tx tx = simple_setup();
    tx.output_note[0].owner = grumpkin::g1::element::random_element();
    tx.output_note[1].owner = grumpkin::g1::element::random_element();

    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner.private_key).valid);
}

// Signature
TEST_F(join_split_tests, test_wrong_account_private_key_fails)
{
    join_split_tx tx = simple_setup();
    tx.account_private_key = grumpkin::fr::random_element();

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "account_private_key incorrect");
}

TEST_F(join_split_tests, test_wrong_public_owner_sig_fail)
{
    join_split_tx tx = simple_setup();
    tx.proof_id = ProofIds::DEPOSIT;
    tx.public_value = 1;
    tx.public_owner = fr::random_element();

    // sign over a different public owner
    tx.signature = sign_join_split_tx(tx, { user.owner.private_key, tx.signing_pub_key });

    tx.public_owner = fr::random_element();

    auto result = verify_logic(tx);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "verify signature failed");
}

TEST_F(join_split_tests, test_spend_zero_nonce_notes_with_signing_key_fails)
{
    join_split_tx tx = simple_setup();
    auto result = sign_and_verify_logic(tx, user.signing_keys[0].private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "verify signature failed");
}

TEST_F(join_split_tests, test_spend_registered_notes_with_owner_key_fails)
{
    auto tx = simple_setup({ 2, 3 }, 6, 1);
    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "verify signature failed");
}

// Account membership
TEST_F(join_split_tests, test_wrong_merkle_root_fails)
{
    join_split_tx tx = simple_setup();
    tx.old_data_root = fr::random_element();

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "input note not a member");
}

TEST_F(join_split_tests, test_wrong_alias_hash_fails)
{
    join_split_tx tx = simple_setup({ 2, 3 }, 6, 1);
    tx.alias_hash = rollup::fixtures::generate_alias_hash("chicken");

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "account check_membership failed");
}

TEST_F(join_split_tests, test_nonregistered_signing_key_fails)
{
    join_split_tx tx = simple_setup({ 2, 3 }, 6, 1);
    auto keys = rollup::fixtures::create_key_pair(nullptr);
    tx.signing_pub_key = keys.public_key;

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "account check_membership failed");
}

TEST_F(join_split_tests, test_wrong_note_hash_path_fails)
{
    join_split_tx tx = simple_setup();
    auto gibberish_path = fr_hash_path(32, std::make_pair(fr::random_element(), fr::random_element()));
    tx.input_path[0] = gibberish_path;

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "input note not a member");
}

TEST_F(join_split_tests, test_tainted_output_owner_fails)
{
    join_split_tx tx = simple_setup();
    tx.proof_id = ProofIds::DEPOSIT;
    tx.public_value = 1;
    tx.signing_pub_key = user.owner.public_key;
    uint8_t public_owner[32] = { 0x01, 0xaa, 0x42, 0xd4, 0x72, 0x88, 0x8e, 0xae, 0xa5, 0x56, 0x39,
                                 0x46, 0xeb, 0x5c, 0xf5, 0x6c, 0x81, 0x6,  0x4d, 0x80, 0xc6, 0xf5,
                                 0xa5, 0x38, 0xcc, 0x87, 0xae, 0x54, 0xae, 0xdb, 0x75, 0xd9 };
    tx.public_owner = from_buffer<fr>(public_owner);
    tx.signature = sign_join_split_tx(tx, { user.owner.private_key, user.owner.public_key });

    auto composer = new_join_split_composer(tx);
    auto prover = composer.create_unrolled_prover();
    auto proof = prover.construct_proof();

    EXPECT_EQ(proof.proof_data[InnerProofOffsets::PUBLIC_OWNER], 0x01);
    proof.proof_data[InnerProofFields::PUBLIC_OWNER] = 0x02;

    EXPECT_FALSE(verify_proof(proof));
}

TEST_F(join_split_tests, test_invalid_bridge_id)
{
    join_split_tx tx = simple_setup();
    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 1;

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "asset ids don't match");
}

TEST_F(join_split_tests, test_defi_deposit)
{
    join_split_tx tx = setup_defi_case_5();

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_TRUE(result.valid);
}

TEST_F(join_split_tests, test_defi_invalid_tx_fee_asset_id_fails)
{
    join_split_tx tx = setup_defi_case_5();
    tx.asset_id = 666;

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "asset ids don't match");
}

TEST_F(join_split_tests, test_defi_deposit_of_zero_fails)
{
    join_split_tx tx = simple_setup();
    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.output_note[0].value = 0;
    tx.output_note[1].value = 0;
    tx.partial_claim_note.deposit_value = 0;

    bridge_id bridge_id = { .bridge_address_id = 0,
                            .input_asset_id = tx.asset_id,
                            .output_asset_id_a = 111,
                            .output_asset_id_b = 0,
                            .opening_nonce = 0,
                            .config = bridge_id::bit_config{ .first_input_virtual = false,
                                                             .second_input_virtual = false,
                                                             .first_output_virtual = false,
                                                             .second_output_virtual = false,
                                                             .second_input_real = false,
                                                             .second_output_real = false },
                            .aux_data = 0 };
    tx.partial_claim_note.bridge_id = bridge_id.to_uint256_t();

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "defi deposit value of zero not allowed");
}

TEST_F(join_split_tests, test_defi_non_zero_public_value_fails)
{
    join_split_tx tx = simple_setup();
    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.output_note[0].value = 0;
    tx.output_note[1].value = 100;
    tx.partial_claim_note.deposit_value = 50;
    tx.public_value = 1;

    bridge_id bridge_id = { 0,
                            tx.asset_id,
                            0,
                            0,
                            0,
                            bridge_id::bit_config{ .first_input_virtual = false,
                                                   .second_input_virtual = false,
                                                   .first_output_virtual = false,
                                                   .second_output_virtual = false,
                                                   .second_input_real = false,
                                                   .second_output_real = false },
                            0 };
    tx.partial_claim_note.bridge_id = bridge_id.to_uint256_t();

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "public value incorrect");
}

TEST_F(join_split_tests, test_defi_non_zero_output_note_1_ignored)
{
    join_split_tx tx = simple_setup();
    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.output_note[0].value = 10; // This should be ignored in fee calculation!
    tx.output_note[1].value = 100;
    tx.partial_claim_note.deposit_value = 50;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_id bridge_id = { 0,
                            tx.asset_id,
                            0,
                            0,
                            0,
                            { .first_input_virtual = false,
                              .second_input_virtual = false,
                              .first_output_virtual = false,
                              .second_output_virtual = false,
                              .second_input_real = false,
                              .second_output_real = false } };
    tx.partial_claim_note.bridge_id = bridge_id.to_uint256_t();

    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner.private_key).valid);
}

TEST_F(join_split_tests, test_defi_allow_chain_1_fails)
{
    join_split_tx tx = simple_setup();
    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.output_note[1].value = 100;
    tx.partial_claim_note.deposit_value = 50;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;
    bridge_id bridge_id = { 0,
                            tx.asset_id,
                            0,
                            0,
                            0,
                            { .first_input_virtual = false,
                              .second_input_virtual = false,
                              .first_output_virtual = false,
                              .second_output_virtual = false,
                              .second_input_real = false,
                              .second_output_real = false } };
    tx.partial_claim_note.bridge_id = bridge_id.to_uint256_t();

    tx.allow_chain = 1;

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "cannot chain from a partial claim note");
}

// Virtual note repayment.
TEST_F(join_split_tests, test_repayment_logic)
{
    join_split_tx tx = simple_setup({ 0, 4 });
    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_id bridge_id = { 0,
                            tx.asset_id,
                            0,
                            0,
                            defi_interaction_nonce,
                            { .first_input_virtual = false,
                              .second_input_virtual = true,
                              .first_output_virtual = false,
                              .second_output_virtual = false,
                              .second_input_real = false,
                              .second_output_real = false } };
    tx.partial_claim_note.bridge_id = bridge_id.to_uint256_t();

    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner.private_key).valid);
}

TEST_F(join_split_tests, test_virtual_note_repay_different_asset_fail)
{
    join_split_tx tx = simple_setup({ 0, 4 });
    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;
    tx.output_note[1].asset_id = 3;

    bridge_id bridge_id = { 0,
                            tx.asset_id,
                            0,
                            0,
                            defi_interaction_nonce,
                            { .first_input_virtual = false,
                              .second_input_virtual = true,
                              .first_output_virtual = false,
                              .second_output_virtual = false,
                              .second_input_real = false,
                              .second_output_real = false } };
    tx.partial_claim_note.bridge_id = bridge_id.to_uint256_t();

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "asset ids don't match");
}

TEST_F(join_split_tests, test_virtual_note_repay_unequal_value_fails)
{
    join_split_tx tx = simple_setup({ 1, 4 });
    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_id bridge_id = { 0,
                            tx.asset_id,
                            0,
                            0,
                            defi_interaction_nonce,
                            { .first_input_virtual = false,
                              .second_input_virtual = true,
                              .first_output_virtual = false,
                              .second_output_virtual = false,
                              .second_input_real = false,
                              .second_output_real = false } };
    tx.partial_claim_note.bridge_id = bridge_id.to_uint256_t();

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "input note values must match");
}

TEST_F(join_split_tests, test_repayment_incorrect_nonce_fails)
{
    join_split_tx tx = simple_setup({ 0, 4 });
    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_id bridge_id = { 0,
                            tx.asset_id,
                            0,
                            0,
                            defi_interaction_nonce + 1,
                            { .first_input_virtual = false,
                              .second_input_virtual = true,
                              .first_output_virtual = false,
                              .second_output_virtual = false,
                              .second_input_real = false,
                              .second_output_real = false } };

    tx.partial_claim_note.bridge_id = bridge_id.to_uint256_t();

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "incorrect interaction nonce in bridge id");
}

TEST_F(join_split_tests, test_repayment_first_note_fails)
{
    join_split_tx tx = simple_setup({ 4, 0 });
    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_id bridge_id = { 0,
                            tx.asset_id,
                            0,
                            0,
                            defi_interaction_nonce,
                            { .first_input_virtual = false,
                              .second_input_virtual = true,
                              .first_output_virtual = false,
                              .second_output_virtual = false,
                              .second_input_real = false,
                              .second_output_real = false } };

    tx.partial_claim_note.bridge_id = bridge_id.to_uint256_t();

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "unsupported case");
}

// Virtual note join-split.
TEST_F(join_split_tests, test_virtual_note_split_logic)
{
    join_split_tx tx = setup_two_virtual_input_notes();

    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner.private_key).valid);
}

TEST_F(join_split_tests, test_virtual_note_can_only_send)
{
    join_split_tx tx = setup_two_virtual_input_notes();
    tx.proof_id = ProofIds::WITHDRAW;
    tx.public_value = tx.input_note[0].value + tx.input_note[1].value;
    tx.public_owner = fr::random_element();

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "can only send");
}

TEST_F(join_split_tests, test_virtual_note_split_single_input_note)
{
    join_split_tx tx = setup_two_virtual_input_notes();

    tx.num_input_notes = 1;
    tx.input_note[1].value = 0;
    tx.output_note[0].value = tx.input_note[0].value >> 1;
    tx.output_note[1].value = tx.input_note[0].value >> 2;
    tx.output_note[1].input_nullifier = compute_nullifier(tx.input_note[1].commit(), user.owner.private_key, false);

    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner.private_key).valid);
}

TEST_F(join_split_tests, test_virtual_note_split_nonzero_public_value_fails)
{
    join_split_tx tx = setup_two_virtual_input_notes();
    tx.public_value = 10;

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "public value incorrect");
}

TEST_F(join_split_tests, test_input_two_virtual_notes_fails)
{
    join_split_tx tx = setup_two_virtual_input_notes();
    tx.input_note[1].asset_id = 3;

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "unsupported case");
}

TEST_F(join_split_tests, test_virtual_note_split_tx_asset_id_inconsistent)
{
    join_split_tx tx = setup_two_virtual_input_notes();
    tx.asset_id = 0;

    auto result = sign_and_verify_logic(tx, user.owner.private_key);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "asset ids don't match");
}

TEST_F(join_split_tests, test_deposit_full_proof)
{
    join_split_tx tx = zero_input_setup();
    tx.proof_id = ProofIds::DEPOSIT;
    tx.public_value = 10;
    tx.public_owner = fr::random_element();
    tx.output_note[0].value = 7;

    auto proof = sign_and_create_proof(tx, user.owner.private_key);
    auto proof_data = inner_proof_data(proof.proof_data);

    auto input_note1_commitment = tx.input_note[0].commit();
    auto input_note2_commitment = tx.input_note[1].commit();
    uint256_t nullifier1 = compute_nullifier(input_note1_commitment, user.owner.private_key, false);
    uint256_t nullifier2 = compute_nullifier(input_note2_commitment, user.owner.private_key, false);
    auto output_note1_commitment = tx.output_note[0].commit();
    auto output_note2_commitment = tx.output_note[1].commit();

    EXPECT_EQ(proof_data.proof_id, ProofIds::DEPOSIT);
    EXPECT_EQ(proof_data.note_commitment1, output_note1_commitment);
    EXPECT_EQ(proof_data.note_commitment2, output_note2_commitment);
    EXPECT_EQ(proof_data.nullifier1, nullifier1);
    EXPECT_EQ(proof_data.nullifier2, nullifier2);
    EXPECT_EQ(proof_data.public_value, tx.public_value);
    EXPECT_EQ(proof_data.public_owner, tx.public_owner);
    EXPECT_EQ(proof_data.asset_id, tx.asset_id);
    EXPECT_EQ(proof_data.merkle_root, tree->root());
    EXPECT_EQ(proof_data.tx_fee, uint256_t(3));
    EXPECT_EQ(proof_data.tx_fee_asset_id, tx.asset_id);
    EXPECT_EQ(proof_data.bridge_id, uint256_t(0));
    EXPECT_EQ(proof_data.defi_deposit_value, uint256_t(0));
    EXPECT_EQ(proof_data.defi_root, fr(0));

    EXPECT_TRUE(verify_proof(proof));
}

TEST_F(join_split_tests, test_withdraw_full_proof)
{
    join_split_tx tx = simple_setup();
    tx.proof_id = ProofIds::WITHDRAW;
    tx.public_value = 10;
    tx.public_owner = fr::random_element();
    tx.output_note[0].value -= 13;

    auto proof = sign_and_create_proof(tx, user.owner.private_key);
    auto proof_data = inner_proof_data(proof.proof_data);

    auto input_note1_commitment = tx.input_note[0].commit();
    auto input_note2_commitment = tx.input_note[1].commit();
    uint256_t nullifier1 = compute_nullifier(input_note1_commitment, user.owner.private_key, true);
    uint256_t nullifier2 = compute_nullifier(input_note2_commitment, user.owner.private_key, true);
    auto output_note1_commitment = tx.output_note[0].commit();
    auto output_note2_commitment = tx.output_note[1].commit();

    EXPECT_EQ(proof_data.proof_id, ProofIds::WITHDRAW);
    EXPECT_EQ(proof_data.note_commitment1, output_note1_commitment);
    EXPECT_EQ(proof_data.note_commitment2, output_note2_commitment);
    EXPECT_EQ(proof_data.nullifier1, nullifier1);
    EXPECT_EQ(proof_data.nullifier2, nullifier2);
    EXPECT_EQ(proof_data.public_value, tx.public_value);
    EXPECT_EQ(proof_data.public_owner, tx.public_owner);
    EXPECT_EQ(proof_data.asset_id, tx.asset_id);
    EXPECT_EQ(proof_data.merkle_root, tree->root());
    EXPECT_EQ(proof_data.tx_fee, uint256_t(3));
    EXPECT_EQ(proof_data.tx_fee_asset_id, tx.asset_id);
    EXPECT_EQ(proof_data.bridge_id, uint256_t(0));
    EXPECT_EQ(proof_data.defi_deposit_value, uint256_t(0));
    EXPECT_EQ(proof_data.defi_root, fr(0));

    EXPECT_TRUE(verify_proof(proof));
}

TEST_F(join_split_tests, test_private_send_full_proof)
{
    join_split_tx tx = simple_setup();
    tx.output_note[0].value -= 3;

    auto proof = sign_and_create_proof(tx, user.owner.private_key);
    auto proof_data = inner_proof_data(proof.proof_data);

    auto input_note1_commitment = tx.input_note[0].commit();
    auto input_note2_commitment = tx.input_note[1].commit();
    auto output_note1_commitment = tx.output_note[0].commit();
    auto output_note2_commitment = tx.output_note[1].commit();
    uint256_t nullifier1 = compute_nullifier(input_note1_commitment, user.owner.private_key, true);
    uint256_t nullifier2 = compute_nullifier(input_note2_commitment, user.owner.private_key, true);

    EXPECT_EQ(proof_data.proof_id, ProofIds::SEND);
    EXPECT_EQ(proof_data.note_commitment1, output_note1_commitment);
    EXPECT_EQ(proof_data.note_commitment2, output_note2_commitment);
    EXPECT_EQ(proof_data.nullifier1, nullifier1);
    EXPECT_EQ(proof_data.nullifier2, nullifier2);
    EXPECT_EQ(proof_data.public_value, uint256_t(0));
    EXPECT_EQ(proof_data.public_owner, fr(0));
    EXPECT_EQ(proof_data.asset_id, uint256_t(0));
    EXPECT_EQ(proof_data.merkle_root, tree->root());
    EXPECT_EQ(proof_data.tx_fee, uint256_t(3));
    EXPECT_EQ(proof_data.tx_fee_asset_id, tx.asset_id);
    EXPECT_EQ(proof_data.bridge_id, uint256_t(0));
    EXPECT_EQ(proof_data.defi_deposit_value, uint256_t(0));
    EXPECT_EQ(proof_data.defi_root, fr(0));
    EXPECT_EQ(proof_data.backward_link, fr(0));
    EXPECT_EQ(proof_data.allow_chain, uint256_t(0));

    EXPECT_TRUE(verify_proof(proof));
}

TEST_F(join_split_tests, test_defi_deposit_full_proof)
{
    join_split_tx tx = simple_setup();
    // 150 in, fee is 10.
    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.output_note[1].value = 90;
    tx.partial_claim_note.deposit_value = 50;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    const bridge_id bridge_id = { 0,
                                  tx.asset_id,
                                  0,
                                  1,
                                  0,
                                  bridge_id::bit_config{ .first_input_virtual = false,
                                                         .second_input_virtual = false,
                                                         .first_output_virtual = false,
                                                         .second_output_virtual = false,
                                                         .second_input_real = false,
                                                         .second_output_real = true },
                                  0 };

    tx.partial_claim_note.bridge_id = bridge_id.to_uint256_t();

    auto proof = sign_and_create_proof(tx, user.owner.private_key);

    EXPECT_TRUE(verify_proof(proof));

    auto proof_data = inner_proof_data(proof.proof_data);

    auto partial_value_commitment = value::create_partial_commitment(
        tx.partial_claim_note.note_secret, tx.input_note[0].owner, tx.input_note[0].nonce, 0);
    claim::claim_note claim_note = {
        tx.partial_claim_note.deposit_value,  tx.partial_claim_note.bridge_id, 0, 0, partial_value_commitment,
        tx.partial_claim_note.input_nullifier
    };

    auto input_note1_commitment = tx.input_note[0].commit();
    auto input_note2_commitment = tx.input_note[1].commit();
    auto output_note1_commitment = claim_note.partial_commit();
    auto output_note2_commitment = tx.output_note[1].commit();
    uint256_t nullifier1 = compute_nullifier(input_note1_commitment, user.owner.private_key, true);
    uint256_t nullifier2 = compute_nullifier(input_note2_commitment, user.owner.private_key, true);

    EXPECT_EQ(proof_data.proof_id, ProofIds::DEFI_DEPOSIT);
    EXPECT_EQ(proof_data.note_commitment1, output_note1_commitment);
    EXPECT_EQ(proof_data.note_commitment2, output_note2_commitment);
    EXPECT_EQ(proof_data.nullifier1, nullifier1);
    EXPECT_EQ(proof_data.nullifier2, nullifier2);
    EXPECT_EQ(proof_data.public_value, uint256_t(0));
    EXPECT_EQ(proof_data.public_owner, fr(0));
    EXPECT_EQ(proof_data.asset_id, uint256_t(0));
    EXPECT_EQ(proof_data.merkle_root, tree->root());
    EXPECT_EQ(proof_data.tx_fee, uint256_t(10));
    EXPECT_EQ(proof_data.tx_fee_asset_id, bridge_id.input_asset_id);
    EXPECT_EQ(proof_data.bridge_id, tx.partial_claim_note.bridge_id);
    EXPECT_EQ(proof_data.defi_deposit_value, tx.partial_claim_note.deposit_value);
    EXPECT_EQ(proof_data.defi_root, fr(0));

    EXPECT_TRUE(verify_proof(proof));
}

TEST_F(join_split_tests, test_non_zero_output_note_pubkey_x)
{
    {
        join_split_tx tx = simple_setup();
        tx.output_note[0].creator_pubkey = user.owner.public_key.x;
        tx.output_note[1].creator_pubkey = user.owner.public_key.x;
        EXPECT_TRUE(sign_and_verify_logic(tx, user.owner.private_key).valid);
    }
    {
        join_split_tx tx = simple_setup();
        tx.output_note[0].creator_pubkey = user.owner.public_key.x;
        EXPECT_TRUE(sign_and_verify_logic(tx, user.owner.private_key).valid);
    }
    {
        join_split_tx tx = simple_setup();
        tx.output_note[1].creator_pubkey = user.owner.public_key.x;
        EXPECT_TRUE(sign_and_verify_logic(tx, user.owner.private_key).valid);
    }
}

TEST_F(join_split_tests, test_incorrect_output_note_pubkey_x)
{
    {
        join_split_tx tx = simple_setup();
        tx.output_note[0].creator_pubkey = rollup::fixtures::create_key_pair(nullptr).public_key.x;
        EXPECT_FALSE(sign_and_verify_logic(tx, user.owner.private_key).valid);
    }
    {
        join_split_tx tx = simple_setup();
        tx.output_note[1].creator_pubkey = rollup::fixtures::create_key_pair(nullptr).public_key.x;
        EXPECT_FALSE(sign_and_verify_logic(tx, user.owner.private_key).valid);
    }
}

TEST_F(join_split_tests, test_repayment_full_proof)
{
    join_split_tx tx = simple_setup({ 0, 4 });
    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;

    bridge_id bridge_id = { 0,
                            tx.asset_id,
                            0,
                            0,
                            defi_interaction_nonce,
                            { .first_input_virtual = false,
                              .second_input_virtual = true,
                              .first_output_virtual = false,
                              .second_output_virtual = false,
                              .second_input_real = false,
                              .second_output_real = false } };

    tx.partial_claim_note.bridge_id = bridge_id.to_uint256_t();
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    auto proof = sign_and_create_proof(tx, user.owner.private_key);
    auto proof_data = inner_proof_data(proof.proof_data);

    auto partial_commitment = value::create_partial_commitment(
        tx.partial_claim_note.note_secret, tx.input_note[0].owner, tx.input_note[0].nonce, 0);
    claim::claim_note claim_note = {
        tx.partial_claim_note.deposit_value,  tx.partial_claim_note.bridge_id, 0, 0, partial_commitment,
        tx.partial_claim_note.input_nullifier
    };

    auto input_note1_commitment = tx.input_note[0].commit();
    auto input_note2_commitment = tx.input_note[1].commit();
    auto output_note1_commitment = claim_note.partial_commit();
    auto output_note2_commitment = tx.output_note[1].commit();
    uint256_t nullifier1 = compute_nullifier(input_note1_commitment, user.owner.private_key, true);
    uint256_t nullifier2 = compute_nullifier(input_note2_commitment, user.owner.private_key, true);

    EXPECT_EQ(proof_data.proof_id, ProofIds::DEFI_DEPOSIT);
    EXPECT_EQ(proof_data.note_commitment1, output_note1_commitment);
    EXPECT_EQ(proof_data.note_commitment2, output_note2_commitment);
    EXPECT_EQ(proof_data.nullifier1, nullifier1);
    EXPECT_EQ(proof_data.nullifier2, nullifier2);
    EXPECT_EQ(proof_data.public_value, uint256_t(0));
    EXPECT_EQ(proof_data.public_owner, fr(0));
    EXPECT_EQ(proof_data.asset_id, uint256_t(0));
    EXPECT_EQ(proof_data.merkle_root, tree->root());
    EXPECT_EQ(proof_data.tx_fee, uint256_t(10));
    EXPECT_EQ(proof_data.tx_fee_asset_id, bridge_id.input_asset_id);
    EXPECT_EQ(proof_data.bridge_id, tx.partial_claim_note.bridge_id);
    EXPECT_EQ(proof_data.defi_deposit_value, tx.partial_claim_note.deposit_value);
    EXPECT_EQ(proof_data.defi_root, fr(0));

    EXPECT_TRUE(verify_proof(proof));
}

TEST_F(join_split_tests, test_virtual_note_split_full_proof)
{
    join_split_tx tx = setup_two_virtual_input_notes();
    ;

    auto proof = sign_and_create_proof(tx, user.owner.private_key);

    auto proof_data = inner_proof_data(proof.proof_data);

    auto input_note1_commitment = tx.input_note[0].commit();
    auto input_note2_commitment = tx.input_note[1].commit();
    auto output_note1_commitment = tx.output_note[0].commit();
    auto output_note2_commitment = tx.output_note[1].commit();
    uint256_t nullifier1 = compute_nullifier(input_note1_commitment, user.owner.private_key, true);
    uint256_t nullifier2 = compute_nullifier(input_note2_commitment, user.owner.private_key, true);

    EXPECT_EQ(proof_data.proof_id, ProofIds::SEND);
    EXPECT_EQ(proof_data.note_commitment1, output_note1_commitment);
    EXPECT_EQ(proof_data.note_commitment2, output_note2_commitment);
    EXPECT_EQ(proof_data.nullifier1, nullifier1);
    EXPECT_EQ(proof_data.nullifier2, nullifier2);
    EXPECT_EQ(proof_data.public_value, uint256_t(0));
    EXPECT_EQ(proof_data.public_owner, fr(0));
    EXPECT_EQ(proof_data.asset_id, uint256_t(0));
    EXPECT_EQ(proof_data.merkle_root, tree->root());
    EXPECT_EQ(proof_data.tx_fee, uint256_t(0));
    EXPECT_EQ(proof_data.tx_fee_asset_id, tx.asset_id);
    EXPECT_EQ(proof_data.defi_deposit_value, uint256_t(0));
    EXPECT_EQ(proof_data.defi_root, fr(0));

    EXPECT_TRUE(verify_proof(proof));
}

} // namespace join_split
} // namespace proofs
} // namespace rollup