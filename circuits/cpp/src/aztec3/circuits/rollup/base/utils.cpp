#include "index.hpp"
#include "init.hpp"

#include <aztec3/circuits/kernel/private/utils.hpp>
#include <aztec3/circuits/mock/mock_kernel_circuit.hpp>
#include "aztec3/circuits/abis/private_kernel/new_contract_data.hpp"
#include "aztec3/circuits/abis/rollup/base/previous_rollup_data.hpp"

namespace {
using NT = aztec3::utils::types::NativeTypes;
using AggregationObject = aztec3::utils::types::NativeTypes::AggregationObject;

using aztec3::circuits::abis::AppendOnlyTreeSnapshot;
using aztec3::circuits::abis::BaseRollupInputs;
using aztec3::circuits::abis::BaseRollupPublicInputs;
using aztec3::circuits::abis::ConstantRollupData;
using aztec3::circuits::abis::MembershipWitness;
using aztec3::circuits::abis::NullifierLeafPreimage;
using aztec3::circuits::abis::PreviousRollupData;
using aztec3::circuits::abis::private_kernel::NewContractData;

using aztec3::circuits::kernel::private_kernel::utils::dummy_previous_kernel_with_vk_proof;

using plonk::TurboComposer;
} // namespace

namespace aztec3::circuits::rollup::base::utils {

BaseRollupInputs<NT> dummy_base_rollup_inputs_with_vk_proof()
{
    // TODO standardize function naming
    ConstantRollupData<NT> constantRollupData = ConstantRollupData<NT>::empty();

    std::array<NullifierLeafPreimage<NT>, 2 * KERNEL_NEW_NULLIFIERS_LENGTH> low_nullifier_leaf_preimages;
    std::array<MembershipWitness<NT, NULLIFIER_TREE_HEIGHT>, 2 * KERNEL_NEW_NULLIFIERS_LENGTH>
        low_nullifier_membership_witness;

    for (size_t i = 0; i < 2 * KERNEL_NEW_NULLIFIERS_LENGTH; ++i) {
        low_nullifier_leaf_preimages[i] = NullifierLeafPreimage<NT>::empty();
        low_nullifier_membership_witness[i] = MembershipWitness<NT, NULLIFIER_TREE_HEIGHT>::empty();
    }

    std::array<MembershipWitness<NT, PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT>, 2>
        historic_private_data_tree_root_membership_witnesses = {
            MembershipWitness<NT, PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT>::empty(),
            MembershipWitness<NT, PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT>::empty()
        };

    std::array<MembershipWitness<NT, CONTRACT_TREE_ROOTS_TREE_HEIGHT>, 2>
        historic_contract_tree_root_membership_witnesses = {
            MembershipWitness<NT, CONTRACT_TREE_ROOTS_TREE_HEIGHT>::empty(),
            MembershipWitness<NT, CONTRACT_TREE_ROOTS_TREE_HEIGHT>::empty()
        };

    // Kernels
    std::array<abis::private_kernel::PreviousKernelData<NT>, 2> kernel_data;
    // grab mocked previous kernel (need a valid vk, proof, aggobj)
    kernel_data[0] = dummy_previous_kernel_with_vk_proof();
    kernel_data[1] = dummy_previous_kernel_with_vk_proof();

    BaseRollupInputs<NT> baseRollupInputs = { .kernel_data = kernel_data,
                                              .start_private_data_tree_snapshot = {
                                                  .root = native_base_rollup::MerkleTree(PRIVATE_DATA_TREE_HEIGHT).root(),
                                                  .next_available_leaf_index = 0,
                                              },
                                              .start_nullifier_tree_snapshot = AppendOnlyTreeSnapshot<NT>::empty(),
                                              .start_contract_tree_snapshot = {
                                                  .root = native_base_rollup::MerkleTree(CONTRACT_TREE_HEIGHT).root(),
                                                  .next_available_leaf_index = 0,
                                              },
                                              .low_nullifier_leaf_preimages = low_nullifier_leaf_preimages,
                                              .low_nullifier_membership_witness = low_nullifier_membership_witness,
                                              .new_commitments_subtree_sibling_path = { 0 },
                                              .new_nullifiers_subtree_sibling_path = { 0 },
                                              .new_contracts_subtree_sibling_path = { 0 },
                                              .historic_private_data_tree_root_membership_witnesses =
                                                  historic_private_data_tree_root_membership_witnesses,
                                              .historic_contract_tree_root_membership_witnesses =
                                                  historic_contract_tree_root_membership_witnesses,
                                              .constants = constantRollupData };
    return baseRollupInputs;
}

PreviousRollupData<NT> dummy_previous_rollup_with_vk_proof()
{
    BaseRollupInputs emptyInputs = dummy_base_rollup_inputs_with_vk_proof();
    BaseRollupPublicInputs outputs = aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(emptyInputs);

    // just for mocked vk and proof
    // TODO create generic utility for mocked vk and proof
    PreviousKernelData<NT> mocked_kernel = dummy_previous_kernel_with_vk_proof();

    PreviousRollupData<NT> previous_rollup = {
        .base_rollup_public_inputs = outputs,
        .proof = mocked_kernel.proof,
        .vk = mocked_kernel.vk,
        .vk_index = 0,
        .vk_sibling_path = MembershipWitness<NT, ROLLUP_VK_TREE_HEIGHT>::empty(),
    };

    return previous_rollup;
}

} // namespace aztec3::circuits::rollup::base::utils