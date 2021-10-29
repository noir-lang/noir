#include <common/test.hpp>
#include <common/map.hpp>
#include <common/throw_or_abort.hpp>
#include <common/container.hpp>
#include "index.hpp"
#include "../rollup/index.hpp"
#include "../root_rollup/index.hpp"
#include "../notes/native/index.hpp"
#include "../../fixtures/test_context.hpp"

namespace rollup {
namespace proofs {
namespace root_verifier {

using namespace barretenberg;
using namespace notes::native;

namespace {
std::shared_ptr<waffle::DynamicFileReferenceStringFactory> srs;
join_split::circuit_data js_cd;
proofs::account::circuit_data account_cd;
proofs::circuit_data claim_cd;
rollup::circuit_data tx_rollup_cd;
root_rollup::circuit_data root_rollup_cd;
root_rollup::circuit_data root_rollup_cd_bad;
root_verifier::circuit_data root_verifier_cd;

struct RollupShape {
    uint INNER_ROLLUP_TXS;
    uint ROLLUPS_PER_ROLLUP;
    bool operator==(RollupShape const&) const = default;
};
} // namespace

class root_verifier_logic : public ::testing::Test {
  protected:
    static constexpr auto CRS_PATH = "../srs_db/ignition";
    static constexpr auto FIXTURE_PATH = "../src/aztec/rollup/proofs/root_verifier/fixtures";
    static constexpr auto TEST_PROOFS_PATH = "../src/aztec/rollup/proofs/root_verifier/fixtures/test_proofs";
    typedef std::vector<std::vector<std::vector<uint8_t>>> RollupStructure;

    root_verifier_logic()
        : context(js_cd, account_cd, claim_cd)
        , js_proofs(get_js_proofs(2))
    {}

    static void SetUpTestCase()
    {
        auto recreate = !exists(FIXTURE_PATH);
        srs = std::make_shared<waffle::DynamicFileReferenceStringFactory>(CRS_PATH);

        account_cd = proofs::account::compute_or_load_circuit_data(srs, FIXTURE_PATH);
        js_cd = join_split::compute_or_load_circuit_data(srs, FIXTURE_PATH);
        claim_cd = proofs::claim::get_circuit_data(srs, FIXTURE_PATH);

        if (recreate) {
            tx_rollup_cd = rollup::get_circuit_data(1U, js_cd, account_cd, claim_cd, srs, FIXTURE_PATH);
            // create 1x1 circuit data; this will be the only shape accepted by the root verifier circuit.
            root_rollup_cd = root_rollup::get_circuit_data(1U, tx_rollup_cd, srs, FIXTURE_PATH);
            root_verifier_cd =
                root_verifier::get_circuit_data(root_rollup_cd, srs, { root_rollup_cd.verification_key }, FIXTURE_PATH);
            // create 1x2 key to use later
            root_rollup_cd_bad = root_rollup::get_circuit_data(2U, tx_rollup_cd, srs, FIXTURE_PATH);
        } else {
            tx_rollup_cd = rollup::get_circuit_data(
                1U, js_cd, account_cd, claim_cd, srs, FIXTURE_PATH, false, false, true, false, true);
            root_rollup_cd =
                root_rollup::get_circuit_data(1U, tx_rollup_cd, srs, FIXTURE_PATH, false, false, true, false, true);
            root_verifier_cd = get_circuit_data(root_rollup_cd,
                                                srs,
                                                { root_rollup_cd.verification_key },
                                                FIXTURE_PATH,
                                                false,
                                                false,
                                                false,
                                                false,
                                                false);
            // create 1x2 key to use later
            root_rollup_cd_bad =
                root_rollup::get_circuit_data(2U, tx_rollup_cd, srs, FIXTURE_PATH, false, false, true, false, true);
        }
    }

    root_verifier_tx create_root_verifier_tx(
        std::string const& test_name,
        RollupShape const& shape,
        RollupStructure const& rollup_structure,
        std::vector<std::vector<uint256_t>> bridge_ids = { {}, {}, {} },
        std::vector<uint256_t> bridge_ids_union = {},
        std::vector<std::vector<uint256_t>> asset_ids = { { 0 }, { 0 }, { 0 } },
        std::vector<uint256_t> asset_ids_union = { 0 },
        std::vector<fixtures::native::defi_interaction::note> const& interaction_notes = {})
    {
        auto root_rollup = create_root_rollup_tx(test_name,
                                                 shape,
                                                 rollup_structure,
                                                 bridge_ids,
                                                 bridge_ids_union,
                                                 asset_ids,
                                                 asset_ids_union,
                                                 interaction_notes);

        auto fixture_name = format(test_name, "_root_rollup");
        auto proof_buf = compute_or_load_root_rollup(fixture_name, shape, root_rollup);
        if (proof_buf.empty()) {
            throw_or_abort("Failed to create root rollup proof.");
        }
        return root_verifier::create_root_verifier_tx(proof_buf, (uint32_t)root_rollup_cd.rollup_size);
    }

