#include "index.hpp"
#include "../notes/native/index.hpp"
#include "../../fixtures/test_context.hpp"
#include "../../fixtures/compute_or_load_fixture.hpp"
#include "../join_split/create_noop_join_split_proof.hpp"
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
rollup::circuit_data rollup_5_keyless;
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
        account_cd = account::get_circuit_data(srs);
        js_cd = join_split::get_circuit_data(srs);
        claim_cd = claim::get_circuit_data(srs);
        rollup_1_keyless = rollup::get_circuit_data(1, js_cd, account_cd, claim_cd, srs, "", false, false, false);
        rollup_2_keyless = rollup::get_circuit_data(2, js_cd, account_cd, claim_cd, srs, "", false, false, false);
        rollup_3_keyless = rollup::get_circuit_data(3, js_cd, account_cd, claim_cd, srs, "", false, false, false);
        rollup_4_keyless = rollup::get_circuit_data(4, js_cd, account_cd, claim_cd, srs, "", false, false, false);
        rollup_5_keyless = rollup::get_circuit_data(5, js_cd, account_cd, claim_cd, srs, "", false, false, false);
    }

    auto create_tx_with_1_defi()
    {
        context.append_value_notes({ 100, 50 });
        context.start_next_root_rollup();

        const notes::native::bridge_id bid = { .bridge_address_id = 0,
                                               .input_asset_id_a = 0,
                                               .input_asset_id_b = 0,
                                               .output_asset_id_a = 111,
                                               .output_asset_id_b = 222,
                                               .config =
                                                   notes::native::bridge_id::bit_config{ .second_input_in_use = false,
                                                                                         .second_output_in_use = true },
                                               .aux_data = 0 };
        // MIKE started here
        auto defi_proof1 = context.create_defi_proof({ 0, 1 }, { 100, 50 }, { 40, 110 }, bid);

        return create_rollup_tx(context.world_state, 1, { defi_proof1 }, { bid });
    }

    auto create_tx_with_3_defi()
    {
        context.append_value_notes({ 100, 50 });
        context.append_value_notes({ 100, 50, 100, 50 }, 8);
        context.append_value_notes({ 200, 40 }, 13);
        context.start_next_root_rollup();

        const notes::native::bridge_id bid1 = { .bridge_address_id = 0,
                                                .input_asset_id_a = 8,
                                                .input_asset_id_b = 0,
                                                .output_asset_id_a = 0,
                                                .output_asset_id_b = 1,
                                                .config =
                                                    notes::native::bridge_id::bit_config{
                                                        .second_input_in_use = false, .second_output_in_use = true },
                                                .aux_data = 0 };

        const notes::native::bridge_id bid2 = { .bridge_address_id = 1,
                                                .input_asset_id_a = 13,
                                                .input_asset_id_b = 0,
                                                .output_asset_id_a = 0,
                                                .output_asset_id_b = 1,
                                                .config =
                                                    notes::native::bridge_id::bit_config{
                                                        .second_input_in_use = false, .second_output_in_use = true },
                                                .aux_data = 0 };
        auto js_proof = context.create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 73 });         // fee = 7
        auto defi_proof1 = context.create_defi_proof({ 2, 3 }, { 100, 50 }, { 40, 100 }, bid1, 8);  // fee = 10
        auto defi_proof2 = context.create_defi_proof({ 4, 5 }, { 100, 50 }, { 30, 80 }, bid1, 8);   // fee = 40
        auto defi_proof3 = context.create_defi_proof({ 6, 7 }, { 200, 40 }, { 20, 207 }, bid2, 13); // fee = 13

        return create_rollup_tx(
            context.world_state, 4, { js_proof, defi_proof1, defi_proof2, defi_proof3 }, { bid1, bid2 }, { 0, 8, 13 });
    }

    auto create_tx_with_defi_loan()
    {
        context.append_value_notes({ 150, 50, 40, 160 });
        context.append_value_notes({ 100, 50 }, 1);
        context.start_next_root_rollup();

        const notes::native::bridge_id bid1 = { .bridge_address_id = 0,
                                                .input_asset_id_a = 0,
                                                .input_asset_id_b = 0,
                                                .output_asset_id_a = 2,
                                                .output_asset_id_b = virtual_asset_id_flag,
                                                .config =
                                                    notes::native::bridge_id::bit_config{
                                                        .second_input_in_use = false, .second_output_in_use = true },
                                                .aux_data = 0 };

        const notes::native::bridge_id bid2 = { .bridge_address_id = 1,
                                                .input_asset_id_a = 1,
                                                .input_asset_id_b = 0,
                                                .output_asset_id_a = 3,
                                                .output_asset_id_b = virtual_asset_id_flag,
                                                .config =
                                                    notes::native::bridge_id::bit_config{
                                                        .second_input_in_use = false, .second_output_in_use = true },
                                                .aux_data = 0 };
        auto defi_proof1 = context.create_defi_proof({ 0, 1 }, { 150, 50 }, { 180, 0 }, bid1, 0); // fee = 20
        auto defi_proof2 = context.create_defi_proof({ 2, 3 }, { 40, 160 }, { 190, 0 }, bid1, 0); // fee = 10
        auto defi_proof3 = context.create_defi_proof({ 4, 5 }, { 100, 50 }, { 150, 0 }, bid2, 1); // fee = 0

        return create_rollup_tx(
            context.world_state, 4, { defi_proof1, defi_proof2, defi_proof3 }, { bid1, bid2 }, { 0, 1 });
    }

    auto create_js_proof(join_split::join_split_tx& tx)
    {
        context.js_tx_factory.finalise_and_sign_tx(tx, context.user.owner);
        return join_split::create_proof(tx, js_cd);
    }

    auto create_tx_with_3_defi_include_non_fee_asset()
    {
        context.append_value_notes({ 100, 50 });
        context.append_value_notes({ 100, 50, 100, 50 }, 8);
        context.append_value_notes({ 200, 40 }, 25); // not a fee paying asset
        context.start_next_root_rollup();

        const notes::native::bridge_id bid1 = { .bridge_address_id = 0,
                                                .input_asset_id_a = 8,
                                                .input_asset_id_b = 0,
                                                .output_asset_id_a = 0,
                                                .output_asset_id_b = 1,
                                                .config =
                                                    notes::native::bridge_id::bit_config{
                                                        .second_input_in_use = false, .second_output_in_use = true },
                                                .aux_data = 0 };

        const notes::native::bridge_id bid2 = { .bridge_address_id = 2,
                                                .input_asset_id_a = 25,
                                                .input_asset_id_b = 0,
                                                .output_asset_id_a = 0,
                                                .output_asset_id_b = 1,
                                                .config =
                                                    notes::native::bridge_id::bit_config{
                                                        .second_input_in_use = false, .second_output_in_use = true },
                                                .aux_data = 0 };
        auto js_proof = context.create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 73 });         // fee = 7
        auto defi_proof1 = context.create_defi_proof({ 2, 3 }, { 100, 50 }, { 40, 100 }, bid1, 8);  // fee = 10
        auto defi_proof2 = context.create_defi_proof({ 4, 5 }, { 100, 50 }, { 30, 80 }, bid1, 8);   // fee = 40
        auto defi_proof3 = context.create_defi_proof({ 6, 7 }, { 200, 40 }, { 20, 207 }, bid2, 25); // fee = 13

        return create_rollup_tx(
            context.world_state, 4, { js_proof, defi_proof1, defi_proof2, defi_proof3 }, { bid1, bid2 }, { 0, 8 });
    }

    void test_chain_off_disallowed_note_fails(uint32_t allow_chain, size_t indicator);

    fixtures::TestContext context;
    const uint32_t virtual_asset_id_flag = (uint32_t(1) << (MAX_NUM_ASSETS_BIT_LENGTH - 1));
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
    auto join_split_proof = join_split::create_noop_join_split_proof(js_cd, context.world_state.data_tree.root());

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
    auto rollup =
        create_rollup_tx(context.world_state, rollup_size, { join_split_proof }, {}, { 0 }, { data_root_index });

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
    EXPECT_EQ(result.err, "check_nullifiers_inserted_0_old_value");
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
    EXPECT_EQ(result.err, "batch_update_membership_old_subtree");
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
    EXPECT_EQ(result.err, "check_nullifiers_inserted_0_old_value");
}

