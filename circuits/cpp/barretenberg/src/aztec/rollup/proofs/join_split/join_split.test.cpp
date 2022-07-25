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
using key_pair = rollup::fixtures::grumpkin_key_pair;

auto create_account_leaf_data(fr const& account_alias_hash,
                              grumpkin::g1::affine_element const& owner_key,
                              grumpkin::g1::affine_element const& signing_key)
{
    return notes::native::account::account_note{ account_alias_hash, owner_key, signing_key }.commit();
}

class join_split_tests : public ::testing::Test {
  protected:
    static constexpr size_t ACCOUNT_INDEX = 14;
    static void SetUpTestCase()
    {
        auto null_crs_factory = std::make_shared<waffle::ReferenceStringFactory>();
        init_proving_key(null_crs_factory, false);
        auto crs_factory = std::make_unique<waffle::FileReferenceStringFactory>("../srs_db");
        init_verification_key(std::move(crs_factory));
    }

    virtual void SetUp()
    {
        store = std::make_unique<MemoryStore>();
        tree = std::make_unique<MerkleTree<MemoryStore>>(*store, 32);
        user = rollup::fixtures::create_user_context();

        default_value_note = { .value = 100,
                               .asset_id = asset_id,
                               .account_required = false,
                               .owner = user.owner.public_key,
                               .secret = user.note_secret,
                               .creator_pubkey = 0,
                               .input_nullifier = fr::random_element() };

        // Initialise value_notes array as default:
        for (auto& value_note : value_notes) {
            value_note = default_value_note;
            value_note.input_nullifier = fr::random_element(); // to ensure uniqueness
        }

        value_notes[0].creator_pubkey = user.owner.public_key.x;

        value_notes[1].value = 50;
        value_notes[1].creator_pubkey = rollup::fixtures::create_key_pair(nullptr).public_key.x;

        value_notes[2].value = 90;
        value_notes[2].account_required = true,

        value_notes[3].value = 40;
        value_notes[3].account_required = true;

        const uint32_t virtual_asset_id = virtual_asset_id_flag + defi_interaction_nonce;

        value_notes[4].asset_id = virtual_asset_id;

        value_notes[5].value = 30;
        value_notes[5].asset_id = virtual_asset_id;

        value_notes[6].value = 90;
        value_notes[6].asset_id = asset_id + 1;
        value_notes[6].creator_pubkey = user.owner.public_key.x;

        // Value chosen to cause tests to fail.
        value_notes[7].value = 110;
        value_notes[7].asset_id = asset_id + 1;
        value_notes[7].creator_pubkey = user.owner.public_key.x;

        // Similar to value_notes[0], but a different value of 90, to match defi_deposit_value in tests.
        value_notes[8].value = 90;
        value_notes[8].creator_pubkey = user.owner.public_key.x;

        // Similar previous virtual notes, but a different value of 90, to match defi_deposit_value in tests.
        value_notes[9].value = 90;
        value_notes[9].asset_id = virtual_asset_id + 1;

        // Value chosen to cause tests to fail.
        value_notes[10].value = 110;
        value_notes[10].asset_id = virtual_asset_id + 1;

        // Similar previous virtual notes, but a different value of 90, to match defi_deposit_value in tests.
        value_notes[11].value = 90;
        value_notes[11].asset_id = virtual_asset_id;

        // Asset id field designed to be invalid.
        value_notes[12].value = 90;
        value_notes[12].asset_id = virtual_asset_id_flag + asset_id;

        // 'zero'
        value_notes[13].value = 0;
        value_notes[13].asset_id = 0;
    }

    /**
     * Add two account notes for the user.
     */
    void preload_account_notes()
    {
        tree->update_element(
            tree->size(),
            create_account_leaf_data(user.alias_hash, user.owner.public_key, user.signing_keys[0].public_key));
        tree->update_element(
            tree->size(),
            create_account_leaf_data(user.alias_hash, user.owner.public_key, user.signing_keys[1].public_key));
    }

    /**
     * See the test's SetUp() function for the value_notes.
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
    join_split_tx create_join_split_tx(std::array<uint32_t, 2> const& input_indices,
                                       std::array<value::value_note, 2> const& input_notes,
                                       uint32_t account_note_index = 0,
                                       bool account_required = false)
    {
        uint32_t tx_asset_id = input_notes[0].asset_id;
        auto input_nullifier1 = compute_nullifier(input_notes[0].commit(), user.owner.private_key, true);
        auto input_nullifier2 = compute_nullifier(input_notes[1].commit(), user.owner.private_key, true);
        value::value_note output_note1 = { .value = input_notes[0].value + input_notes[1].value,
                                           .asset_id = tx_asset_id,
                                           .account_required = account_required,
                                           .owner = user.owner.public_key,
                                           .secret = user.note_secret,
                                           .creator_pubkey = 0,
                                           .input_nullifier = input_nullifier1 };
        value::value_note output_note2 = { .value = 0,
                                           .asset_id = tx_asset_id,
                                           .account_required = account_required,
                                           .owner = user.owner.public_key,
                                           .secret = user.note_secret,
                                           .creator_pubkey = 0,
                                           .input_nullifier = input_nullifier2 };

        join_split_tx tx;
        tx.proof_id = ProofIds::SEND;
        tx.public_value = 0;
        tx.num_input_notes = 2;
        tx.input_index = input_indices;
        tx.old_data_root = tree->root();
        tx.input_path = { tree->get_hash_path(input_indices[0]), tree->get_hash_path(input_indices[1]) };
        tx.input_note = input_notes;
        tx.output_note = { output_note1, output_note2 };
        tx.public_owner = fr(0);
        tx.account_note_index = account_note_index;
        tx.account_note_path = tree->get_hash_path(account_note_index);
        tx.signing_pub_key = user.signing_keys[0].public_key;
        tx.asset_id = tx_asset_id;
        tx.account_private_key = user.owner.private_key;
        tx.partial_claim_note.input_nullifier = 0;
        tx.alias_hash = !account_required ? rollup::fixtures::generate_alias_hash("penguin") : user.alias_hash;
        tx.account_required = account_required;
        // default to no chaining:
        tx.backward_link = 0;
        tx.allow_chain = 0;
        return tx;
    }

    /**
     * Add account notes and value notes.
     * Return a join split tx that spends them.
     */
    join_split_tx simple_setup(std::array<uint32_t, 2> const& input_indices = { 0, 1 },
                               uint32_t account_note_index = 0,
                               bool account_required = false)
    {
        // The tree, user and notes are initialised in SetUp().
        preload_value_notes();
        preload_account_notes(); // indices: [ACCOUNT_INDEX, ACCOUNT_INDEX + 1]
        return create_join_split_tx(input_indices,
                                    { value_notes[input_indices[0]], value_notes[input_indices[1]] },
                                    account_note_index,
                                    account_required);

        /** Default tx:
         * SEND tx
         * public_value = 0
         * input_note_1:  value = 100, asset_id = 1, account_required = 0
         * input_note_2:  value = 50,  asset_id = 1, account_required = 0
         * output_note_1: value = 150, asset_id = 1, account_required = 0, same owner as inputs
         * output_note_2: value = 0,   asset_id = 1, account_required = 0, same owner as inputs
         */
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
        tx.account_required = false;
        tx.account_note_index = 0;
        tx.account_note_path = tree->get_hash_path(0);
        tx.signing_pub_key = user.signing_keys[0].public_key;
        tx.backward_link = 0;
        tx.allow_chain = 0;
        return tx;
    }

    waffle::plonk_proof sign_and_create_proof(join_split_tx& tx, key_pair const& signing_key)
    {
        tx.signature = sign_join_split_tx(tx, signing_key);

        auto prover = new_join_split_prover(tx, false);
        return prover.construct_proof();
    }

