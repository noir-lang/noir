#include "c_bind.h"
#include "index.hpp"
#include "init.hpp"

#include "aztec3/circuits/abis/append_only_tree_snapshot.hpp"
#include "aztec3/circuits/abis/combined_accumulated_data.hpp"
#include "aztec3/circuits/abis/global_variables.hpp"
#include "aztec3/circuits/abis/kernel_circuit_public_inputs.hpp"
#include "aztec3/circuits/abis/new_contract_data.hpp"
#include "aztec3/circuits/abis/previous_kernel_data.hpp"
#include "aztec3/circuits/abis/rollup/merge/previous_rollup_data.hpp"
#include "aztec3/circuits/hash.hpp"
#include "aztec3/circuits/kernel/private/utils.hpp"
#include "aztec3/circuits/rollup/base/init.hpp"
#include "aztec3/circuits/rollup/components/components.hpp"
#include "aztec3/circuits/rollup/test_utils/init.hpp"
#include "aztec3/circuits/rollup/test_utils/utils.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/dummy_circuit_builder.hpp"

#include <barretenberg/barretenberg.hpp>

#include <gtest/gtest.h>

#include <cstdint>
#include <iostream>
#include <memory>
#include <vector>

namespace {


using aztec3::circuits::abis::PreviousKernelData;


// using aztec3::circuits::mock::mock_circuit;
using aztec3::circuits::rollup::test_utils::utils::compare_field_hash_to_expected;
using aztec3::circuits::rollup::test_utils::utils::get_empty_kernel;
using aztec3::circuits::rollup::test_utils::utils::get_empty_l1_to_l2_messages;
using aztec3::circuits::rollup::test_utils::utils::get_initial_nullifier_tree_empty;
using aztec3::circuits::rollup::test_utils::utils::get_root_rollup_inputs;

using aztec3::circuits::abis::AppendOnlyTreeSnapshot;

using aztec3::circuits::rollup::native_base_rollup::BaseOrMergeRollupPublicInputs;
using aztec3::circuits::rollup::native_base_rollup::BaseRollupInputs;
using aztec3::circuits::rollup::native_base_rollup::ConstantRollupData;
using aztec3::circuits::rollup::native_base_rollup::NT;

using aztec3::circuits::rollup::native_root_rollup::RootRollupInputs;
using aztec3::circuits::rollup::native_root_rollup::RootRollupPublicInputs;

using aztec3::circuits::abis::NewContractData;

using MemoryStore = stdlib::merkle_tree::MemoryStore;
using MerkleTree = stdlib::merkle_tree::MerkleTree<MemoryStore>;

using KernelData = aztec3::circuits::abis::PreviousKernelData<NT>;
}  // namespace

namespace aztec3::circuits::rollup::root::native_root_rollup_circuit {

class root_rollup_tests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { barretenberg::srs::init_crs_factory("../barretenberg/cpp/srs_db/ignition"); }

    // TODO(1998): uncomment once https://github.com/AztecProtocol/aztec-packages/issues/1998 is solved and
    //             use new pattern such as call_func_and_wrapper from test_helper.hpp

    // static void run_cbind(RootRollupInputs& root_rollup_inputs,
    //                       RootRollupPublicInputs& expected_public_inputs,
    //                       bool compare_pubins = true)
    // {
    //     info("Retesting via cbinds....");
    //     // info("Verification key size: ", vk_size);

    //     std::vector<uint8_t> root_rollup_inputs_vec;
    //     serialize::write(root_rollup_inputs_vec, root_rollup_inputs);

    //     // uint8_t const* proof_data;
    //     // size_t proof_data_size;
    //     uint8_t const* public_inputs_buf = nullptr;
    //     size_t public_inputs_size = 0;
    //     // info("simulating circuit via cbind");
    //     uint8_t* const circuit_failure_ptr =
    //         root_rollup__sim(root_rollup_inputs_vec.data(), &public_inputs_size, &public_inputs_buf);
    //     ASSERT_TRUE(circuit_failure_ptr == nullptr);
    //     // info("Proof size: ", proof_data_size);
    //     // info("PublicInputs size: ", public_inputs_size);

