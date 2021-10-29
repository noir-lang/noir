// Uncomment to simulate running in CI.
// #define DISABLE_HEAVY_TESTS

#include <common/test.hpp>
#include <common/map.hpp>
#include <common/throw_or_abort.hpp>
#include "index.hpp"
#include "../rollup/index.hpp"
#include "../root_rollup/index.hpp"
#include "../notes/native/index.hpp"
#include "../../fixtures/test_context.hpp"
#include "../inner_proof_data.hpp"
#include "../../constants.hpp"
#include <ecc/curves/bn254/g1.hpp>

#include <fstream>
#include <common/serialize.hpp>

namespace rollup {
namespace proofs {
namespace root_verifier {

using namespace barretenberg;
using namespace notes::native;
using namespace plonk::stdlib::merkle_tree;

namespace {
std::shared_ptr<waffle::DynamicFileReferenceStringFactory> srs;
numeric::random::Engine* rand_engine = &numeric::random::get_debug_engine(true);
fixtures::user_context user = fixtures::create_user_context(rand_engine);
join_split::circuit_data join_split_cd;
proofs::account::circuit_data account_cd;
proofs::claim::circuit_data claim_cd;
std::vector<std::vector<uint8_t>> js_proofs;
} // namespace

class root_verifier_full : public ::testing::Test {
  protected:
    static constexpr auto CRS_PATH = "../srs_db/ignition";
    static constexpr auto FIXTURE_PATH = "../src/aztec/rollup/proofs/root_verifier/fixtures";
    static constexpr auto TEST_PROOFS_PATH = "../src/aztec/rollup/proofs/root_verifier/fixtures/test_proofs";

    typedef std::vector<std::vector<std::vector<uint8_t>>> RollupStructure;

    root_verifier_full()
    {
        rand_engine = &numeric::random::get_debug_engine(true);
        user = fixtures::create_user_context(rand_engine);
    }

    static void SetUpTestCase()
    {
        mkdir(FIXTURE_PATH, 0700);
        mkdir(TEST_PROOFS_PATH, 0700);
        srs = std::make_shared<waffle::DynamicFileReferenceStringFactory>(CRS_PATH);
        account_cd = proofs::account::compute_circuit_data(srs);
        join_split_cd = join_split::compute_circuit_data(srs);
        claim_cd = proofs::claim::get_circuit_data(srs, FIXTURE_PATH, true, false, false);

        MemoryStore store;
        MerkleTree<MemoryStore> data_tree(store, DATA_TREE_DEPTH, 0);
        // Create 1 js proofs to play with.
        for (size_t i = 0; i < 1; ++i) {
            auto js_proof = root_rollup::compute_or_load_fixture(TEST_PROOFS_PATH, format("js", i), [&] {
                return join_split::create_noop_join_split_proof(join_split_cd, data_tree.root());
            });
            js_proofs.push_back(js_proof);
        }
    }

    root_verifier_tx create_root_verifier_tx(std::string const& test_name,
                                             uint32_t rollup_id,
                                             rollup::circuit_data const& tx_rollup_cd,
                                             root_rollup::circuit_data const& root_rollup_cd,
                                             RollupStructure const& rollup_structure)
    {
        root_rollup::verify_result result;

        auto root_rollup = create_root_rollup_tx(test_name, rollup_id, tx_rollup_cd, rollup_structure);
        result = root_rollup::verify(root_rollup, root_rollup_cd);
        ASSERT(!result.proof_data.empty());

        return root_verifier::create_root_verifier_tx(result);
    }

    root_rollup::root_rollup_tx create_root_rollup_tx(std::string const& test_name,
                                                      uint32_t rollup_id,
                                                      rollup::circuit_data const& tx_rollup_cd,
                                                      RollupStructure const& rollup_structure)
    {
        std::vector<rollup::rollup_tx> rollups;
        std::vector<std::vector<uint8_t>> rollups_data;

        for (auto txs : rollup_structure) {
            auto name = format(test_name, "_rollup", rollups_data.size() + 1);
            auto rollup = rollup::create_rollup_tx(world_state, tx_rollup_cd.rollup_size, txs, {}, { 0 });
            auto rollup_data = root_rollup::compute_or_load_fixture(
                TEST_PROOFS_PATH, name, [&] { return rollup::verify(rollup, tx_rollup_cd).proof_data; });
            assert(!rollup_data.empty());
            rollups_data.push_back(rollup_data);
        }

        return root_rollup::create_root_rollup_tx(
            world_state, rollup_id, world_state.defi_tree.root(), rollups_data, {}, { 0 });
    }