    root_rollup::root_rollup_tx create_root_rollup_tx(
        std::string const& test_name,
        RollupShape const& shape,
        RollupStructure const& rollup_structure,
        std::vector<std::vector<uint256_t>> bridge_ids = { {}, {}, {} },
        std::vector<uint256_t> bridge_ids_union = {},
        std::vector<std::vector<uint256_t>> asset_ids = { { 0 }, { 0 }, { 0 } },
        std::vector<uint256_t> asset_ids_union = { 0 },
        std::vector<fixtures::native::defi_interaction::note> const& interaction_notes = {})
    {
        uint32_t rollup_id = static_cast<uint32_t>(context.world_state.root_tree.size() - 1);
        auto old_defi_root = context.world_state.defi_tree.root();
        context.world_state.add_defi_notes(interaction_notes);

        std::vector<std::vector<uint8_t>> inner_data;
        for (size_t i = 0; i < rollup_structure.size(); ++i) {
            auto tx_proofs = rollup_structure[i];
            auto rollup = rollup::create_rollup_tx(
                context.world_state, shape.INNER_ROLLUP_TXS, tx_proofs, bridge_ids[i], asset_ids[i]);
            auto fixture_name = format(test_name, "_rollup", rollup_id, "_inner", inner_data.size());
            auto proof_data = compute_or_load_rollup(fixture_name, shape, rollup);
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

    std::vector<uint8_t> compute_or_load_root_rollup(std::string const& name,
                                                     RollupShape shape,
                                                     root_rollup::root_rollup_tx& root_rollup)
    {
        return root_rollup::compute_or_load_fixture(TEST_PROOFS_PATH, name, [&] {
            // compute_or_load_rollup is called first, so tx_rollup_cd has been built.
            // We need to ensure we have a proving key to build the root rollup proof fixtures.
            if (!root_rollup_cd.proving_key) {
                root_rollup_cd =
                    root_rollup::get_circuit_data(shape.ROLLUPS_PER_ROLLUP, tx_rollup_cd, srs, FIXTURE_PATH);
                root_verifier_cd.root_rollup_circuit_data = root_rollup_cd;
            }
            auto result = root_rollup::verify(root_rollup, root_rollup_cd);
            root_rollup::root_rollup_broadcast_data broadcast_data(result.broadcast_data);
            return join({ to_buffer(broadcast_data), result.proof_data });
        });
    }

    std::vector<uint8_t> compute_or_load_rollup(std::string const& name, RollupShape shape, rollup::rollup_tx& rollup)
    {
        return root_rollup::compute_or_load_fixture(TEST_PROOFS_PATH, name, [&] {
            // We need to ensure we have a proving key to build the inner proof fixtures.
            if (!tx_rollup_cd.proving_key) {
                tx_rollup_cd =
                    rollup::get_circuit_data(shape.INNER_ROLLUP_TXS, js_cd, account_cd, claim_cd, srs, FIXTURE_PATH);
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
            auto js_proof = context.create_join_split_proof({}, {}, { 100, 50 }, 150);
            proofs.push_back(js_proof);
        }
        return proofs;
    }

    fixtures::TestContext context;
    std::vector<std::vector<uint8_t>> js_proofs;
};

TEST_F(root_verifier_logic, passing)
{
    RollupShape shape = { 1U, 1U };
    RollupStructure structure{ { js_proofs[0] } };

    root_verifier_tx tx_data = create_root_verifier_tx("root_verifier", shape, structure);
    auto result = verify_logic(tx_data, root_verifier_cd);
    ASSERT_TRUE(result.logic_verified);
}

TEST_F(root_verifier_logic, failing_invalid_shape)
{
    RollupShape shape = { 1U, 1U };
    RollupStructure structure{ { js_proofs[0] } };

    root_verifier_tx tx_data = create_root_verifier_tx("root_verifier", shape, structure);

    // try to pass a root rollup circuit with invalid shape
    root_verifier_cd.root_rollup_circuit_data = root_rollup_cd_bad;
    auto result = verify_logic(tx_data, root_verifier_cd);
    ASSERT_FALSE(result.logic_verified);
}

} // namespace root_verifier
} // namespace proofs
} // namespace rollup