// Uncomment to simulate running in CI.
// #define DISABLE_HEAVY_TESTS

#include "../../fixtures/user_context.hpp"
#include "../../constants.hpp"
#include "../rollup/verify.hpp"
#include "../rollup/rollup_proof_data.hpp"
#include "../join_split/create_noop_join_split_proof.hpp"
#include "../claim/get_circuit_data.hpp"
#include "compute_or_load_fixture.hpp"
#include "compute_circuit_data.hpp"
#include "root_rollup_circuit.hpp"
#include "create_root_rollup_tx.hpp"
#include "verify.hpp"
#include <common/test.hpp>

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
proofs::claim::circuit_data claim_cd;
std::vector<std::vector<uint8_t>> js_proofs;
} // namespace

class root_rollup_full_tests : public ::testing::Test {
  protected:
    static constexpr auto CRS_PATH = "../srs_db/ignition";
    static constexpr auto FIXTURE_PATH = "../src/aztec/rollup/proofs/root_rollup/fixtures";
    static constexpr auto TEST_PROOFS_PATH = "../src/aztec/rollup/proofs/root_rollup/fixtures/test_proofs";

    typedef std::vector<std::vector<std::vector<uint8_t>>> RollupStructure;

    root_rollup_full_tests()
    {
        rand_engine = &numeric::random::get_debug_engine(true);
        user = fixtures::create_user_context(rand_engine);
    }

    static void SetUpTestCase()
    {
        srs = std::make_shared<waffle::DynamicFileReferenceStringFactory>(CRS_PATH);
        account_cd = account::compute_or_load_circuit_data(srs, FIXTURE_PATH);
        join_split_cd = join_split::compute_or_load_circuit_data(srs, FIXTURE_PATH);
        claim_cd = proofs::claim::get_circuit_data(srs, FIXTURE_PATH, true, false, false);

        MemoryStore store;
        MerkleTree<MemoryStore> data_tree(store, DATA_TREE_DEPTH, 0);
        // Create 5 js proofs to play with.
        for (size_t i = 0; i < 5; ++i) {
            auto js_proof = compute_or_load_fixture(TEST_PROOFS_PATH, format("js", i), [&] {
                return create_noop_join_split_proof(join_split_cd, data_tree.root());
            });
            js_proofs.push_back(js_proof);
        }
    }

    root_rollup_tx create_root_rollup_tx(std::string const& test_name,
                                         uint32_t rollup_id,
                                         rollup::circuit_data const& tx_rollup_cd,
                                         RollupStructure const& rollup_structure)
    {
        std::vector<rollup::rollup_tx> rollups;
        std::vector<std::vector<uint8_t>> rollups_data;

        for (auto txs : rollup_structure) {
            auto name = format(test_name, "_rollup", rollups.size() + 1);
            auto rollup = rollup::create_rollup(world_state, tx_rollup_cd.rollup_size, txs);
            auto rollup_data = compute_or_load_fixture(
                TEST_PROOFS_PATH, name, [&] { return rollup::verify(rollup, tx_rollup_cd).proof_data; });
            assert(!rollup_data.empty());
            rollups.push_back(rollup);
            rollups_data.push_back(rollup_data);
        }

        return root_rollup::create_root_rollup_tx(rollup_id, rollups_data, world_state);
    }

    world_state::WorldState<MemoryStore> world_state;
};

