#include "index.hpp"
#include "../notes/native/index.hpp"
#include "../../fixtures/test_context.hpp"
#include <common/test.hpp>
#include <common/map.hpp>

namespace rollup {
namespace proofs {
namespace rollup {

using namespace barretenberg;
using namespace notes;

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
        : context(js_cd, account_cd, claim_cd)
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

    auto create_tx_with_1_defi()
    {
        context.append_value_notes({ 100, 50 });
        context.start_next_root_rollup();

        notes::native::bridge_id bid = { 0, 2, 0, 0, 0 };
        auto defi_proof1 = context.create_defi_proof({ 0, 1 }, { 100, 50 }, { 40, 110 }, bid);

        return create_rollup_tx(context.world_state, 1, { defi_proof1 }, { bid });
    }

    auto create_tx_with_3_defi()
    {
        context.append_value_notes({ 100, 50, 100, 50, 100, 50 });
        context.append_value_notes({ 200, 40 }, 1);
        context.start_next_root_rollup();

        notes::native::bridge_id bid1 = { 0, 2, 0, 0, 0 };
        notes::native::bridge_id bid2 = { 1, 2, 1, 0, 0 };
        auto js_proof = context.create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });
        auto defi_proof1 = context.create_defi_proof({ 2, 3 }, { 100, 50 }, { 40, 110 }, bid1);
        auto defi_proof2 = context.create_defi_proof({ 4, 5 }, { 100, 50 }, { 30, 120 }, bid1);
        auto defi_proof3 = context.create_defi_proof({ 6, 7 }, { 200, 40 }, { 20, 220 }, bid2, 1);

        return create_rollup_tx(
            context.world_state, 4, { js_proof, defi_proof1, defi_proof2, defi_proof3 }, { bid1, bid2 });
    }

    fixtures::TestContext context;
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
    auto join_split_proof = create_noop_join_split_proof(js_cd, context.world_state.data_tree.root());

    auto rollup = create_rollup_tx(context.world_state, rollup_size, { join_split_proof });
    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_0_proof_in_1_rollup)
{
    auto rollup = create_empty_rollup(context.world_state);
    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_1_proof_in_1_rollup)
{
    size_t rollup_size = 1;

    context.append_account_notes();
    context.append_value_notes({ 100, 50 });
    context.start_next_root_rollup();
    auto join_split_proof = context.create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup_tx(context.world_state, rollup_size, { join_split_proof });
    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_1_proof_with_old_root_in_1_rollup)
{
    size_t rollup_size = 1;

    // Insert rollup 0 at index 1.
    context.append_account_notes();
    context.append_value_notes({ 100, 50 });
    context.start_next_root_rollup();

    // Create proof which references root at index 1.
    auto join_split_proof = context.create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 });
    auto data_root_index = 1U;

    // Insert rollup 1.
    context.append_value_notes({ 30, 40 });
    context.start_next_root_rollup();

    // Create rollup 2 with old join-split.
    auto rollup = create_rollup_tx(context.world_state, rollup_size, { join_split_proof }, {}, { data_root_index });

    inner_proof_data data(join_split_proof);
    EXPECT_TRUE(data.merkle_root != rollup.old_data_root);

    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_1_proof_with_invalid_old_null_root_fails)
{
    size_t rollup_size = 1;

    context.append_account_notes();
    context.append_value_notes({ 100, 50 });
    context.start_next_root_rollup();
    auto join_split_proof = context.create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 });
    auto rollup = create_rollup_tx(context.world_state, rollup_size, { join_split_proof });

    rollup.old_null_root = fr::random_element();

    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(result.logic_verified);
}

TEST_F(rollup_tests, test_incorrect_data_start_index_fails)
{
    size_t rollup_size = 1;

    context.append_account_notes();
    context.append_value_notes({ 100, 50 });
    context.start_next_root_rollup();
    auto join_split_proof = context.create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup_tx(context.world_state, rollup_size, { join_split_proof });
    rollup.data_start_index = 0;
    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(result.logic_verified);
}

TEST_F(rollup_tests, test_larger_total_output_value_fails)
{
    size_t rollup_size = 1;

    context.append_account_notes();
    context.append_value_notes({ 100, 50 });
    context.start_next_root_rollup();
    auto join_split_proof = context.create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 90 });

    auto rollup = create_rollup_tx(context.world_state, rollup_size, { join_split_proof });
    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(result.logic_verified);
}

TEST_F(rollup_tests, test_reuse_spent_note_fails)
{
    size_t rollup_size = 1;

    context.append_value_notes({ 100, 50 });
    context.start_next_root_rollup();
    auto join_split_proof = context.create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 });
    inner_proof_data inner_proof_data(join_split_proof);
    context.world_state.nullify(inner_proof_data.nullifier1);

    auto rollup = create_rollup_tx(context.world_state, rollup_size, { join_split_proof });
    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(result.logic_verified);
}

