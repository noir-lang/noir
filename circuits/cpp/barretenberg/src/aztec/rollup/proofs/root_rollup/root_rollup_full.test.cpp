// Uncomment to simulate running in CI.
// #define DISABLE_HEAVY_TESTS

#include "../../fixtures/user_context.hpp"
#include "../../constants.hpp"
#include "../rollup/verify.hpp"
#include "../join_split/create_noop_join_split_proof.hpp"
#include "../join_split/join_split_circuit.hpp"
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
std::vector<std::vector<uint8_t>> js_proofs;
} // namespace

class root_rollup_full_tests : public ::testing::Test {
  protected:
    static constexpr auto CRS_PATH = "../srs_db/ignition";
    static constexpr auto FIXTURE_PATH = "../src/aztec/rollup/proofs/root_rollup/fixtures";
    static constexpr auto TEST_PROOFS_PATH = "../src/aztec/rollup/proofs/root_rollup/fixtures/test_proofs";

    typedef std::vector<std::vector<std::vector<uint8_t>>> RollupStructure;

    root_rollup_full_tests()
        : data_tree(store, DATA_TREE_DEPTH, 0)
        , null_tree(store, NULL_TREE_DEPTH, 1)
        , root_tree(store, ROOT_TREE_DEPTH, 2)
    {
        update_root_tree_with_data_root(0);
        rand_engine = &numeric::random::get_debug_engine(true);
        user = fixtures::create_user_context(rand_engine);
    }

    static void SetUpTestCase()
    {
        srs = std::make_shared<waffle::DynamicFileReferenceStringFactory>(CRS_PATH);
        account_cd = account::compute_circuit_data(srs);
        join_split_cd = join_split::compute_circuit_data(srs);

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
            auto rollup = rollup::create_rollup(txs, data_tree, null_tree, root_tree, tx_rollup_cd.rollup_size);
            auto rollup_data = compute_or_load_fixture(
                TEST_PROOFS_PATH, name, [&] { return rollup::verify_rollup(rollup, tx_rollup_cd).proof_data; });
            assert(!rollup_data.empty());
            rollups.push_back(rollup);
            rollups_data.push_back(rollup_data);
        }

        return root_rollup::create_root_rollup_tx(rollup_id, rollups_data, data_tree, root_tree);
    }

    void update_root_tree_with_data_root(size_t index)
    {
        auto data_root = to_buffer(data_tree.root());
        root_tree.update_element(index, data_root);
    }

    MemoryStore store;
    MerkleTree<MemoryStore> data_tree;
    MerkleTree<MemoryStore> null_tree;
    MerkleTree<MemoryStore> root_tree;
};

HEAVY_TEST_F(root_rollup_full_tests, test_root_rollup_3x2)
{
    static constexpr auto inner_rollup_txs = 2U;
    static constexpr auto rollups_per_rollup = 3U;

    auto tx_rollup_cd =
        rollup::get_circuit_data(inner_rollup_txs, join_split_cd, account_cd, srs, FIXTURE_PATH, true, true, true);
    auto root_rollup_cd = get_circuit_data(rollups_per_rollup, tx_rollup_cd, srs, FIXTURE_PATH, true, false, false);

    auto old_data_root = data_tree.root();
    auto old_null_root = null_tree.root();
    auto old_root_root = root_tree.root();

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
    EXPECT_EQ(rollup_data.new_data_root, data_tree.root());
    EXPECT_EQ(rollup_data.new_null_root, null_tree.root());
    EXPECT_EQ(rollup_data.new_data_roots_root, root_tree.root());
    EXPECT_EQ(rollup_data.num_txs, 0U);

    auto inner_data = rollup_data.inner_proofs[3];
    auto zero_arr = std::array<uint8_t, 64>();
    EXPECT_EQ(inner_data.public_input, uint256_t(0));
    EXPECT_EQ(inner_data.public_output, uint256_t(0));
    EXPECT_EQ(inner_data.new_note1, zero_arr);
    EXPECT_EQ(inner_data.new_note2, zero_arr);
    EXPECT_EQ(inner_data.nullifier1, uint256_t(0));
    EXPECT_EQ(inner_data.nullifier2, uint256_t(0));
    EXPECT_EQ(inner_data.input_owner, fr(0));
    EXPECT_EQ(inner_data.output_owner, fr(0));
}

