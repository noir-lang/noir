#include "index.hpp"
#include "../notes/native/index.hpp"
#include <common/test.hpp>
#include <common/map.hpp>

namespace rollup {
namespace proofs {
namespace rollup {

using namespace barretenberg;
using namespace notes::native::value;
using namespace notes::native::account;
using WorldState = world_state::WorldState<MemoryStore>;

namespace {
join_split::circuit_data js_cd;
account::circuit_data account_cd;
claim::circuit_data claim_cd;
rollup::circuit_data rollup_1_keyless;
rollup::circuit_data rollup_2_keyless;
rollup::circuit_data rollup_3_keyless;
rollup::circuit_data rollup_4_keyless;
} // namespace

class rollup_tests : public ::testing::Test {
  protected:
    rollup_tests()
        : rand_engine(&numeric::random::get_debug_engine(true))
        , user(fixtures::create_user_context(rand_engine))
        , js_tx_factory(world_state, user)
        , account_tx_factory(world_state, user)
        , claim_tx_factory(world_state, user)
    {}

    static void SetUpTestCase()
    {
        std::string CRS_PATH = "../srs_db";
        auto srs = std::make_shared<waffle::DynamicFileReferenceStringFactory>(CRS_PATH);
        account_cd = account::compute_circuit_data(srs);
        js_cd = join_split::compute_circuit_data(srs);
        claim_cd = claim::get_circuit_data(srs, "", true, false, false);
        rollup_1_keyless = rollup::get_circuit_data(1, js_cd, account_cd, claim_cd, srs, "", false, false, false);
        rollup_2_keyless = rollup::get_circuit_data(2, js_cd, account_cd, claim_cd, srs, "", false, false, false);
        rollup_3_keyless = rollup::get_circuit_data(3, js_cd, account_cd, claim_cd, srs, "", false, false, false);
        rollup_4_keyless = rollup::get_circuit_data(4, js_cd, account_cd, claim_cd, srs, "", false, false, false);
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
        return join_split::create_proof(tx, signer, js_cd);
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
        return join_split::create_proof(tx, signer, js_cd);
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

    auto create_tx_with_3_defi()
    {
        append_notes({ 100, 50, 100, 50, 100, 50, 100, 50 });
        world_state.update_root_tree_with_data_root();

        notes::native::bridge_id bid1 = { 0, 2, 0, 0, 0 };
        notes::native::bridge_id bid2 = { 1, 2, 0, 0, 0 };
        auto js_proof = create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });
        auto defi_proof1 = create_defi_proof({ 2, 3 }, { 100, 50 }, { 40, 110 }, bid1);
        auto defi_proof2 = create_defi_proof({ 4, 5 }, { 100, 50 }, { 30, 120 }, bid1);
        auto defi_proof3 = create_defi_proof({ 6, 7 }, { 100, 50 }, { 20, 130 }, bid2);

        return create_rollup(world_state, 4, { js_proof, defi_proof1, defi_proof2, defi_proof3 }, { bid1, bid2 });
    }