TEST_F(rollup_tests, test_max_num_txs)
{
    size_t rollup_size = 1;
    auto join_split_proof = create_noop_join_split_proof(js_cd, context.world_state.data_tree.root());

    auto rollup = create_rollup_tx(context.world_state, rollup_size, { join_split_proof });
    rollup.num_txs = (uint32_t(1) << MAX_TXS_BIT_LENGTH) - 1;
    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_overflow_num_txs_fails)
{
    size_t rollup_size = 1;
    auto join_split_proof = create_noop_join_split_proof(js_cd, context.world_state.data_tree.root());

    auto rollup = create_rollup_tx(context.world_state, rollup_size, { join_split_proof });
    rollup.num_txs = uint32_t(1) << MAX_TXS_BIT_LENGTH;

    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(result.logic_verified);
}

// Asset Id
TEST_F(rollup_tests, test_invalid_asset_id_fails)
{
    size_t rollup_size = 1;
    uint32_t invalid_asset_id = NUM_ASSETS;

    context.append_account_notes();
    context.append_value_notes({ 100, 50 }, invalid_asset_id);
    context.start_next_root_rollup();
    auto join_split_proof =
        context.create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 }, 0, 0, 0, 0, invalid_asset_id);

    auto rollup = create_rollup_tx(context.world_state, rollup_size, { join_split_proof });
    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(result.logic_verified);
}

// Account
TEST_F(rollup_tests, test_1_account_proof_in_1_rollup)
{
    size_t rollup_size = 1;

    auto create_account = context.create_account_proof();
    auto rollup = create_rollup_tx(context.world_state, rollup_size, { create_account });
    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_reuse_nullified_account_alias_id_fails)
{
    size_t rollup_size = 1;

    context.append_account_notes();
    auto account_alias_id = fixtures::generate_account_alias_id(context.user.alias_hash, 0);
    context.nullify_account_alias_id(account_alias_id);
    context.start_next_root_rollup();

    auto account_proof = context.create_account_proof();
    auto rollup = create_rollup_tx(context.world_state, rollup_size, { account_proof });
    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(result.logic_verified);
}