    world_state::WorldState<MemoryStore> world_state;
};

HEAVY_TEST_F(root_verifier_full, good_data_passes)
{
    static constexpr auto inner_rollup_txs = 1U;
    static constexpr auto rollups_per_rollup = 1U;

    auto tx_rollup_cd = rollup::get_circuit_data(
        inner_rollup_txs, join_split_cd, account_cd, claim_cd, srs, FIXTURE_PATH, true, true, true);
    auto root_rollup_cd =
        root_rollup::get_circuit_data(rollups_per_rollup, tx_rollup_cd, srs, FIXTURE_PATH, true, true, true);
    auto root_verifier_cd =
        get_circuit_data(root_rollup_cd, srs, { root_rollup_cd.verification_key }, FIXTURE_PATH, true, false, false);

    auto tx = create_root_verifier_tx("test_root_verifier_1x1", 0, tx_rollup_cd, root_rollup_cd, { { js_proofs[0] } });
    auto result = verify(tx, root_verifier_cd);
    ASSERT_TRUE(result.verified);
}

HEAVY_TEST_F(root_verifier_full, bad_byte_failure)
{
    static constexpr auto inner_rollup_txs = 1U;
    static constexpr auto rollups_per_rollup = 1U;

    auto tx_rollup_cd = rollup::get_circuit_data(
        inner_rollup_txs, join_split_cd, account_cd, claim_cd, srs, FIXTURE_PATH, true, true, true);
    auto root_rollup_cd =
        root_rollup::get_circuit_data(rollups_per_rollup, tx_rollup_cd, srs, FIXTURE_PATH, true, true, true);
    auto root_verifier_cd =
        get_circuit_data(root_rollup_cd, srs, { root_rollup_cd.verification_key }, FIXTURE_PATH, true, false, false);

    auto tx = create_root_verifier_tx("test_root_verifier_1x1", 0, tx_rollup_cd, root_rollup_cd, { { js_proofs[0] } });

    // change the first byte of the root rollup proof data.
    tx.proof_data[0] = (tx.proof_data[0] == 0) ? 1 : 0;
    auto result = verify(tx, root_verifier_cd);
    ASSERT_FALSE(result.verified);
}

HEAVY_TEST_F(root_verifier_full, bad_valid_point_failure)
{
    static constexpr auto inner_rollup_txs = 1U;
    static constexpr auto rollups_per_rollup = 1U;

    auto tx_rollup_cd = rollup::get_circuit_data(
        inner_rollup_txs, join_split_cd, account_cd, claim_cd, srs, FIXTURE_PATH, true, true, true);
    auto root_rollup_cd =
        root_rollup::get_circuit_data(rollups_per_rollup, tx_rollup_cd, srs, FIXTURE_PATH, true, true, true);
    auto root_verifier_cd =
        get_circuit_data(root_rollup_cd, srs, { root_rollup_cd.verification_key }, FIXTURE_PATH, true, false, false);

    auto tx = create_root_verifier_tx("test_root_verifier_1x1", 0, tx_rollup_cd, root_rollup_cd, { { js_proofs[0] } });

    auto data = root_verifier_proof_data(tx.proof_data);

    /*
     * Check that the first recursive proof element occurring in tx.proof_data is not the identity
     * element of the curve (exceedingly unlikely). Then invert this element and check that the proof does not
     * verify. We do this inversion 'by hand', inverting the y-coordinate, for simplicity.
     */

    g1::affine_element P = data.recursion_output[0];
    ASSERT_FALSE(P.is_point_at_infinity());
    auto minus_P_ct = outer_curve::g1_ct(-P);

    fr minus_y = minus_P_ct.y.binary_basis_limbs->element.additive_constant;
    uint8_t* ptr = tx.proof_data.data();
    // skip some public inputs fields and the x-coordinate of the first recursive proof element.
    ptr += RootVerifierProofFields::NUM_FIELDS * 32 + (4 * 32);
    fr::serialize_to_buffer(minus_y, ptr);

    auto result = root_verifier::verify(tx, root_verifier_cd);
    ASSERT_FALSE(result.verified);
}

} // namespace root_verifier
} // namespace proofs
} // namespace rollup