    bool sign_and_verify(join_split_tx& tx, key_pair const& signing_key)
    {
        return verify_proof(sign_and_create_proof(tx, signing_key));
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

    verify_result sign_and_verify_logic(join_split_tx& tx, key_pair const& signing_key)
    {
        tx.signature = sign_join_split_tx(tx, signing_key);
        return verify_logic(tx);
    }

    rollup::fixtures::user_context user;
    std::unique_ptr<MemoryStore> store;
    std::unique_ptr<MerkleTree<MemoryStore>> tree;
    bridge_call_data empty_bridge_call_data = { .bridge_address_id = 0,
                                                .input_asset_id_a = 0,
                                                .input_asset_id_b = 0,
                                                .output_asset_id_a = 0,
                                                .output_asset_id_b = 0,
                                                .config = { .second_input_in_use = false,
                                                            .second_output_in_use = false },
                                                .aux_data = 0 };
    value::value_note default_value_note;
    value::value_note value_notes[14];
    const uint32_t asset_id = 1;
    const uint32_t defi_interaction_nonce = 7;
    const uint32_t virtual_asset_id_flag = (uint32_t(1) << (MAX_NUM_ASSETS_BIT_LENGTH - 1));
    const uint256_t max_value = notes::NOTE_VALUE_MAX;
};

/**
 * List of negative tests - ways of triggering each error message:
 *
 * BRIDGE CALL DATA
 *
 * - "Expected second_input_in_use, given input_asset_id_b != 0"
 *     - input_asset_id_b != 0, but config.second_input_in_use = false;
 * - "Expected second_output_in_use, given output_asset_id_b != 0"
 *     - output_asset_id_b != 0, but config.second_output_in_use = false;
 * - "input asset ids must be different for the second bridge input to be in-use"
 *     - config.second_input_in_use but equality: input_asset_id_a == input_asset_id_b (bridge inputs)
 * - "real output asset ids must be different for the second bridge output to be in-use"
 *     - config.second_output_in_use and both real but equality: output_asset_id_a == output_asset_id_b (bridge outputs)
 * - "output_asset_id_a detected as virtual, but has incorrect placeholder value"
 *     - first output virtual, but output_asset_id_a != 2**29
 * - "output_asset_id_b detected as virtual, but has incorrect placeholder value"
 *     - first output virtual, but output_asset_id_b != 2**29
 *
 * MAIN
 *
 * INPUT VALIDATION
 *
 * - "invalid num_input_notes"
 *     - provide an invalid # notes
 * - "public value invalid":
 *     - deposit, no public value
 *     - withdraw, no public value
 *     - send, public value
 *     - defi deposit, public value
 * - "public owner invalid"
 *     - deposit, no public owner
 *     - withdraw, no public owner
 *     - send, public owner
 *     - defi deposit, public owner
 * - "invalid proof id"
 *     - provide an invalid proof id
 * - "joining same note"
 *     - pass two identical input notes
 * - "can only deposit"
 *     - num_input_notes = 0, Send
 *     - num_input_notes = 0, Withdraw
 *     - num_input_notes = 0, Defi Desposit
 * - "asset ids don't match"
 *     - inputs.asset_id != input_note1.asset_id
 *     - inputs.asset_id != output_note1.asset_id
 *     - inputs.asset_id != output_note2.asset_id
 * - "input asset ids must match unless defi-depositing"
 *     - input_note_2_in_use && is_deposit, but differing input note asset_ids
 *     - input_note_2_in_use && is_send, but differing input note asset_ids
 *     - input_note_2_in_use && is_withdraw, but differing input note asset_ids
 *
 * DEFI DEPOSIT CHECKS
 *
 * - "Expected a nonzero defi_deposit_value for a defi-deposit"
 *     - proof_id = DEFI_DEPOSIT, but defi_deposit_value = 0.
 * - "all of input note 2 must be defi-deposited"
 *     - (is_defi_deposit && input_note_2_in_use && different_input_asset_ids), but
 *       defi_deposit_value != input_note_2_value
 * - "Expected bridge_call_data_local.input_asset_id_a == input_note_1.asset_id for a defi-deposit"
 *     - is_defi_deposit, but bridge_call_data_local.input_asset_id_a != input_note_1.asset_id
 * - "Expected input_note_2_in_use, given bridge_call_data_local.config.second_input_in_use"
 *     - bridge_call_data_local.config.second_input_in_use, but input_note_2_in_use = false
 * - "Expected bridge_call_data_local.config.second_input_in_use, given input_note_2_in_use &&
 * different_input_asset_ids"
 *     - input_note_2_in_use && different_input_asset_ids, but bridge_call_data_local.config.second_input_in_use =
 * false,
 * - "Expected bridge_call_data_local.input_asset_id_b == input_note_2.asset_id, given
 *    bridge_call_data_local.config.second_input_in_use"
 *     - bridge_call_data_local.config.second_input_in_use, but bridge_call_data_local.input_asset_id_b !=
 * input_note_2.asset_id
 *
 * TRANSACTION CHAINING CHECKS
 *
 * (Not listed here)
 *
 * VALUE CONSERVATION EQUATION (BALANCING CHECKS)
 * - To trigger "total_in_value < total_out_value"
 *     - total_out_value > total_in_value
 *     - defi deposit, input_note_2 in use, different input note asset_ids, input_note_1_value < input_note_2_value
 *
 * ACCOUNT
 *
 * - "input note owners don't match"
 *     - different owners in the two input notes
 * - "input note account_required don't match"
 *     - different account_required in the two input notes
 * - "account private key is zero"
 *     - set inputs.account_private_key = 0
 * - "account_private_key incorrect"
 *     - input an incorrect inputs.account_private_key, relative to the input_note_1.owner
 * - "account_required incorrect"
 *     - set inputs.account_required != input_note_1.account_required
 * - "output note 1 creator_pubkey mismatch"
 *     - set the output_note_1.creator_pubkey to be different from input_note_1.owner (and nonzero)
 * - "output note 2 creator_pubkey mismatch"
 *     - set the output_note_2.creator_pubkey to be different from input_note_1.owner (and nonzero)
 * - "account check_membership failed"
 *     - signing_key_exists = false  AND !inputs.account_required = false
 *
 * NOTE
 *
 * - "padding note non zero"
 *     - input_note_1.value != 0 AND num_input_notes = 0
 *     - input_note_2.value != 0 AND num_input_notes = 0
 * - "input note not a member"
 *     - fails merkle membership check due to:
 *         - bad merkle_root
 *         - bad hash_path
 *         - bad index
 *     - if note doesn't exist, isn't propagated, and is in_use
 *
 * NULLIFIER
 * - "output note 1 has incorrect input nullifier"
 *     - Put an incorrect input_nullifier into output_note_1
 * - "output note 2 has incorrect input nullifier"
 *     - Put an incorrect input_nullifier into output_note_2
 */

// *************************************************************************************************************
// Input validation
// *************************************************************************************************************

TEST_F(join_split_tests, test_invalid_num_input_notes_fails)
{
    // Only num_input_notes = 0, 1, 2 should be accepted.
    join_split_tx tx = simple_setup();
    tx.num_input_notes = 100; // <-- testing this fails.
    tx.input_note[1].value = 0;
    tx.output_note[0].value = tx.input_note[0].value;
    tx.output_note[1].input_nullifier = compute_nullifier(tx.input_note[1].commit(), user.owner.private_key, false);

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "invalid num_input_notes");
}

TEST_F(join_split_tests, test_deposit_public_value_invalid_fails)
{
    join_split_tx tx = zero_input_setup();
    tx.proof_id = ProofIds::DEPOSIT;
    tx.public_value = 0; // <-- invalid, should be nonzero
    tx.public_owner = fr::random_element();

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "public value invalid");
}

TEST_F(join_split_tests, test_send_public_value_invalid_fails)
{
    join_split_tx tx = zero_input_setup();
    tx.public_value = 10; // <-- invalid - should be 0
    tx.public_owner = fr::random_element();

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "public value invalid");
}

TEST_F(join_split_tests, test_withdraw_public_value_invalid_fails)
{
    join_split_tx tx = simple_setup();
    tx.proof_id = ProofIds::WITHDRAW;
    tx.public_value = 0; // <-- invalid - should be nonzero
    tx.public_owner = fr::random_element();

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "public value invalid");
}

TEST_F(join_split_tests, test_defi_deposit_public_value_invalid_fails)
{
    join_split_tx tx = simple_setup();
    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.output_note[0].value = 0;
    tx.output_note[1].value = 99;
    tx.partial_claim_note.deposit_value = 50;

    tx.public_value = 1; // <-- invalid - should be 0

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "public value invalid");
}

TEST_F(join_split_tests, test_deposit_public_owner_invalid_fails)
{
    join_split_tx tx = zero_input_setup();
    tx.proof_id = ProofIds::DEPOSIT;
    tx.public_value = 10;
    tx.public_owner = 0; // <-- invalid - should be nonzero

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "public owner invalid");
}

