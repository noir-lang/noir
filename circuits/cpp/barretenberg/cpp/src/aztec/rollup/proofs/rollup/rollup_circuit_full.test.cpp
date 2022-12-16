#include "index.hpp"
#include "../notes/native/index.hpp"
#include "../../fixtures/test_context.hpp"
#include <common/test.hpp>

namespace rollup {
namespace proofs {
namespace rollup {

using namespace barretenberg;
using namespace notes::native::value;
using namespace notes::native::account;
using namespace notes::native::value;
using namespace plonk::stdlib::merkle_tree;

namespace {
#ifdef CI
constexpr bool CIRCUIT_CHANGE_EXPECTED = false;
#else
// During development, if the circuit vk hash/gate count is expected to change, set the following to true.
constexpr bool CIRCUIT_CHANGE_EXPECTED = false;
#endif

std::shared_ptr<waffle::DynamicFileReferenceStringFactory> srs;
join_split::circuit_data js_cd;
account::circuit_data account_cd;
claim::circuit_data claim_cd;
} // namespace

class rollup_full_tests : public ::testing::Test {
  protected:
    rollup_full_tests()
        : context(js_cd, account_cd, claim_cd)
    {}

    static void SetUpTestCase()
    {
        std::string CRS_PATH = "../srs_db/ignition";
        srs = std::make_shared<waffle::DynamicFileReferenceStringFactory>(CRS_PATH);
        account_cd = account::get_circuit_data(srs);
        js_cd = join_split::get_circuit_data(srs);
        claim_cd = claim::get_circuit_data(srs);
    }

    fixtures::TestContext context;
    const uint32_t asset_id = 0;
    const uint32_t tx_fee = 7;
};

// Full proofs.
HEAVY_TEST_F(rollup_full_tests, test_1_proof_in_1_rollup_full_proof_and_detect_circuit_change)
{
    size_t rollup_size = 1;

    context.append_account_notes();
    context.append_value_notes({ 100, 50 });
    context.start_next_root_rollup();

    auto join_split_proof = context.create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 110 - tx_fee }, 30, 0);
    auto rollup = create_rollup_tx(context.world_state, rollup_size, { join_split_proof });

    auto rollup_circuit_data =
        rollup::get_circuit_data(rollup_size, js_cd, account_cd, claim_cd, srs, "", true, false, false);
    auto result = verify(rollup, rollup_circuit_data);

    ASSERT_TRUE(result.verified);

    auto rollup_data = rollup_proof_data(result.proof_data);
    EXPECT_EQ(rollup_data.rollup_id, 1UL);
    EXPECT_EQ(rollup_data.rollup_size, rollup_size);
    EXPECT_EQ(rollup_data.data_start_index, 4UL);
    EXPECT_EQ(rollup_data.old_data_root, rollup.old_data_root);
    EXPECT_EQ(rollup_data.new_data_root, rollup.new_data_root);
    EXPECT_EQ(rollup_data.old_null_root, rollup.old_null_root);
    EXPECT_EQ(rollup_data.new_null_root, rollup.new_null_roots.back());
    EXPECT_EQ(rollup_data.old_data_roots_root, rollup.data_roots_root);
    EXPECT_EQ(rollup_data.new_data_roots_root, rollup.data_roots_root);
    for (size_t i = 0; i < rollup_data.total_tx_fees.size(); ++i) {
        EXPECT_EQ(rollup_data.total_tx_fees[i], i == asset_id ? tx_fee : 0UL);
    }
    EXPECT_EQ(rollup_data.inner_proofs.size(), 1UL);

    auto tx_data = inner_proof_data(join_split_proof);
    auto inner_data = rollup_data.inner_proofs[0];
    EXPECT_EQ(inner_data.proof_id, tx_data.proof_id);
    EXPECT_EQ(inner_data.note_commitment1, tx_data.note_commitment1);
    EXPECT_EQ(inner_data.note_commitment2, tx_data.note_commitment2);
    EXPECT_EQ(inner_data.nullifier1, tx_data.nullifier1);
    EXPECT_EQ(inner_data.nullifier2, tx_data.nullifier2);
    EXPECT_EQ(inner_data.public_value, tx_data.public_value);
    EXPECT_EQ(inner_data.public_owner, tx_data.public_owner);
    EXPECT_EQ(inner_data.asset_id, tx_data.asset_id);

    // The below part detects the changes in the rollup circuit
    constexpr uint32_t CIRCUIT_GATE_COUNT = 1153136;
    constexpr uint32_t GATES_NEXT_POWER_OF_TWO = 2097152;
    const uint256_t VK_HASH("b6481781e449ba7c4a3bff935cc08421ab9b88527d0a70fa454dd9288dba8c46");

    auto number_of_gates_rollup = rollup_circuit_data.num_gates;
    auto vk_hash_rollup = rollup_circuit_data.verification_key->sha256_hash();

