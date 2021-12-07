#include <common/test.hpp>
#include <common/map.hpp>
#include <common/throw_or_abort.hpp>
#include "index.hpp"
#include "../rollup/index.hpp"
#include "../notes/native/index.hpp"
#include "../../fixtures/test_context.hpp"
#include "../../fixtures/compute_or_load_fixture.hpp"

// #pragma GCC diagnostic ignored "-Wunused-variable"
// #pragma GCC diagnostic ignored "-Wunused-parameter"
namespace rollup {
namespace proofs {
namespace root_rollup {

using namespace barretenberg;
using namespace notes::native;
using namespace plonk::stdlib::merkle_tree;

namespace {
#ifdef CI
bool persist = false;
#else
bool persist = true;
#endif
std::shared_ptr<waffle::DynamicFileReferenceStringFactory> srs;
join_split::circuit_data js_cd;
proofs::account::circuit_data account_cd;
proofs::circuit_data claim_cd;
rollup::circuit_data tx_rollup_cd;
circuit_data root_rollup_cd;
} // namespace

class root_rollup_tests : public ::testing::Test {
  protected:
    static constexpr auto CRS_PATH = "../srs_db/ignition";
    static constexpr auto FIXTURE_PATH = "../src/aztec/rollup/proofs/root_rollup/fixtures";
    static constexpr auto TEST_PROOFS_PATH = "../src/aztec/rollup/proofs/root_rollup/fixtures/test_proofs";
    static constexpr auto INNER_ROLLUP_TXS = 2U;
    static constexpr auto ROLLUPS_PER_ROLLUP = 3U;

    typedef std::vector<std::vector<std::vector<uint8_t>>> RollupStructure;

    root_rollup_tests()
        : context(js_cd, account_cd, claim_cd)
        , js_proofs(get_js_proofs(5))
    {}

    static void SetUpTestCase()
    {
        auto recreate = !exists(FIXTURE_PATH);
        mkdir(FIXTURE_PATH, 0700);
        mkdir(TEST_PROOFS_PATH, 0700);
        srs = std::make_shared<waffle::DynamicFileReferenceStringFactory>(CRS_PATH);

        account_cd = proofs::account::compute_circuit_data(srs);
        js_cd = join_split::compute_circuit_data(srs);
        claim_cd = proofs::claim::get_circuit_data(srs, "", true, false, false);

        if (recreate) {
            // If no fixtures dir, recreate all proving keys, verification keys, padding proofs etc.
            tx_rollup_cd = rollup::get_circuit_data(
                INNER_ROLLUP_TXS, js_cd, account_cd, claim_cd, srs, FIXTURE_PATH, true, persist, persist);
        } else {
            // Otherwise we should only need the inner proofs verification key for logic tests.
            tx_rollup_cd = rollup::get_circuit_data(
                INNER_ROLLUP_TXS, js_cd, account_cd, claim_cd, srs, FIXTURE_PATH, false, false, true, false, true);
        }
        root_rollup_cd =
            get_circuit_data(ROLLUPS_PER_ROLLUP, tx_rollup_cd, srs, FIXTURE_PATH, false, false, false, false, false);
    }

    root_rollup_tx create_root_rollup_tx(std::string const& test_name,
                                         RollupStructure const& rollup_structure,
                                         std::vector<std::vector<uint256_t>> bridge_ids = { {}, {}, {} },
                                         std::vector<uint256_t> bridge_ids_union = {},
                                         std::vector<std::vector<uint256_t>> asset_ids = { { 0 }, { 0 }, { 0 } },
                                         std::vector<uint256_t> asset_ids_union = { 0 },
                                         std::vector<native::defi_interaction::note> const& interaction_notes = {})
    {
        uint32_t rollup_id = static_cast<uint32_t>(context.world_state.root_tree.size() - 1);
        auto old_defi_root = context.world_state.defi_tree.root();
        context.world_state.add_defi_notes(interaction_notes);

        std::vector<std::vector<uint8_t>> inner_data;
        for (size_t i = 0; i < rollup_structure.size(); ++i) {
            auto tx_proofs = rollup_structure[i];
            auto rollup =
                rollup::create_rollup_tx(context.world_state, INNER_ROLLUP_TXS, tx_proofs, bridge_ids[i], asset_ids[i]);
            auto fixture_name = format(test_name, "_rollup", rollup_id, "_inner", inner_data.size());
            auto proof_data = compute_or_load_rollup(fixture_name, rollup);
            if (proof_data.empty()) {
                throw_or_abort("Failed to create inner rollup proof.");
            }
            inner_data.push_back(proof_data);
        }

        return root_rollup::create_root_rollup_tx(context.world_state,
                                                  rollup_id,
                                                  old_defi_root,
                                                  inner_data,
                                                  bridge_ids_union,
                                                  asset_ids_union,
                                                  interaction_notes);
    }

