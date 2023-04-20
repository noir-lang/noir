#include "aztec3/circuits/abis/append_only_tree_snapshot.hpp"
#include "aztec3/circuits/abis/membership_witness.hpp"
#include "aztec3/circuits/abis/new_contract_data.hpp"
#include "aztec3/circuits/abis/previous_kernel_data.hpp"
#include "aztec3/circuits/abis/rollup/merge/previous_rollup_data.hpp"
#include "aztec3/circuits/abis/rollup/nullifier_leaf_preimage.hpp"
#include "aztec3/circuits/rollup/base/init.hpp"
#include "aztec3/circuits/rollup/base/utils.hpp"
#include "aztec3/circuits/kernel/private/utils.hpp"
#include "aztec3/circuits/rollup/merge/utils.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/dummy_composer.hpp"
#include "barretenberg/crypto/sha256/sha256.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/stdlib/merkle_tree/memory_tree.hpp"
#include "index.hpp"
#include "init.hpp"
#include "c_bind.h"

#include <aztec3/circuits/apps/test_apps/escrow/deposit.hpp>
#include <aztec3/circuits/apps/test_apps/basic_contract_deployment/basic_contract_deployment.hpp>

#include <aztec3/circuits/abis/call_context.hpp>
#include <aztec3/circuits/abis/call_stack_item.hpp>
#include <aztec3/circuits/abis/contract_deployment_data.hpp>
#include <aztec3/circuits/abis/function_data.hpp>
#include <aztec3/circuits/abis/signed_tx_request.hpp>
#include <aztec3/circuits/abis/tx_context.hpp>
#include <aztec3/circuits/abis/tx_request.hpp>
#include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>
#include <aztec3/circuits/abis/private_kernel/private_inputs.hpp>
#include <aztec3/circuits/abis/kernel_circuit_public_inputs.hpp>
#include <aztec3/circuits/abis/combined_accumulated_data.hpp>
#include <aztec3/circuits/abis/combined_constant_data.hpp>
#include <aztec3/circuits/abis/private_historic_tree_roots.hpp>
#include <aztec3/circuits/abis/combined_historic_tree_roots.hpp>
#include <aztec3/circuits/abis/private_kernel/globals.hpp>

#include <aztec3/circuits/apps/function_execution_context.hpp>

#include <aztec3/circuits/mock/mock_kernel_circuit.hpp>

#include <barretenberg/common/map.hpp>
#include <barretenberg/common/test.hpp>
#include <cstdint>
#include <gtest/gtest.h>
#include <iostream>
#include <memory>
#include <vector>

namespace {

using aztec3::circuits::abis::CallContext;
using aztec3::circuits::abis::CallStackItem;
using aztec3::circuits::abis::ContractDeploymentData;
using aztec3::circuits::abis::FunctionData;
using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;
using aztec3::circuits::abis::PrivateCircuitPublicInputs;
using aztec3::circuits::abis::SignedTxRequest;
using aztec3::circuits::abis::TxContext;
using aztec3::circuits::abis::TxRequest;

using aztec3::circuits::abis::CombinedAccumulatedData;
using aztec3::circuits::abis::CombinedConstantData;
using aztec3::circuits::abis::CombinedHistoricTreeRoots;
using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::PreviousKernelData;
using aztec3::circuits::abis::PrivateHistoricTreeRoots;
using aztec3::circuits::abis::private_kernel::Globals;
using aztec3::circuits::abis::private_kernel::PrivateCallData;
using aztec3::circuits::abis::private_kernel::PrivateInputs;

using aztec3::circuits::apps::test_apps::basic_contract_deployment::constructor;
using aztec3::circuits::apps::test_apps::escrow::deposit;

// using aztec3::circuits::mock::mock_circuit;
using aztec3::circuits::kernel::private_kernel::utils::dummy_previous_kernel;
using aztec3::circuits::mock::mock_kernel_circuit;
using aztec3::circuits::rollup::base::utils::dummy_base_rollup_inputs;
using aztec3::circuits::rollup::merge::utils::previous_rollup_datas;
// using aztec3::circuits::mock::mock_kernel_inputs;

using aztec3::circuits::abis::AppendOnlyTreeSnapshot;

using aztec3::circuits::abis::MembershipWitness;
using aztec3::circuits::abis::NullifierLeafPreimage;
using aztec3::circuits::rollup::native_base_rollup::BaseOrMergeRollupPublicInputs;
using aztec3::circuits::rollup::native_base_rollup::BaseRollupInputs;
using aztec3::circuits::rollup::native_base_rollup::ConstantRollupData;
using aztec3::circuits::rollup::native_base_rollup::NT;

using aztec3::circuits::abis::PreviousRollupData;
using aztec3::circuits::rollup::native_root_rollup::RootRollupInputs;
using aztec3::circuits::rollup::native_root_rollup::RootRollupPublicInputs;

using aztec3::circuits::abis::FunctionData;
using aztec3::circuits::abis::NewContractData;
using aztec3::circuits::abis::OptionallyRevealedData;

using MemoryTree = proof_system::plonk::stdlib::merkle_tree::MemoryTree;

} // namespace

