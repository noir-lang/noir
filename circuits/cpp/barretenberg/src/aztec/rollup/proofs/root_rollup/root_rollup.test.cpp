#include <common/test.hpp>
#include <common/map.hpp>
#include <common/throw_or_abort.hpp>
#include "index.hpp"
#include "../rollup/index.hpp"
#include "../notes/native/index.hpp"

// #pragma GCC diagnostic ignored "-Wunused-variable"
// #pragma GCC diagnostic ignored "-Wunused-parameter"
namespace rollup {
namespace proofs {
namespace root_rollup {

using namespace barretenberg;
using namespace notes::native;
using namespace plonk::stdlib::merkle_tree;

namespace {
std::shared_ptr<waffle::DynamicFileReferenceStringFactory> srs;
numeric::random::Engine* rand_engine = &numeric::random::get_debug_engine(true);
fixtures::user_context user = fixtures::create_user_context(rand_engine);
join_split::circuit_data join_split_cd;
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
    {
        rand_engine = &numeric::random::get_debug_engine(true);
        user = fixtures::create_user_context(rand_engine);
    }

    static void SetUpTestCase()
    {
        auto recreate = !exists(FIXTURE_PATH);
        srs = std::make_shared<waffle::DynamicFileReferenceStringFactory>(CRS_PATH);

        account_cd = proofs::account::compute_or_load_circuit_data(srs, FIXTURE_PATH);
        join_split_cd = join_split::compute_or_load_circuit_data(srs, FIXTURE_PATH);
        claim_cd = proofs::claim::get_circuit_data(srs, FIXTURE_PATH);

        if (recreate) {
            // If no fixtures dir, recreate all proving keys, verification keys, padding proofs etc.
            tx_rollup_cd = rollup::get_circuit_data(
                INNER_ROLLUP_TXS, join_split_cd, account_cd, claim_cd, srs, FIXTURE_PATH, true, true, true);
            root_rollup_cd = get_circuit_data(ROLLUPS_PER_ROLLUP, tx_rollup_cd, srs, FIXTURE_PATH, true, true, true);
        } else {
            // Otherwise we should only need the inner proofs verification key for logic tests.
            tx_rollup_cd = rollup::get_circuit_data(INNER_ROLLUP_TXS,
                                                    join_split_cd,
                                                    account_cd,
                                                    claim_cd,
                                                    srs,
                                                    FIXTURE_PATH,
                                                    false,
                                                    false,
                                                    true,
                                                    false,
                                                    true);
            root_rollup_cd = get_circuit_data(
                ROLLUPS_PER_ROLLUP, tx_rollup_cd, srs, FIXTURE_PATH, false, false, false, false, false);
        }
    }

    join_split_tx create_join_split_tx(std::array<uint32_t, 2> in_note_idx,
                                       std::array<uint32_t, 2> in_note_value,
                                       std::array<uint32_t, 2> out_note_value,
                                       uint32_t public_input)
    {
        uint32_t asset_id = 0;
        uint32_t nonce = 0;
        value::value_note input_note1 = { in_note_value[0], asset_id, nonce, user.owner.public_key, user.note_secret };
        value::value_note input_note2 = { in_note_value[1], asset_id, nonce, user.owner.public_key, user.note_secret };
        value::value_note output_note1 = {
            out_note_value[0], asset_id, nonce, user.owner.public_key, user.note_secret
        };
        value::value_note output_note2 = {
            out_note_value[1], asset_id, nonce, user.owner.public_key, user.note_secret
        };

        join_split_tx tx;
        tx.public_input = public_input;
        tx.public_output = 0;
        tx.asset_id = 0;
        tx.num_input_notes = 0;
        tx.input_index = { in_note_idx[0], in_note_idx[1] };
        tx.old_data_root = world_state.data_tree.root();
        tx.input_path = { world_state.data_tree.get_hash_path(in_note_idx[0]),
                          world_state.data_tree.get_hash_path(in_note_idx[1]) };
        tx.input_note = { input_note1, input_note2 };
        tx.output_note = { output_note1, output_note2 };
        tx.account_index = 0;
        tx.account_path = world_state.data_tree.get_hash_path(0);
        tx.signing_pub_key = user.owner.public_key;
        tx.account_private_key = user.owner.private_key;
        tx.alias_hash = 0;
        tx.nonce = 0;
        tx.input_owner = fr::random_element();
        tx.output_owner = fr::random_element();
        tx.claim_note = { 0, 0, user.note_secret, 0 };

        return tx;
    }