    std::vector<uint8_t> compute_or_load_rollup(std::string const& name, rollup::rollup_tx& rollup)
    {
        return fixtures::compute_or_load_fixture(TEST_PROOFS_PATH, name, [&] {
            // We need to ensure we have a proving key to build the inner proof fixtures.
            if (!tx_rollup_cd.proving_key) {
                tx_rollup_cd = rollup::get_circuit_data(
                    INNER_ROLLUP_TXS, js_cd, account_cd, claim_cd, srs, FIXTURE_PATH, true, true, true);
                root_rollup_cd.inner_rollup_circuit_data = tx_rollup_cd;
            }
            return rollup::verify(rollup, tx_rollup_cd).proof_data;
        });
    }

    // Create and return n deposit join split proofs.
    std::vector<std::vector<uint8_t>> get_js_proofs(uint32_t n)
    {
        std::vector<std::vector<uint8_t>> proofs;
        for (uint32_t i = 0; i < n; ++i) {
            auto js_proof = fixtures::compute_or_load_fixture(TEST_PROOFS_PATH, format("js", i), [&] {
                return context.create_join_split_proof({}, {}, { 100, 50 }, 150);
            });
            proofs.push_back(js_proof);
        }
        return proofs;
    }

    root_rollup_tx create_full_logic_root_rollup_tx()
    {
        uint32_t aid1 = 1, aid2 = 2, aid3 = 3;
        context.append_value_notes({ 100, 50 });
        context.append_value_notes({ 100, 50 }, aid1);
        context.append_value_notes({ 100, 50, 100, 50, 100, 50 }, aid2);
        context.append_value_notes({ 100, 50 }, aid3);
        context.start_next_root_rollup();

        const notes::native::bridge_id bid2 = {
            .bridge_address_id = 1,
            .input_asset_id = aid2,
            .output_asset_id_a = 0,
            .output_asset_id_b = 1,
            .opening_nonce = 0,
            .config = notes::native::bridge_id::bit_config{ .first_input_asset_virtual = false,
                                                            .second_input_asset_virtual = false,
                                                            .first_output_asset_virtual = false,
                                                            .second_output_asset_virtual = false,
                                                            .second_input_valid = false,
                                                            .second_output_valid = true },
            .aux_data = 0
        };

        const notes::native::bridge_id bid3 = {
            .bridge_address_id = 2,
            .input_asset_id = aid3,
            .output_asset_id_a = 0,
            .output_asset_id_b = 1,
            .opening_nonce = 0,
            .config = notes::native::bridge_id::bit_config{ .first_input_asset_virtual = false,
                                                            .second_input_asset_virtual = false,
                                                            .first_output_asset_virtual = false,
                                                            .second_output_asset_virtual = false,
                                                            .second_input_valid = false,
                                                            .second_output_valid = true },
            .aux_data = 0
        };

        auto js_proof1 = context.create_join_split_proof({ 0, 1 }, { 100, 50 }, { 70, 80 - 7 }); // fee = 7
        auto js_proof2 =
            context.create_join_split_proof({ 2, 3 }, { 100, 50 }, { 20, 130 - 15 }, 0, 0, 0, aid1); // fee = 15
        auto js_proof3 =
            context.create_join_split_proof({ 4, 5 }, { 100, 50 }, { 10, 140 - 9 }, 0, 0, 0, aid2);     // fee = 9
        auto defi_proof1 = context.create_defi_proof({ 6, 7 }, { 100, 50 }, { 30, 102 }, bid2, aid2);   // fee = 18
        auto defi_proof2 = context.create_defi_proof({ 8, 9 }, { 100, 50 }, { 50, 80 }, bid2, aid2);    // fee = 20
        auto defi_proof3 = context.create_defi_proof({ 10, 11 }, { 100, 50 }, { 20, 111 }, bid3, aid3); // fee = 19

        // Add some defi interaction notes.
        std::vector<notes::native::defi_interaction::note> interaction_notes = { { 1, 0, 3, 4, 5, false },
                                                                                 { 2, 1, 4, 5, 6, true } };
        return create_root_rollup_tx(
            "root_full_logic",
            { { js_proof1, js_proof2 }, { defi_proof1, defi_proof3 }, { js_proof3, defi_proof2 } },
            { {}, { bid2, bid3 }, { bid2 } },
            { bid2, bid3 },
            { { 0, aid1 }, { aid2, aid3 }, { aid2 } },
            { 0, aid1, aid3, aid2 },
            interaction_notes);
    }