TEST_F(join_split_tests, test_send_public_owner_invalid_fails)
{
    join_split_tx tx = zero_input_setup();
    tx.public_value = 0;
    tx.public_owner = fr::random_element(); // <-- invalid - should be 0

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "public owner invalid");
}

TEST_F(join_split_tests, test_withdraw_public_owner_invalid_fails)
{
    join_split_tx tx = simple_setup();
    tx.proof_id = ProofIds::WITHDRAW;
    tx.public_value = 10;
    tx.public_owner = 0; // <-- invalid - should be nonzero

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "public owner invalid");
}

TEST_F(join_split_tests, test_defi_deposit_public_owner_invalid_fails)
{
    join_split_tx tx = simple_setup();
    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.output_note[0].value = 0;
    tx.output_note[1].value = 99;
    tx.partial_claim_note.deposit_value = 50;

    tx.public_owner = 1; // <-- invalid - should be 0

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "public owner invalid");
}

TEST_F(join_split_tests, test_wrong_proof_id)
{
    join_split_tx tx = zero_input_setup();
    tx.proof_id = ProofIds::DEFI_CLAIM;

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "invalid proof id");
}

TEST_F(join_split_tests, test_joining_same_note_fails)
{
    join_split_tx tx = simple_setup({ 1, 1 });
    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "joining same note");
}

TEST_F(join_split_tests, test_send_with_0_input_notes_fails)
{
    join_split_tx tx = zero_input_setup();

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "can only deposit");
}

TEST_F(join_split_tests, test_withdraw_with_0_input_notes_fails)
{
    join_split_tx tx = zero_input_setup();
    tx.proof_id = ProofIds::WITHDRAW;
    tx.public_value = 10;
    tx.public_owner = fr::random_element();

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "can only deposit");
}

TEST_F(join_split_tests, test_defi_deposit_with_0_input_notes_fails)
{
    join_split_tx tx = zero_input_setup();
    tx.proof_id = ProofIds::DEFI_DEPOSIT;

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "can only deposit");
}

TEST_F(join_split_tests, test_wrong_asset_id_fails)
{
    join_split_tx tx = simple_setup();
    tx.asset_id = 3;

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "asset ids don't match");
}

TEST_F(join_split_tests, test_different_input_note_1_asset_id_fails)
{
    join_split_tx tx = simple_setup();
    tx.input_note[0].asset_id = 3;

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "asset ids don't match");
}

TEST_F(join_split_tests, test_different_output_note_1_asset_id_fails)
{
    join_split_tx tx = simple_setup();
    tx.output_note[0].asset_id = 3;

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "asset ids don't match");
}

TEST_F(join_split_tests, test_different_output_note_2_asset_id_fails)
{
    join_split_tx tx = simple_setup();
    tx.output_note[1].asset_id = 3;

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "asset ids don't match");
}

TEST_F(join_split_tests, test_deposit_but_different_input_note_2_asset_id_fails)
{
    join_split_tx tx = simple_setup();
    tx.proof_id = ProofIds::DEPOSIT;
    tx.public_value = 10;
    tx.public_owner = fr::random_element();
    tx.input_note[1].asset_id = 3;

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "input asset ids must match unless defi-depositing");
}

TEST_F(join_split_tests, test_send_but_different_input_note_2_asset_id_fails)
{
    join_split_tx tx = simple_setup();
    tx.input_note[1].asset_id = 3;

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "input asset ids must match unless defi-depositing");
}

TEST_F(join_split_tests, test_withdraw_but_different_input_note_2_asset_id_fails)
{
    join_split_tx tx = simple_setup();
    tx.proof_id = ProofIds::WITHDRAW;
    tx.public_value = 10;
    tx.public_owner = fr::random_element();
    tx.input_note[1].asset_id = 3;

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "input asset ids must match unless defi-depositing");
}

// *************************************************************************************************************
// Input note combinations. Deposit/Send/Withdraw.
// *************************************************************************************************************

TEST_F(join_split_tests, test_0_input_notes)
{
    join_split_tx tx = zero_input_setup();
    tx.proof_id = ProofIds::DEPOSIT;
    tx.public_value = 30;
    tx.public_owner = fr::random_element();
    tx.output_note[0].value = 30;

    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner).valid);
}

// Bespoke test seeking bug.
TEST_F(join_split_tests, test_0_input_notes_create_dupicate_output_notes_fails)
{
    join_split_tx tx = zero_input_setup();

    tx.proof_id = ProofIds::DEPOSIT;
    tx.public_value = 30;
    tx.public_owner = fr::random_element();
    tx.output_note[0].value = 15;
    tx.output_note[1] = tx.output_note[0]; // <-- attempt to maliciously create duplicate output notes.
    tx.input_note[1] = tx.input_note[0];   // <-- for output notes to be equal, input_nullifiers must be equal.

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "joining same note");
}

// Bespoke test seeking bug.
TEST_F(join_split_tests, test_0_input_notes_create_dupicate_output_notes_fails_2)
{
    join_split_tx tx = zero_input_setup();

    tx.proof_id = ProofIds::DEPOSIT;
    tx.public_value = 30;
    tx.public_owner = fr::random_element();
    tx.output_note[0].value = 15;
    tx.output_note[1] = tx.output_note[0]; // <-- attempt to maliciously create duplicate output notes.
    tx.input_note[1] = tx.input_note[0];
    tx.input_note[1].secret +=
        1; // <-- to avoid 'joining same note', modify input_note[1], but then hit the error that requirement for
           // different input_nullifiers will force the output_notes to be different.

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "output note 2 has incorrect input nullifier");
}

TEST_F(join_split_tests, test_dummy_input_note_1_non_0_value_fails)
{
    // Note: `tx.num_input_notes = 0` implies both inputs are 'dummy'
    join_split_tx tx = zero_input_setup();
    tx.proof_id = ProofIds::DEPOSIT;
    tx.public_value = 10;
    tx.public_owner = fr::random_element();
    tx.input_note[0].value = 10;
    tx.output_note[0].value = 20;

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "padding note non zero");
}

TEST_F(join_split_tests, test_dummy_input_note_2_non_0_value_fails)
{
    // Note: `tx.num_input_notes = 0` implies both inputs are 'dummy'
    join_split_tx tx = zero_input_setup();
    tx.proof_id = ProofIds::DEPOSIT;
    tx.public_value = 10;
    tx.public_owner = fr::random_element();
    tx.input_note[1].value = 10;
    tx.output_note[0].value = 20;

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "padding note non zero");
}

TEST_F(join_split_tests, test_1_input_note)
{
    join_split_tx tx = simple_setup();
    tx.num_input_notes = 1; // <-- testing this
    tx.input_note[1].value = 0;
    tx.output_note[0].value = tx.input_note[0].value;
    tx.output_note[1].input_nullifier = compute_nullifier(tx.input_note[1].commit(), user.owner.private_key, false);

    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner).valid);
}

// Bespoke test seeking bug.
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

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "can only deposit");
}

TEST_F(join_split_tests, test_2_input_notes)
{
    join_split_tx tx = simple_setup();
    auto result = sign_and_verify_logic(tx, user.owner);
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

    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner).valid);
}

// *************************************************************************************************************
// Chaining
// *************************************************************************************************************

class test_valid_allow_chain_permutations : public join_split_tests, public ::testing::WithParamInterface<uint32_t> {};
TEST_P(test_valid_allow_chain_permutations, )
{
    join_split_tx tx = simple_setup();
    // sending to self is implied here, by the fixture's default values
    tx.allow_chain = GetParam();
    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_TRUE(result.valid);
    EXPECT_EQ(result.public_inputs[InnerProofFields::ALLOW_CHAIN], GetParam());
}
INSTANTIATE_TEST_SUITE_P(join_split_tests, test_valid_allow_chain_permutations, ::testing::Values(0, 1, 2, 3));

TEST_F(join_split_tests, test_allow_chain_out_of_range_fails)
{
    join_split_tx tx = simple_setup();
    tx.backward_link = fr::random_element(); // choose a value unrelated to the inputs being spent
    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "backward_link unrelated to inputs");
}

