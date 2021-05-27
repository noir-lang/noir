#include "compute_circuit_data.hpp"
#include "create_rollup.hpp"
#include "rollup_proof_data.hpp"
#include "verify.hpp"
#include "../../fixtures/user_context.hpp"
#include "../inner_proof_data.hpp"
#include "../join_split/join_split.hpp"
#include "../join_split/join_split_circuit.hpp"
#include "../join_split/create_proof.hpp"
#include "../account/account.hpp"
#include "../account/account_tx_factory.hpp"
#include "../account/create_proof.hpp"
#include "../join_split/sign_join_split_tx.hpp"
#include "../notes/native/value/encrypt.hpp"
#include "../notes/native/account/encrypt.hpp"
#include "../notes/native/account/compute_account_alias_id_nullifier.hpp"
#include "../notes/native/claim/bridge_id.hpp"
#include "../notes/native/claim/encrypt.hpp"
#include "../notes/native/claim/create_partial_value_note.hpp"
#include "../notes/constants.hpp"
#include "../join_split/join_split_tx_factory.hpp"
#include "../join_split/compute_circuit_data.hpp"
#include "../join_split/create_noop_join_split_proof.hpp"
#include "../inner_proof_data.hpp"
#include "../../constants.hpp"
#include "../../world_state/world_state.hpp"
#include <common/test.hpp>
#include <stdlib/merkle_tree/memory_store.hpp>
#include <stdlib/merkle_tree/membership.hpp>

namespace rollup {
namespace proofs {
namespace rollup {

using namespace barretenberg;
using namespace notes::native::value;
using namespace notes::native::account;
using WorldState = world_state::WorldState<MemoryStore>;

namespace {
join_split::circuit_data join_split_cd;
account::circuit_data account_cd;
rollup::circuit_data rollup_1_keyless;
rollup::circuit_data rollup_2_keyless;
rollup::circuit_data rollup_3_keyless;
} // namespace

class rollup_tests : public ::testing::Test {
  protected:
    rollup_tests()
        : rand_engine(&numeric::random::get_debug_engine(true))
        , user(fixtures::create_user_context(rand_engine))
        , js_tx_factory(world_state, user)
        , account_tx_factory(world_state, user)
    {}

    static void SetUpTestCase()
    {
        std::string CRS_PATH = "../srs_db";
        auto srs = std::make_shared<waffle::DynamicFileReferenceStringFactory>(CRS_PATH);
        account_cd = account::compute_circuit_data(srs);
        join_split_cd = join_split::compute_circuit_data(srs);
        rollup_1_keyless = rollup::get_circuit_data(1, join_split_cd, account_cd, srs, "", false, false, false);
        rollup_2_keyless = rollup::get_circuit_data(2, join_split_cd, account_cd, srs, "", false, false, false);
        rollup_3_keyless = rollup::get_circuit_data(3, join_split_cd, account_cd, srs, "", false, false, false);
    }

    void append_notes(std::vector<uint32_t> const& values, uint32_t asset_id = 0)
    {
        for (auto v : values) {
            notes::native::value::value_note note = { v, asset_id, 0, user.owner.public_key, user.note_secret };
            world_state.append_data_note(note);
        }
    }

    void append_account_notes()
    {
        auto account_alias_id = fixtures::generate_account_alias_id(user.alias_hash, 1);
        notes::native::account::account_note note1 = { account_alias_id,
                                                       user.owner.public_key,
                                                       user.signing_keys[0].public_key };
        notes::native::account::account_note note2 = { account_alias_id,
                                                       user.owner.public_key,
                                                       user.signing_keys[1].public_key };
        world_state.append_data_note(note1);
        world_state.append_data_note(note2);
    }

    void nullify_account_alias_id(fr const& account_alias_id)
    {
        world_state.nullify(notes::native::account::compute_account_alias_id_nullifier(account_alias_id));
    }

