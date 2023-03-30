// #include <barretenberg/common/serialize.hpp>
// #include <barretenberg/stdlib/types/types.hpp>
// #include <aztec3/oracle/oracle.hpp>
// #include <aztec3/circuits/apps/oracle_wrapper.hpp>
// #include <barretenberg/numeric/random/engine.hpp>
#include "aztec3/circuits/abis/append_only_tree_snapshot.hpp"
#include "aztec3/circuits/abis/membership_witness.hpp"
#include "aztec3/circuits/abis/private_kernel/new_contract_data.hpp"
#include "aztec3/circuits/abis/private_kernel/previous_kernel_data.hpp"
#include "aztec3/circuits/abis/rollup/base/previous_rollup_data.hpp"
#include "aztec3/circuits/abis/rollup/nullifier_leaf_preimage.hpp"
#include "aztec3/circuits/rollup/base/init.hpp"
#include "aztec3/circuits/rollup/base/utils.hpp"
#include "aztec3/circuits/kernel/private/utils.hpp"
#include "aztec3/constants.hpp"
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
#include <aztec3/circuits/abis/private_kernel/public_inputs.hpp>
#include <aztec3/circuits/abis/private_kernel/accumulated_data.hpp>
#include <aztec3/circuits/abis/private_kernel/constant_data.hpp>
#include <aztec3/circuits/abis/private_kernel/old_tree_roots.hpp>
#include <aztec3/circuits/abis/private_kernel/globals.hpp>
// #include <aztec3/circuits/abis/private_kernel/private_inputs.hpp>
// #include <aztec3/circuits/abis/private_kernel/private_inputs.hpp>

#include <aztec3/circuits/apps/function_execution_context.hpp>

// #include <aztec3/circuits/mock/mock_circuit.hpp>
#include <aztec3/circuits/mock/mock_kernel_circuit.hpp>

#include <barretenberg/common/map.hpp>
#include <barretenberg/common/test.hpp>
#include <cstdint>
#include <gtest/gtest.h>
#include <iostream>
#include <memory>
#include <vector>

// #include <aztec3/constants.hpp>
// #include <barretenberg/crypto/pedersen/pedersen.hpp>
// #include <barretenberg/stdlib/hash/pedersen/pedersen.hpp>

namespace {

using aztec3::circuits::abis::CallContext;
using aztec3::circuits::abis::CallStackItem;
using aztec3::circuits::abis::CallType;
using aztec3::circuits::abis::ContractDeploymentData;
using aztec3::circuits::abis::FunctionData;
using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;
using aztec3::circuits::abis::PrivateCircuitPublicInputs;
using aztec3::circuits::abis::SignedTxRequest;
using aztec3::circuits::abis::TxContext;
using aztec3::circuits::abis::TxRequest;

using aztec3::circuits::abis::private_kernel::AccumulatedData;
using aztec3::circuits::abis::private_kernel::ConstantData;
using aztec3::circuits::abis::private_kernel::Globals;
using aztec3::circuits::abis::private_kernel::OldTreeRoots;
using aztec3::circuits::abis::private_kernel::PreviousKernelData;
using aztec3::circuits::abis::private_kernel::PrivateCallData;
using aztec3::circuits::abis::private_kernel::PrivateInputs;
using aztec3::circuits::abis::private_kernel::PublicInputs;

using aztec3::circuits::apps::test_apps::basic_contract_deployment::constructor;
using aztec3::circuits::apps::test_apps::escrow::deposit;

// using aztec3::circuits::mock::mock_circuit;
using aztec3::circuits::kernel::private_kernel::utils::dummy_previous_kernel_with_vk_proof;
using aztec3::circuits::mock::mock_kernel_circuit;
using aztec3::circuits::rollup::base::utils::dummy_base_rollup_inputs_with_vk_proof;
using aztec3::circuits::rollup::base::utils::dummy_previous_rollup_with_vk_proof;
// using aztec3::circuits::mock::mock_kernel_inputs;

using aztec3::circuits::abis::AppendOnlyTreeSnapshot;

using aztec3::circuits::abis::MembershipWitness;
using aztec3::circuits::abis::NullifierLeafPreimage;
using aztec3::circuits::rollup::native_base_rollup::BaseRollupInputs;
using aztec3::circuits::rollup::native_base_rollup::BaseRollupPublicInputs;
using aztec3::circuits::rollup::native_base_rollup::ConstantRollupData;
using aztec3::circuits::rollup::native_base_rollup::NT;

using aztec3::circuits::abis::PreviousRollupData;
using aztec3::circuits::rollup::native_root_rollup::RootRollupInputs;
using aztec3::circuits::rollup::native_root_rollup::RootRollupPublicInputs;

using aztec3::circuits::abis::FunctionData;
using aztec3::circuits::abis::OptionallyRevealedData;
using aztec3::circuits::abis::private_kernel::NewContractData;

} // namespace