    numeric::random::Engine* rand_engine;
    WorldState world_state;
    fixtures::user_context user;
    join_split::JoinSplitTxFactory<WorldState> js_tx_factory;
    account::AccountTxFactory<WorldState> account_tx_factory;
    claim::ClaimTxFactory<WorldState> claim_tx_factory;
};

TEST_F(rollup_tests, test_padding_proof)
{
    Composer composer = Composer(js_cd.proving_key, js_cd.verification_key, js_cd.num_gates);
    join_split::join_split_circuit(composer, join_split::noop_tx());
    auto verifier = composer.create_unrolled_verifier();
    EXPECT_TRUE(verifier.verify_proof({ js_cd.padding_proof }));
}

TEST_F(rollup_tests, test_1_deposit_proof_in_1_rollup)
{
    size_t rollup_size = 1;
    auto join_split_proof = create_noop_join_split_proof(js_cd, world_state.data_tree.root());

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
    auto join_split_proof = create_noop_join_split_proof(js_cd, world_state.data_tree.root());

    auto rollup = create_rollup(world_state, rollup_size, { join_split_proof });
    rollup.num_txs = (uint32_t(1) << MAX_TXS_BIT_LENGTH) - 1;
    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_overflow_num_txs_fails)
{
    size_t rollup_size = 1;
    auto join_split_proof = create_noop_join_split_proof(js_cd, world_state.data_tree.root());

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

TEST_F(rollup_tests, test_defi_deposit_sums_accumulated)
{
    auto tx = create_tx_with_3_defi();
    auto result = verify_logic(tx, rollup_4_keyless);
    ASSERT_TRUE(result.logic_verified);

    auto rollup_data = rollup_proof_data(result.public_inputs);

    // Check correct defi deposit_sums.
    EXPECT_EQ(rollup_data.bridge_ids[0], tx.bridge_ids[0]);
    EXPECT_EQ(rollup_data.bridge_ids[1], tx.bridge_ids[1]);
    EXPECT_EQ(rollup_data.bridge_ids[2], 0);
    EXPECT_EQ(rollup_data.bridge_ids[3], 0);
    EXPECT_EQ(rollup_data.deposit_sums[0], 70);
    EXPECT_EQ(rollup_data.deposit_sums[1], 20);
    EXPECT_EQ(rollup_data.deposit_sums[2], 0);
    EXPECT_EQ(rollup_data.deposit_sums[3], 0);
}

TEST_F(rollup_tests, test_defi_interaction_nonce_added_to_claim_notes)
{
    auto tx = create_tx_with_3_defi();
    auto result = verify_logic(tx, rollup_4_keyless);
    ASSERT_TRUE(result.logic_verified);

    auto rollup_data = rollup_proof_data(result.public_inputs);

    // Check regular join-split output note1 unchanged (as we change it for defi deposits).
    notes::native::value::value_note value_note = { 70, 0, 0, user.owner.public_key, user.note_secret };
    auto expected = encrypt(value_note);
    EXPECT_EQ(rollup_data.inner_proofs[0].new_note1, expected);

    // Check correct interaction nonce in claim notes.
    auto check_defi_proof = [&](uint32_t i, uint32_t claim_note_interaction_nonce) {
        auto defi_proof = rollup_data.inner_proofs[i];
        auto deposit_value = defi_proof.public_output;
        auto bid = defi_proof.asset_id;

        auto partial_state =
            notes::native::claim::create_partial_value_note(user.note_secret, user.owner.public_key, 0);
        notes::native::claim::claim_note claim_note = {
            deposit_value, bid, claim_note_interaction_nonce, partial_state
        };
        auto expected = encrypt(claim_note);

        EXPECT_EQ(defi_proof.new_note1, expected);
    };

    check_defi_proof(1, 4);
    check_defi_proof(2, 4);
    check_defi_proof(3, 5);
}

TEST_F(rollup_tests, test_defi_claim_proofs)
{
    auto rollup1_tx = create_tx_with_3_defi();
    auto bids = rollup1_tx.bridge_ids;
    auto result = verify_logic(rollup1_tx, rollup_4_keyless);
    ASSERT_TRUE(result.logic_verified);

    // Update root tree to simulate creation of root rollup.
    world_state.update_root_tree_with_data_root();

    rollup::rollup_proof_data data(result.public_inputs);

    // Create defi interaction notes for interactions in rollup 1.
    uint32_t din_insertion_index = data.rollup_id * NUM_BRIDGE_CALLS_PER_BLOCK;
    std::vector<notes::native::defi_interaction::defi_interaction_note> dins = {
        { bids[0], din_insertion_index, 70, 700, 7000, true }, { bids[1], din_insertion_index + 1, 20, 2, 3, true }
    };
    world_state.add_defi_notes(dins);

    // TODO: Publish on deposit and extract directly.
    auto partial_state = notes::native::claim::create_partial_value_note(user.note_secret, user.owner.public_key, 0);

    // Create claim proofs for each claim note in previous rollup.
    auto claim_proofs = mapi(data.inner_proofs, [&](auto inner, auto i) {
        if (inner.proof_id != ProofIds::DEFI_DEPOSIT) {
            return std::vector<uint8_t>();
        }

        uint32_t bid_index = 0;
        while (inner.asset_id != bids[bid_index]) {
            bid_index++;
        }
        auto nonce = din_insertion_index + bid_index;
        auto claim_note_index = data.data_start_index + uint32_t(2 * i);
        notes::native::claim::claim_note claim_note = { inner.public_output, inner.asset_id, nonce, partial_state };
        auto tx = claim_tx_factory.create_claim_tx(
            world_state.defi_tree.root(), claim_note_index, claim_note, dins[bid_index]);
        return claim::create_proof(tx, claim_cd);
    });

    auto rollup2_tx = create_rollup(world_state, 4, { claim_proofs[1], claim_proofs[2], claim_proofs[3] }, bids);
    auto result2 = verify_logic(rollup2_tx, rollup_4_keyless);
    EXPECT_TRUE(result2.logic_verified);
}

TEST_F(rollup_tests, test_defi_claim_proof_has_valid_defi_root)
{
    auto rollup1_tx = create_tx_with_3_defi();
    auto bids = rollup1_tx.bridge_ids;
    auto result = verify_logic(rollup1_tx, rollup_4_keyless);
    ASSERT_TRUE(result.logic_verified);

    // Update root tree to simulate creation of root rollup.
    world_state.update_root_tree_with_data_root();

    rollup::rollup_proof_data data(result.public_inputs);

    // Create defi interaction notes for interactions in rollup 1.
    uint32_t din_insertion_index = data.rollup_id * NUM_BRIDGE_CALLS_PER_BLOCK;
    std::vector<notes::native::defi_interaction::defi_interaction_note> dins = {
        { bids[0], din_insertion_index, 70, 700, 7000, true }, { bids[1], din_insertion_index + 1, 20, 2, 3, true }
    };
    world_state.add_defi_notes(dins);

    auto partial_state = notes::native::claim::create_partial_value_note(user.note_secret, user.owner.public_key, 0);
    auto inner = data.inner_proofs[1];
    notes::native::claim::claim_note claim_note = {
        inner.public_output, inner.asset_id, din_insertion_index, partial_state
    };

    // Create claim proof with trash defi root.
    auto tx = claim_tx_factory.create_claim_tx(world_state.defi_tree.root(), 10, claim_note, dins[0]);
    tx.defi_root = fr::random_element();
    auto claim_proof = claim::create_proof(tx, claim_cd);

    auto rollup2_tx = create_rollup(world_state, 4, { claim_proof }, bids);
    auto result2 = verify_logic(rollup2_tx, rollup_4_keyless);
    EXPECT_FALSE(result2.logic_verified);
}

} // namespace rollup
} // namespace proofs
} // namespace rollup