    std::vector<uint8_t> create_proof(join_split_tx const& tx, std::string const& fixture_prefix)
    {
        auto fixture_name = format(fixture_prefix,
                                   "_",
                                   tx.input_index[0],
                                   "_",
                                   tx.input_index[1],
                                   "_",
                                   uint32_t(tx.input_note[0].value),
                                   "_",
                                   uint32_t(tx.input_note[1].value),
                                   "_",
                                   uint32_t(tx.output_note[0].value),
                                   "_",
                                   uint32_t(tx.output_note[1].value),
                                   "_",
                                   uint32_t(tx.public_input),
                                   "_",
                                   uint32_t(tx.claim_note.deposit_value));

        return compute_or_load_fixture(TEST_PROOFS_PATH, fixture_name, [&] {
            Composer composer =
                Composer(join_split_cd.proving_key, join_split_cd.verification_key, join_split_cd.num_gates);

            join_split::join_split_circuit(composer, tx);
            if (composer.failed) {
                throw_or_abort(format("Join-split logic failed: ", composer.err));
            }
            auto prover = composer.create_unrolled_prover();
            auto join_split_proof = prover.construct_proof();

            return join_split_proof.proof_data;
        });
    }

    std::vector<uint8_t> create_join_split_proof(std::array<uint32_t, 2> in_note_idx,
                                                 std::array<uint32_t, 2> in_note_value,
                                                 std::array<uint32_t, 2> out_note_value,
                                                 uint32_t public_input,
                                                 std::string const& fixture_prefix = "js")
    {
        auto tx = create_join_split_tx(in_note_idx, in_note_value, out_note_value, public_input);
        tx.signature = sign_join_split_tx(tx, user.owner);
        return create_proof(tx, fixture_prefix);
    }

    std::vector<uint8_t> create_defi_proof(std::array<uint32_t, 2> in_note_idx,
                                           std::array<uint32_t, 2> in_note_value,
                                           std::array<uint32_t, 2> out_note_value,
                                           uint256_t bridge_id,
                                           std::string const& fixture_prefix = "defi")
    {
        auto tx = create_join_split_tx(in_note_idx, in_note_value, out_note_value, 0);
        tx.num_input_notes = 2;
        tx.claim_note.bridge_id = bridge_id;
        tx.claim_note.deposit_value = tx.output_note[0].value;
        tx.output_note[0].value = 0;
        tx.signature = sign_join_split_tx(tx, user.owner);
        return create_proof(tx, fixture_prefix);
    }

    std::vector<uint8_t> create_claim_proof(
        barretenberg::fr const& defi_root,
        uint32_t claim_note_index,
        notes::native::claim::claim_note const& claim_note,
        notes::native::defi_interaction::defi_interaction_note const& defi_interaction_note)
    {
        proofs::claim::claim_tx tx;
        tx.data_root = world_state.data_tree.root();
        tx.defi_root = defi_root;
        tx.claim_note_index = claim_note_index;
        tx.claim_note_path = world_state.data_tree.get_hash_path(claim_note_index);
        tx.claim_note = claim_note;
        tx.defi_interaction_note_path = world_state.defi_tree.get_hash_path(defi_interaction_note.interaction_nonce);
        tx.defi_interaction_note = defi_interaction_note;
        tx.output_value_a = claim_note.deposit_value * defi_interaction_note.total_output_a_value /
                            defi_interaction_note.total_input_value;
        tx.output_value_b = claim_note.deposit_value * defi_interaction_note.total_output_b_value /
                            defi_interaction_note.total_input_value;

        Composer composer = Composer(claim_cd.proving_key, claim_cd.verification_key, claim_cd.num_gates);

        proofs::claim::claim_circuit(composer, tx);
        if (composer.failed) {
            throw_or_abort(format("Claim proof logic failed: ", composer.err));
        }
        auto prover = composer.create_unrolled_prover();
        auto proof = prover.construct_proof();

        return proof.proof_data;
    }