HEAVY_TEST_F(root_rollup_full_tests, test_root_rollup_3x2)
{
    static constexpr auto inner_rollup_txs = 2U;
    static constexpr auto rollups_per_rollup = 3U;

    auto tx_rollup_cd = rollup::get_circuit_data(
        inner_rollup_txs, join_split_cd, account_cd, claim_cd, srs, FIXTURE_PATH, true, true, true);
    auto root_rollup_cd = get_circuit_data(rollups_per_rollup, tx_rollup_cd, srs, FIXTURE_PATH, true, false, false);

    auto old_data_root = world_state.data_tree.root();
    auto old_null_root = world_state.null_tree.root();
    auto old_root_root = world_state.root_tree.root();

    auto tx_data = create_root_rollup_tx(
        "test_root_rollup_3x2", 0, tx_rollup_cd, { { js_proofs[0], js_proofs[1] }, { js_proofs[2] } });
    auto result = verify(tx_data, root_rollup_cd);
    ASSERT_TRUE(result.verified);

    auto rollup_data = rollup::rollup_proof_data(result.proof_data);
    EXPECT_EQ(rollup_data.rollup_id, 0U);
    EXPECT_EQ(rollup_data.rollup_size, 8U);
    EXPECT_EQ(rollup_data.data_start_index, 0U);
    EXPECT_EQ(rollup_data.old_data_root, old_data_root);
    EXPECT_EQ(rollup_data.old_null_root, old_null_root);
    EXPECT_EQ(rollup_data.old_data_roots_root, old_root_root);
    EXPECT_EQ(rollup_data.new_data_root, world_state.data_tree.root());
    EXPECT_EQ(rollup_data.new_null_root, world_state.null_tree.root());
    EXPECT_EQ(rollup_data.new_data_roots_root, world_state.root_tree.root());

    auto inner_data = rollup_data.inner_proofs[3];
    EXPECT_EQ(inner_data.public_input, uint256_t(0));
    EXPECT_EQ(inner_data.public_output, uint256_t(0));
    EXPECT_EQ(inner_data.new_note1, grumpkin::g1::affine_element(0));
    EXPECT_EQ(inner_data.new_note2, grumpkin::g1::affine_element(0));
    EXPECT_EQ(inner_data.nullifier1, uint256_t(0));
    EXPECT_EQ(inner_data.nullifier2, uint256_t(0));
    EXPECT_EQ(inner_data.input_owner, fr(0));
    EXPECT_EQ(inner_data.output_owner, fr(0));
}

HEAVY_TEST_F(root_rollup_full_tests, test_root_rollup_2x3)
{
    static constexpr auto inner_rollup_txs = 3U;
    static constexpr auto rollups_per_rollup = 2U;

    auto tx_rollup_cd = rollup::get_circuit_data(
        inner_rollup_txs, join_split_cd, account_cd, claim_cd, srs, FIXTURE_PATH, true, true, true);
    auto root_rollup_cd = get_circuit_data(rollups_per_rollup, tx_rollup_cd, srs, FIXTURE_PATH, true, false, false);

    auto old_data_root = world_state.data_tree.root();
    auto old_null_root = world_state.null_tree.root();
    auto old_root_root = world_state.root_tree.root();

    auto tx_data = create_root_rollup_tx("test_root_rollup_2x3", 0, tx_rollup_cd, { { js_proofs[0] } });
    auto result = verify(tx_data, root_rollup_cd);
    ASSERT_TRUE(result.verified);

    auto rollup_data = rollup::rollup_proof_data(result.proof_data);
    EXPECT_EQ(rollup_data.rollup_id, 0U);
    EXPECT_EQ(rollup_data.rollup_size, 8U);
    EXPECT_EQ(rollup_data.data_start_index, 0U);
    EXPECT_EQ(rollup_data.old_data_root, old_data_root);
    EXPECT_EQ(rollup_data.old_null_root, old_null_root);
    EXPECT_EQ(rollup_data.old_data_roots_root, old_root_root);
    EXPECT_EQ(rollup_data.new_data_root, world_state.data_tree.root());
    EXPECT_EQ(rollup_data.new_null_root, world_state.null_tree.root());
    EXPECT_EQ(rollup_data.new_data_roots_root, world_state.root_tree.root());

    for (size_t i = 1; i < rollup_data.inner_proofs.size(); ++i) {
        auto inner_data = rollup_data.inner_proofs[i];
        EXPECT_EQ(inner_data.public_input, uint256_t(0));
        EXPECT_EQ(inner_data.public_output, uint256_t(0));
        EXPECT_EQ(inner_data.new_note1, grumpkin::g1::affine_element(0));
        EXPECT_EQ(inner_data.new_note2, grumpkin::g1::affine_element(0));
        EXPECT_EQ(inner_data.nullifier1, uint256_t(0));
        EXPECT_EQ(inner_data.nullifier2, uint256_t(0));
        EXPECT_EQ(inner_data.input_owner, fr(0));
        EXPECT_EQ(inner_data.output_owner, fr(0));
    }
}

} // namespace root_rollup
} // namespace proofs
} // namespace rollup