    fixtures::TestContext context;
    std::vector<std::vector<uint8_t>> js_proofs;
};

/*
 * Due the the length of time it takes to produce inner proofs, they're saved in fixtures.
 * If they need to be recomputed due to a circuit change or otherwise, delete files in ./fixtures/test_proofs. (You
 * might need to delete all of the fixtures). The fixtures' names are named so as to reduce unnecessary (re)computation
 * between tests. i.e. If a rollup has a structure shorter than its name suggests, it's because it can reuse the
 * fixtures from the longer rollup structure due to them having the same leading structure.
 */
TEST_F(root_rollup_tests, test_1_real_2_padding)
{
    auto tx_data = create_root_rollup_tx("root_1", { { js_proofs[0] } });
    auto result = verify_logic(tx_data, root_rollup_cd);
    ASSERT_TRUE(result.logic_verified);
}

TEST_F(root_rollup_tests, test_2_real_1_padding)
{
    auto tx_data = create_root_rollup_tx("root_211", { { js_proofs[0], js_proofs[1] }, { js_proofs[2] } });
    auto result = verify_logic(tx_data, root_rollup_cd);
    ASSERT_TRUE(result.logic_verified);
}

TEST_F(root_rollup_tests, test_3_real_0_padding)
{
    auto tx_data = create_root_rollup_tx(
        "root_221", { { js_proofs[0], js_proofs[1] }, { js_proofs[2], js_proofs[3] }, { js_proofs[4] } });
    auto result = verify_logic(tx_data, root_rollup_cd);
    ASSERT_TRUE(result.logic_verified);
}

TEST_F(root_rollup_tests, test_incorrect_new_data_root_fails)
{
    auto tx_data = create_root_rollup_tx("bad_new_data_root_fail", { { js_proofs[0] } });
    tx_data.new_data_roots_root = fr::random_element();
    auto result = verify_logic(tx_data, root_rollup_cd);
    ASSERT_FALSE(result.logic_verified);
}

TEST_F(root_rollup_tests, test_inner_rollups_out_of_order_fail)
{
    auto tx_data =
        create_root_rollup_tx("root_221", { { js_proofs[0], js_proofs[1] }, { js_proofs[2], js_proofs[3] } });
    std::swap(tx_data.rollups[0], tx_data.rollups[1]);

    auto result = verify_logic(tx_data, root_rollup_cd);
    ASSERT_FALSE(result.logic_verified);
}

TEST_F(root_rollup_tests, test_invalid_padding_proof_fail)
{
    auto tx_data = create_root_rollup_tx(
        "root_221", { { js_proofs[0], js_proofs[1] }, { js_proofs[2], js_proofs[3] }, { js_proofs[4] } });
    tx_data.num_inner_proofs = 2;
    auto result = verify_logic(tx_data, root_rollup_cd);
    ASSERT_FALSE(result.logic_verified);
}

TEST_F(root_rollup_tests, test_invalid_last_proof_fail)
{
    auto tx_data = create_root_rollup_tx("root_221", { { js_proofs[0], js_proofs[1] } });
    tx_data.num_inner_proofs = 2;
    auto result = verify_logic(tx_data, root_rollup_cd);
    ASSERT_FALSE(result.logic_verified);
}

TEST_F(root_rollup_tests, test_defi_valid_previous_defi_hash_for_0_interactions)
{
    auto tx_data = create_root_rollup_tx("root_1", { { js_proofs[0] } });
    auto result = verify_logic(tx_data, root_rollup_cd);
    ASSERT_TRUE(result.logic_verified);

    std::vector<uint8_t> sha256_input;
    for (size_t i = 0; i < NUM_INTERACTION_RESULTS_PER_BLOCK; i++) {
        notes::native::defi_interaction::note note = { 0, 0, 0, 0, 0, false };
        auto buf = note.to_byte_array();
        uint256_t note_hash = uint256_t(sha256::sha256_to_field(buf));
        write(sha256_input, note_hash);
    }
    auto expected = sha256::sha256_to_field(sha256_input);

    root_rollup_broadcast_data data(result.broadcast_data);
    ASSERT_EQ(data.previous_defi_interaction_hash, expected);
}

TEST_F(root_rollup_tests, test_encode_inputs)
{
    auto tx_data = create_root_rollup_tx("root_211", { { js_proofs[0], js_proofs[1] }, { js_proofs[2] } });
    auto result = verify_logic(tx_data, root_rollup_cd);
    ASSERT_TRUE(result.logic_verified);

    root_rollup_broadcast_data broadcast_data(result.broadcast_data);
    root_rollup_proof_data proof_data(result.public_inputs);

    auto hash_output = broadcast_data.compute_hash();
    EXPECT_EQ(hash_output, proof_data.input_hash);
}

TEST_F(root_rollup_tests, test_asset_ids_missing_fails)
{
    auto tx_data = create_full_logic_root_rollup_tx();
    tx_data.asset_ids[0] = tx_data.asset_ids[1]; // asset_ids = [0, aid3, aid3, aid2]

    auto result = verify_logic(tx_data, root_rollup_cd);
    ASSERT_FALSE(result.logic_verified);
}

TEST_F(root_rollup_tests, test_asset_ids_repeating_fails)
{
    auto tx_data = create_full_logic_root_rollup_tx();
    tx_data.asset_ids[1] = tx_data.asset_ids[0]; // asset_ids = [0, aid1, aid1, aid2]

    auto result = verify_logic(tx_data, root_rollup_cd);
    ASSERT_FALSE(result.logic_verified);
}

TEST_F(root_rollup_tests, test_asset_ids_reordering)
{
    auto tx_data = create_full_logic_root_rollup_tx();
    std::swap(tx_data.asset_ids[1], tx_data.asset_ids[2]); // asset_ids = [0, aid1, aid2, aid3]

    auto result = verify_logic(tx_data, root_rollup_cd);
    ASSERT_TRUE(result.logic_verified);
}

TEST_F(root_rollup_tests, test_bridge_ids_missing_fails)
{
    auto tx_data = create_full_logic_root_rollup_tx();
    tx_data.bridge_ids[0] = tx_data.bridge_ids[1]; // bridge_ids = [bid3, bid3, 0, 0]

    auto result = verify_logic(tx_data, root_rollup_cd);
    ASSERT_FALSE(result.logic_verified);
}

TEST_F(root_rollup_tests, test_bridge_ids_repeating_fails)
{
    auto tx_data = create_full_logic_root_rollup_tx();
    tx_data.bridge_ids[1] = tx_data.bridge_ids[0]; // bridge_ids = [bid2, bid2, 0, 0]

    auto result = verify_logic(tx_data, root_rollup_cd);
    ASSERT_FALSE(result.logic_verified);
}

TEST_F(root_rollup_tests, test_bridge_ids_reordering)
{
    auto tx_data = create_full_logic_root_rollup_tx();
    std::swap(tx_data.bridge_ids[1], tx_data.bridge_ids[0]); // bridge_ids = [bid3, bid2, 0, 0]

    auto result = verify_logic(tx_data, root_rollup_cd);
    ASSERT_TRUE(result.logic_verified);
}

// Full logic tests
TEST_F(root_rollup_tests, test_full_logic)
{
    auto tx_data = create_full_logic_root_rollup_tx();
    auto result = verify_logic(tx_data, root_rollup_cd);
    ASSERT_TRUE(result.logic_verified);

    root_rollup_broadcast_data rollup_data = result.broadcast_data;
    EXPECT_EQ(rollup_data.bridge_ids[0], fr(tx_data.bridge_ids[0]));
    EXPECT_EQ(rollup_data.bridge_ids[1], fr(tx_data.bridge_ids[1]));
    EXPECT_EQ(rollup_data.bridge_ids[2], 0);
    EXPECT_EQ(rollup_data.bridge_ids[3], 0);
    EXPECT_EQ(rollup_data.deposit_sums[0], 80);
    EXPECT_EQ(rollup_data.deposit_sums[1], 20);
    EXPECT_EQ(rollup_data.deposit_sums[2], 0);
    EXPECT_EQ(rollup_data.deposit_sums[3], 0);
    EXPECT_EQ(rollup_data.defi_interaction_notes[0], tx_data.defi_interaction_notes[0].commit());
    EXPECT_EQ(rollup_data.defi_interaction_notes[1], tx_data.defi_interaction_notes[1].commit());
    EXPECT_EQ(rollup_data.total_tx_fees[0], 7);  // asset_id = 0 (ETH)
    EXPECT_EQ(rollup_data.total_tx_fees[1], 15); // aid1
    EXPECT_EQ(rollup_data.total_tx_fees[2], 9);  // aid3
    EXPECT_EQ(rollup_data.total_tx_fees[3], 28); // aid2
    EXPECT_EQ(rollup_data.asset_ids[0], fr(tx_data.asset_ids[0]));
    EXPECT_EQ(rollup_data.asset_ids[1], fr(tx_data.asset_ids[1]));
    EXPECT_EQ(rollup_data.asset_ids[2], fr(tx_data.asset_ids[2]));
    EXPECT_EQ(rollup_data.asset_ids[3], fr(tx_data.asset_ids[3]));

    std::vector<uint8_t> sha256_input;
    for (size_t i = 0; i < NUM_INTERACTION_RESULTS_PER_BLOCK; i++) {
        auto buf = tx_data.defi_interaction_notes[i].to_byte_array();
        uint256_t note_hash = uint256_t(sha256::sha256_to_field(buf));
        write(sha256_input, note_hash);
    }
    auto expected_hash = sha256::sha256_to_field(sha256_input);

    EXPECT_EQ(rollup_data.previous_defi_interaction_hash, expected_hash);
}

} // namespace root_rollup
} // namespace proofs
} // namespace rollup