    root_rollup_tx create_root_rollup_tx(std::string const& test_name,
                                         uint32_t rollup_id,
                                         RollupStructure const& rollup_structure,
                                         std::vector<std::vector<uint32_t>> const& data_roots_indicies_ = { {} },
                                         std::vector<std::vector<uint32_t>> const& claim_note_nonces_ = {})
    {
        auto data_roots_indicies = data_roots_indicies_;
        data_roots_indicies.resize(rollup_structure.size());
        auto claim_note_nonces = claim_note_nonces_;
        claim_note_nonces.resize(rollup_structure.size());

        std::vector<rollup::rollup_tx> rollups;
        std::vector<std::vector<uint8_t>> rollups_data;

        for (size_t i = 0; i < rollup_structure.size(); ++i) {
            auto tx_proofs = rollup_structure[i];
            auto rollup = rollup::create_rollup(world_state,
                                                INNER_ROLLUP_TXS,
                                                tx_proofs,
                                                {},
                                                data_roots_indicies[i]); //, claim_note_nonces[i]);
            auto fixture_name = format(test_name, "_rollup", rollup_id, "_inner", rollups.size());
            auto rollup_data = compute_or_load_rollup(fixture_name, rollup);
            if (rollup_data.empty()) {
                throw std::runtime_error("Failed to create inner rollup proof.");
            }
            rollups.push_back(rollup);
            rollups_data.push_back(rollup_data);
        }

        return root_rollup::create_root_rollup_tx(rollup_id, rollups_data, world_state);
    }

    std::vector<uint8_t> compute_or_load_rollup(std::string const& name, rollup::rollup_tx& rollup)
    {
        return compute_or_load_fixture(TEST_PROOFS_PATH, name, [&] {
            // We need to ensure we have a proving key to build the inner proof fixtures.
            if (!tx_rollup_cd.proving_key) {
                account_cd = proofs::account::compute_or_load_circuit_data(srs, FIXTURE_PATH);
                join_split_cd = join_split::compute_or_load_circuit_data(srs, FIXTURE_PATH);
                tx_rollup_cd = rollup::get_circuit_data(
                    INNER_ROLLUP_TXS, join_split_cd, account_cd, claim_cd, srs, FIXTURE_PATH, true, true, true);
                root_rollup_cd.inner_rollup_circuit_data = tx_rollup_cd;
            }
            return rollup::verify(rollup, tx_rollup_cd).proof_data;
        });
    }

    // Create and return n deposit join split proofs.
    auto get_js_proofs(uint32_t n)
    {
        std::vector<std::vector<uint8_t>> proofs;
        for (uint32_t i = 0; i < n * 2; i += 2) {
            auto js_proof = create_join_split_proof({ i, i + 1 }, { 0, 0 }, { 100, 50 }, 150);
            proofs.push_back(js_proof);
        }
        return proofs;
    }

    void add_rollup_with_1_js()
    {
        auto js_proofs = get_js_proofs(1);
        create_root_rollup_tx(__FUNCTION__, 0, { { js_proofs[0] } });
    }

    auto create_rollup_tx_with_1_defi()
    {
        bridge_id bid = { 0, 2, 0, 0, 0 };
        auto defi_proof1 = create_defi_proof({ 0, 1 }, { 100, 50 }, { 40, 110 }, bid);
        auto tx = create_root_rollup_tx(__FUNCTION__, 1, { { defi_proof1 } });
        tx.bridge_ids = { bid };
        return tx;
    }