namespace aztec3::circuits::rollup::root::native_root_rollup_circuit {

class root_rollup_tests : public ::testing::Test {
  protected:
    void run_cbind(RootRollupInputs& root_rollup_inputs,
                   RootRollupPublicInputs& expected_public_inputs,
                   bool compare_pubins = true)
    {
        info("Retesting via cbinds....");
        // TODO might be able to get rid of proving key buffer
        uint8_t const* pk_buf;
        size_t pk_size = root_rollup__init_proving_key(&pk_buf);
        (void)pk_size;
        // info("Proving key size: ", pk_size);

        // TODO might be able to get rid of verification key buffer
        uint8_t const* vk_buf;
        size_t vk_size = root_rollup__init_verification_key(pk_buf, &vk_buf);
        (void)vk_size;
        // info("Verification key size: ", vk_size);

        std::vector<uint8_t> root_rollup_inputs_vec;
        write(root_rollup_inputs_vec, root_rollup_inputs);

        // uint8_t const* proof_data;
        // size_t proof_data_size;
        uint8_t const* public_inputs_buf;
        // info("simulating circuit via cbind");
        size_t public_inputs_size = root_rollup__sim(root_rollup_inputs_vec.data(), &public_inputs_buf);
        // info("Proof size: ", proof_data_size);
        // info("PublicInputs size: ", public_inputs_size);

        if (compare_pubins) {
            RootRollupPublicInputs public_inputs;
            uint8_t const* public_inputs_buf_tmp = public_inputs_buf;
            read(public_inputs_buf_tmp, public_inputs);
            ASSERT_EQ(public_inputs.calldata_hash.size(), expected_public_inputs.calldata_hash.size());
            for (size_t i = 0; i < public_inputs.calldata_hash.size(); i++) {
                ASSERT_EQ(public_inputs.calldata_hash[i], expected_public_inputs.calldata_hash[i]);
            }

            std::vector<uint8_t> expected_public_inputs_vec;
            write(expected_public_inputs_vec, expected_public_inputs);

            ASSERT_EQ(public_inputs_size, expected_public_inputs_vec.size());
            // Just compare the first 10 bytes of the serialized public outputs
            if (public_inputs_size > 10) {
                // for (size_t 0; i < public_inputs_size; i++) {
                for (size_t i = 0; i < 10; i++) {
                    ASSERT_EQ(public_inputs_buf[i], expected_public_inputs_vec[i]);
                }
            }
        }

        free((void*)pk_buf);
        free((void*)vk_buf);
        // free((void*)proof_data);
        free((void*)public_inputs_buf);
    }

  protected:
    template <size_t N>
    std::array<fr, N> get_subtree_sibling_path(MemoryTree tree,
                                               size_t const& leafIndex,
                                               size_t const& subtree_depth_to_skip)
    {
        std::array<fr, N> siblingPath;
        auto path = tree.get_sibling_path(leafIndex);
        for (size_t i = 0; i < N; i++) {
            siblingPath[i] = path[subtree_depth_to_skip + i];
        }
        return siblingPath;
    }