// Rollups of size 2.
TEST_F(rollup_tests, test_1_proof_in_2_rollup)
{
    size_t rollup_size = 2;

    context.append_account_notes();
    context.append_value_notes({ 100, 50 });
    context.start_next_root_rollup();
    auto join_split_proof = context.create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup_tx(context.world_state, rollup_size, { join_split_proof });
    auto result = verify_logic(rollup, rollup_2_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_2_proofs_in_2_rollup)
{
    size_t rollup_size = 2;

    context.append_account_notes();
    context.append_value_notes({ 0, 0, 100, 50, 80, 60 });
    context.start_next_root_rollup();
    auto join_split_proof1 = context.create_join_split_proof({ 4, 5 }, { 100, 50 }, { 70, 80 });
    auto join_split_proof2 = context.create_join_split_proof({ 6, 7 }, { 80, 60 }, { 70, 70 });
    auto txs = std::vector{ join_split_proof1, join_split_proof2 };

    auto rollup = create_rollup_tx(context.world_state, rollup_size, txs);
    auto result = verify_logic(rollup, rollup_2_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_1_js_proof_1_account_proof_in_2_rollup)
{
    size_t rollup_size = 2;

    context.append_account_notes();
    context.append_value_notes({ 0, 0, 100, 50, 80, 60 });
    context.start_next_root_rollup();
    auto join_split_proof = context.create_join_split_proof({ 4, 5 }, { 100, 50 }, { 70, 80 });
    auto account_proof = context.create_account_proof();
    auto txs = std::vector{ join_split_proof, account_proof };

    auto rollup = create_rollup_tx(context.world_state, rollup_size, txs);
    auto result = verify_logic(rollup, rollup_2_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_create_rollup_picks_correct_data_start_index)
{
    size_t rollup_size = 2;

    context.append_account_notes();
    // Add a couple of additional notes taking total to 6.
    context.append_value_notes({ 100, 50, 0, 0 });
    context.start_next_root_rollup();
    auto join_split_proof = context.create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup_tx(context.world_state, rollup_size, { join_split_proof });

    EXPECT_EQ(rollup.data_start_index, 8UL);
}

TEST_F(rollup_tests, test_same_input_note_in_two_proofs_fails)
{
    size_t rollup_size = 2;

    context.append_account_notes();
    context.append_value_notes({ 0, 0, 100, 50, 80, 60 });
    context.start_next_root_rollup();
    auto join_split_proof1 = context.create_join_split_proof({ 4, 5 }, { 100, 50 }, { 70, 80 });
    auto join_split_proof2 = context.create_join_split_proof({ 6, 5 }, { 80, 50 }, { 70, 60 });
    auto txs = std::vector{ join_split_proof1, join_split_proof2 };

    auto rollup = create_rollup_tx(context.world_state, rollup_size, txs);
    auto result = verify_logic(rollup, rollup_2_keyless);

    EXPECT_FALSE(result.logic_verified);
}

TEST_F(rollup_tests, test_nullifier_hash_path_consistency)
{
    size_t rollup_size = 2;

    context.append_account_notes();
    context.append_value_notes({ 0, 0, 100, 50, 80, 60 });
    context.start_next_root_rollup();
    auto join_split_proof1 = context.create_join_split_proof({ 4, 5 }, { 100, 50 }, { 70, 80 });
    auto join_split_proof2 = context.create_join_split_proof({ 6, 7 }, { 80, 60 }, { 70, 70 });
    auto txs = std::vector{ join_split_proof1, join_split_proof2 };

    auto rollup = create_rollup_tx(context.world_state, rollup_size, txs);

    std::swap(rollup.new_null_roots[2], rollup.new_null_roots[3]);
    std::swap(rollup.old_null_paths[2], rollup.old_null_paths[3]);

    auto result = verify_logic(rollup, rollup_2_keyless);

    EXPECT_FALSE(result.logic_verified);
}

// Rollups of size 3.
TEST_F(rollup_tests, test_1_proof_in_3_rollup)
{
    size_t rollup_size = 3;

    context.append_account_notes();
    context.append_value_notes({ 100, 50 });
    context.start_next_root_rollup();
    auto join_split_proof = context.create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 });

    auto rollup = create_rollup_tx(context.world_state, rollup_size, { join_split_proof });
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
    notes::native::value::value_note note1 = { 70, 0, 0, context.user.owner.public_key, context.user.note_secret };
    auto expected1 = encrypt(note1);
    EXPECT_EQ(rollup_data.inner_proofs[0].new_note1, expected1);

    notes::native::value::value_note note2 = { 80, 0, 0, context.user.owner.public_key, context.user.note_secret };
    auto expected2 = encrypt(note2);
    EXPECT_EQ(rollup_data.inner_proofs[0].new_note2, expected2);

    // Check correct interaction nonce in claim notes.
    auto check_defi_proof = [&](uint32_t i, uint32_t claim_note_interaction_nonce) {
        auto defi_proof = rollup_data.inner_proofs[i];
        auto deposit_value = defi_proof.public_output;
        auto bid = defi_proof.asset_id;

        auto partial_state =
            notes::native::claim::create_partial_value_note(context.user.note_secret, context.user.owner.public_key, 0);
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

    std::vector<native::defi_interaction::note> dins = { { bids[0], 0, 70, 700, 7000, true },
                                                         { bids[1], 0, 20, 2, 3, true } };
    context.start_next_root_rollup(dins);

    rollup::rollup_proof_data data(result.public_inputs);

    // js and defi proofs to be rolled up with claim proofs
    auto js_proof = context.create_join_split_proof({}, {}, { 100, 30 }, 130);
    auto defi_proof1 = context.create_defi_proof(
        { data.data_start_index, data.data_start_index + 1 }, { 70, 80 }, { 120, 30 }, bids[0]);

    // Create claim proofs for each claim note in previous rollup.
    auto claim_proofs = mapi(data.inner_proofs, [&](auto inner, auto i) {
        if (inner.proof_id != ProofIds::DEFI_DEPOSIT) {
            return std::vector<uint8_t>();
        }
        auto claim_note_index = data.data_start_index + uint32_t(2 * i);
        return context.create_claim_proof(inner.asset_id, inner.public_output, claim_note_index);
    });

    auto rollup2_tx =
        create_rollup_tx(context.world_state, 4, { js_proof, claim_proofs[1], defi_proof1, claim_proofs[2] }, bids);
    auto result2 = verify_logic(rollup2_tx, rollup_4_keyless);
    EXPECT_TRUE(result2.logic_verified);
}

TEST_F(rollup_tests, test_defi_claim_proof_has_valid_defi_root)
{
    auto rollup1_tx = create_tx_with_1_defi();
    auto bids = rollup1_tx.bridge_ids;
    auto result = verify_logic(rollup1_tx, rollup_1_keyless);
    ASSERT_TRUE(result.logic_verified);

    std::vector<native::defi_interaction::note> dins = { { bids[0], 0, 70, 700, 7000, true },
                                                         { bids[1], 0, 20, 2, 3, true } };
    context.start_next_root_rollup(dins);

    rollup::rollup_proof_data data(result.public_inputs);

    // Create claim proof with trash defi root.
    auto inner = data.inner_proofs[0];
    auto tx = context.create_claim_tx(inner.asset_id, inner.public_output, 2);
    tx.defi_root = fr::random_element();
    auto claim_proof = claim::create_proof(tx, context.claim_cd);

    auto rollup2_tx = create_rollup_tx(context.world_state, 1, { claim_proof }, bids);
    auto result2 = verify_logic(rollup2_tx, rollup_1_keyless);
    EXPECT_FALSE(result2.logic_verified);
}

} // namespace rollup
} // namespace proofs
} // namespace rollup