    auto create_root_rollup_tx_with_3_defi()
    {
        auto js_proofs = get_js_proofs(4);
        auto tx1 = create_root_rollup_tx(__FUNCTION__, 0, { { js_proofs[0], js_proofs[1] }, { js_proofs[2] } });
        auto result1 = verify_logic(tx1, root_rollup_cd);
        assert(result1.logic_verified);

        bridge_id bid1 = { 0, 2, 0, 0, 0 };
        bridge_id bid2 = { 1, 2, 0, 0, 0 };
        auto defi_proof1 = create_defi_proof({ 0, 1 }, { 100, 50 }, { 40, 110 }, bid1, __FUNCTION__);
        auto defi_proof2 = create_defi_proof({ 2, 3 }, { 100, 50 }, { 30, 120 }, bid1, __FUNCTION__);
        auto defi_proof3 = create_defi_proof({ 4, 5 }, { 100, 50 }, { 20, 130 }, bid2, __FUNCTION__);
        auto tx2 = create_root_rollup_tx(__FUNCTION__,
                                         1,
                                         { { js_proofs[3], defi_proof1 }, { defi_proof2, defi_proof3 } },
                                         { { 0 } },
                                         { { 0, 4 }, { 4, 5 } });
        tx2.bridge_ids = { bid1, bid2 };
        return tx2;
    }

    world_state::WorldState<MemoryStore> world_state;
};

/*
 * The fixtures names are named so as to reduce unnecessary (re)computation.
 * i.e. If a rollup has a structure shorter than its name suggests, it's because it can reuse the fixtures from
 * the longer rollup structure due to them having the same leading structure.
 */
TEST_F(root_rollup_tests, test_1_real_2_padding)
{
    auto js_proofs = get_js_proofs(1);
    auto tx_data = create_root_rollup_tx("root_1", 0, { { js_proofs[0] } });
    auto result = verify_logic(tx_data, root_rollup_cd);
    ASSERT_TRUE(result.logic_verified);
}

TEST_F(root_rollup_tests, test_2_real_1_padding)
{
    auto js_proofs = get_js_proofs(3);
    auto tx_data = create_root_rollup_tx("root_211", 0, { { js_proofs[0], js_proofs[1] }, { js_proofs[2] } });
    auto result = verify_logic(tx_data, root_rollup_cd);
    ASSERT_TRUE(result.logic_verified);
}

TEST_F(root_rollup_tests, test_3_real_0_padding)
{
    auto js_proofs = get_js_proofs(5);
    auto tx_data = create_root_rollup_tx(
        "root_221", 0, { { js_proofs[0], js_proofs[1] }, { js_proofs[2], js_proofs[3] }, { js_proofs[4] } });
    auto result = verify_logic(tx_data, root_rollup_cd);
    ASSERT_TRUE(result.logic_verified);
}

TEST_F(root_rollup_tests, test_incorrect_new_data_root_fails)
{
    auto js_proofs = get_js_proofs(1);
    auto tx_data = create_root_rollup_tx("bad_new_data_root_fail", 0, { { js_proofs[0] } });
    tx_data.new_data_roots_root = fr::random_element();
    auto result = verify_logic(tx_data, root_rollup_cd);
    ASSERT_FALSE(result.logic_verified);
}

TEST_F(root_rollup_tests, test_inner_rollups_out_of_order_fail)
{
    auto js_proofs = get_js_proofs(4);
    auto tx_data =
        create_root_rollup_tx("root_221", 0, { { js_proofs[0], js_proofs[1] }, { js_proofs[2], js_proofs[3] } });
    std::swap(tx_data.rollups[0], tx_data.rollups[1]);

    auto result = verify_logic(tx_data, root_rollup_cd);
    ASSERT_FALSE(result.logic_verified);
}

TEST_F(root_rollup_tests, test_invalid_padding_proof_fail)
{
    auto js_proofs = get_js_proofs(5);
    auto tx_data = create_root_rollup_tx(
        "root_221", 0, { { js_proofs[0], js_proofs[1] }, { js_proofs[2], js_proofs[3] }, { js_proofs[4] } });
    tx_data.num_inner_proofs = 2;
    auto result = verify_logic(tx_data, root_rollup_cd);
    ASSERT_FALSE(result.logic_verified);
}

TEST_F(root_rollup_tests, test_invalid_last_proof_fail)
{
    auto js_proofs = get_js_proofs(2);
    auto tx_data = create_root_rollup_tx("root_221", 0, { { js_proofs[0], js_proofs[1] } });
    tx_data.num_inner_proofs = 2;
    auto result = verify_logic(tx_data, root_rollup_cd);
    ASSERT_FALSE(result.logic_verified);
}

TEST_F(root_rollup_tests, test_defi_valid_previous_defi_hash_for_0_interactions)
{
    auto js_proofs = get_js_proofs(1);
    auto tx_data = create_root_rollup_tx("root_1", 0, { { js_proofs[0] } });
    // This note should be ignored (rollup 0)
    // TODO: tx_data.defi_interaction_notes.push_back()
    auto result = verify_logic(tx_data, root_rollup_cd);

    ASSERT_TRUE(result.logic_verified);

    auto previous_defi_interaction_hash = result.public_inputs.back();

    std::vector<uint8_t> sha256_input;
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; i++) {
        notes::native::defi_interaction::defi_interaction_note note = { 0, 0, 0, 0, 0, false };
        auto buf = note.to_byte_array();
        sha256_input.insert(sha256_input.end(), buf.begin(), buf.end());
    }
    auto expected = sha256::sha256(sha256_input);

