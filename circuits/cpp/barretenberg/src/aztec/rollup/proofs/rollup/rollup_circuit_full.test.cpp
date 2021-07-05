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
std::shared_ptr<waffle::DynamicFileReferenceStringFactory> srs;
join_split::circuit_data js_cd;
account::circuit_data account_cd;
claim::circuit_data claim_cd;
} // namespace

class rollup_tests_full : public ::testing::Test {
  protected:
    rollup_tests_full()
        : context(js_cd, account_cd, claim_cd)
    {}

    static void SetUpTestCase()
    {
        std::string CRS_PATH = "../srs_db/ignition";
        srs = std::make_shared<waffle::DynamicFileReferenceStringFactory>(CRS_PATH);
        account_cd = account::compute_circuit_data(srs);
        js_cd = join_split::compute_circuit_data(srs);
        claim_cd = claim::get_circuit_data(srs, "", true, false, false);
    }

    fixtures::TestContext context;
    const uint32_t asset_id = 0;
    const uint256_t tx_fee = 7;
};

// Full proofs.
HEAVY_TEST_F(rollup_tests_full, test_1_proof_in_1_rollup_full_proof)
{
    size_t rollup_size = 1;

    context.append_account_notes();
    context.append_value_notes({ 100, 50 });
    context.start_next_root_rollup();

    auto join_split_proof = context.create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 50 }, 30, 60);
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
    EXPECT_EQ(inner_data.public_input, tx_data.public_input);
    EXPECT_EQ(inner_data.public_output, tx_data.public_output);
    EXPECT_EQ(inner_data.asset_id, tx_data.asset_id);
    EXPECT_EQ(inner_data.new_note1, tx_data.new_note1);
    EXPECT_EQ(inner_data.new_note2, tx_data.new_note2);
    EXPECT_EQ(inner_data.nullifier1, tx_data.nullifier1);
    EXPECT_EQ(inner_data.nullifier2, tx_data.nullifier2);
    EXPECT_EQ(inner_data.input_owner, tx_data.input_owner);
    EXPECT_EQ(inner_data.output_owner, tx_data.output_owner);
}

HEAVY_TEST_F(rollup_tests_full, test_1_proof_in_2_rollup_full_proof)
{
    size_t rollup_size = 2;

    context.append_account_notes();
    context.append_value_notes({ 100, 50 });
    context.start_next_root_rollup();

    auto join_split_proof = context.create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 });
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
    EXPECT_EQ(inner_data.public_input, tx_data.public_input);
    EXPECT_EQ(inner_data.public_output, tx_data.public_output);
    EXPECT_EQ(inner_data.asset_id, tx_data.asset_id);
    EXPECT_EQ(inner_data.new_note1, tx_data.new_note1);
    EXPECT_EQ(inner_data.new_note2, tx_data.new_note2);
    EXPECT_EQ(inner_data.nullifier1, tx_data.nullifier1);
    EXPECT_EQ(inner_data.nullifier2, tx_data.nullifier2);
    EXPECT_EQ(inner_data.input_owner, tx_data.input_owner);
    EXPECT_EQ(inner_data.output_owner, tx_data.output_owner);
}

HEAVY_TEST_F(rollup_tests_full, test_2_proofs_in_2_rollup_full_proof)
{
    size_t rollup_size = 2;

    context.append_account_notes();
    context.append_value_notes({ 0, 0, 100, 50, 80, 60 });
    context.start_next_root_rollup();

    auto join_split_proof1 = context.create_join_split_proof({ 4, 5 }, { 100, 50 }, { 70, 50 }, 30, 60);
    auto join_split_proof2 = context.create_join_split_proof({ 6, 7 }, { 80, 60 }, { 70, 70 });
    auto txs = std::vector<std::vector<uint8_t>>{ join_split_proof1, join_split_proof2 };

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
        EXPECT_EQ(rollup_data.total_tx_fees[i], i == asset_id ? tx_fee * 2 : 0UL);
    }
    EXPECT_EQ(rollup_data.inner_proofs.size(), txs.size());

    for (size_t i = 0; i < txs.size(); ++i) {
        auto tx_data = inner_proof_data(txs[i]);
        auto inner_data = rollup_data.inner_proofs[i];
        EXPECT_EQ(inner_data.proof_id, tx_data.proof_id);
        EXPECT_EQ(inner_data.public_input, tx_data.public_input);
        EXPECT_EQ(inner_data.public_output, tx_data.public_output);
        EXPECT_EQ(inner_data.asset_id, tx_data.asset_id);
        EXPECT_EQ(inner_data.new_note1, tx_data.new_note1);
        EXPECT_EQ(inner_data.new_note2, tx_data.new_note2);
        EXPECT_EQ(inner_data.nullifier1, tx_data.nullifier1);
        EXPECT_EQ(inner_data.nullifier2, tx_data.nullifier2);
        EXPECT_EQ(inner_data.input_owner, tx_data.input_owner);
        EXPECT_EQ(inner_data.output_owner, tx_data.output_owner);
    }
}

HEAVY_TEST_F(rollup_tests_full, test_1_js_proof_1_account_proof_in_2_rollup_full_proof)
{
    size_t rollup_size = 2;

    context.append_account_notes();
    context.append_value_notes({ 0, 0, 100, 50, 80, 60 });
    context.start_next_root_rollup();

    auto join_split_proof = context.create_join_split_proof({ 4, 5 }, { 100, 50 }, { 70, 50 }, 30, 60);
    auto account_proof = context.create_account_proof();
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
        EXPECT_EQ(inner_data.public_input, tx_data.public_input);
        EXPECT_EQ(inner_data.public_output, tx_data.public_output);
        EXPECT_EQ(inner_data.asset_id, tx_data.asset_id);
        EXPECT_EQ(inner_data.new_note1, tx_data.new_note1);
        EXPECT_EQ(inner_data.new_note2, tx_data.new_note2);
        EXPECT_EQ(inner_data.nullifier1, tx_data.nullifier1);
        EXPECT_EQ(inner_data.nullifier2, tx_data.nullifier2);
        EXPECT_EQ(inner_data.input_owner, tx_data.input_owner);
        EXPECT_EQ(inner_data.output_owner, tx_data.output_owner);
    }
}

HEAVY_TEST_F(rollup_tests_full, test_1_proof_in_3_of_4_rollup_full_proof)
{
    size_t rollup_size = 3;

    context.append_account_notes();
    context.append_value_notes({ 100, 50 });
    context.start_next_root_rollup();

    auto join_split_proof = context.create_join_split_proof({ 2, 3 }, { 100, 50 }, { 70, 80 });
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
        EXPECT_EQ(inner_data.public_input, tx_data.public_input);
        EXPECT_EQ(inner_data.public_output, tx_data.public_output);
        EXPECT_EQ(inner_data.new_note1, tx_data.new_note1);
        EXPECT_EQ(inner_data.new_note2, tx_data.new_note2);
        EXPECT_EQ(inner_data.nullifier1, tx_data.nullifier1);
        EXPECT_EQ(inner_data.nullifier2, tx_data.nullifier2);
        EXPECT_EQ(inner_data.input_owner, tx_data.input_owner);
        EXPECT_EQ(inner_data.output_owner, tx_data.output_owner);
    }

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

} // namespace rollup
} // namespace proofs
} // namespace rollup