    if (!CIRCUIT_CHANGE_EXPECTED) {
        EXPECT_EQ(number_of_gates_rollup, CIRCUIT_GATE_COUNT) << "The gate count for the rollup circuit is changed.";
        EXPECT_EQ(from_buffer<uint256_t>(vk_hash_rollup), VK_HASH)
            << "The verification key hash for the rollup circuit is changed.";
    }
    // For the next power of two limit, we need to consider that we reserve four gates for adding
    // randomness/zero-knowledge
    EXPECT_LE(number_of_gates_rollup, GATES_NEXT_POWER_OF_TWO - waffle::ComposerBase::NUM_RESERVED_GATES)
        << "You have exceeded the next power of two limit for the rollup circuit.";
}

HEAVY_TEST_F(rollup_full_tests, test_1_proof_in_2_rollup_full_proof)
{
    size_t rollup_size = 2;

    context.append_account_notes();
    context.append_value_notes({ 100, 50 });
    context.start_next_root_rollup();

    auto join_split_proof = context.create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 - tx_fee });
    auto rollup = create_rollup_tx(context.world_state, rollup_size, { join_split_proof });

    auto rollup_circuit_data =
        rollup::get_circuit_data(rollup_size, js_cd, account_cd, claim_cd, srs, "", true, false, false);
    auto result = verify(rollup, rollup_circuit_data);

    ASSERT_TRUE(result.verified);

    auto rollup_data = rollup_proof_data(result.proof_data);
    EXPECT_EQ(rollup_data.rollup_id, 1UL);
    EXPECT_EQ(rollup_data.rollup_size, rollup_size);
    EXPECT_EQ(rollup_data.data_start_index, 4UL);
    EXPECT_EQ(rollup_data.old_data_root, rollup.old_data_root);
    EXPECT_EQ(rollup_data.new_data_root, rollup.new_data_root);
    EXPECT_EQ(rollup_data.old_null_root, rollup.old_null_root);
    EXPECT_EQ(rollup_data.new_null_root, rollup.new_null_roots.back());
    EXPECT_EQ(rollup_data.old_data_roots_root, rollup.data_roots_root);
    EXPECT_EQ(rollup_data.new_data_roots_root, rollup.data_roots_root);
    for (size_t i = 0; i < rollup_data.total_tx_fees.size(); ++i) {
        EXPECT_EQ(rollup_data.total_tx_fees[i], i == asset_id ? tx_fee : 0UL);
    }
    EXPECT_EQ(rollup_data.inner_proofs.size(), 2UL);

    auto tx_data = inner_proof_data(join_split_proof);
    auto inner_data = rollup_data.inner_proofs[0];
    EXPECT_EQ(inner_data.proof_id, tx_data.proof_id);
    EXPECT_EQ(inner_data.note_commitment1, tx_data.note_commitment1);
    EXPECT_EQ(inner_data.note_commitment2, tx_data.note_commitment2);
    EXPECT_EQ(inner_data.nullifier1, tx_data.nullifier1);
    EXPECT_EQ(inner_data.nullifier2, tx_data.nullifier2);
    EXPECT_EQ(inner_data.public_value, tx_data.public_value);
    EXPECT_EQ(inner_data.public_owner, tx_data.public_owner);
    EXPECT_EQ(inner_data.asset_id, tx_data.asset_id);
}