TEST_F(join_split_tests, test_unrelated_backward_link_fails)
{
    join_split_tx tx = simple_setup();
    tx.allow_chain = 4;
    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "allow_chain out of range");
}

class test_allow_chain_to_other_users_fail : public join_split_tests, public ::testing::WithParamInterface<uint32_t> {};
TEST_P(test_allow_chain_to_other_users_fail, )
{
    join_split_tx tx = simple_setup();
    tx.allow_chain = GetParam();
    tx.output_note[tx.allow_chain - 1].owner = grumpkin::g1::element::random_element(); // i.e. not owned by self.
    auto result = sign_and_verify_logic(tx, user.owner);
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
    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_TRUE(result.valid);
}
INSTANTIATE_TEST_SUITE_P(join_split_tests, test_propagated_notes_skip_membership_check, ::testing::Values(1, 2));

TEST_F(join_split_tests, test_propagated_input_note1_no_double_spend)
{
    join_split_tx tx = simple_setup();
    tx.backward_link = value_notes[0].commit();

    // Let's try to double-spend input_note[0]:
    tx.input_note[1] = tx.input_note[0];

    tx.output_note[0] = {
        tx.input_note[0].value,           tx.input_note[0].asset_id, tx.input_note[0].account_required,
        tx.input_note[0].owner,           tx.input_note[0].secret,   tx.input_note[0].creator_pubkey,
        tx.output_note[0].input_nullifier
    };
    tx.output_note[1] = {
        tx.input_note[0].value,           tx.input_note[0].asset_id,   tx.input_note[0].account_required,
        tx.input_note[0].owner,           tx.input_note[0].secret + 1, tx.input_note[0].creator_pubkey,
        tx.output_note[0].input_nullifier
    };

    auto result = sign_and_verify_logic(tx, user.owner);

    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "joining same note");
}

// *************************************************************************************************************
// Public values
// *************************************************************************************************************

TEST_F(join_split_tests, test_max_public_input)
{
    join_split_tx tx = zero_input_setup();
    tx.proof_id = ProofIds::DEPOSIT;
    tx.public_value = max_value;
    tx.output_note[0].value = max_value;
    tx.public_owner = fr::random_element();

    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner).valid);
}

TEST_F(join_split_tests, test_overflow_public_value_fails)
{
    join_split_tx tx = zero_input_setup();
    tx.proof_id = ProofIds::DEPOSIT;
    tx.public_value = max_value + 1;
    tx.public_owner = fr::random_element();

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "safe_uint_t range constraint failure: public_value");
}

// *************************************************************************************************************
// Tx fee
// *************************************************************************************************************

TEST_F(join_split_tests, test_non_zero_tx_fee)
{
    join_split_tx tx = zero_input_setup();
    tx.proof_id = ProofIds::DEPOSIT;
    tx.public_value += 10;
    tx.public_owner = fr::random_element();

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_TRUE(result.valid);
    EXPECT_EQ(result.public_inputs[InnerProofFields::TX_FEE], 10);
}

TEST_F(join_split_tests, test_non_zero_tx_fee_zero_public_values)
{
    join_split_tx tx = simple_setup();
    tx.output_note[0].value -= 10;

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_TRUE(result.valid);
    EXPECT_EQ(result.public_inputs[InnerProofFields::TX_FEE], 10);
}

TEST_F(join_split_tests, test_max_tx_fee)
{
    join_split_tx tx = zero_input_setup();
    auto tx_fee = (uint256_t(1) << rollup::TX_FEE_BIT_LENGTH) - 1;
    tx.proof_id = ProofIds::DEPOSIT;
    tx.public_value += tx_fee;
    tx.public_owner = fr::random_element();

    auto result = sign_and_verify_logic(tx, user.owner);
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

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "safe_uint_t range constraint failure: subtract: total_in_value < total_out_value");
}

TEST_F(join_split_tests, test_total_output_value_larger_than_total_input_value_fails)
{
    join_split_tx tx = simple_setup();
    tx.output_note[0].value += 1;

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "safe_uint_t range constraint failure: subtract: total_in_value < total_out_value");
}

// *************************************************************************************************************
// Account
// *************************************************************************************************************

TEST_F(join_split_tests, test_different_input_note_owners_fails)
{
    join_split_tx tx = simple_setup({ 1, 2 });
    tx.input_note[0].owner = grumpkin::g1::affine_element::hash_to_curve(1).second;

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "input note owners don't match");
}

TEST_F(join_split_tests, test_different_input_note_account_requireds_fails)
{
    join_split_tx tx = simple_setup({ 1, 2 });

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "input note account_required don't match");
}

// Test omitted, because circle ci test is terminated when reaching this ASSERT.
// TEST_F(join_split_tests, test_zero_account_private_key_fails)
// {
//     join_split_tx tx = simple_setup({ 2, 3 }, ACCOUNT_INDEX, 1);
//     tx.account_private_key = 0;

//     auto result = sign_and_verify_logic(tx, user.owner);
//     EXPECT_FALSE(result.valid);
//     EXPECT_EQ(result.err, "input scalar to fixed_base_scalar_mul_internal cannot be 0");
//     // EXPECT_EQ(result.err, "account private key is zero");
// }

TEST_F(join_split_tests, test_spend_notes_with_registered_account)
{
    join_split_tx tx = simple_setup({ 2, 3 }, ACCOUNT_INDEX, 1);
    EXPECT_TRUE(sign_and_verify_logic(tx, user.signing_keys[0]).valid);
}

TEST_F(join_split_tests, test_different_note_account_required_vs_account_required_fails)
{
    join_split_tx tx = simple_setup({ 2, 3 }, ACCOUNT_INDEX, 0);
    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "account_required incorrect");
}

TEST_F(join_split_tests, test_wrong_input_note_owner_fails)
{
    join_split_tx tx = simple_setup();
    tx.input_note[0].owner = grumpkin::g1::element::random_element();
    tx.input_note[1].owner = tx.input_note[0].owner;

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "account_private_key incorrect");
}

TEST_F(join_split_tests, test_random_output_note_owners)
{
    join_split_tx tx = simple_setup();
    tx.output_note[0].owner = grumpkin::g1::element::random_element();
    tx.output_note[1].owner = grumpkin::g1::element::random_element();

    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner).valid);
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
    tx.signature = sign_join_split_tx(tx, user.owner);

    auto prover = new_join_split_prover(tx, false);
    auto proof = prover.construct_proof();

    EXPECT_EQ(proof.proof_data[InnerProofOffsets::PUBLIC_OWNER], 0x01);
    proof.proof_data[InnerProofFields::PUBLIC_OWNER] = 0x02;

    EXPECT_FALSE(verify_proof(proof));
}

// *************************************************************************************************************
// Signature
// *************************************************************************************************************

TEST_F(join_split_tests, test_wrong_account_private_key_fails)
{
    join_split_tx tx = simple_setup();
    tx.account_private_key = grumpkin::fr::random_element();

    auto result = sign_and_verify_logic(tx, user.owner);
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
    tx.signature = sign_join_split_tx(tx, user.owner);

    tx.public_owner = fr::random_element();

    auto result = verify_logic(tx);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "verify signature failed");
}

TEST_F(join_split_tests, test_spend_notes_with_signing_key_when_account_required_is_false_fails)
{
    join_split_tx tx = simple_setup();
    auto result = sign_and_verify_logic(tx, user.signing_keys[0]);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "verify signature failed");
}

TEST_F(join_split_tests, test_spend_registered_notes_with_owner_key_fails)
{
    auto tx = simple_setup({ 2, 3 }, ACCOUNT_INDEX, 1);
    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "verify signature failed");
}

// *************************************************************************************************************
// Account membership
// *************************************************************************************************************

TEST_F(join_split_tests, test_wrong_alias_hash_fails)
{
    join_split_tx tx = simple_setup({ 2, 3 }, ACCOUNT_INDEX, 1);
    tx.alias_hash = rollup::fixtures::generate_alias_hash("chicken");

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "account check_membership failed");
}

TEST_F(join_split_tests, test_nonregistered_signing_key_fails)
{
    join_split_tx tx = simple_setup({ 2, 3 }, ACCOUNT_INDEX, 1);
    auto keys = rollup::fixtures::create_key_pair(nullptr);
    tx.signing_pub_key = keys.public_key;

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "account check_membership failed");
}

// *************************************************************************************************************
// Note membership
// *************************************************************************************************************