    std::vector<uint8_t> create_join_split_proof(std::array<uint32_t, 2> in_note_idx,
                                                 std::array<uint32_t, 2> in_note_value,
                                                 std::array<uint32_t, 2> out_note_value,
                                                 uint256_t public_input = 0,
                                                 uint256_t public_output = 0,
                                                 uint256_t tx_fee = 7,
                                                 uint32_t account_note_idx = 0,
                                                 uint32_t asset_id = 0,
                                                 uint32_t nonce = 0)
    {
        auto tx = js_tx_factory.create_join_split_tx(in_note_idx,
                                                     in_note_value,
                                                     out_note_value,
                                                     public_input,
                                                     public_output,
                                                     tx_fee,
                                                     account_note_idx,
                                                     asset_id,
                                                     nonce);
        auto signer = nonce ? user.signing_keys[0] : user.owner;
        return join_split::create_proof(tx, signer, join_split_cd);
    }

    std::vector<uint8_t> create_defi_proof(std::array<uint32_t, 2> in_note_idx,
                                           std::array<uint32_t, 2> in_note_value,
                                           std::array<uint32_t, 2> out_note_value,
                                           uint256_t bridge_id,
                                           uint32_t asset_id = 0,
                                           uint32_t nonce = 0)
    {

        auto tx = js_tx_factory.create_defi_deposit_tx(in_note_idx, in_note_value, out_note_value, bridge_id, asset_id);
        auto signer = nonce ? user.signing_keys[0] : user.owner;
        return join_split::create_proof(tx, signer, join_split_cd);
    }

    std::vector<uint8_t> create_account_proof(uint32_t nonce = 0, uint32_t account_note_idx = 0)
    {
        auto tx = account_tx_factory.create_tx(nonce, account_note_idx);
        auto signer = nonce ? user.signing_keys[0] : user.owner;
        return account::create_proof(tx, signer, account_cd);
    }

    auto create_tx_with_1_defi()
    {
        append_notes({ 100, 50 });
        world_state.update_root_tree_with_data_root();

        notes::native::bridge_id bid = { 0, 2, 0, 0, 0 };
        auto defi_proof1 = create_defi_proof({ 0, 1 }, { 100, 50 }, { 40, 110 }, bid);

        return create_rollup(world_state, 1, { defi_proof1 }, { bid });
    }

    numeric::random::Engine* rand_engine;
    WorldState world_state;
    fixtures::user_context user;
    join_split::JoinSplitTxFactory<WorldState> js_tx_factory;
    account::AccountTxFactory<WorldState> account_tx_factory;
};

TEST_F(rollup_tests, test_padding_proof)
{
    Composer composer = Composer(join_split_cd.proving_key, join_split_cd.verification_key, join_split_cd.num_gates);
    join_split::join_split_circuit(composer, join_split::noop_tx());
    auto verifier = composer.create_unrolled_verifier();
    EXPECT_TRUE(verifier.verify_proof({ join_split_cd.padding_proof }));
}

TEST_F(rollup_tests, test_1_deposit_proof_in_1_rollup)
{
    size_t rollup_size = 1;
    auto join_split_proof = create_noop_join_split_proof(join_split_cd, world_state.data_tree.root());

    auto rollup = create_rollup(world_state, rollup_size, { join_split_proof });
    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_0_proof_in_1_rollup)
{
    auto rollup = create_empty_rollup(world_state);
    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_1_proof_in_1_rollup)
{
    size_t rollup_size = 1;

    append_account_notes();
    append_notes({ 100, 50 });
    world_state.update_root_tree_with_data_root();
    auto join_split_proof = create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup(world_state, rollup_size, { join_split_proof });
    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_1_proof_with_old_root_in_1_rollup)
{
    size_t rollup_size = 1;

    // Insert rollup 0 at index 1.
    append_account_notes();
    append_notes({ 100, 50 });
    world_state.update_root_tree_with_data_root();

    // Create proof which references root at index 1.
    auto join_split_proof = create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 });
    auto data_root_index = 1U;

    // Insert rollup 1.
    append_notes({ 30, 40 });
    world_state.update_root_tree_with_data_root();

    // Create rollup 2 with old join-split.
    auto rollup = create_rollup(world_state, rollup_size, { join_split_proof }, {}, { data_root_index });

