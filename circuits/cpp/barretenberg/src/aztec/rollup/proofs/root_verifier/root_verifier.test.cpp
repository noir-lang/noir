#include <common/test.hpp>
#include <common/map.hpp>
#include <common/throw_or_abort.hpp>
#include <common/container.hpp>
#include "index.hpp"
#include "../rollup/index.hpp"
#include "../root_rollup/index.hpp"
#include "../notes/native/index.hpp"
#include "../../fixtures/test_context.hpp"
#include "../../fixtures/compute_or_load_fixture.hpp"

namespace rollup {
namespace proofs {
namespace root_verifier {

using namespace barretenberg;
using namespace notes::native;

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
root_rollup::circuit_data root_rollup_cd;
root_rollup::circuit_data root_rollup_cd_bad;
root_verifier::circuit_data root_verifier_cd;

struct RollupShape {
    uint INNER_ROLLUP_TXS;
    uint ROLLUPS_PER_ROLLUP;
    bool operator==(RollupShape const&) const = default;
};
} // namespace

class root_verifier_tests : public ::testing::Test {
  protected:
    static constexpr auto CRS_PATH = "../srs_db/ignition";
    static constexpr auto FIXTURE_PATH = "../src/aztec/rollup/proofs/root_verifier/fixtures";
    static constexpr auto TEST_PROOFS_PATH = "../src/aztec/rollup/proofs/root_verifier/fixtures/test_proofs";
    typedef std::vector<std::vector<std::vector<uint8_t>>> RollupStructure;

    root_verifier_tests()
        : context(js_cd, account_cd, claim_cd)
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
            tx_rollup_cd =
                rollup::get_circuit_data(1U, js_cd, account_cd, claim_cd, srs, FIXTURE_PATH, true, persist, persist);
            // create 1x1 circuit data; this will be the only shape accepted by the root verifier circuit.
            root_rollup_cd = root_rollup::get_circuit_data(1U, tx_rollup_cd, srs, FIXTURE_PATH, true, persist, persist);
            root_verifier_cd = root_verifier::get_circuit_data(
                root_rollup_cd, srs, { root_rollup_cd.verification_key }, FIXTURE_PATH, true, persist, persist);
            // create 1x2 key to use later
            root_rollup_cd_bad =
                root_rollup::get_circuit_data(2U, tx_rollup_cd, srs, FIXTURE_PATH, true, persist, persist);
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
                                                true,
                                                false,
                                                true);
            // create 1x2 key to use later
            root_rollup_cd_bad =
                root_rollup::get_circuit_data(2U, tx_rollup_cd, srs, FIXTURE_PATH, false, false, true, false, true);
        }
    }

    root_verifier_tx create_root_verifier_tx()
    {
        auto root_rollup = fixtures::compute_or_load_fixture(TEST_PROOFS_PATH, "root_rollup", [&]() {
            auto js_proof = context.create_join_split_proof({}, {}, { 100, 50 }, 150);
            auto rollup_tx = rollup::create_rollup_tx(context.world_state, tx_rollup_cd.rollup_size, { js_proof });
            auto rollup_data = rollup::verify(rollup_tx, tx_rollup_cd).proof_data;
            ASSERT(!rollup_data.empty());
            auto root_rollup_tx = root_rollup::create_root_rollup_tx(
                context.world_state, 0, context.world_state.defi_tree.root(), { rollup_data });
            auto result = root_rollup::verify(root_rollup_tx, root_rollup_cd);
            ASSERT(!result.proof_data.empty());
            return join({ to_buffer(result.broadcast_data), result.proof_data });
        });

        return root_verifier::create_root_verifier_tx(root_rollup, 1);
    }

    fixtures::TestContext context;
};

TEST_F(root_verifier_tests, passing)
{
    root_verifier_tx tx_data = create_root_verifier_tx();
    auto result = verify_logic(tx_data, root_verifier_cd);
    ASSERT_TRUE(result.logic_verified);
}

TEST_F(root_verifier_tests, failing_invalid_shape)
{
    root_verifier_tx tx_data = create_root_verifier_tx();

    // try to pass a root rollup circuit with invalid shape
    root_verifier_cd.root_rollup_circuit_data = root_rollup_cd_bad;
    auto result = verify_logic(tx_data, root_verifier_cd);
    ASSERT_FALSE(result.logic_verified);
}

} // namespace root_verifier
} // namespace proofs
} // namespace rollup