TEST_F(join_split_tests, test_wrong_merkle_root_fails)
{
    join_split_tx tx = simple_setup();
    tx.old_data_root = fr::random_element();

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "input note not a member");
}

TEST_F(join_split_tests, test_wrong_note_hash_path_fails)
{
    join_split_tx tx = simple_setup();
    auto gibberish_path = fr_hash_path(32, std::make_pair(fr::random_element(), fr::random_element()));
    tx.input_path[0] = gibberish_path;

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "input note not a member");
}

TEST_F(join_split_tests, test_wrong_leaf_index_fails)
{
    join_split_tx tx = simple_setup();
    tx.input_index[0] = 99;

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "input note not a member");
}

// *************************************************************************************************************
// Nullifier
// *************************************************************************************************************

TEST_F(join_split_tests, test_incorrect_input_nullifier_in_output_note_1_fails)
{
    join_split_tx tx = simple_setup();
    tx.output_note[0].input_nullifier = 1; // incorrect nullifier

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "output note 1 has incorrect input nullifier");
}

TEST_F(join_split_tests, test_incorrect_input_nullifier_in_output_note_2_fails)
{
    join_split_tx tx = simple_setup();
    tx.output_note[1].input_nullifier = 1; // incorrect nullifier

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "output note 2 has incorrect input nullifier");
}

// *************************************************************************************************************
// Defi deposit
// *************************************************************************************************************

TEST_F(join_split_tests, test_defi_deposit_one_virtual_input)
{
    join_split_tx tx = simple_setup({ 4, 13 });

    tx.num_input_notes = 1;
    tx.output_note[1].input_nullifier =
        compute_nullifier(tx.input_note[1].commit(), user.owner.private_key, false); // input note 2 is a dummy note

    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    /**
     * tx represents:
     *   - 100 in1 (virtual)
     *   - 0 in2 (not in use)
     *   - 90 deposited
     *   - 10 paid as fee
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_TRUE(result.valid);
}

TEST_F(join_split_tests, test_defi_deposit_one_real_one_virtual_inputs)
{
    join_split_tx tx = simple_setup({ 0, 11 });

    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    bridge_call_data.input_asset_id_b = tx.input_note[1].asset_id;
    bridge_call_data.config.second_input_in_use = true;
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    /**
     * tx represents:
     *   - 100 in1 (real)
     *   - 90 in2 (virtual)
     *   - 90 deposited via bridge input 1 (real)
     *   - 90 deposited via bridge input 2 (virtual)
     *   - 10 paid as fee (in1's asset_id)
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_TRUE(result.valid);
}

TEST_F(join_split_tests, test_defi_deposit_one_virtual_one_real_inputs)
{
    join_split_tx tx = simple_setup({ 10, 7 });

    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 110;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    bridge_call_data.input_asset_id_b = tx.input_note[1].asset_id;
    bridge_call_data.config.second_input_in_use = true;
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    /**
     * tx represents:
     *   - 110 in1 (virtual)
     *   - 110 in2 (real)
     *   - 110 deposited via bridge input 1 (virtual)
     *   - 110 deposited via bridge input 2 (virtual)
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_TRUE(result.valid);
}

TEST_F(join_split_tests, test_defi_deposit_one_real_one_virtual_inputs_same_asset_ids)
{
    join_split_tx tx = simple_setup({ 0, 12 });

    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    bridge_call_data.input_asset_id_b = tx.input_note[1].asset_id;
    bridge_call_data.config.second_input_in_use = true;
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    /**
     * tx represents:
     *   - 100 in1 (real)
     *   - 90 in2 (virtual flag, but same 'asset id' (29 bits) as input 1)
     *   - 90 deposited via bridge input 1 (real)
     *   - 90 deposited via bridge input 2 (virtual)
     *   - 10 paid as fee (in1's asset_id)
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_TRUE(result.valid);
}

TEST_F(join_split_tests, test_defi_deposit_two_real_inputs_different_asset_ids)
{
    join_split_tx tx = simple_setup({ 0, 6 });

    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    bridge_call_data.input_asset_id_b = tx.input_note[1].asset_id;
    bridge_call_data.config.second_input_in_use = true;
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    /**
     * tx represents:
     *   - 100 in1 (real)
     *   - 90 in2 (real, different asset_id = asset_id + 1)
     *   - 90 deposited via bridge input 1
     *   - 90 deposited via bridge input 2
     *   - 10 paid as fee (in1's asset_id)
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_TRUE(result.valid);
}

TEST_F(join_split_tests, test_defi_deposit_two_virtual_inputs_different_asset_ids)
{
    join_split_tx tx = simple_setup({ 4, 9 });

    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    bridge_call_data.input_asset_id_b = tx.input_note[1].asset_id;
    bridge_call_data.config.second_input_in_use = true;
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    /**
     * tx represents:
     *   - 100 in (virtual)
     *   - 90 in (virtual, different asset_id = virtual_asset_id + 1)
     *   - 90 deposited via bridge input 1
     *   - 90 deposited via bridge input 2
     *   - 10 paid as fee (in1's asset_id)
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_TRUE(result.valid);
}

TEST_F(join_split_tests,
       test_defi_deposit_two_virtual_inputs_different_asset_ids_and_input_note_2_gt_defi_deposit_fails)
{
    join_split_tx tx = simple_setup({ 4, 10 });

    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    bridge_call_data.input_asset_id_b = tx.input_note[1].asset_id;
    bridge_call_data.config.second_input_in_use = true;
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    /**
     * tx represents:
     *   - 100 in (virtual)
     *   - 110 in (virtual, different asset_id = virtual_asset_id + 1) <-- fails, since input note 2 value should equal
     *     defi deposit value.
     *   - 90 deposited via bridge input 1
     *   - 90 deposited via bridge input 2
     *   - 10 paid as fee (in1's asset_id)
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "all of input note 2 must be defi-deposited");
}

TEST_F(join_split_tests, test_defi_deposit_two_real_inputs_different_asset_ids_and_input_note_2_gt_defi_deposit_fails)
{
    join_split_tx tx = simple_setup({ 0, 7 });

    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    bridge_call_data.input_asset_id_b = tx.input_note[1].asset_id;
    bridge_call_data.config.second_input_in_use = true;
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    /**
     * tx represents:
     *   - 100 in1 (real)
     *   - 110 in2 (real, different asset_id = asset_id + 1) <-- fails, since input note 2 value should equal
     *     defi deposit value.
     *   - 90 deposited via bridge input 1
     *   - 90 deposited via bridge input 2
     *   - 10 paid as fee (in1's asset_id)
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "all of input note 2 must be defi-deposited");
}

TEST_F(join_split_tests, test_defi_deposit_two_real_inputs_different_asset_ids_and_input_note_2_lt_defi_deposit_fails)
{
    join_split_tx tx = simple_setup({ 0, 6 });

    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 95;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    bridge_call_data.input_asset_id_b = tx.input_note[1].asset_id;
    bridge_call_data.config.second_input_in_use = true;
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    /**
     * tx represents:
     *   - 100 in1 (real)
     *   - 90 in2 (real, different asset_id = asset_id + 1) <-- fails, since input note 2 value should equal
     *     defi deposit value.
     *   - 95 deposited via bridge input 1
     *   - 95 deposited via bridge input 2 <-- not possible, given input note 2 value
     *   - 5 paid as fee (in1's asset_id)
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "all of input note 2 must be defi-deposited");
}

TEST_F(join_split_tests, test_defi_invalid_tx_fee_asset_id_fails)
{
    join_split_tx tx = simple_setup({ 0, 11 });

    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    bridge_call_data.input_asset_id_b = tx.input_note[1].asset_id;
    bridge_call_data.config.second_input_in_use = true;
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    tx.asset_id = 666; // <-- different fee asset_id from both input notes.

    /**
     * tx represents:
     *   - 100 in1 (real)
     *   - 90 in2 (virtual)
     *   - 90 deposited via bridge input 1 (real)
     *   - 90 deposited via bridge input 2 (virtual)
     *   - 10 paid as fee (incorrect asset_id)
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "asset ids don't match");
}

TEST_F(join_split_tests, test_defi_deposit_of_zero_fails)
{
    join_split_tx tx = simple_setup();

    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.output_note[0].value = 0;
    tx.output_note[1].value = 0;
    tx.partial_claim_note.deposit_value = 0; // <-- should be > 0

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "Expected a nonzero defi_deposit_value for a defi-deposit");
}

TEST_F(join_split_tests, test_defi_non_zero_output_note_1_ignored)
{
    join_split_tx tx = simple_setup();
    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.output_note[0].value = 10; // This should be ignored in fee calculation.
    tx.output_note[1].value = 100;
    tx.partial_claim_note.deposit_value = 50;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    /**
     * tx represents:
     *   - 100 in1 (real)
     *   - 50 in2 (real, same asset_id)
     *   - 50 deposited via bridge input 1 (real, same asset_id)
     *   - 10 out1 (should be ignored, since we're doing a defi deposit)
     *   - 100 out2 (real, same asset_id)
     *   - 0 paid as fee
     */

    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner).valid);
}