    //     if (compare_pubins) {
    //         RootRollupPublicInputs public_inputs;
    //         uint8_t const* public_inputs_buf_tmp = public_inputs_buf;
    //         serialize::read(public_inputs_buf_tmp, public_inputs);
    //         ASSERT_EQ(public_inputs.calldata_hash.size(), expected_public_inputs.calldata_hash.size());
    //         for (size_t i = 0; i < public_inputs.calldata_hash.size(); i++) {
    //             ASSERT_EQ(public_inputs.calldata_hash[i], expected_public_inputs.calldata_hash[i]);
    //         }

    //         std::vector<uint8_t> expected_public_inputs_vec;
    //         serialize::write(expected_public_inputs_vec, expected_public_inputs);

    //         ASSERT_EQ(public_inputs_size, expected_public_inputs_vec.size());
    //         // Just compare the first 10 bytes of the serialized public outputs
    //         if (public_inputs_size > 10) {
    //             // for (size_t 0; i < public_inputs_size; i++) {
    //             for (size_t i = 0; i < 10; i++) {
    //                 ASSERT_EQ(public_inputs_buf[i], expected_public_inputs_vec[i]);
    //             }
    //         }
    //     }

    //     // free((void*)proof_data);
    //     free((void*)public_inputs_buf);
    // }
};

TEST_F(root_rollup_tests, native_check_block_hashes_empty_blocks)
{
    std::vector<uint8_t> const zero_bytes_vec = test_utils::utils::get_empty_calldata_leaf();
    auto call_data_hash_inner = sha256::sha256(zero_bytes_vec);

    // Compute a new calldata hash based on TWO of the above rollups
    std::array<uint8_t, 64> hash_input;
    for (uint8_t i = 0; i < 32; ++i) {
        hash_input[i] = call_data_hash_inner[i];
        hash_input[32 + i] = call_data_hash_inner[i];
    }
    std::vector<uint8_t> const calldata_hash_input_bytes_vec(hash_input.begin(), hash_input.end());
    auto calldata_hash = sha256::sha256(calldata_hash_input_bytes_vec);

    // get messages
    std::array<fr, NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP> const l1_to_l2_messages = get_empty_l1_to_l2_messages();

    // hash messages
    std::vector<uint8_t> const messages_hash_input_bytes_vec(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP * 32, 0);
    auto messages_hash = sha256::sha256(messages_hash_input_bytes_vec);

    utils::DummyCircuitBuilder builder =
        utils::DummyCircuitBuilder("root_rollup_tests__native_check_block_hashes_empty_blocks");
    std::array<KernelData, 4> const kernels = {
        get_empty_kernel(), get_empty_kernel(), get_empty_kernel(), get_empty_kernel()
    };

    RootRollupInputs inputs = get_root_rollup_inputs(builder, kernels, l1_to_l2_messages);
    RootRollupPublicInputs outputs = aztec3::circuits::rollup::native_root_rollup::root_rollup_circuit(builder, inputs);

    // check calldata hash
    ASSERT_TRUE(compare_field_hash_to_expected(outputs.calldata_hash, calldata_hash));
    // Check messages hash
    ASSERT_TRUE(compare_field_hash_to_expected(outputs.l1_to_l2_messages_hash, messages_hash));

    EXPECT_FALSE(builder.failed());

    // TODO(1998): see above
    // run_cbind(inputs, outputs, true);
}