HEAVY_TEST_F(rollup_full_tests, test_1_js_proof_1_account_proof_in_2_rollup_full_proof)
{
    size_t rollup_size = 2;

    context.append_account_notes();
    context.append_value_notes({ 0, 0, 100, 50, 80, 60 });
    context.start_next_root_rollup();

    auto join_split_proof = context.create_join_split_proof({ 4, 5 }, { 100, 50 }, { 70, 110 - tx_fee }, 30);
    auto account_proof = context.create_add_signing_keys_to_account_proof();
    auto txs = std::vector<std::vector<uint8_t>>{ join_split_proof, account_proof };
    auto rollup = create_rollup_tx(context.world_state, rollup_size, txs);
    auto rollup_circuit_data =
        rollup::get_circuit_data(rollup_size, js_cd, account_cd, claim_cd, srs, "", true, false, false);
    auto result = verify(rollup, rollup_circuit_data);

    ASSERT_TRUE(result.verified);

    auto rollup_data = rollup_proof_data(result.proof_data);
    EXPECT_EQ(rollup_data.rollup_id, 1UL);
    EXPECT_EQ(rollup_data.rollup_size, rollup_size);
    EXPECT_EQ(rollup_data.data_start_index, 8UL);
    EXPECT_EQ(rollup_data.old_data_root, rollup.old_data_root);
    EXPECT_EQ(rollup_data.new_data_root, rollup.new_data_root);
    EXPECT_EQ(rollup_data.old_null_root, rollup.old_null_root);
    EXPECT_EQ(rollup_data.new_null_root, rollup.new_null_roots.back());
    EXPECT_EQ(rollup_data.old_data_roots_root, rollup.data_roots_root);
    EXPECT_EQ(rollup_data.new_data_roots_root, rollup.data_roots_root);
    for (size_t i = 0; i < rollup_data.total_tx_fees.size(); ++i) {
        EXPECT_EQ(rollup_data.total_tx_fees[i], i == asset_id ? tx_fee : 0UL);
    }
    EXPECT_EQ(rollup_data.inner_proofs.size(), txs.size());

    for (size_t i = 0; i < txs.size(); ++i) {
        auto tx_data = inner_proof_data(txs[i]);
        auto inner_data = rollup_data.inner_proofs[i];
        EXPECT_EQ(inner_data.proof_id, tx_data.proof_id);
        EXPECT_EQ(inner_data.note_commitment1, tx_data.note_commitment1);
        EXPECT_EQ(inner_data.note_commitment2, tx_data.note_commitment2);
        EXPECT_EQ(inner_data.nullifier1, tx_data.nullifier1);
        EXPECT_EQ(inner_data.nullifier2, tx_data.nullifier2);
        EXPECT_EQ(inner_data.public_value, tx_data.public_value);
        EXPECT_EQ(inner_data.public_owner, tx_data.public_owner);
        EXPECT_EQ(inner_data.asset_id, tx_data.asset_id);
    }
}

HEAVY_TEST_F(rollup_full_tests, test_3_rollup_pads_to_4)
{
    size_t rollup_size = 3;

    context.append_account_notes();
    context.append_value_notes({ 100, 50 });
    context.start_next_root_rollup();

    auto join_split_proof = context.create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 - tx_fee });
    auto rollup = create_rollup_tx(context.world_state, rollup_size, { join_split_proof });

    auto rollup_circuit_data =
        rollup::get_circuit_data(rollup_size, js_cd, account_cd, claim_cd, srs, "", true, false, false);
    auto result = verify(rollup, rollup_circuit_data);

    ASSERT_TRUE(result.verified);

    auto rollup_data = rollup_proof_data(result.proof_data);
    EXPECT_EQ(rollup_data.rollup_id, 1UL);
    EXPECT_EQ(rollup_data.rollup_size, 4UL);
    EXPECT_EQ(rollup_data.data_start_index, 8UL);
    EXPECT_EQ(rollup_data.old_data_root, rollup.old_data_root);
    EXPECT_EQ(rollup_data.new_data_root, rollup.new_data_root);
    EXPECT_EQ(rollup_data.old_null_root, rollup.old_null_root);
    EXPECT_EQ(rollup_data.new_null_root, rollup.new_null_roots.back());
    EXPECT_EQ(rollup_data.old_data_roots_root, rollup.data_roots_root);
    EXPECT_EQ(rollup_data.new_data_roots_root, rollup.data_roots_root);
    for (size_t i = 0; i < rollup_data.total_tx_fees.size(); ++i) {
        EXPECT_EQ(rollup_data.total_tx_fees[i], i == asset_id ? tx_fee : 0UL);
    }
    EXPECT_EQ(rollup_data.inner_proofs.size(), 4UL);

    auto tx_data = inner_proof_data(join_split_proof);

    {
        auto inner_data = rollup_data.inner_proofs[0];
        EXPECT_EQ(inner_data.note_commitment1, tx_data.note_commitment1);
        EXPECT_EQ(inner_data.note_commitment2, tx_data.note_commitment2);
        EXPECT_EQ(inner_data.nullifier1, tx_data.nullifier1);
        EXPECT_EQ(inner_data.nullifier2, tx_data.nullifier2);
        EXPECT_EQ(inner_data.public_value, tx_data.public_value);
        EXPECT_EQ(inner_data.public_owner, tx_data.public_owner);
        EXPECT_EQ(inner_data.asset_id, tx_data.asset_id);
    }

    for (size_t i = 1; i < rollup_data.inner_proofs.size(); ++i) {
        auto inner_data = rollup_data.inner_proofs[i];
        EXPECT_EQ(inner_data.note_commitment1, fr(0));
        EXPECT_EQ(inner_data.note_commitment2, fr(0));
        EXPECT_EQ(inner_data.nullifier1, uint256_t(0));
        EXPECT_EQ(inner_data.nullifier2, uint256_t(0));
        EXPECT_EQ(inner_data.public_value, uint256_t(0));
        EXPECT_EQ(inner_data.public_owner, fr(0));
        EXPECT_EQ(inner_data.asset_id, uint256_t(0));
    }
}

} // namespace rollup
} // namespace proofs
} // namespace rollup