TEST_F(join_split_tests, test_defi_allow_chain_1_fails)
{
    join_split_tx tx = simple_setup();
    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.output_note[1].value = 100;
    tx.partial_claim_note.deposit_value = 50;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    tx.allow_chain = 1;

    /**
     * tx represents:
     *   - 100 in1 (real)
     *   - 50 in2 (real, same asset_id)
     *   - 50 deposited via bridge input 1 (real, same asset_id)
     *   - 100 out2 (real, same asset_id)
     *   - 0 paid as fee
     *   - trying to chain off output_note_1, which is not allowed for defi deposits.
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "cannot chain from a partial claim note");
}

TEST_F(join_split_tests, test_defi_deposit_incorrect_input_nullifier_in_partial_claim_note_fails)
{
    join_split_tx tx = simple_setup({ 4, 13 });

    tx.num_input_notes = 1;
    tx.output_note[1].input_nullifier =
        compute_nullifier(tx.input_note[1].commit(), user.owner.private_key, false); // input note 2 is a dummy note

    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = 1; // incorrect nullifier

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    /**
     * tx represents:
     *   - 100 in1 (virtual)
     *   - 0 in2 (not in use)
     *   - 90 deposited
     *   - 10 paid as fee
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "partial claim note has incorrect input nullifier");
}

// *************************************************************************************************************
// BridgeCallData checks
// *************************************************************************************************************

TEST_F(join_split_tests, test_defi_deposit_bridge_call_data_second_bridge_input_nonzero_but_not_in_use_fails)
{
    join_split_tx tx = simple_setup({ 0, 11 });

    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    bridge_call_data.input_asset_id_b = tx.input_note[1].asset_id;
    bridge_call_data.config.second_input_in_use = false; // <-- to cause the contradiction
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    /**
     * tx represents:
     *   - 100 in1 (real)
     *   - 90 in2 (virtual)
     *   - 90 deposited via bridge input 1
     *   - 90 deposited via bridge input 2 (but bit_config contradicts this)
     *   - 10 paid as fee (in1's asset_id)
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "Expected second_input_in_use, given input_asset_id_b != 0");
}

TEST_F(join_split_tests, test_defi_deposit_bridge_call_data_second_bridge_output_nonzero_but_not_in_use_fails)
{
    join_split_tx tx = simple_setup({ 4, 13 });

    tx.num_input_notes = 1;
    tx.output_note[1].input_nullifier =
        compute_nullifier(tx.input_note[1].commit(), user.owner.private_key, false); // input note 2 is a dummy note

    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    bridge_call_data.output_asset_id_b = virtual_asset_id_flag;
    bridge_call_data.config.second_output_in_use = false; // <-- to cause the contradiction
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    /**
     * tx represents:
     *   - 100 in1 (real)
     *   - 0 in2 (not in use)
     *   - 90 deposited via bridge input 1
     *   - expectation of a virtual bridge output, but in_use flag not set.
     *   - 10 paid as fee (in1's asset_id)
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "Expected second_output_in_use, given output_asset_id_b != 0");
}

TEST_F(join_split_tests, test_defi_deposit_second_bridge_input_in_use_but_same_bridge_input_asset_ids_fails)
{
    join_split_tx tx = simple_setup();

    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 50;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;
    tx.output_note[1].value = 90;

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    bridge_call_data.input_asset_id_b = tx.input_note[1].asset_id; // <-- same asset_id
    bridge_call_data.config.second_input_in_use = true;
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    /**
     * tx represents:
     *   - 100 in1 (real)
     *   - 50 in2 (real, same asset_id)
     *   - 50 deposited via bridge input 1
     *   - 50 deposited via bridge input 2 <-- not allowed, since both inputs have the same asset_id (so they should be
     * combined into a single deposit of 100 via bridge input 1)
     *   - 90 out2
     *   - 10 paid as fee (in1's asset_id)
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "input asset ids must be different for the second bridge input to be in-use");
}

TEST_F(join_split_tests, test_defi_deposit_second_bridge_output_in_use_and_same_virtual_bridge_output_asset_ids)
{
    join_split_tx tx = simple_setup({ 4, 13 });

    tx.num_input_notes = 1;
    tx.output_note[1].input_nullifier =
        compute_nullifier(tx.input_note[1].commit(), user.owner.private_key, false); // input note 2 is a dummy note

    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    bridge_call_data.output_asset_id_a = virtual_asset_id_flag;
    bridge_call_data.output_asset_id_b =
        virtual_asset_id_flag; // <-- same, but that's ok, since they're virtual asset_id placeholders.
    bridge_call_data.config.second_output_in_use = true;
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    /**
     * tx represents:
     *   - 100 in1 (real)
     *   - 0 in2 (not in use)
     *   - 90 deposited via bridge input 1
     *   - expectation of two _virtual_ bridge outputs, so they have the same asset_id (placeholder values of 2 ** 29).
     *   - 10 paid as fee (in1's asset_id)
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_TRUE(result.valid);
}

TEST_F(join_split_tests, test_defi_deposit_second_bridge_output_in_use_but_same_real_bridge_output_asset_ids_fails)
{
    join_split_tx tx = simple_setup({ 4, 13 });

    tx.num_input_notes = 1;
    tx.output_note[1].input_nullifier =
        compute_nullifier(tx.input_note[1].commit(), user.owner.private_key, false); // input note 2 is a dummy note

    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    bridge_call_data.output_asset_id_a = asset_id;
    bridge_call_data.output_asset_id_b = asset_id; // <-- same - not ok, since they're both real output asset_ids
    bridge_call_data.config.second_output_in_use = true;
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    /**
     * tx represents:
     *   - 100 in1 (real)
     *   - 0 in2 (not in use)
     *   - 90 deposited via bridge input 1
     *   - expectation of two bridge outputs, but they have the same real asset_id (and so they should be combined into
     * bridge output 1).
     *   - 10 paid as fee (in1's asset_id)
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "real output asset ids must be different for the second bridge output to be in-use");
}

TEST_F(join_split_tests, test_defi_deposit_first_bridge_output_asset_id_virtual_but_incorrect_placeholder_value_fails)
{
    join_split_tx tx = simple_setup({ 4, 13 });

    tx.num_input_notes = 1;
    tx.output_note[1].input_nullifier =
        compute_nullifier(tx.input_note[1].commit(), user.owner.private_key, false); // input note 2 is a dummy note

    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    bridge_call_data.output_asset_id_a = virtual_asset_id_flag + 1; // should just be virtual_asset_id_flag (=2**29)
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    /**
     * tx represents:
     *   - 100 in1 (real)
     *   - 0 in2 (not in use)
     *   - 90 deposited via bridge input 1
     *   - expectation of one _virtual_ bridge output, but it has an invlaid placeholder asset_id.
     *   - 10 paid as fee (in1's asset_id)
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "output_asset_id_a detected as virtual, but has incorrect placeholder value");
}

TEST_F(join_split_tests, test_defi_deposit_second_bridge_output_asset_id_virtual_but_incorrect_placeholder_value_fails)
{
    join_split_tx tx = simple_setup({ 4, 13 });

    tx.num_input_notes = 1;
    tx.output_note[1].input_nullifier =
        compute_nullifier(tx.input_note[1].commit(), user.owner.private_key, false); // input note 2 is a dummy note

    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    bridge_call_data.output_asset_id_a = virtual_asset_id_flag;
    bridge_call_data.output_asset_id_b = virtual_asset_id_flag + 1; // should just be virtual_asset_id_flag (=2**29)
    bridge_call_data.config.second_output_in_use = true;
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    /**
     * tx represents:
     *   - 100 in1 (real)
     *   - 0 in2 (not in use)
     *   - 90 deposited via bridge input 1
     *   - expectation of two _virtual_ bridge outputs, but the 2nd has an invlaid placeholder asset_id.
     *   - 10 paid as fee (in1's asset_id)
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "output_asset_id_b detected as virtual, but has incorrect placeholder value");
}

// *************************************************************************************************************
// BridgeCallData data vs input note data
// *************************************************************************************************************

TEST_F(join_split_tests, test_defi_wrong_first_asset_id_in_bridge_call_data_fails)
{
    join_split_tx tx = simple_setup({ 4, 13 });

    tx.num_input_notes = 1;
    tx.output_note[1].input_nullifier =
        compute_nullifier(tx.input_note[1].commit(), user.owner.private_key, false); // input note 2 is a dummy note

    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id + 1; // wrong asset_id vs input notes
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    /**
     * tx represents:
     *   - 100 in1 (virtual)
     *   - 0 in2 (not in use)
     *   - 90 deposited
     *   - 10 paid as fee
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err,
              "Expected bridge_call_data_local.input_asset_id_a == input_note_1.asset_id for a defi-deposit");
}

TEST_F(join_split_tests, test_defi_bridge_call_data_config_second_input_in_use_but_input_note_2_not_in_use_fails)
{
    join_split_tx tx = simple_setup({ 4, 13 });

    tx.num_input_notes = 1;
    tx.output_note[1].input_nullifier =
        compute_nullifier(tx.input_note[1].commit(), user.owner.private_key, false); // input note 2 is a dummy note

    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    bridge_call_data.config.second_input_in_use =
        true; // <-- causes error, since there's no input note 2 that can be fed into the second bridge input.
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    /**
     * tx represents:
     *   - 100 in1 (virtual)
     *   - 0 in2 (not in use)
     *   - 90 deposited
     *   - 10 paid as fee
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "Expected input_note_2_in_use, given bridge_call_data_local.config.second_input_in_use");
}

TEST_F(join_split_tests, test_defi_missing_second_asset_in_bridge_call_data_fails)
{
    join_split_tx tx = simple_setup({ 0, 11 });

    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    /**
     * tx represents:
     *   - 100 in1 (real)
     *   - 90 in2 (virtual)
     *   - 90 deposited via bridge input 1 (real)
     *   - 90 SHOULD BE deposited via bridge input 2 (virtual), but bridge_call_data is incorrect.
     *   - 10 paid as fee (in1's asset_id)
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err,
              "Expected bridge_call_data_local.config.second_input_in_use, given input_note_2_in_use && "
              "different_input_asset_ids");
}

TEST_F(join_split_tests, test_defi_wrong_second_asset_id_in_bridge_call_data_fails)
{
    join_split_tx tx = simple_setup({ 4, 9 });

    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    bridge_call_data.input_asset_id_b = tx.input_note[1].asset_id + 1; // <-- wrong asset_id, vs input note 2's asset_id
    bridge_call_data.config.second_input_in_use = true;
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    /**
     * tx represents:
     *   - 100 in1 (virtual)
     *   - 90 in2 (virtual, different asset_id = virtual_asset_id + 1)
     *   - 90 deposited via bridge input 1
     *   - 90 deposited via bridge input 2 (but bridge_call_data is incorrect)
     *   - 10 paid as fee (in1's asset_id)
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err,
              "Expected bridge_call_data_local.input_asset_id_b == input_note_2.asset_id, given "
              "bridge_call_data_local.config.second_input_in_use");
}

// *************************************************************************************************************
// Virtual note repayment.
// *************************************************************************************************************

TEST_F(join_split_tests, test_repayment_logic)
{
    join_split_tx tx = simple_setup({ 0, 11 });

    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;
    tx.output_note[1].value = 10; // <-- repaying some value back to the defi-depositor

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    bridge_call_data.input_asset_id_b = virtual_asset_id_flag + defi_interaction_nonce; // virtual_asset_id
    bridge_call_data.config.second_input_in_use = true;
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    /**
     * tx represents:
     *   - 100 in1 (real)
     *   - 90 in2 (virtual)
     *   - 90 deposited via bridge input 1
     *   - 90 deposited via bridge input 2
     *   - 10 out2 repayment back to depositor (in1's asset_id)
     */

    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner).valid);
}