HEAVY_TEST_F(root_rollup_full_tests, test_root_rollup_2x3)
{
    static constexpr auto inner_rollup_txs = 3U;
    static constexpr auto rollups_per_rollup = 2U;

    auto tx_rollup_cd =
        rollup::get_circuit_data(inner_rollup_txs, join_split_cd, account_cd, srs, FIXTURE_PATH, true, true, true);
    auto root_rollup_cd = get_circuit_data(rollups_per_rollup, tx_rollup_cd, srs, FIXTURE_PATH, true, false, false);

    auto old_data_root = data_tree.root();
    auto old_null_root = null_tree.root();
    auto old_root_root = root_tree.root();

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
    EXPECT_EQ(rollup_data.new_data_root, data_tree.root());
    EXPECT_EQ(rollup_data.new_null_root, null_tree.root());
    EXPECT_EQ(rollup_data.new_data_roots_root, root_tree.root());
    EXPECT_EQ(rollup_data.num_txs, 0U);

    for (size_t i = 1; i < rollup_data.inner_proofs.size(); ++i) {
        auto inner_data = rollup_data.inner_proofs[i];
        auto zero_arr = std::array<uint8_t, 64>();
        EXPECT_EQ(inner_data.public_input, uint256_t(0));
        EXPECT_EQ(inner_data.public_output, uint256_t(0));
        EXPECT_EQ(inner_data.new_note1, zero_arr);
        EXPECT_EQ(inner_data.new_note2, zero_arr);
        EXPECT_EQ(inner_data.nullifier1, uint256_t(0));
        EXPECT_EQ(inner_data.nullifier2, uint256_t(0));
        EXPECT_EQ(inner_data.input_owner, fr(0));
        EXPECT_EQ(inner_data.output_owner, fr(0));
    }
}

HEAVY_TEST_F(root_rollup_full_tests, test_bad_js_proof_fails)
{
    static constexpr auto inner_rollup_txs = 2U;
    static constexpr auto rollups_per_rollup = 1U;

    // Create a bad js proof.
    auto bad_proof = create_noop_join_split_proof(join_split_cd, data_tree.root(), false);

    // Our inner rollup should fail.
    auto tx_rollup_cd =
        rollup::get_circuit_data(inner_rollup_txs, join_split_cd, account_cd, srs, FIXTURE_PATH, true, true, true);
    auto inner_rollup_tx =
        rollup::create_rollup({ js_proofs[0], bad_proof }, data_tree, null_tree, root_tree, tx_rollup_cd.rollup_size);
    Composer inner_composer = Composer(tx_rollup_cd.proving_key, tx_rollup_cd.verification_key, tx_rollup_cd.num_gates);
    rollup::rollup_circuit(inner_composer, inner_rollup_tx, tx_rollup_cd.verification_keys, tx_rollup_cd.num_txs);
    ASSERT_FALSE(inner_composer.failed);
    auto inner_prover = inner_composer.create_unrolled_prover();
    auto inner_proof = inner_prover.construct_proof();
    auto inner_verifier = inner_composer.create_unrolled_verifier();
    ASSERT_FALSE(inner_verifier.verify_proof(inner_proof));

    // Root rollup should fail.
    auto root_rollup_cd = get_circuit_data(rollups_per_rollup, tx_rollup_cd, srs, FIXTURE_PATH, true, false, false);
    auto root_rollup_tx = root_rollup::create_root_rollup_tx(0, { inner_proof.proof_data }, data_tree, root_tree);
    Composer root_composer =
        Composer(root_rollup_cd.proving_key, root_rollup_cd.verification_key, root_rollup_cd.num_gates);
    root_rollup_circuit(root_composer,
                        root_rollup_tx,
                        root_rollup_cd.inner_rollup_circuit_data.rollup_size,
                        root_rollup_cd.rollup_size,
                        root_rollup_cd.inner_rollup_circuit_data.verification_key);
    ASSERT_FALSE(root_composer.failed);
    auto root_prover = root_composer.create_prover();
    auto root_proof = root_prover.construct_proof();
    auto root_verifier = root_composer.create_verifier();
    ASSERT_FALSE(root_verifier.verify_proof(root_proof));
}

} // namespace root_rollup
} // namespace proofs
} // namespace rollup