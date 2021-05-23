#include <crypto/sha256/sha256.hpp>
#include <common/test.hpp>
#include <common/throw_or_abort.hpp>
#include "../../fixtures/user_context.hpp"
#include "../../constants.hpp"
#include "../rollup/verify.hpp"
#include "../join_split/create_noop_join_split_proof.hpp"
#include "../join_split/sign_join_split_tx.hpp"
#include "../join_split/join_split_circuit.hpp"
#include "compute_or_load_fixture.hpp"
#include "compute_circuit_data.hpp"
#include "root_rollup_circuit.hpp"
#include "create_root_rollup_tx.hpp"
#include "verify.hpp"

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
account::circuit_data account_cd;
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
        : data_tree(store, DATA_TREE_DEPTH, 0)
        , null_tree(store, NULL_TREE_DEPTH, 1)
        , root_tree(store, ROOT_TREE_DEPTH, 2)
        , defi_tree(store, DEFI_TREE_DEPTH, 3)
    {
        update_root_tree_with_data_root(0);
        rand_engine = &numeric::random::get_debug_engine(true);
        user = fixtures::create_user_context(rand_engine);
    }

    static void SetUpTestCase()
    {
        auto recreate = !exists(FIXTURE_PATH);
        srs = std::make_shared<waffle::DynamicFileReferenceStringFactory>(CRS_PATH);

        if (recreate) {
            // If no fixtures dir, recreate all proving keys, verification keys, padding proofs etc.
            account_cd = account::compute_or_load_circuit_data(srs, FIXTURE_PATH);
            join_split_cd = join_split::compute_or_load_circuit_data(srs, FIXTURE_PATH);
            tx_rollup_cd = rollup::get_circuit_data(
                INNER_ROLLUP_TXS, join_split_cd, account_cd, srs, FIXTURE_PATH, true, true, true);
            root_rollup_cd = get_circuit_data(ROLLUPS_PER_ROLLUP, tx_rollup_cd, srs, FIXTURE_PATH, true, true, true);
        } else {
            // Otherwise we should only need the inner proofs verification key for logic tests.
            tx_rollup_cd = rollup::get_circuit_data(
                INNER_ROLLUP_TXS, join_split_cd, account_cd, srs, FIXTURE_PATH, false, false, true, false, true);
            root_rollup_cd = get_circuit_data(
                ROLLUPS_PER_ROLLUP, tx_rollup_cd, srs, FIXTURE_PATH, false, false, false, false, false);
        }

        // TODO: REMOVE ME
        join_split_cd = join_split::compute_or_load_circuit_data(srs, FIXTURE_PATH);
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
        tx.old_data_root = data_tree.root();
        tx.input_path = { data_tree.get_hash_path(in_note_idx[0]), data_tree.get_hash_path(in_note_idx[1]) };
        tx.input_note = { input_note1, input_note2 };
        tx.output_note = { output_note1, output_note2 };
        tx.account_index = 0;
        tx.account_path = data_tree.get_hash_path(0);
        tx.signing_pub_key = user.owner.public_key;
        tx.account_private_key = user.owner.private_key;
        tx.alias_hash = 0;
        tx.nonce = 0;
        tx.input_owner = fr::random_element();
        tx.output_owner = fr::random_element();
        tx.claim_note = { 0, 0, user.note_secret, 0 };

        return tx;
    }

    std::vector<uint8_t> create_proof(join_split_tx const& tx)
    {
        auto fixture_name = format("js_",
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
                                                 uint32_t public_input)
    {
        auto tx = create_join_split_tx(in_note_idx, in_note_value, out_note_value, public_input);
        tx.signature = sign_join_split_tx(tx, user.owner);
        return create_proof(tx);
    }

    std::vector<uint8_t> create_defi_proof(std::array<uint32_t, 2> in_note_idx,
                                           std::array<uint32_t, 2> in_note_value,
                                           std::array<uint32_t, 2> out_note_value,
                                           uint256_t bridge_id)
    {
        auto tx = create_join_split_tx(in_note_idx, in_note_value, out_note_value, 0);
        tx.num_input_notes = 2;
        tx.claim_note.bridge_id = bridge_id;
        tx.claim_note.deposit_value = tx.output_note[0].value;
        tx.output_note[0].value = 0;
        tx.signature = sign_join_split_tx(tx, user.owner);
        return create_proof(tx);
    }

    root_rollup_tx create_root_rollup_tx(std::string const& test_name,
                                         uint32_t rollup_id,
                                         RollupStructure const& rollup_structure)
    {
        std::vector<rollup::rollup_tx> rollups;
        std::vector<std::vector<uint8_t>> rollups_data;

        for (auto tx_proofs : rollup_structure) {
            auto rollup = rollup::create_rollup(tx_proofs, data_tree, null_tree, root_tree, INNER_ROLLUP_TXS);
            auto fixture_name = format(test_name, "_rollup", rollups.size() + 1);
            auto rollup_data = compute_or_load_rollup(fixture_name, rollup);
            ASSERT(!rollup_data.empty());
            rollups.push_back(rollup);
            rollups_data.push_back(rollup_data);
        }

        return root_rollup::create_root_rollup_tx(rollup_id, rollups_data, data_tree, root_tree, defi_tree);
    }

    void update_root_tree_with_data_root(size_t index)
    {
        auto data_root = to_buffer(data_tree.root());
        root_tree.update_element(index, data_root);
    }

    std::vector<uint8_t> compute_or_load_rollup(std::string const& name, rollup::rollup_tx& rollup)
    {
        return compute_or_load_fixture(TEST_PROOFS_PATH, name, [&] {
            // We need to ensure we have a proving key to build the inner proof fixtures.
            if (!tx_rollup_cd.proving_key) {
                account_cd = account::compute_or_load_circuit_data(srs, FIXTURE_PATH);
                join_split_cd = join_split::compute_or_load_circuit_data(srs, FIXTURE_PATH);
                tx_rollup_cd = rollup::get_circuit_data(
                    INNER_ROLLUP_TXS, join_split_cd, account_cd, srs, FIXTURE_PATH, true, true, true);
                root_rollup_cd.inner_rollup_circuit_data = tx_rollup_cd;
            }
            return rollup::verify_rollup(rollup, tx_rollup_cd).proof_data;
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
        create_root_rollup_tx("root_1", 0, { { js_proofs[0] } });
    }

    auto create_rollup_tx_with_1_defi()
    {
        bridge_id bid = { 0, 2, 0, 0, 0 };
        auto defi_proof1 = create_defi_proof({ 0, 1 }, { 100, 50 }, { 40, 110 }, bid);
        auto tx = create_root_rollup_tx("test_root_rollup_with_defi_proof", 1, { { defi_proof1 } });
        tx.bridge_ids = { bid };
        return tx;
    }

    MemoryStore store;
    MerkleTree<MemoryStore> data_tree;
    MerkleTree<MemoryStore> null_tree;
    MerkleTree<MemoryStore> root_tree;
    MerkleTree<MemoryStore> defi_tree;
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

TEST_F(root_rollup_tests, test_defi_claim_notes_added_interaction_nonce) {}

TEST_F(root_rollup_tests, test_defi_interaction_notes_added_to_defi_tree) {}

} // namespace root_rollup
} // namespace proofs
} // namespace rollup