TEST_F(join_split_tests, test_virtual_note_repay_different_asset_id_fail)
{
    join_split_tx tx = simple_setup({ 0, 11 });

    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;
    tx.output_note[1].asset_id = 3; // <-- different from any of the input notes' asset_ids

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    bridge_call_data.input_asset_id_b = virtual_asset_id_flag + defi_interaction_nonce; // virtual_asset_id
    bridge_call_data.config.second_input_in_use = true;
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    /**
     * tx represents:
     *   - 100 in1 (real)
     *   - 90 in2 (virtual)
     *   - 90 deposited via bridge input 1
     *   - 90 deposited via bridge input 2
     *   - 10 out2 repayment back to depositor (INVALID asset_id)
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "asset ids don't match");
}

TEST_F(join_split_tests, test_real_input_value_lt_virtual_input_value_fails)
{
    join_split_tx tx = simple_setup({ 1, 11 });
    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    bridge_call_data.input_asset_id_b = tx.input_note[1].asset_id;
    bridge_call_data.config.second_input_in_use = true;
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    /**
     * tx represents:
     *   - 50 in1 (real)
     *   - 90 in2 (virtual)
     *   - 90 deposited via bridge input 1 (shouldn't be possible, given low in1 value)
     *   - 90 deposited via bridge input 2
     *   - 10 out2 repayment back to depositor (INVALID asset_id)
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    // In this context, failure of this subtraction (due to underflow) implies input note 1's value was less than input
    // note 2's value (and hence the defi_deposit_value) (which is not allowed).
    EXPECT_EQ(result.err, "safe_uint_t range constraint failure: subtract: total_in_value < total_out_value");
}

// *************************************************************************************************************
// Virtual note send tx
// *************************************************************************************************************

TEST_F(join_split_tests, test_send_two_virtual_notes)
{
    join_split_tx tx = simple_setup({ 4, 5 });

    /**
     * SEND tx represents:
     *   - 100 in1 (virtual)
     *   - 30 in2 (virtual, same asset_id)
     *   - 130 out1
     *   - 0 out2
     */

    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner).valid);
}

TEST_F(join_split_tests, test_send_one_virtual_note)
{
    join_split_tx tx = simple_setup({ 4, 13 });
    tx.num_input_notes = 1;
    tx.output_note[1].input_nullifier = compute_nullifier(tx.input_note[1].commit(), user.owner.private_key, false);

    /**
     *  SEND tx represents:
     *   - 100 in1 (virtual)
     *   - 0 in2 (not in use)
     *   - 100 out1
     *   - 0 out2
     */

    EXPECT_TRUE(sign_and_verify_logic(tx, user.owner).valid);
}

TEST_F(join_split_tests, test_send_two_virtual_notes_nonzero_public_value_fails)
{
    join_split_tx tx = simple_setup({ 4, 5 });
    tx.public_value = 10;

    /**
     * SEND tx represents:
     *   - 100 in1 (virtual)
     *   - 30 in2 (virtual, same asset_id)
     *   - 130 out1
     *   - 0 out2
     *   - public_value = 10 (not allowed for a send tx)
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "public value invalid");
}

TEST_F(join_split_tests, test_send_two_virtual_inputs_different_asset_ids_fails)
{
    join_split_tx tx = simple_setup({ 4, 9 });

    /**
     * SEND tx represents:
     *   - 100 in1 (virtual)
     *   - 90 in2 (virtual, different asset_id)
     *   - 190 out1
     *   - 0 out2
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "input asset ids must match unless defi-depositing");
}

TEST_F(join_split_tests, test_send_two_virtual_inputs_different_fee_asset_id_fails)
{
    join_split_tx tx = simple_setup({ 4, 5 });
    tx.asset_id = 0;

    /**
     * SEND tx represents:
     *   - 100 in1 (virtual)
     *   - 30 in2 (virtual, same asset_id)
     *   - 130 out1
     *   - 0 out2
     *   - public_asset_id = 0 (should be same as input_note_1's asset_id)
     */

    auto result = sign_and_verify_logic(tx, user.owner);
    EXPECT_FALSE(result.valid);
    EXPECT_EQ(result.err, "asset ids don't match");
}

