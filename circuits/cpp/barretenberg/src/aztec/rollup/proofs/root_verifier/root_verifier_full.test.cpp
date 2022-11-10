#include <common/test.hpp>
#include <common/map.hpp>
#include <common/throw_or_abort.hpp>
#include "index.hpp"
#include "../rollup/index.hpp"
#include "../root_rollup/index.hpp"
#include "../notes/native/index.hpp"
#include "../../fixtures/test_context.hpp"
#include "../../fixtures/compute_or_load_fixture.hpp"
#include "../../constants.hpp"
#include <ecc/curves/bn254/g1.hpp>

#include <fstream>
#include <filesystem>
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
proofs::rollup::circuit_data tx_rollup_cd;
proofs::root_rollup::circuit_data root_rollup_cd;
proofs::root_verifier::circuit_data root_verifier_cd;
std::vector<uint8_t> js_proof;
} // namespace

class root_verifier_full_tests : public ::testing::Test {
  protected:
    static constexpr auto CRS_PATH = "../srs_db/ignition";
    static constexpr auto FIXTURE_PATH = "../src/aztec/rollup/proofs/root_verifier/fixtures";
    static constexpr auto TEST_PROOFS_PATH = "../src/aztec/rollup/proofs/root_verifier/fixtures/test_proofs";

    typedef std::vector<std::vector<std::vector<uint8_t>>> RollupStructure;

    root_verifier_full_tests()
    {
        rand_engine = &numeric::random::get_debug_engine(true);
        user = fixtures::create_user_context(rand_engine);
    }

    static void SetUpTestCase()
    {
        std::filesystem::create_directories(FIXTURE_PATH);
        std::filesystem::create_directories(TEST_PROOFS_PATH);
        srs = std::make_shared<waffle::DynamicFileReferenceStringFactory>(CRS_PATH);
        account_cd = proofs::account::get_circuit_data(srs);
        join_split_cd = join_split::get_circuit_data(srs);
        claim_cd = proofs::claim::get_circuit_data(srs);
        tx_rollup_cd =
            rollup::get_circuit_data(1, join_split_cd, account_cd, claim_cd, srs, FIXTURE_PATH, true, false, false);
        root_rollup_cd = root_rollup::get_circuit_data(1, tx_rollup_cd, srs, FIXTURE_PATH, true, false, false);
        root_verifier_cd = get_circuit_data(
            root_rollup_cd, srs, { root_rollup_cd.verification_key }, FIXTURE_PATH, true, false, false);

        MemoryStore store;
        MerkleTree<MemoryStore> data_tree(store, DATA_TREE_DEPTH, 0);
        js_proof = join_split::create_noop_join_split_proof(join_split_cd, data_tree.root());
    }

    root_verifier_tx create_root_verifier_tx()
    {
        auto rollup_tx = rollup::create_rollup_tx(world_state, tx_rollup_cd.rollup_size, { js_proof });
        auto rollup_data = rollup::verify(rollup_tx, tx_rollup_cd).proof_data;
        ASSERT(!rollup_data.empty());
        auto root_rollup_tx = root_rollup::create_root_rollup_tx(
            world_state, 0, world_state.defi_tree.root(), world_state.defi_tree.get_hash_path(0), { rollup_data });
        auto result = root_rollup::verify(root_rollup_tx, root_rollup_cd);
        ASSERT(!result.proof_data.empty());

        return root_verifier::create_root_verifier_tx(result);
    }

    world_state::WorldState<MemoryStore> world_state;
};

HEAVY_TEST_F(root_verifier_full_tests, good_data_passes_and_detect_circuit_change)
{
    auto tx = create_root_verifier_tx();
    auto result = verify(tx, root_verifier_cd, root_rollup_cd);
    ASSERT_TRUE(result.verified);
    // The below part detects changes in the root verifier circuit
    size_t number_of_gates_root_verifier = result.number_of_gates;
    auto vk_hash_root_verifier = result.verification_key->sha256_hash();
    // If the below assertions fail, consider changing the variable is_circuit_change_expected to 1 in
    // rollup/constants.hpp and see if atleast the next power of two limit is not exceeded. Please change the constant
    // values accordingly and set is_circuit_change_expected to 0 in rollup/constants.hpp before merging.
    if (!(circuit_gate_count::is_circuit_change_expected)) {
        EXPECT_TRUE(number_of_gates_root_verifier == circuit_gate_count::ROOT_VERIFIER)
            << "The gate count for the root verifier circuit is changed.";
        EXPECT_TRUE(from_buffer<uint256_t>(vk_hash_root_verifier) == circuit_vk_hash::ROOT_VERIFIER)
            << "The verification key hash for the root verifier circuit is changed.";
        // For the next power of two limit, we need to consider that we reserve four gates for adding
        // randomness/zero-knowledge
        EXPECT_TRUE(number_of_gates_root_verifier <=
                    circuit_gate_next_power_of_two::ROOT_VERIFIER - waffle::ComposerBase::NUM_RESERVED_GATES)
            << "You have exceeded the next power of two limit for the root verifier circuit.";
    } else {
        EXPECT_TRUE(number_of_gates_root_verifier <=
                    circuit_gate_next_power_of_two::ROOT_VERIFIER - waffle::ComposerBase::NUM_RESERVED_GATES)
            << "You have exceeded the next power of two limit for the root verifier circuit.";
    }
}

HEAVY_TEST_F(root_verifier_full_tests, bad_byte_failure)
{
    auto tx = create_root_verifier_tx();

    // change the first byte of the root rollup proof data.
    tx.proof_data[0] = (tx.proof_data[0] == 0) ? 1 : 0;
    auto result = verify(tx, root_verifier_cd, root_rollup_cd);
    ASSERT_FALSE(result.verified);
}

HEAVY_TEST_F(root_verifier_full_tests, bad_valid_point_failure)
{
    auto tx = create_root_verifier_tx();

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

    auto result = root_verifier::verify(tx, root_verifier_cd, root_rollup_cd);
    ASSERT_FALSE(result.verified);
}

} // namespace root_verifier
} // namespace proofs
} // namespace rollup