    inner_proof_data data(join_split_proof);
    EXPECT_TRUE(data.merkle_root != rollup.old_data_root);

    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_1_proof_with_invalid_old_null_root_fails)
{
    size_t rollup_size = 1;

    append_account_notes();
    append_notes({ 100, 50 });
    world_state.update_root_tree_with_data_root();
    auto join_split_proof = create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 });
    auto rollup = create_rollup(world_state, rollup_size, { join_split_proof });

    rollup.old_null_root = fr::random_element();

    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(result.logic_verified);
}

TEST_F(rollup_tests, test_incorrect_data_start_index_fails)
{
    size_t rollup_size = 1;

    append_account_notes();
    append_notes({ 100, 50 });
    world_state.update_root_tree_with_data_root();
    auto join_split_proof = create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup(world_state, rollup_size, { join_split_proof });
    rollup.data_start_index = 0;
    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(result.logic_verified);
}

TEST_F(rollup_tests, test_larger_total_output_value_fails)
{
    size_t rollup_size = 1;

    append_account_notes();
    append_notes({ 100, 50 });
    world_state.update_root_tree_with_data_root();
    auto join_split_proof = create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 90 });

    auto rollup = create_rollup(world_state, rollup_size, { join_split_proof });
    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(result.logic_verified);
}

TEST_F(rollup_tests, test_reuse_spent_note_fails)
{
    size_t rollup_size = 1;

    append_notes({ 100, 50 });
    world_state.update_root_tree_with_data_root();
    auto join_split_proof = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });
    inner_proof_data inner_proof_data(join_split_proof);
    world_state.nullify(inner_proof_data.nullifier1);

    auto rollup = create_rollup(world_state, rollup_size, { join_split_proof });
    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(result.logic_verified);
}

TEST_F(rollup_tests, test_max_num_txs)
{
    size_t rollup_size = 1;
    auto join_split_proof = create_noop_join_split_proof(join_split_cd, world_state.data_tree.root());

    auto rollup = create_rollup(world_state, rollup_size, { join_split_proof });
    rollup.num_txs = (uint32_t(1) << MAX_TXS_BIT_LENGTH) - 1;
    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_overflow_num_txs_fails)
{
    size_t rollup_size = 1;
    auto join_split_proof = create_noop_join_split_proof(join_split_cd, world_state.data_tree.root());

    auto rollup = create_rollup(world_state, rollup_size, { join_split_proof });
    rollup.num_txs = uint32_t(1) << MAX_TXS_BIT_LENGTH;

    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(result.logic_verified);
}

// Asset Id
TEST_F(rollup_tests, test_invalid_asset_id_fails)
{
    size_t rollup_size = 1;
    uint32_t invalid_asset_id = NUM_ASSETS;

    append_account_notes();
    append_notes({ 100, 50 }, invalid_asset_id);
    world_state.update_root_tree_with_data_root();
    auto join_split_proof = create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 }, 0, 0, 0, 0, invalid_asset_id);

    auto rollup = create_rollup(world_state, rollup_size, { join_split_proof });
    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(result.logic_verified);
}

// Account
TEST_F(rollup_tests, test_1_account_proof_in_1_rollup)
{
    size_t rollup_size = 1;

    auto create_account = create_account_proof();
    auto rollup = create_rollup(world_state, rollup_size, { create_account });
    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_reuse_nullified_account_alias_id_fails)
{
    size_t rollup_size = 1;

    append_account_notes();
    auto account_alias_id = fixtures::generate_account_alias_id(user.alias_hash, 0);
    nullify_account_alias_id(account_alias_id);
    world_state.update_root_tree_with_data_root();

    auto account_proof = create_account_proof();
    auto rollup = create_rollup(world_state, rollup_size, { account_proof });
    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(result.logic_verified);
}