// *************************************************************************************************************
// Creator pubkey
// *************************************************************************************************************

TEST_F(join_split_tests, test_non_zero_output_note_creator_pubkey_x)
{
    {
        join_split_tx tx = simple_setup();
        tx.output_note[0].creator_pubkey = user.owner.public_key.x;
        tx.output_note[1].creator_pubkey = user.owner.public_key.x;
        EXPECT_TRUE(sign_and_verify_logic(tx, user.owner).valid);
    }
    {
        join_split_tx tx = simple_setup();
        tx.output_note[0].creator_pubkey = user.owner.public_key.x;
        EXPECT_TRUE(sign_and_verify_logic(tx, user.owner).valid);
    }
    {
        join_split_tx tx = simple_setup();
        tx.output_note[1].creator_pubkey = user.owner.public_key.x;
        EXPECT_TRUE(sign_and_verify_logic(tx, user.owner).valid);
    }
}

TEST_F(join_split_tests, test_incorrect_output_note_creator_pubkey_x)
{
    {
        join_split_tx tx = simple_setup();
        tx.output_note[0].creator_pubkey =
            rollup::fixtures::create_key_pair(nullptr)
                .public_key.x; // setting creator to be different from sender (the owner of the input notes).
        auto result = sign_and_verify_logic(tx, user.owner);
        EXPECT_FALSE(result.valid);
        EXPECT_EQ(result.err, "output note 1 creator_pubkey mismatch");
    }
    {
        join_split_tx tx = simple_setup();
        tx.output_note[1].creator_pubkey =
            rollup::fixtures::create_key_pair(nullptr)
                .public_key.x; // setting creator to be different from sender (the owner of the input notes).
        auto result = sign_and_verify_logic(tx, user.owner);
        EXPECT_FALSE(result.valid);
        EXPECT_EQ(result.err, "output note 2 creator_pubkey mismatch");
    }
}

// *************************************************************************************************************
// Full proofs
// *************************************************************************************************************

TEST_F(join_split_tests, test_deposit_full_proof)
{
    join_split_tx tx = zero_input_setup();
    tx.proof_id = ProofIds::DEPOSIT;
    tx.public_value = 10;
    tx.public_owner = fr::random_element();
    tx.output_note[0].value = 7;

    /**
     * DEPOSIT tx represents:
     *   - public_value = 10
     *   - out1 = 7
     *   - fee = 3
     */

    auto proof = sign_and_create_proof(tx, user.owner);
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
    EXPECT_EQ(proof_data.bridge_call_data, uint256_t(0));
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

    /**
     * WITHDRAW tx represents:
     *   - 10 public_value
     *   - 100 in1 (real)
     *   - 50 in2 (real, same asset_id)
     *   - 137 out1
     *   - 3 paid as fee (in1's asset_id)
     */

    auto proof = sign_and_create_proof(tx, user.owner);
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
    EXPECT_EQ(proof_data.bridge_call_data, uint256_t(0));
    EXPECT_EQ(proof_data.defi_deposit_value, uint256_t(0));
    EXPECT_EQ(proof_data.defi_root, fr(0));

    EXPECT_TRUE(verify_proof(proof));
}

TEST_F(join_split_tests, test_private_send_full_proof)
{
    join_split_tx tx = simple_setup();
    tx.output_note[0].value -= 3;

    /**
     * SEND tx represents:
     *   - 100 in1 (real)
     *   - 50 in2 (real, same asset_id)
     *   - 147 out1
     *   - 3 paid as fee (in1's asset_id)
     */

    auto proof = sign_and_create_proof(tx, user.owner);
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
    EXPECT_EQ(proof_data.bridge_call_data, uint256_t(0));
    EXPECT_EQ(proof_data.defi_deposit_value, uint256_t(0));
    EXPECT_EQ(proof_data.defi_root, fr(0));
    EXPECT_EQ(proof_data.backward_link, fr(0));
    EXPECT_EQ(proof_data.allow_chain, uint256_t(0));

    EXPECT_TRUE(verify_proof(proof));
}

TEST_F(join_split_tests, test_defi_deposit_full_proof)
{
    join_split_tx tx = simple_setup();

    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 50;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;
    tx.output_note[1].value = 90;

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    bridge_call_data.output_asset_id_b = 1;
    bridge_call_data.config.second_output_in_use = true;
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    /**
     * tx represents:
     *   - 100 in1 (real)
     *   - 50 in2 (real, same asset_id)
     *   - 50 deposited via bridge input 1
     *   - expecting two real assets to be returned by the bridge
     *   - 90 out2
     *   - 10 paid as fee (in1's asset_id)
     */

    auto proof = sign_and_create_proof(tx, user.owner);
    EXPECT_TRUE(verify_proof(proof));

    auto proof_data = inner_proof_data(proof.proof_data);

    auto partial_value_commitment = value::create_partial_commitment(
        tx.partial_claim_note.note_secret, tx.input_note[0].owner, tx.input_note[0].account_required, 0);
    claim::claim_note claim_note = {
        tx.partial_claim_note.deposit_value,  tx.partial_claim_note.bridge_call_data, 0, 0, partial_value_commitment,
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
    EXPECT_EQ(proof_data.tx_fee_asset_id, bridge_call_data.input_asset_id_a);
    EXPECT_EQ(proof_data.bridge_call_data, tx.partial_claim_note.bridge_call_data);
    EXPECT_EQ(proof_data.defi_deposit_value, tx.partial_claim_note.deposit_value);
    EXPECT_EQ(proof_data.defi_root, fr(0));

    EXPECT_TRUE(verify_proof(proof));
}

TEST_F(join_split_tests, test_repayment_full_proof)
{
    join_split_tx tx = simple_setup({ 0, 11 });

    tx.proof_id = ProofIds::DEFI_DEPOSIT;
    tx.partial_claim_note.deposit_value = 90;
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;
    tx.output_note[1].value = 10; // <-- repaying some value back to the defi-depositor

    bridge_call_data bridge_call_data = empty_bridge_call_data;
    bridge_call_data.input_asset_id_a = tx.input_note[0].asset_id;
    bridge_call_data.input_asset_id_b = virtual_asset_id_flag + defi_interaction_nonce; // virtual_asset_id
    bridge_call_data.config.second_input_in_use = true;
    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();

    /**
     * tx represents:
     *   - 100 in1 (real)
     *   - 90 in2 (virtual)
     *   - 90 deposited via bridge input 1
     *   - 90 deposited via bridge input 2
     *   - 10 out2 (repay back to depositor in in1's asset_id)
     */

    tx.partial_claim_note.bridge_call_data = bridge_call_data.to_uint256_t();
    tx.partial_claim_note.input_nullifier = tx.output_note[0].input_nullifier;

    auto proof = sign_and_create_proof(tx, user.owner);
    auto proof_data = inner_proof_data(proof.proof_data);

    auto partial_commitment = value::create_partial_commitment(
        tx.partial_claim_note.note_secret, tx.input_note[0].owner, tx.input_note[0].account_required, 0);
    claim::claim_note claim_note = {
        tx.partial_claim_note.deposit_value,  tx.partial_claim_note.bridge_call_data, 0, 0, partial_commitment,
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
    EXPECT_EQ(proof_data.tx_fee, uint256_t(0));
    EXPECT_EQ(proof_data.tx_fee_asset_id, bridge_call_data.input_asset_id_a);
    EXPECT_EQ(proof_data.bridge_call_data, tx.partial_claim_note.bridge_call_data);
    EXPECT_EQ(proof_data.defi_deposit_value, tx.partial_claim_note.deposit_value);
    EXPECT_EQ(proof_data.defi_root, fr(0));

    EXPECT_TRUE(verify_proof(proof));
}

TEST_F(join_split_tests, test_send_two_virtual_notes_full_proof)
{
    join_split_tx tx = simple_setup({ 4, 5 });

    /**
     * SEND tx represents:
     *   - 100 in1 (virtual)
     *   - 30 in2 (virtual, same asset_id)
     *   - 130 out1
     *   - 0 out2
     */

    auto proof = sign_and_create_proof(tx, user.owner);

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