TEST_F(rollup_tests, test_max_num_txs)
{
    size_t rollup_size = 1;
    auto join_split_proof = join_split::create_noop_join_split_proof(js_cd, context.world_state.data_tree.root());

    auto rollup = create_rollup_tx(context.world_state, rollup_size, { join_split_proof });
    rollup.num_txs = (uint32_t(1) << MAX_TXS_BIT_LENGTH) - 1;
    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_overflow_num_txs_fails)
{
    size_t rollup_size = 1;
    auto join_split_proof = join_split::create_noop_join_split_proof(js_cd, context.world_state.data_tree.root());

    auto rollup = create_rollup_tx(context.world_state, rollup_size, { join_split_proof });
    rollup.num_txs = uint32_t(1) << MAX_TXS_BIT_LENGTH;

    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(result.logic_verified);
}

// Asset Ids
TEST_F(rollup_tests, test_invalid_asset_id_fails)
{
    size_t rollup_size = 1;
    uint32_t invalid_asset_id = MAX_NUM_ASSETS;

    context.append_account_notes();
    context.append_value_notes({ 100, 50 }, invalid_asset_id);
    context.start_next_root_rollup();
    auto join_split_proof =
        context.create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 }, 0, 0, 0, invalid_asset_id);

    auto rollup = create_rollup_tx(context.world_state, rollup_size, { join_split_proof }, {}, { invalid_asset_id });
    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(result.logic_verified);
    EXPECT_EQ(result.err, "asset_id out of scope");
}

TEST_F(rollup_tests, test_asset_id_repeated_fails)
{
    auto tx = create_tx_with_3_defi();
    tx.asset_ids.push_back(tx.asset_ids[0]);
    auto result = verify_logic(tx, rollup_4_keyless);

    ASSERT_FALSE(result.logic_verified);
    EXPECT_EQ(result.err, "proof asset id matched 2 times");
}

TEST_F(rollup_tests, test_proof_asset_id_not_in_assets)
{
    // txs can have non-fee paying assets
    // these need to be accepted and asset constraint bypassed
    auto tx = create_tx_with_3_defi_include_non_fee_asset();
    // 4th tx has asset that is not included in list of assets on rollup
    auto result = verify_logic(tx, rollup_4_keyless);
    ASSERT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_asset_id_reordering_works)
{
    auto tx = create_tx_with_3_defi();
    tx.asset_ids = { 8, 0, 13 };
    auto result = verify_logic(tx, rollup_4_keyless);

    ASSERT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_asset_id_output_order)
{
    auto tx = create_tx_with_3_defi();
    auto result = verify_logic(tx, rollup_4_keyless);
    ASSERT_TRUE(result.logic_verified);

    auto rollup_data = rollup_proof_data(result.public_inputs);

    // Check correct asset ids order.
    EXPECT_EQ(rollup_data.asset_ids[0], tx.asset_ids[0]); // asset_id 0
    EXPECT_EQ(rollup_data.asset_ids[1], tx.asset_ids[1]); // asset_id 8
    EXPECT_EQ(rollup_data.asset_ids[2], tx.asset_ids[2]); // asset_id 13
    EXPECT_EQ(rollup_data.asset_ids[3], MAX_NUM_ASSETS);  // padding

    // Check correct tx_fee accumulation.
    EXPECT_EQ(rollup_data.total_tx_fees[0], 7);  // asset_id 0
    EXPECT_EQ(rollup_data.total_tx_fees[1], 25); // asset_id 8, net_tx_fee = 50/2
    EXPECT_EQ(rollup_data.total_tx_fees[2], 6);  // asset_id 13, net_tx_fee = 13/2
    EXPECT_EQ(rollup_data.total_tx_fees[3], 0);  // padding
}

