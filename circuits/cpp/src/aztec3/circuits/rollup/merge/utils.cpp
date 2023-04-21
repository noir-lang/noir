#include "aztec3/circuits/rollup/merge/utils.hpp"
#include "barretenberg/plonk/proof_system/verification_key/verification_key.hpp"

#include <aztec3/circuits/kernel/private/utils.hpp>
#include "aztec3/circuits/rollup/base/utils.hpp"

namespace {
using aztec3::circuits::abis::MembershipWitness;
using aztec3::circuits::kernel::private_kernel::utils::dummy_previous_kernel;
} // namespace

namespace aztec3::circuits::rollup::merge {

std::array<PreviousRollupData, 2> previous_rollup_datas()
{
    DummyComposer composer = DummyComposer();
    auto input1 = base::utils::dummy_base_rollup_inputs();
    BaseOrMergeRollupPublicInputs base_public_input1 =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, input1);

    auto input2 = input1;
    input2.start_private_data_tree_snapshot = base_public_input1.end_private_data_tree_snapshot;
    input2.start_nullifier_tree_snapshot = base_public_input1.end_nullifier_tree_snapshot;
    input2.start_contract_tree_snapshot = base_public_input1.end_contract_tree_snapshot;
    BaseOrMergeRollupPublicInputs base_public_input2 =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, input2);

    // just for mocked vk and proof
    // Need a way to extract a proof from Base Rollup Circuit. Until then use kernel as a hack.
    PreviousKernelData<NT> mocked_kernel = dummy_previous_kernel();

    PreviousRollupData previous_rollup1 = {
        .base_or_merge_rollup_public_inputs = base_public_input1,
        .proof = mocked_kernel.proof,
        .vk = mocked_kernel.vk,
        .vk_index = 0,
        .vk_sibling_path = MembershipWitness<NT, ROLLUP_VK_TREE_HEIGHT>(),
    };
    PreviousRollupData previous_rollup2 = {
        .base_or_merge_rollup_public_inputs = base_public_input2,
        .proof = mocked_kernel.proof,
        .vk = mocked_kernel.vk,
        .vk_index = 0,
        .vk_sibling_path = MembershipWitness<NT, ROLLUP_VK_TREE_HEIGHT>(),
    };

    return { previous_rollup1, previous_rollup2 };
}

MergeRollupInputs dummy_merge_rollup_inputs()
{
    MergeRollupInputs merge_rollup_inputs = { .previous_rollup_data = previous_rollup_datas() };
    return merge_rollup_inputs;
}
} // namespace aztec3::circuits::rollup::merge