namespace aztec3::circuits::rollup::root::native_root_rollup_circuit {

class root_rollup_tests : public ::testing::Test {
  protected:
    void run_cbind(RootRollupInputs& root_rollup_inputs,
                   RootRollupPublicInputs& expected_public_inputs,
                   bool compare_pubins = true)
    {
        // TODO might be able to get rid of proving key buffer
        uint8_t const* pk_buf;
        size_t pk_size = root_rollup__init_proving_key(&pk_buf);
        info("Proving key size: ", pk_size);

        // TODO might be able to get rid of verification key buffer
        uint8_t const* vk_buf;
        size_t vk_size = root_rollup__init_verification_key(pk_buf, &vk_buf);
        info("Verification key size: ", vk_size);

        std::vector<uint8_t> root_rollup_inputs_vec;
        write(root_rollup_inputs_vec, root_rollup_inputs);

        // uint8_t const* proof_data;
        // size_t proof_data_size;
        uint8_t const* public_inputs_buf;
        info("creating proof");
        size_t public_inputs_size = root_rollup__sim(root_rollup_inputs_vec.data(), &public_inputs_buf);
        // info("Proof size: ", proof_data_size);
        info("PublicInputs size: ", public_inputs_size);

        if (compare_pubins) {
            RootRollupPublicInputs public_inputs;
            info("about to read...");
            uint8_t const* public_inputs_buf_tmp = public_inputs_buf;
            read(public_inputs_buf_tmp, public_inputs);
            info("about to assert...");
            ASSERT_EQ(public_inputs.calldata_hash.size(), expected_public_inputs.calldata_hash.size());
            for (size_t i = 0; i < public_inputs.calldata_hash.size(); i++) {
                ASSERT_EQ(public_inputs.calldata_hash[i], expected_public_inputs.calldata_hash[i]);
            }

            info("about to write expected...");
            std::vector<uint8_t> expected_public_inputs_vec;
            write(expected_public_inputs_vec, expected_public_inputs);

            info("about to assert buffers eq...");
            ASSERT_EQ(public_inputs_size, expected_public_inputs_vec.size());
            // Just compare the first 10 bytes of the serialized public outputs
            if (public_inputs_size > 10) {
                // for (size_t 0; i < public_inputs_size; i++) {
                for (size_t i = 0; i < 10; i++) {
                    ASSERT_EQ(public_inputs_buf[i], expected_public_inputs_vec[i]);
                }
            }
        }
        (void)root_rollup_inputs;     // unused
        (void)expected_public_inputs; // unused
        (void)compare_pubins;         // unused

        free((void*)pk_buf);
        free((void*)vk_buf);
        // free((void*)proof_data);
        free((void*)public_inputs_buf);
        info("finished retesting via cbinds...");
    }