    RootRollupInputs getEmptyRootRollupInputs()
    {
        MemoryTree historic_data_tree = MemoryTree(PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT);
        MemoryTree historic_contract_tree = MemoryTree(CONTRACT_TREE_ROOTS_TREE_HEIGHT);

        auto historic_data_sibling_path =
            get_subtree_sibling_path<PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT>(historic_data_tree, 0, 0);
        auto historic_contract_sibling_path =
            get_subtree_sibling_path<CONTRACT_TREE_ROOTS_TREE_HEIGHT>(historic_contract_tree, 0, 0);

        RootRollupInputs rootRollupInputs = {
            .previous_rollup_data = previous_rollup_datas(),
            .new_historic_private_data_tree_root_sibling_path = historic_data_sibling_path,
            .new_historic_contract_tree_root_sibling_path = historic_contract_sibling_path,
        };
        return rootRollupInputs;
    }
};

TEST_F(root_rollup_tests, native_calldata_hash_empty_blocks)
{
    utils::DummyComposer composer = utils::DummyComposer();
    std::vector<uint8_t> zero_bytes_vec(704, 0);
    auto call_data_hash_inner = sha256::sha256(zero_bytes_vec);

    std::array<uint8_t, 64> hash_input;
    for (uint8_t i = 0; i < 32; ++i) {
        hash_input[i] = call_data_hash_inner[i];
        hash_input[32 + i] = call_data_hash_inner[i];
    }

    std::vector<uint8_t> calldata_hash_input_bytes_vec(hash_input.begin(), hash_input.end());

    auto hash = sha256::sha256(calldata_hash_input_bytes_vec);

    RootRollupInputs inputs = getEmptyRootRollupInputs();
    RootRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_root_rollup::root_rollup_circuit(composer, inputs);

    std::array<fr, 2> calldata_hash_fr = outputs.calldata_hash;
    auto high_buffer = calldata_hash_fr[0].to_buffer();
    auto low_buffer = calldata_hash_fr[1].to_buffer();

    std::array<uint8_t, 32> calldata_hash;
    for (uint8_t i = 0; i < 16; ++i) {
        calldata_hash[i] = high_buffer[16 + i];
        calldata_hash[16 + i] = low_buffer[16 + i];
    }

    ASSERT_EQ(hash, calldata_hash);

    run_cbind(inputs, outputs, true);
}

TEST_F(root_rollup_tests, native_root_missing_nullifier_logic)
{
    utils::DummyComposer composer = utils::DummyComposer();
    MemoryTree data_tree = MemoryTree(PRIVATE_DATA_TREE_HEIGHT);
    MemoryTree contract_tree = MemoryTree(CONTRACT_TREE_HEIGHT);

    // historic trees
    MemoryTree historic_data_tree = MemoryTree(PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT);
    MemoryTree historic_contract_tree = MemoryTree(CONTRACT_TREE_ROOTS_TREE_HEIGHT);

    std::array<BaseRollupInputs, 2> base_inputs = { dummy_base_rollup_inputs(), dummy_base_rollup_inputs() };
    // Insert commitments into base rollups
    for (uint8_t rollup_i = 0; rollup_i < 2; rollup_i++) {
        for (uint8_t kernel_j = 0; kernel_j < 2; kernel_j++) {
            for (uint8_t commitment_k = 0; commitment_k < 2; commitment_k++) {
                auto val = fr(rollup_i * 8 + kernel_j * 4 + commitment_k + 1);
                base_inputs[rollup_i].kernel_data[kernel_j].public_inputs.end.new_commitments[commitment_k] = val;
                data_tree.update_element(rollup_i * 8 + kernel_j * 4 + commitment_k, val);
            }
        }
        // Compute sibling path for inserting commitment subtree
        base_inputs[rollup_i].new_commitments_subtree_sibling_path =
            get_subtree_sibling_path<PRIVATE_DATA_SUBTREE_INCLUSION_CHECK_DEPTH>(
                data_tree, rollup_i * 8, PRIVATE_DATA_SUBTREE_DEPTH);

        if (rollup_i == 0) {
            // Update starting point for second rollup before
            base_inputs[1].start_private_data_tree_snapshot = {
                .root = data_tree.root(),
                .next_available_leaf_index = 8,
            };
        }
    }

    // TODO: Add nullifiers

    // Contract tree
    NewContractData<NT> new_contract = {
        .contract_address = fr(1),
        .portal_contract_address = fr(3),
        .function_tree_root = fr(2),
    };
    base_inputs[1].kernel_data[0].public_inputs.end.new_contracts[0] = new_contract;
    auto contract_leaf = crypto::pedersen_commitment::compress_native(
        {
            new_contract.contract_address,
            new_contract.portal_contract_address,
            new_contract.function_tree_root,
        },
        GeneratorIndex::CONTRACT_LEAF);

    // Update contract tree
    contract_tree.update_element(2, contract_leaf);
    base_inputs[1].new_contracts_subtree_sibling_path =
        get_subtree_sibling_path<CONTRACT_SUBTREE_INCLUSION_CHECK_DEPTH>(contract_tree, 2, CONTRACT_SUBTREE_DEPTH);

    // Historic trees
    auto historic_data_sibling_path =
        get_subtree_sibling_path<PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT>(historic_data_tree, 0, 0);
    auto historic_contract_sibling_path =
        get_subtree_sibling_path<CONTRACT_TREE_ROOTS_TREE_HEIGHT>(historic_contract_tree, 0, 0);

    // The start historic data snapshot
    AppendOnlyTreeSnapshot<NT> start_historic_data_tree_snapshot = { .root = historic_data_tree.root(),
                                                                     .next_available_leaf_index = 0 };
    AppendOnlyTreeSnapshot<NT> start_historic_contract_tree_snapshot = { .root = historic_contract_tree.root(),
                                                                         .next_available_leaf_index = 0 };

    // Insert the newest data root into the historic tree
    historic_data_tree.update_element(0, data_tree.root());
    historic_contract_tree.update_element(0, contract_tree.root());

    // Compute the end snapshot
    AppendOnlyTreeSnapshot<NT> end_historic_data_tree_snapshot = { .root = historic_data_tree.root(),
                                                                   .next_available_leaf_index = 1 };
    AppendOnlyTreeSnapshot<NT> end_historic_contract_tree_snapshot = { .root = historic_contract_tree.root(),
                                                                       .next_available_leaf_index = 1 };

    std::array<BaseOrMergeRollupPublicInputs, 2> base_outputs;
    base_outputs[0] = aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, base_inputs[0]);