    ASSERT_EQ(from_buffer<fr>(expected), previous_defi_interaction_hash);
}

TEST_F(root_rollup_tests, test_defi_proof_in_rollup)
{
    add_rollup_with_1_js();
    auto tx_data = create_rollup_tx_with_1_defi();
    auto result = verify_logic(tx_data, root_rollup_cd);

    ASSERT_TRUE(result.logic_verified);
}

TEST_F(root_rollup_tests, test_defi_bridge_id_zero_fails)
{
    add_rollup_with_1_js();
    auto tx_data = create_rollup_tx_with_1_defi();
    tx_data.bridge_ids = { 0 };
    auto result = verify_logic(tx_data, root_rollup_cd);

    ASSERT_FALSE(result.logic_verified);
}

TEST_F(root_rollup_tests, test_defi_bridge_id_repeated_fails)
{
    add_rollup_with_1_js();
    auto tx_data = create_rollup_tx_with_1_defi();
    tx_data.bridge_ids.push_back(tx_data.bridge_ids[0]);
    auto result = verify_logic(tx_data, root_rollup_cd);

    ASSERT_FALSE(result.logic_verified);
}

TEST_F(root_rollup_tests, test_defi_bridge_id_unmatched_fails)
{
    add_rollup_with_1_js();
    auto tx_data = create_rollup_tx_with_1_defi();
    tx_data.bridge_ids[0] = { 1, 2, 0, 0 };
    auto result = verify_logic(tx_data, root_rollup_cd);

    ASSERT_FALSE(result.logic_verified);
}

TEST_F(root_rollup_tests, test_defi_claim_notes_added_interaction_nonce)
{
    add_rollup_with_1_js();
    auto tx_data = create_rollup_tx_with_1_defi();
    auto result = verify_logic(tx_data, root_rollup_cd);

    ASSERT_TRUE(result.logic_verified);

    const auto public_input_start_idx = rollup::RollupProofFields::INNER_PROOFS_DATA;
    const auto output_note1_x = result.public_inputs[public_input_start_idx + InnerProofFields::NEW_NOTE1_X];
    const auto output_note1_y = result.public_inputs[public_input_start_idx + InnerProofFields::NEW_NOTE1_Y];
    const auto deposit_value = result.public_inputs[public_input_start_idx + InnerProofFields::PUBLIC_OUTPUT];
    const auto bid = tx_data.bridge_ids[0];

    auto partial_state = notes::native::claim::create_partial_value_note(user.note_secret, user.owner.public_key, 0);
    notes::native::claim::claim_note claim_note = { deposit_value, bid, 0, partial_state };
    auto expected = encrypt(claim_note);

    point_ct result_encrypted_claim_note{ output_note1_x, output_note1_y };

    EXPECT_EQ(result_encrypted_claim_note.x.get_value(), expected.x);
    EXPECT_EQ(result_encrypted_claim_note.y.get_value(), expected.y);
}

TEST_F(root_rollup_tests, test_process_defi_deposits)
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

TEST_F(root_rollup_tests, test_defi_interaction_notes_added_to_defi_tree) {}

TEST_F(root_rollup_tests, test_claim_proof_has_valid_defi_root)
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

} // namespace root_rollup
} // namespace proofs
} // namespace rollup