  protected:
    RootRollupInputs getEmptyRootRollupInputs()
    {
        std::array<PreviousRollupData<NT>, 2> previous_rollup_data = {
            dummy_previous_rollup_with_vk_proof(),
            dummy_previous_rollup_with_vk_proof(),
        };

        RootRollupInputs rootRollupInputs = {
            .previous_rollup_data = previous_rollup_data,
            .new_historic_private_data_tree_root_sibling_path = { 0 },
            .new_historic_contract_tree_root_sibling_path = { 0 },
        };

        return rootRollupInputs;
    }
};

TEST_F(root_rollup_tests, calldata_hash_empty_blocks)
{
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
    RootRollupPublicInputs outputs = aztec3::circuits::rollup::native_root_rollup::root_rollup_circuit(inputs);

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

template <size_t N>
NT::fr iterate_through_tree_via_sibling_path(NT::fr leaf, NT::uint32 leafIndex, std::array<NT::fr, N> siblingPath)
{
    for (size_t i = 0; i < siblingPath.size(); i++) {
        if (leafIndex & (1 << i)) {
            leaf = crypto::pedersen_hash::hash_multiple({ siblingPath[i], leaf });
        } else {
            leaf = crypto::pedersen_hash::hash_multiple({ leaf, siblingPath[i] });
        }
    }
    return leaf;
}

template <size_t N> std::array<fr, N> get_sibling_path(stdlib::merkle_tree::MemoryTree tree, size_t leafIndex)
{
    std::array<fr, N> siblingPath;
    auto path = tree.get_hash_path(leafIndex);

    for (size_t i = 0; i < N; i++) {
        //
        if (leafIndex & (1 << i)) {
            siblingPath[i] = path[i].first;
        } else {
            siblingPath[i] = path[i].second;
        }
    }
    return siblingPath;
}

TEST_F(root_rollup_tests, blabber)
{
    // Base Rollup 1 and 2
    BaseRollupInputs base_inputs_1 = dummy_base_rollup_inputs_with_vk_proof();
    BaseRollupInputs base_inputs_2 = dummy_base_rollup_inputs_with_vk_proof();

    // Insert commitments into base rollup 2
    stdlib::merkle_tree::MemoryTree data_tree = stdlib::merkle_tree::MemoryTree(PRIVATE_DATA_TREE_HEIGHT);

    for (uint8_t i = 0; i < 2; i++) {
        for (uint8_t j = 0; j < 4; j++) {
            base_inputs_2.kernel_data[i].public_inputs.end.new_commitments[j] = fr(i * 4 + j + 1);
            data_tree.update_element(i * 4 + j, fr(i * 4 + j + 1));
        }
    }

    // Compute a sibling path for the new commitment subtree. Get the full first, and then shorten it.
    auto sibling_path = get_sibling_path<PRIVATE_DATA_TREE_HEIGHT>(data_tree, 0);
    std::array<fr, PRIVATE_DATA_SUBTREE_INCLUSION_CHECK_DEPTH> new_commitments_subtree_sibling_path;
    for (size_t i = PRIVATE_DATA_SUBTREE_INCLUSION_CHECK_DEPTH; i < PRIVATE_DATA_TREE_HEIGHT; i++) {
        new_commitments_subtree_sibling_path[i - PRIVATE_DATA_SUBTREE_INCLUSION_CHECK_DEPTH] = sibling_path[i];
    }
    std::cout << "new_commitments_subtree_sibling_path" << new_commitments_subtree_sibling_path << std::endl;
    base_inputs_2.new_commitments_subtree_sibling_path = new_commitments_subtree_sibling_path;

    std::cout << data_tree.root() << std::endl;

    // Historic trees
    stdlib::merkle_tree::MemoryTree historic_data_tree =
        stdlib::merkle_tree::MemoryTree(PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT);

    auto historic_sibling_path = get_sibling_path<PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT>(historic_data_tree, 0);

    AppendOnlyTreeSnapshot<NT> historic_data_tree_snapshot = { .root = historic_data_tree.root(),
                                                               .next_available_leaf_index = 0 };

    base_inputs_1.constants.start_tree_of_historic_private_data_tree_roots_snapshot = historic_data_tree_snapshot;
    base_inputs_2.constants.start_tree_of_historic_private_data_tree_roots_snapshot = historic_data_tree_snapshot;

    historic_data_tree.update_element(0, historic_data_tree.root());
    AppendOnlyTreeSnapshot<NT> historic_data_tree_snapshot_2 = { .root = historic_data_tree.root(),
                                                                 .next_available_leaf_index = 1 };

    BaseRollupPublicInputs outputs1 = aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(base_inputs_1);
    BaseRollupPublicInputs outputs2 = aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(base_inputs_2);

    PreviousKernelData<NT> mocked_kernel0 = base_inputs_1.kernel_data[0];
    PreviousKernelData<NT> mocked_kernel1 = base_inputs_1.kernel_data[1];

    PreviousRollupData<NT> r1 = {
        .base_rollup_public_inputs = outputs1,
        .proof = mocked_kernel0.proof,
        .vk = mocked_kernel0.vk,
        .vk_index = 0,
        .vk_sibling_path = MembershipWitness<NT, ROLLUP_VK_TREE_HEIGHT>(),
    };

    PreviousRollupData<NT> r2 = {
        .base_rollup_public_inputs = outputs1,
        .proof = mocked_kernel1.proof,
        .vk = mocked_kernel1.vk,
        .vk_index = 1,
        .vk_sibling_path = MembershipWitness<NT, ROLLUP_VK_TREE_HEIGHT>(),
    };

    std::array<PreviousRollupData<NT>, 2> previous_rollup_data = { r1, r2 };

    RootRollupInputs rootRollupInputs = {
        .previous_rollup_data = previous_rollup_data,
        .new_historic_private_data_tree_root_sibling_path = historic_sibling_path,
        .new_historic_contract_tree_root_sibling_path = { 0 },
    };

    RootRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_root_rollup::root_rollup_circuit(rootRollupInputs);

    std::cout << outputs << std::endl;

    // We expect this thing afterwards historic_data_tree_snapshot_2

    std::cout << "pre_historic : " << historic_data_tree_snapshot << std::endl;
    std::cout << "post_historic: " << historic_data_tree_snapshot_2 << std::endl;

    std::cout << "data root: " << data_tree.root() << std::endl;

    std::cout << r2.base_rollup_public_inputs.end_private_data_tree_snapshot << std::endl;

    run_cbind(rootRollupInputs, outputs, true);
}

} // namespace aztec3::circuits::rollup::root::native_root_rollup_circuit