// Rollups of size 2.
TEST_F(rollup_tests, test_1_proof_in_2_rollup)
{
    size_t rollup_size = 2;

    append_account_notes();
    append_notes({ 100, 50 });
    world_state.update_root_tree_with_data_root();
    auto join_split_proof = create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup(world_state, rollup_size, { join_split_proof });
    auto result = verify_logic(rollup, rollup_2_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_2_proofs_in_2_rollup)
{
    size_t rollup_size = 2;

    append_account_notes();
    append_notes({ 0, 0, 100, 50, 80, 60 });
    world_state.update_root_tree_with_data_root();
    auto join_split_proof1 = create_join_split_proof({ 4, 5 }, { 100, 50 }, { 70, 80 });
    auto join_split_proof2 = create_join_split_proof({ 6, 7 }, { 80, 60 }, { 70, 70 });
    auto txs = std::vector{ join_split_proof1, join_split_proof2 };

    auto rollup = create_rollup(world_state, rollup_size, txs);
    auto result = verify_logic(rollup, rollup_2_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_1_js_proof_1_account_proof_in_2_rollup)
{
    size_t rollup_size = 2;

    append_account_notes();
    append_notes({ 0, 0, 100, 50, 80, 60 });
    world_state.update_root_tree_with_data_root();
    auto join_split_proof = create_join_split_proof({ 4, 5 }, { 100, 50 }, { 70, 80 });
    auto account_proof = create_account_proof();
    auto txs = std::vector{ join_split_proof, account_proof };

    auto rollup = create_rollup(world_state, rollup_size, txs);
    auto result = verify_logic(rollup, rollup_2_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_create_rollup_picks_correct_data_start_index)
{
    size_t rollup_size = 2;

    append_account_notes();
    // Add a couple of additional notes taking total to 6.
    append_notes({ 100, 50, 0, 0 });
    world_state.update_root_tree_with_data_root();
    auto join_split_proof = create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup(world_state, rollup_size, { join_split_proof });

    EXPECT_EQ(rollup.data_start_index, 8UL);
}

TEST_F(rollup_tests, test_same_input_note_in_two_proofs_fails)
{
    size_t rollup_size = 2;

    append_account_notes();
    append_notes({ 0, 0, 100, 50, 80, 60 });
    world_state.update_root_tree_with_data_root();
    auto join_split_proof1 = create_join_split_proof({ 4, 5 }, { 100, 50 }, { 70, 80 });
    auto join_split_proof2 = create_join_split_proof({ 6, 5 }, { 80, 50 }, { 70, 60 });
    auto txs = std::vector{ join_split_proof1, join_split_proof2 };

    auto rollup = create_rollup(world_state, rollup_size, txs);
    auto result = verify_logic(rollup, rollup_2_keyless);

    EXPECT_FALSE(result.logic_verified);
}

TEST_F(rollup_tests, test_nullifier_hash_path_consistency)
{
    size_t rollup_size = 2;

    append_account_notes();
    append_notes({ 0, 0, 100, 50, 80, 60 });
    world_state.update_root_tree_with_data_root();
    auto join_split_proof1 = create_join_split_proof({ 4, 5 }, { 100, 50 }, { 70, 80 });
    auto join_split_proof2 = create_join_split_proof({ 6, 7 }, { 80, 60 }, { 70, 70 });
    auto txs = std::vector{ join_split_proof1, join_split_proof2 };

    auto rollup = create_rollup(world_state, rollup_size, txs);

    std::swap(rollup.new_null_roots[2], rollup.new_null_roots[3]);
    std::swap(rollup.new_null_paths[2], rollup.new_null_paths[3]);
    std::swap(rollup.old_null_paths[2], rollup.old_null_paths[3]);

    auto result = verify_logic(rollup, rollup_2_keyless);

    EXPECT_FALSE(result.logic_verified);
}

// Rollups of size 3.
TEST_F(rollup_tests, test_1_proof_in_3_rollup)
{
    size_t rollup_size = 3;

    append_account_notes();
    append_notes({ 100, 50 });
    world_state.update_root_tree_with_data_root();
    auto join_split_proof = create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup(world_state, rollup_size, { join_split_proof });
    auto result = verify_logic(rollup, rollup_3_keyless);

    EXPECT_TRUE(result.logic_verified);
}

// Defi tests.
TEST_F(rollup_tests, test_defi_proof_in_rollup)
{
    auto tx = create_tx_with_1_defi();
    auto result = verify_logic(tx, rollup_1_keyless);

    ASSERT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_defi_bridge_id_zero_fails)
{
    auto tx = create_tx_with_1_defi();
    tx.bridge_ids = { 0 };
    auto result = verify_logic(tx, rollup_1_keyless);

    ASSERT_FALSE(result.logic_verified);
}

TEST_F(rollup_tests, test_defi_bridge_id_repeated_fails)
{
    auto tx = create_tx_with_1_defi();
    tx.bridge_ids.push_back(tx.bridge_ids[0]);
    auto result = verify_logic(tx, rollup_1_keyless);

    ASSERT_FALSE(result.logic_verified);
}

TEST_F(rollup_tests, test_defi_bridge_id_unmatched_fails)
{
    auto tx = create_tx_with_1_defi();
    tx.bridge_ids[0] = { 1, 2, 0, 0 };
    auto result = verify_logic(tx, rollup_1_keyless);

    ASSERT_FALSE(result.logic_verified);
}

TEST_F(rollup_tests, test_defi_claim_notes_added_interaction_nonce)
{
    auto tx = create_tx_with_1_defi();
    auto result = verify_logic(tx, rollup_1_keyless);

    ASSERT_TRUE(result.logic_verified);

    auto rollup_data = rollup_proof_data(result.public_inputs);
    auto js_data = rollup_data.inner_proofs[0];
    const auto bid = tx.bridge_ids[0];

    auto partial_state = notes::native::claim::create_partial_value_note(user.note_secret, user.owner.public_key, 0);
    notes::native::claim::claim_note claim_note = { js_data.public_output, js_data.asset_id, 4, partial_state };
    auto expected = encrypt(claim_note);

    EXPECT_EQ(js_data.new_note1, expected);
}

/*
TEST_F(rollup_tests, test_process_defi_deposits)
{
    auto tx = create_root_rollup_tx_with_3_defi();
    auto result = verify_logic(tx, root_rollup_cd);
    assert(result.logic_verified);

    // Check correct defi deposit_sums.
    const auto defi_info_public_inputs = result.public_inputs.size() - 17;
    EXPECT_EQ(uint256_t(result.public_inputs[defi_info_public_inputs]), tx.bridge_ids[0]);
    EXPECT_EQ(uint256_t(result.public_inputs[defi_info_public_inputs + 1]), 70);
    EXPECT_EQ(uint256_t(result.public_inputs[defi_info_public_inputs + 2]), tx.bridge_ids[1]);
    EXPECT_EQ(uint256_t(result.public_inputs[defi_info_public_inputs + 3]), 20);
    EXPECT_EQ(uint256_t(result.public_inputs[defi_info_public_inputs + 4]), 0);
    EXPECT_EQ(uint256_t(result.public_inputs[defi_info_public_inputs + 5]), 0);
    EXPECT_EQ(uint256_t(result.public_inputs[defi_info_public_inputs + 6]), 0);
    EXPECT_EQ(uint256_t(result.public_inputs[defi_info_public_inputs + 7]), 0);

    // Check regular join-split output note1 unchanged (as we change it for defi deposits).
    const auto public_input_start_idx = rollup::RollupProofFields::INNER_PROOFS_DATA;
    const auto output_note1_x = result.public_inputs[public_input_start_idx + InnerProofFields::NEW_NOTE1_X];
    const auto output_note1_y = result.public_inputs[public_input_start_idx + InnerProofFields::NEW_NOTE1_Y];
    notes::native::value::value_note value_note = { 100, 0, 0, user.owner.public_key, user.note_secret };
    auto expected = encrypt(value_note);
    EXPECT_EQ(output_note1_x, expected.x);
    EXPECT_EQ(output_note1_y, expected.y);

    // Check correct interaction nonce in claim notes.
    auto check_defi_proof = [&](verify_result const& result, uint32_t i, uint32_t claim_note_interaction_nonce) {
        const auto public_input_start_idx =
            rollup::RollupProofFields::INNER_PROOFS_DATA + (i * InnerProofFields::NUM_PUBLISHED);
        const auto output_note1_x = result.public_inputs[public_input_start_idx + InnerProofFields::NEW_NOTE1_X];
        const auto output_note1_y = result.public_inputs[public_input_start_idx + InnerProofFields::NEW_NOTE1_Y];
        const auto deposit_value = result.public_inputs[public_input_start_idx + InnerProofFields::PUBLIC_OUTPUT];
        const auto bid = result.public_inputs[public_input_start_idx + InnerProofFields::ASSET_ID];

        auto partial_state =
            notes::native::claim::create_partial_value_note(user.note_secret, user.owner.public_key, 0);
        notes::native::claim::claim_note claim_note = {
            deposit_value, bid, claim_note_interaction_nonce, partial_state
        };
        info(claim_note);
        auto expected = encrypt(claim_note);

        EXPECT_EQ(output_note1_x, expected.x);
        EXPECT_EQ(output_note1_y, expected.y);
    };

    check_defi_proof(result, 1, 4);
    check_defi_proof(result, 2, 4);
    check_defi_proof(result, 3, 5);
}

TEST_F(rollup_tests, test_claim_proof_has_valid_defi_root)
{
    auto test_name = ::testing::UnitTest::GetInstance()->current_test_info()->name();
    auto rollup1_tx = create_root_rollup_tx_with_3_defi();
    auto result = verify_logic(rollup1_tx, root_rollup_cd);
    assert(result.logic_verified);

    rollup::rollup_proof_data data(result.public_inputs);
    auto bids = rollup1_tx.bridge_ids;

    // Create defi interaction notes for interactions in rollup 1, to insert in rollup 2.
    uint32_t din_insertion_index = data.rollup_id * NUM_BRIDGE_CALLS_PER_BLOCK;
    std::vector<notes::native::defi_interaction::defi_interaction_note> dins = {
        { bids[0], din_insertion_index, 70, 700, 7000, true }, { bids[1], din_insertion_index + 1, 0, 0, 0, true }
    };

    // We need to add interaction notes before creating claim notes, so record the old state of the tree.
    auto& defi_tree = world_state.defi_tree;
    auto old_defi_root = defi_tree.root();
    auto old_defi_path = defi_tree.get_hash_path(0);
    world_state.add_defi_notes(dins);
    auto new_defi_root = defi_tree.root();
    auto new_defi_path = defi_tree.get_hash_path(0);

    // TODO: Publish on deposit and extract directly.
    auto partial_state = notes::native::claim::create_partial_value_note(user.note_secret, user.owner.public_key, 0);

    auto claim_proofs = mapi(data.inner_proofs, [&](auto inner, auto i) {
        // We only inserted defi deposits in rollup 1.
        if (inner.proof_id != ProofIds::DEFI_DEPOSIT) {
            return std::vector<uint8_t>();
        }

        uint32_t bid_index = 0;
        while (inner.asset_id != bids[bid_index]) {
            bid_index++;
        }
        auto nonce = din_insertion_index + bid_index;
        auto claim_note_index = data.data_start_index + uint32_t(2 * i);
        info("rollup 1 deposit proof ", i, " claim_note_index = ", claim_note_index);
        notes::native::claim::claim_note claim_note = { inner.public_output, inner.asset_id, nonce, partial_state };
        info(claim_note);
        info(encrypt(claim_note));
        return create_claim_proof(new_defi_root, claim_note_index, claim_note, dins[bid_index]);
    });

    auto rollup2_tx =
        create_root_rollup_tx(test_name, 2, { { claim_proofs[1], claim_proofs[2] }, { claim_proofs[3] } });
    rollup2_tx.num_defi_interactions = dins.size();
    rollup2_tx.old_defi_interaction_root = old_defi_root;
    rollup2_tx.old_defi_interaction_path = old_defi_path;
    rollup2_tx.new_defi_interaction_root = new_defi_root;
    rollup2_tx.new_defi_interaction_path = new_defi_path;
    auto result2 = verify_logic(rollup2_tx, root_rollup_cd);
    EXPECT_TRUE(result2.logic_verified);
}
*/
} // namespace rollup
} // namespace proofs
} // namespace rollup