TEST_F(root_rollup_tests, native_root_missing_nullifier_logic)
{
    utils::DummyCircuitBuilder builder =
        utils::DummyCircuitBuilder("root_rollup_tests__native_root_missing_nullifier_logic");

    MemoryStore private_data_tree_store;
    MerkleTree private_data_tree(private_data_tree_store, PRIVATE_DATA_TREE_HEIGHT);

    MemoryStore contract_tree_store;
    MerkleTree contract_tree(contract_tree_store, CONTRACT_TREE_HEIGHT);

    MemoryStore l1_to_l2_messages_tree_store;
    MerkleTree l1_to_l2_messages_tree(l1_to_l2_messages_tree_store, L1_TO_L2_MSG_TREE_HEIGHT);

    MemoryStore public_store;
    MerkleTree public_data_tree(public_store, PUBLIC_DATA_TREE_HEIGHT);

    // Create initial nullifier tree with 32 initial nullifiers
    auto nullifier_tree = get_initial_nullifier_tree_empty();

    MemoryStore blocks_tree_store;
    MerkleTree blocks_tree(blocks_tree_store, HISTORIC_BLOCKS_TREE_HEIGHT);

    std::array<KernelData, 4> kernels = {
        get_empty_kernel(), get_empty_kernel(), get_empty_kernel(), get_empty_kernel()
    };
    std::array<fr, NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP> l1_to_l2_messages = get_empty_l1_to_l2_messages();

    // Calculate the start block hash
    abis::GlobalVariables<NT> globals = abis::GlobalVariables<NT>::empty();
    auto start_block_hash = compute_block_hash_with_globals(globals,
                                                            private_data_tree.root(),
                                                            nullifier_tree.root(),
                                                            contract_tree.root(),
                                                            l1_to_l2_messages_tree.root(),
                                                            public_data_tree.root());
    blocks_tree.update_element(0, start_block_hash);
    AppendOnlyTreeSnapshot<NT> start_blocks_tree_snapshot = { .root = blocks_tree.root(),
                                                              .next_available_leaf_index = 1 };

    // Create commitments
    for (size_t kernel_j = 0; kernel_j < 4; kernel_j++) {
        std::array<fr, MAX_NEW_COMMITMENTS_PER_TX> new_commitments;
        for (uint8_t commitment_k = 0; commitment_k < MAX_NEW_COMMITMENTS_PER_TX; commitment_k++) {
            auto val = fr(kernel_j * MAX_NEW_COMMITMENTS_PER_TX + commitment_k + 1);
            new_commitments[commitment_k] = val;
            private_data_tree.update_element(kernel_j * MAX_NEW_COMMITMENTS_PER_TX + commitment_k, val);
        }
        kernels[kernel_j].public_inputs.end.new_commitments = new_commitments;

        std::array<fr, MAX_NEW_L2_TO_L1_MSGS_PER_TX> new_l2_to_l1_messages;
        for (uint8_t i = 0; i < MAX_NEW_L2_TO_L1_MSGS_PER_TX; i++) {
            auto val = fr(kernel_j * MAX_NEW_L2_TO_L1_MSGS_PER_TX + i + 1);
            new_l2_to_l1_messages[i] = val;
        }
        kernels[kernel_j].public_inputs.end.new_l2_to_l1_msgs = new_l2_to_l1_messages;
    }

    // @todo @LHerskind: Add nullifiers
    // @todo @LHerskind: Add public data writes

    // Contract tree
    NewContractData<NT> const new_contract = {
        .contract_address = fr(1),
        .portal_contract_address = fr(3),
        .function_tree_root = fr(2),
    };
    // Update contract tree
    contract_tree.update_element(2, new_contract.hash());
    kernels[2].public_inputs.end.new_contracts[0] = new_contract;

    // l1 to l2 messages snapshot
    AppendOnlyTreeSnapshot<NT> const start_l1_to_l2_messages_tree_snapshot = { .root = l1_to_l2_messages_tree.root(),
                                                                               .next_available_leaf_index = 0 };

    // Create 16 empty l1 to l2 messages, and update the l1_to_l2 messages tree
    for (size_t i = 0; i < l1_to_l2_messages.size(); i++) {
        l1_to_l2_messages_tree.update_element(i, l1_to_l2_messages[i]);
    }

    // Get the block hash after.
    auto end_block_hash = compute_block_hash_with_globals(globals,
                                                          private_data_tree.root(),
                                                          nullifier_tree.root(),
                                                          contract_tree.root(),
                                                          l1_to_l2_messages_tree.root(),
                                                          public_data_tree.root());
    blocks_tree.update_element(1, end_block_hash);
    AppendOnlyTreeSnapshot<NT> end_blocks_tree_snapshot = { .root = blocks_tree.root(),
                                                            .next_available_leaf_index = 2 };

    // Compute the end snapshot
    AppendOnlyTreeSnapshot<NT> const end_l1_to_l2_messages_tree_snapshot = { .root = l1_to_l2_messages_tree.root(),
                                                                             .next_available_leaf_index =
                                                                                 NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP };

    RootRollupInputs rootRollupInputs = get_root_rollup_inputs(builder, kernels, l1_to_l2_messages);
    RootRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_root_rollup::root_rollup_circuit(builder, rootRollupInputs);

    // Check private data trees
    ASSERT_EQ(
        outputs.start_private_data_tree_snapshot,
        rootRollupInputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.start_private_data_tree_snapshot);
    ASSERT_EQ(
        outputs.end_private_data_tree_snapshot,
        rootRollupInputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.end_private_data_tree_snapshot);
    AppendOnlyTreeSnapshot<NT> const expected_private_data_tree_snapshot = { .root = private_data_tree.root(),
                                                                             .next_available_leaf_index =
                                                                                 4 * MAX_NEW_COMMITMENTS_PER_TX };
    ASSERT_EQ(outputs.end_private_data_tree_snapshot, expected_private_data_tree_snapshot);

    // Check public data trees
    ASSERT_EQ(outputs.start_public_data_tree_root,
              rootRollupInputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.start_public_data_tree_root);
    ASSERT_EQ(outputs.end_public_data_tree_root,
              rootRollupInputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.end_public_data_tree_root);

    // check contract trees
    ASSERT_EQ(outputs.start_contract_tree_snapshot,
              rootRollupInputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.start_contract_tree_snapshot);
    ASSERT_EQ(outputs.end_contract_tree_snapshot,
              rootRollupInputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.end_contract_tree_snapshot);
    AppendOnlyTreeSnapshot<NT> const expected_contract_tree_snapshot{ .root = contract_tree.root(),
                                                                      .next_available_leaf_index = 4 };
    ASSERT_EQ(outputs.end_contract_tree_snapshot, expected_contract_tree_snapshot);

    // @todo @LHerskind: Check nullifier trees

    // Check l1 to l2 messages trees
    ASSERT_EQ(outputs.start_l1_to_l2_messages_tree_snapshot, start_l1_to_l2_messages_tree_snapshot);
    ASSERT_EQ(outputs.start_contract_tree_snapshot,
              rootRollupInputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.start_contract_tree_snapshot);
    ASSERT_EQ(outputs.end_contract_tree_snapshot,
              rootRollupInputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.end_contract_tree_snapshot);
    ASSERT_EQ(outputs.end_l1_to_l2_messages_tree_snapshot, end_l1_to_l2_messages_tree_snapshot);

    ASSERT_EQ(outputs.start_historic_blocks_tree_snapshot, start_blocks_tree_snapshot);
    ASSERT_EQ(outputs.end_historic_blocks_tree_snapshot, end_blocks_tree_snapshot);

    // Compute the expected calldata hash for the root rollup (including the l2 -> l1 messages)
    auto left = components::compute_kernels_calldata_hash({ kernels[0], kernels[1] });
    auto right = components::compute_kernels_calldata_hash({ kernels[2], kernels[3] });
    auto root = accumulate_sha256<NT>({ left[0], left[1], right[0], right[1] });
    ASSERT_EQ(outputs.calldata_hash, root);

    EXPECT_FALSE(builder.failed());

    // TODO(1998): see above
    // run_cbind(rootRollupInputs, outputs, true);
}

}  // namespace aztec3::circuits::rollup::root::native_root_rollup_circuit