    // Setup the start for the second base rollup
    base_inputs[1].start_private_data_tree_snapshot = base_outputs[0].end_private_data_tree_snapshot;
    base_inputs[1].start_nullifier_tree_snapshot = base_outputs[0].end_nullifier_tree_snapshot;
    base_inputs[1].start_contract_tree_snapshot = base_outputs[0].end_contract_tree_snapshot;

    base_outputs[1] = aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, base_inputs[1]);
    base_inputs[1].constants = base_inputs[0].constants;

    PreviousRollupData<NT> r1 = {
        .base_or_merge_rollup_public_inputs = base_outputs[0],
        .proof = base_inputs[0].kernel_data[0].proof, // TODO: this is a hack, we should be able to use the proof from
                                                      // base_outputs[0]
        .vk = base_inputs[0].kernel_data[0].vk,
        .vk_index = 0,
        .vk_sibling_path = MembershipWitness<NT, ROLLUP_VK_TREE_HEIGHT>(),
    };

    PreviousRollupData<NT> r2 = {
        .base_or_merge_rollup_public_inputs = base_outputs[1],
        .proof = base_inputs[1].kernel_data[0].proof, // TODO: this is a hack, we should be able to use the proof from
                                                      // base_outputs[1]
        .vk = base_inputs[1].kernel_data[0].vk,
        .vk_index = 0,
        .vk_sibling_path = MembershipWitness<NT, ROLLUP_VK_TREE_HEIGHT>(),
    };

    RootRollupInputs rootRollupInputs = {
        .previous_rollup_data = { r1, r2 },
        .new_historic_private_data_tree_root_sibling_path = historic_data_sibling_path,
        .new_historic_contract_tree_root_sibling_path = historic_contract_sibling_path,
    };

    RootRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_root_rollup::root_rollup_circuit(composer, rootRollupInputs);

    // Check data trees
    ASSERT_EQ(outputs.start_private_data_tree_snapshot, base_outputs[0].start_private_data_tree_snapshot);
    ASSERT_EQ(outputs.end_private_data_tree_snapshot, base_outputs[1].end_private_data_tree_snapshot);
    AppendOnlyTreeSnapshot<NT> expected_data_tree_snapshot = { .root = data_tree.root(),
                                                               .next_available_leaf_index = 16 };
    ASSERT_EQ(outputs.end_private_data_tree_snapshot, expected_data_tree_snapshot);

    // check contract trees
    ASSERT_EQ(outputs.start_contract_tree_snapshot, base_outputs[0].start_contract_tree_snapshot);
    ASSERT_EQ(outputs.end_contract_tree_snapshot, base_outputs[1].end_contract_tree_snapshot);
    AppendOnlyTreeSnapshot<NT> expected_contract_tree_snapshot{ .root = contract_tree.root(),
                                                                .next_available_leaf_index = 4 };
    ASSERT_EQ(outputs.end_contract_tree_snapshot, expected_contract_tree_snapshot);

    // TODO: Check nullifier trees

    // Check historic data trees
    ASSERT_EQ(outputs.start_tree_of_historic_private_data_tree_roots_snapshot, start_historic_data_tree_snapshot);
    ASSERT_EQ(outputs.end_tree_of_historic_private_data_tree_roots_snapshot, end_historic_data_tree_snapshot);

    // Check historic contract trees
    ASSERT_EQ(outputs.start_tree_of_historic_contract_tree_roots_snapshot, start_historic_contract_tree_snapshot);
    ASSERT_EQ(outputs.end_tree_of_historic_contract_tree_roots_snapshot, end_historic_contract_tree_snapshot);
}

} // namespace aztec3::circuits::rollup::root::native_root_rollup_circuit