// Account
TEST_F(rollup_tests, test_1_account_proof_in_1_rollup)
{
    size_t rollup_size = 1;

    auto create_account = context.create_new_account_proof();
    auto rollup = create_rollup_tx(context.world_state, rollup_size, { create_account });
    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_reuse_nullified_account_alias_hash_fails)
{
    size_t rollup_size = 1;

    context.append_account_notes();
    context.nullify_account_alias_hash(context.user.alias_hash);
    context.start_next_root_rollup();

    auto account_proof = context.create_new_account_proof();
    auto rollup = create_rollup_tx(context.world_state, rollup_size, { account_proof });
    auto result = verify_logic(rollup, rollup_1_keyless);

    EXPECT_FALSE(result.logic_verified);
}

TEST_F(rollup_tests, test_reuse_nullified_account_public_key_fails)
{
    size_t rollup_size = 1;

    context.append_account_notes();
    context.nullify_account_public_key(context.user.owner.public_key);
    context.start_next_root_rollup();

    auto account_proof = context.create_new_account_proof();
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
    auto account_proof = context.create_migrate_account_proof();
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

// Chaining
TEST_F(rollup_tests, test_chain_off_first_output_note_and_consume_in_first_input_note)
{
    size_t rollup_size = 2;

    context.append_value_notes({ 100, 50 });
    context.start_next_root_rollup();

    auto tx1 = context.js_tx_factory.create_join_split_tx({ 0 }, { 100 }, { 70, 30 });
    tx1.allow_chain = 1;
    auto join_split_proof1 = create_js_proof(tx1);

    // Chain off the prior tx's first output note, and join with the second preloaded note from data tree index 1.
    // First input index of tx2 is set to 0 but it's not actually used (since the propagated note doesn't exist in the
    // tree yet).
    auto tx2 = context.js_tx_factory.create_join_split_tx({ 0, 1 }, { 70, 50 }, { 120, 0 });
    tx2.input_note[0] = tx1.output_note[0];
    tx2.backward_link = tx2.input_note[0].commit();
    auto join_split_proof2 = create_js_proof(tx2);

    auto rollup = create_rollup_tx(context.world_state, rollup_size, { join_split_proof1, join_split_proof2 });
    auto result = verify_logic(rollup, rollup_2_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_chain_off_first_output_note_and_consume_in_second_input_note)
{
    size_t rollup_size = 2;

    context.append_value_notes({ 100, 50 });
    context.start_next_root_rollup();

    auto tx1 = context.js_tx_factory.create_join_split_tx({ 0 }, { 100 }, { 70, 30 });
    tx1.allow_chain = 1;
    auto join_split_proof1 = create_js_proof(tx1);

    // Chain off the prior tx's first output note, and join with the second preloaded note from data tree index 1.
    // First input index of tx2 is set to 0 but it's not actually used (since the propagated note doesn't exist in the
    // tree yet).
    auto tx2 = context.js_tx_factory.create_join_split_tx({ 1, 0 }, { 50, 70 }, { 120, 0 });
    tx2.input_note[1] = tx1.output_note[0];
    tx2.backward_link = tx2.input_note[1].commit();
    auto join_split_proof2 = create_js_proof(tx2);

    auto rollup = create_rollup_tx(context.world_state, rollup_size, { join_split_proof1, join_split_proof2 });
    auto result = verify_logic(rollup, rollup_2_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_chain_off_second_output_note_and_consume_in_first_input_note)
{
    size_t rollup_size = 2;

    context.append_value_notes({ 100, 50 });
    context.start_next_root_rollup();

    auto tx1 = context.js_tx_factory.create_join_split_tx({ 0 }, { 100 }, { 70, 30 });
    tx1.allow_chain = 2;
    auto join_split_proof1 = create_js_proof(tx1);

    // Chain off tx1's second output note, and join with the second preloaded note from data tree index 1.
    // Second input index of tx2 is set to 0 but it's not actually used (since the propagated note doesn't exist in the
    // tree yet).
    auto tx2 = context.js_tx_factory.create_join_split_tx({ 0, 1 }, { 30, 50 }, { 80, 0 });
    tx2.input_note[0] = tx1.output_note[1];
    tx2.backward_link = tx2.input_note[0].commit();
    auto join_split_proof2 = create_js_proof(tx2);

    auto rollup = create_rollup_tx(context.world_state, rollup_size, { join_split_proof1, join_split_proof2 });
    auto result = verify_logic(rollup, rollup_2_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_chain_off_second_output_note_and_consume_in_second_input_note)
{
    size_t rollup_size = 2;

    context.append_value_notes({ 100, 50 });
    context.start_next_root_rollup();

    auto tx1 = context.js_tx_factory.create_join_split_tx({ 0 }, { 100 }, { 70, 30 });
    tx1.allow_chain = 2;
    auto join_split_proof1 = create_js_proof(tx1);

    // Chain off tx1's second output note, and join with the second preloaded note from data tree index 1.
    // Second input index of tx2 is set to 0 but it's not actually used (since the propagated note doesn't exist in the
    // tree yet).
    auto tx2 = context.js_tx_factory.create_join_split_tx({ 1, 0 }, { 50, 30 }, { 80, 0 });
    tx2.input_note[1] = tx1.output_note[1];
    tx2.backward_link = tx2.input_note[1].commit();
    auto join_split_proof2 = create_js_proof(tx2);

    auto rollup = create_rollup_tx(context.world_state, rollup_size, { join_split_proof1, join_split_proof2 });
    auto result = verify_logic(rollup, rollup_2_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_allow_chain_off_first_output_note_but_dont_consume)
{
    size_t rollup_size = 2;

    context.append_value_notes({ 100, 50 });
    context.start_next_root_rollup();

    auto tx1 = context.js_tx_factory.create_join_split_tx({ 0 }, { 100 }, { 70, 30 });
    tx1.allow_chain = 1;
    auto join_split_proof1 = create_js_proof(tx1);

    // Allow chaining off the tx1's first note, but don't use it as an input note in tx2.
    // The tx will be permitted because no backward_link is specified, and so no propagation is happening.
    auto tx2 = context.js_tx_factory.create_join_split_tx({ 1 }, { 50 }, { 50, 0 });
    tx2.backward_link = 0;
    auto join_split_proof2 = create_js_proof(tx2);

    auto rollup = create_rollup_tx(context.world_state, rollup_size, { join_split_proof1, join_split_proof2 });
    auto result = verify_logic(rollup, rollup_2_keyless);

    EXPECT_TRUE(result.logic_verified);
}

void rollup_tests::test_chain_off_disallowed_note_fails(uint32_t allow_chain, size_t indicator)
{
    // Testing all invalid allow_chain / backward_link permutations between two txs.
    size_t rollup_size = 2;

    context.append_value_notes({ 100, 50 });
    context.start_next_root_rollup();

    auto tx1 = context.js_tx_factory.create_join_split_tx({ 0 }, { 100 }, { 70, 30 });
    tx1.allow_chain = allow_chain;
    auto join_split_proof1 = create_js_proof(tx1);

    // Chain off the output note dictated by the backward_link `indicator`
    join_split::join_split_tx tx2;
    switch (indicator) {
    case 1:
        tx2 = context.js_tx_factory.create_join_split_tx({ 0, 1 }, { 70, 50 }, { 120, 0 });
        break;
    case 2:
        tx2 = context.js_tx_factory.create_join_split_tx({ 0, 1 }, { 30, 50 }, { 80, 0 });
        break;
    }

    tx2.input_note[0] = tx1.output_note[indicator - 1];
    tx2.backward_link = tx1.output_note[indicator - 1].commit();
    auto join_split_proof2 = create_js_proof(tx2);

    auto rollup = create_rollup_tx(context.world_state, rollup_size, { join_split_proof1, join_split_proof2 });
    auto result = verify_logic(rollup, rollup_2_keyless);

    EXPECT_FALSE(result.logic_verified);
    auto assertion = result.err.find("is not permitted to propagate output") != std::string::npos;
    EXPECT_EQ(true, assertion);
}

TEST_F(rollup_tests, test_chain_off_disallowed_note_fails_0)
{
    test_chain_off_disallowed_note_fails(0, 1);
}

TEST_F(rollup_tests, test_chain_off_disallowed_note_fails_1)
{
    test_chain_off_disallowed_note_fails(0, 2);
}

TEST_F(rollup_tests, test_chain_off_disallowed_note_fails_2)
{
    test_chain_off_disallowed_note_fails(1, 2);
}

TEST_F(rollup_tests, test_chain_off_disallowed_note_fails_3)
{
    test_chain_off_disallowed_note_fails(2, 1);
}

// The following are implicitly tested, since create_rollup_tx will independently calculate the relevant tree root,
// which the circuit will then compare against its own calculation.
// - chained_nullifier_zeroed (if allow_chain = 1 or 2)
// - chained_output_commitment_zeroed
// - split_chain_nullifier_not_zeroed
// - split_chain_output_commitment_not_zeroed
TEST_F(rollup_tests, test_gap_in_chain_within_rollup)
{
    size_t rollup_size = 4;

    /*
     * Leaf index: 0    1    2    3    4    5    6    7    8    9
     * Value:      100  50   0    0    70   30   10   60   15   35
     */

    context.append_value_notes({ 100, 50 });
    context.start_next_root_rollup();

    // Chain should be tx1 -> tx2.
    // We'll interrupt the chain with tx1 -> tx3 -> tx2.
    // This should still pass, as the circuit will find tx1.

    auto tx1 = context.js_tx_factory.create_join_split_tx({ 0 }, { 100 }, { 70, 30 });
    tx1.allow_chain = 1;
    auto join_split_proof1 = create_js_proof(tx1);

    // Chain off the prior tx's first note, and join with the second preloaded note from index 1.
    // First index is set to 0 because the input note is being propagated.
    // Second index is set to 0 because num_input_notes is 1.
    auto tx2 = context.js_tx_factory.create_join_split_tx({ 0 }, { 70 }, { 10, 60 });
    tx2.input_note[0] = tx1.output_note[0];
    tx2.backward_link = tx2.input_note[0].commit();
    auto join_split_proof2 = create_js_proof(tx2);

    // Second index is set to 0 because num_input_notes is 1.
    auto tx3 = context.js_tx_factory.create_join_split_tx({ 1 }, { 50 }, { 15, 35 });
    auto join_split_proof3 = create_js_proof(tx3);

    auto rollup =
        create_rollup_tx(context.world_state, rollup_size, { join_split_proof1, join_split_proof3, join_split_proof2 });
    auto result = verify_logic(rollup, rollup_4_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_gap_in_chain_spanning_rollups_without_path_fails)
{
    size_t rollup_size = 2;

    context.append_value_notes({ 100, 50 });
    context.start_next_root_rollup();

    // Chain should be tx1 -> tx2.
    // We'll interrupt the chain with | tx1 -> tx3  | rollup split | tx2 ... |
    // The rollup provider should therefore provide a path for the backward-linked commitment. But they won't here.

    auto tx1 = context.js_tx_factory.create_join_split_tx({ 0 }, { 100 }, { 70, 30 });
    tx1.allow_chain = 1;

    // Add tx1 and tx3 to the first rollup:
    context.append_value_notes({ 70, 30, 15, 35 });

    // Add tx2 to the next rollup
    // First index is set to 0 because the input note is being propagated.
    auto tx2 = context.js_tx_factory.create_join_split_tx({ 0 }, { 70 }, { 10, 60 });
    tx2.input_note[0] = tx1.output_note[0];
    tx2.backward_link = tx2.input_note[0].commit();
    auto join_split_proof2 = create_js_proof(tx2);

    auto rollup = create_rollup_tx(context.world_state, rollup_size, { join_split_proof2 });
    auto result = verify_logic(rollup, rollup_2_keyless);

    EXPECT_FALSE(result.logic_verified);
    auto assertion = result.err.find("Membership check failed for backward_link") !=
                     std::string::npos; // ensure the error message contains this substring. (workaround without using
                                        // the gmock library).
    EXPECT_EQ(true, assertion);
}

TEST_F(rollup_tests, test_gap_in_chain_spanning_rollups_with_linked_commitment_path)
{
    size_t rollup_size = 4;

    context.append_value_notes({ 100, 50 });
    context.start_next_root_rollup();

    // Chain should be tx1 -> tx2.
    // We'll interrupt the chain with | tx1 -> tx3  | rollup split | tx2 ... |
    // The rollup provider should therefore provide a path for the backward-linked commitment. We'll do this here.

    // Add tx1 and tx3 to the first rollup:
    context.append_value_notes({ 70, 30, 15, 35 });
    context.start_next_root_rollup();

    // Add tx2 to the next rollup
    // First index is set to 0 because the input note is being propagated.
    auto tx2 = context.js_tx_factory.create_join_split_tx({ 0 }, { 70 }, { 10, 60 });
    notes::native::value::value_note linked_note = {
        70, 0, 0, context.user.owner.public_key, context.user.note_secret, 0, 2
    }; // there's a discrepency with the way `append_value_notes` and `create_join_split_tx` calculate
       // commitments/nullifiers. Here, we ensure the calculation methods match for the propagated commitment, at least.

    tx2.input_note[0] = linked_note;
    tx2.backward_link = linked_note.commit();
    auto join_split_proof2 = create_js_proof(tx2);

    auto rollup = create_rollup_tx(context.world_state,
                                   rollup_size,
                                   { join_split_proof2 },
                                   {},
                                   { 0 },
                                   {},
                                   { 2 }); // add the correct linked commitment index, so a valid path is retrieved.
    auto result = verify_logic(rollup, rollup_4_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_chain_off_both_output_notes_and_consume_in_next_two_txs_no_gaps)
{
    size_t rollup_size = 4;

    /*
     * Leaf index: 0    1    2    3    4    5    6    7    8    9
     * Value:      100  50   75   200  70   30   120  0    0    105
     */

    context.append_value_notes({ 100, 50, 75, 200 });
    context.start_next_root_rollup();

    auto tx1 = context.js_tx_factory.create_join_split_tx({ 0 }, { 100 }, { 70, 30 });
    tx1.allow_chain = 3; // allow chaining from both output notes
    auto join_split_proof1 = create_js_proof(tx1);

    // tx2 will consume the first of the propagated output notes of tx1
    // tx3 will consume the second propagated output note of tx1

    // Chain off tx1's first output note, and join with the second preloaded note from data tree index 1.
    // First input index of tx2 is set to 0 but it's not actually used.
    auto tx2 = context.js_tx_factory.create_join_split_tx({ 4, 1 }, { 70, 50 }, { 120, 0 });
    tx2.input_note[0] = tx1.output_note[0];
    tx2.backward_link = tx2.input_note[0].commit();
    auto join_split_proof2 = create_js_proof(tx2);

    auto tx3 = context.js_tx_factory.create_join_split_tx({ 2, 5 }, { 75, 30 }, { 0, 105 });
    tx3.input_note[1] = tx1.output_note[1];
    tx3.backward_link = tx3.input_note[1].commit();
    auto join_split_proof3 = create_js_proof(tx3);

    auto rollup =
        create_rollup_tx(context.world_state, rollup_size, { join_split_proof1, join_split_proof2, join_split_proof3 });
    auto result = verify_logic(rollup, rollup_4_keyless);

    EXPECT_TRUE(result.logic_verified);
}

TEST_F(rollup_tests, test_chain_off_both_output_notes_and_consume_within_rollup_with_gaps)
{
    size_t rollup_size = 8;

    /*
     * Leaf index: 0    1    2    3    4    5    6    7    8    9    10   11   12    13   14    15    16
     * Value:      100  50   75   200  300  400  500  600  70   30   120  0    0    105
     */

    // Chain should be tx1 --> tx2
    //                      \-------->tx3
    // We'll interrupt the chain with tx4 & tx5: | tx1 -> tx4 -> tx2 -> tx5 -> tx3 |
    // The rollup provider should therefore provide a path for both of the propagated input commitments (one for tx2,
    // one for tx3).

    context.append_value_notes({ 100, 50, 75, 200, 300, 400, 500, 600 });
    context.start_next_root_rollup();

    auto tx1 = context.js_tx_factory.create_join_split_tx({ 0 }, { 100 }, { 70, 30 });
    tx1.allow_chain = 3; // allow chaining from both output notes
    auto join_split_proof1 = create_js_proof(tx1);

    // tx2 will consume the first of the propagated output notes of tx1
    // tx3 will consume the second propagated output note of tx1

    // Chain off tx1's first output note, and join with the second preloaded note from data tree index 1.
    // First input index of tx2 is set to 0 but it's not actually used.
    auto tx2 = context.js_tx_factory.create_join_split_tx({ 8, 1 }, { 70, 50 }, { 120, 0 });
    tx2.input_note[0] = tx1.output_note[0];
    tx2.backward_link = tx2.input_note[0].commit();
    auto join_split_proof2 = create_js_proof(tx2);

    auto tx3 = context.js_tx_factory.create_join_split_tx({ 2, 9 }, { 75, 30 }, { 0, 105 });
    tx3.input_note[1] = tx1.output_note[1];
    tx3.backward_link = tx3.input_note[1].commit();
    auto join_split_proof3 = create_js_proof(tx3);

    auto tx4 = context.js_tx_factory.create_join_split_tx({ 3, 4 }, { 200, 300 }, { 20, 480 });
    auto join_split_proof4 = create_js_proof(tx4);

    auto tx5 = context.js_tx_factory.create_join_split_tx({ 5, 6 }, { 400, 500 }, { 1, 899 });
    auto join_split_proof5 = create_js_proof(tx5);

    auto rollup = create_rollup_tx(
        context.world_state,
        rollup_size,
        { join_split_proof1, join_split_proof4, join_split_proof2, join_split_proof5, join_split_proof3 });
    auto result = verify_logic(rollup, rollup_5_keyless);

    EXPECT_TRUE(result.logic_verified);
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

    // Check correct tx_fee accumulation.
    EXPECT_EQ(rollup_data.total_tx_fees[0], 7);  // asset_id 0
    EXPECT_EQ(rollup_data.total_tx_fees[1], 25); // asset_id 8
    EXPECT_EQ(rollup_data.total_tx_fees[2], 6);  // asset_id 13
    EXPECT_EQ(rollup_data.total_tx_fees[3], 0);  // padding
}

TEST_F(rollup_tests, test_defi_interaction_nonce_added_to_claim_notes)
{
    auto tx = create_tx_with_3_defi();
    auto result = verify_logic(tx, rollup_4_keyless);
    ASSERT_TRUE(result.logic_verified);

    auto rollup_data = rollup_proof_data(result.public_inputs);

    // Check regular join-split output note1 unchanged (as we change it for defi deposits).
    notes::native::value::value_note note1 = { .value = 70,
                                               .asset_id = 0,
                                               .account_required = false,
                                               .owner = context.user.owner.public_key,
                                               .secret = context.user.note_secret,
                                               .creator_pubkey = 0,
                                               .input_nullifier = rollup_data.inner_proofs[0].nullifier1 };
    EXPECT_EQ(rollup_data.inner_proofs[0].note_commitment1, note1.commit());

    notes::native::value::value_note note2 = { .value = 73,
                                               .asset_id = 0,
                                               .account_required = false,
                                               .owner = context.user.owner.public_key,
                                               .secret = context.user.note_secret,
                                               .creator_pubkey = 0,
                                               .input_nullifier = rollup_data.inner_proofs[0].nullifier2 };
    EXPECT_EQ(rollup_data.inner_proofs[0].note_commitment2, note2.commit());

    std::vector<uint32_t> claim_fees = { 0, 5, 20, 7 };

    // Check correct interaction nonce in claim notes.
    auto check_defi_proof = [&](uint32_t i, uint32_t claim_note_interaction_nonce) {
        auto defi_proof_data = inner_proof_data(tx.txs[i]);
        auto defi_proof = rollup_data.inner_proofs[i];

        auto partial_state = notes::native::value::create_partial_commitment(
            context.user.note_secret, context.user.owner.public_key, 0, 0);
        notes::native::claim::claim_note claim_note = { .deposit_value = defi_proof_data.defi_deposit_value,
                                                        .bridge_id = defi_proof_data.bridge_id,
                                                        .defi_interaction_nonce = claim_note_interaction_nonce,
                                                        .fee = claim_fees[i],
                                                        .value_note_partial_commitment = partial_state,
                                                        .input_nullifier = defi_proof.nullifier1 };

        EXPECT_EQ(defi_proof.note_commitment1, claim_note.commit());
    };

    check_defi_proof(1, NUM_BRIDGE_CALLS_PER_BLOCK);
    check_defi_proof(2, NUM_BRIDGE_CALLS_PER_BLOCK);
    check_defi_proof(3, NUM_BRIDGE_CALLS_PER_BLOCK + 1);
}

TEST_F(rollup_tests, test_defi_claim_proofs)
{
    auto rollup1_tx = create_tx_with_3_defi();
    auto bids = rollup1_tx.bridge_ids;
    auto asset_ids = rollup1_tx.asset_ids;
    auto result = verify_logic(rollup1_tx, rollup_4_keyless);
    ASSERT_TRUE(result.logic_verified);

    std::vector<native::defi_interaction::note> dins = { { .bridge_id = bids[0],
                                                           .interaction_nonce = 0,
                                                           .total_input_value = 70,
                                                           .total_output_value_a = 700,
                                                           .total_output_value_b = 7000,
                                                           .interaction_result = true },
                                                         { .bridge_id = bids[1],
                                                           .interaction_nonce = 1,
                                                           .total_input_value = 20,
                                                           .total_output_value_a = 2,
                                                           .total_output_value_b = 3,
                                                           .interaction_result = true } };
    context.append_account_notes();
    uint32_t initial_din_index = context.start_next_root_rollup(dins);
    rollup::rollup_proof_data data(result.public_inputs);

    // js, acc and defi proofs to be rolled up with claim proofs
    auto acc_proof = context.create_add_signing_keys_to_account_proof(data.data_start_index + 8);
    auto js_proof = context.create_join_split_proof({}, {}, { 100, 30 }, 130);
    const notes::native::bridge_id bid1 = {
        .bridge_address_id = 0,
        .input_asset_id_a = 0,
        .input_asset_id_b = 0,
        .output_asset_id_a = 0,
        .output_asset_id_b = 1,
        .config = notes::native::bridge_id::bit_config{ .second_input_in_use = false, .second_output_in_use = true },
        .aux_data = 0
    };
    bids.push_back(bid1);
    auto defi_proof1 =
        context.create_defi_proof({ data.data_start_index, data.data_start_index + 1 }, { 70, 73 }, { 120, 23 }, bid1);
    std::vector<uint32_t> claim_fees = { 0, 5, 20, 7 };

    std::vector<uint32_t> indices = { 0, 0, 1 };

    // Create claim proofs for each claim note in previous rollup.
    auto claim_proofs = mapi(data.inner_proofs, [&](auto inner, auto i) {
        if (inner.proof_id != ProofIds::DEFI_DEPOSIT) {
            return std::vector<uint8_t>();
        }
        auto claim_note_index = data.data_start_index + uint32_t(2 * i);
        auto inner_tx = inner_proof_data(rollup1_tx.txs[i]);
        auto defi_note_index = initial_din_index + indices[i - 1];
        return context.create_claim_proof(inner_tx.bridge_id,
                                          inner_tx.defi_deposit_value,
                                          claim_note_index,
                                          defi_note_index, // defi proofs are offset by one
                                          claim_fees[i]);
    });

    auto rollup2_tx = create_rollup_tx(
        context.world_state, 4, { js_proof, acc_proof, defi_proof1, claim_proofs[2] }, bids, asset_ids);
    auto result2 = verify_logic(rollup2_tx, rollup_4_keyless);
    EXPECT_TRUE(result2.logic_verified);
}

TEST_F(rollup_tests, test_defi_loan_proofs)
{
    /**
     * Rollup 0: Create defi deposit proofs for drawing 3 loans:
     * +-------------------------------------------------------------------------+
     * | no    collateral_asset    collateral_value     loan_asset    loan_value |
     * +-------------------------------------------------------------------------+
     * | 1     0                   180                  2             1800       |
     * | 2     0                   190                  2             1900       |
     * | 3     1                   150                  3             3000       |
     * +-------------------------------------------------------------------------+
     */
    auto rollup1_tx = create_tx_with_defi_loan();
    auto bids = rollup1_tx.bridge_ids;
    auto asset_ids = rollup1_tx.asset_ids;
    auto result = verify_logic(rollup1_tx, rollup_4_keyless);
    ASSERT_TRUE(result.logic_verified);

    /**
     * Rollup 1: Create defi claim proofs for drawing 3 loans:
     */
    std::vector<native::defi_interaction::note> dins = {
        { bids[0], NUM_BRIDGE_CALLS_PER_BLOCK, 370, 3700, 3700, true },
        { bids[1], NUM_BRIDGE_CALLS_PER_BLOCK + 1, 150, 3000, 3000, true }
    };
    uint32_t initial_din_index = context.start_next_root_rollup(dins);

    rollup::rollup_proof_data data(result.public_inputs);

    // Create claim proofs for each claim note in previous rollup.
    auto loan_claim_proof1 = context.create_claim_proof(bids[0], 180, data.data_start_index + 0, initial_din_index, 10);
    auto loan_claim_proof2 = context.create_claim_proof(bids[0], 190, data.data_start_index + 2, initial_din_index, 5);
    auto loan_claim_proof3 =
        context.create_claim_proof(bids[1], 150, data.data_start_index + 4, initial_din_index + 1, 0);

    auto rollup2_tx = create_rollup_tx(
        context.world_state, 4, { loan_claim_proof1, loan_claim_proof2, loan_claim_proof3 }, bids, asset_ids);
    auto result2 = verify_logic(rollup2_tx, rollup_4_keyless);
    EXPECT_TRUE(result2.logic_verified);
    /**
     * Rollup 2: Create defi deposit proofs for repaying the loans 1 and 3,
     * split the value and virtual notes of loan 2:
     * +--------------------------------------------------------------------+
     * | no    collateral_asset   returned_collateral_value     loan_repay  |
     * +--------------------------------------------------------------------+
     * | 1     0                  180-18 = 152                  yes         |
     * | 2     0                  -                             no, split   |
     * | 3     1                  150-30 = 120                  yes         |
     * +--------------------------------------------------------------------+
     */
    initial_din_index = context.start_next_root_rollup();
    rollup::rollup_proof_data data2(result2.public_inputs);

    // Loan number 1 repayment
    const uint32_t opening_nonce1 = NUM_BRIDGE_CALLS_PER_BLOCK;

    const notes::native::bridge_id bid1 = {
        .bridge_address_id = 0,
        .input_asset_id_a = 2,
        .input_asset_id_b = virtual_asset_id_flag + opening_nonce1,
        .output_asset_id_a = 0,
        .output_asset_id_b = 0,
        .config = notes::native::bridge_id::bit_config{ .second_input_in_use = true, .second_output_in_use = false },
        .aux_data = 0
    };
    const uint32_t virtual_asset_id1 = (uint32_t(1) << 29) + opening_nonce1;
    auto loan_repay_proof1 = context.create_defi_proof({ data2.data_start_index + 0, data2.data_start_index + 1 },
                                                       { 1800, 1800 },
                                                       { 1800, 0 },
                                                       bid1,
                                                       2,
                                                       0,
                                                       virtual_asset_id1);
    // Loan number 3 repayment
    const uint32_t opening_nonce2 = opening_nonce1 + 1;
    const notes::native::bridge_id bid2 = {
        .bridge_address_id = 0,
        .input_asset_id_a = 3,
        .input_asset_id_b = virtual_asset_id_flag + opening_nonce2,
        .output_asset_id_a = 1,
        .output_asset_id_b = 0,
        .config = notes::native::bridge_id::bit_config{ .second_input_in_use = true, .second_output_in_use = false },
        .aux_data = 0
    };
    const uint32_t virtual_asset_id2 = (uint32_t(1) << 29) + opening_nonce2;
    auto loan_repay_proof2 = context.create_defi_proof({ data2.data_start_index + 4, data2.data_start_index + 5 },
                                                       { 3000, 3000 },
                                                       { 3000, 0 },
                                                       bid2,
                                                       3,
                                                       0,
                                                       virtual_asset_id2);
    // Loan number 2 virtual and value note splitting
    auto loan_split_proof1 = context.create_join_split_proof(
        { data2.data_start_index + 3 }, { 1900 }, { 1000, 900 }, 0, 0, 0, virtual_asset_id1);
    auto value_note_split_proof =
        context.create_join_split_proof({ data2.data_start_index + 2 }, { 1900 }, { 900, 1000 }, 0, 0, 0, 2);
    auto rollup3_tx =
        create_rollup_tx(context.world_state,
                         4,
                         { loan_repay_proof1, loan_repay_proof2, loan_split_proof1, value_note_split_proof },
                         { bid1, bid2 },
                         { 2, 3, virtual_asset_id1 });
    auto result3 = verify_logic(rollup3_tx, rollup_4_keyless);
    EXPECT_TRUE(result3.logic_verified);

    /**
     * Rollup 3: Create defi claim proofs for repaying loans 1 and 3.
     * Also, create defi deposit proof for repaying one part of loan 2.
     */
    std::vector<native::defi_interaction::note> dins1 = { { bid1, 12, 1800, 152, 18, true },
                                                          { bid2, 13, 3000, 120, 15, true } };
    initial_din_index = context.start_next_root_rollup(dins1);

    // Finish loan repayments which were initiated in the previous rollup.
    rollup::rollup_proof_data data3(result3.public_inputs);
    auto loan_repay_claim_proof1 =
        context.create_claim_proof(bid1, 1800, data3.data_start_index + 0, initial_din_index, 0);
    auto loan_repay_claim_proof2 =
        context.create_claim_proof(bid2, 3000, data3.data_start_index + 2, initial_din_index + 1, 0);

    // Initiate repayment of one installment of the second loan in this rollup.
    auto loan_repay_proof4 = context.create_defi_proof({ data3.data_start_index + 7, data3.data_start_index + 4 },
                                                       { 1000, 1000 },
                                                       { 1000, 0 },
                                                       bid1,
                                                       2,
                                                       0,
                                                       virtual_asset_id1);

    auto rollup4_tx = create_rollup_tx(context.world_state,
                                       4,
                                       { loan_repay_claim_proof1, loan_repay_claim_proof2, loan_repay_proof4 },
                                       { bid1, bid2 },
                                       { 2, 3 });
    auto result4 = verify_logic(rollup4_tx, rollup_4_keyless);
    EXPECT_TRUE(result4.logic_verified);

    /**
     * Rollup 4: Create defi claim proofs for repaying one installment of loans 2.
     * Also, create defi deposit proof for repaying other part of loan 2.
     */
    std::vector<native::defi_interaction::note> dins2 = { { bid1, 16, 1000, 85, 15, true } };
    initial_din_index = context.start_next_root_rollup(dins2);

    // Finish loan repayment which was initiated in the previous rollup.
    rollup::rollup_proof_data data4(result4.public_inputs);
    auto loan_repay_claim_proof4 =
        context.create_claim_proof(bid1, 1000, data4.data_start_index + 4, initial_din_index, 0);

    // Initiate repayment of the remaining installment of loan 2 in this rollup.
    auto loan_repay_proof5 = context.create_defi_proof({ data3.data_start_index + 6, data3.data_start_index + 5 },
                                                       { 900, 900 },
                                                       { 900, 0 },
                                                       bid1,
                                                       2,
                                                       0,
                                                       virtual_asset_id1);

    auto rollup5_tx = create_rollup_tx(
        context.world_state, 4, { loan_repay_claim_proof4, loan_repay_proof5 }, { bid1, bid2 }, { 2, 3 });
    auto result5 = verify_logic(rollup5_tx, rollup_4_keyless);
    EXPECT_TRUE(result5.logic_verified);
}

TEST_F(rollup_tests, test_defi_claim_proof_has_valid_defi_root)
{
    auto rollup1_tx = create_tx_with_1_defi();
    auto bids = rollup1_tx.bridge_ids;
    auto result = verify_logic(rollup1_tx, rollup_1_keyless);
    ASSERT_TRUE(result.logic_verified);

    std::vector<native::defi_interaction::note> dins = { { bids[0], 0, 70, 700, 7000, true },
                                                         { bids[1], 0, 20, 2, 3, true } };
    uint32_t initial_din_index = context.start_next_root_rollup(dins);

    rollup::rollup_proof_data data(result.public_inputs);

    // Create claim proof with trash defi root.
    auto inner_tx = inner_proof_data(rollup1_tx.txs[0]);
    auto tx = context.create_claim_tx(inner_tx.bridge_id, inner_tx.defi_deposit_value, 2, initial_din_index, 0);
    tx.defi_root = fr::random_element();
    auto claim_proof = claim::create_proof(tx, context.claim_cd);

    auto rollup2_tx = create_rollup_tx(context.world_state, 1, { claim_proof }, bids);
    auto result2 = verify_logic(rollup2_tx, rollup_1_keyless);
    EXPECT_FALSE(result2.logic_verified);
}

} // namespace rollup
} // namespace proofs
} // namespace rollup