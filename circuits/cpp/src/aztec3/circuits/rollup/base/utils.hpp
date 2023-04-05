#pragma once
#include "aztec3/circuits/rollup/base/nullifier_tree_testing_harness.hpp"
#include "index.hpp"
#include "init.hpp"

namespace aztec3::circuits::rollup::base::utils {

namespace {
using NT = aztec3::utils::types::NativeTypes;
using aztec3::circuits::abis::BaseRollupInputs;
} // namespace

BaseRollupInputs<NT> dummy_base_rollup_inputs_with_vk_proof();

NullifierMemoryTreeTestingHarness get_initial_nullifier_tree(size_t spacing);
abis::AppendOnlyTreeSnapshot<NT> get_snapshot_of_tree_state(NullifierMemoryTreeTestingHarness nullifier_tree);

std::tuple<BaseRollupInputs<NT>, abis::AppendOnlyTreeSnapshot<NT>, abis::AppendOnlyTreeSnapshot<NT>>
generate_nullifier_tree_testing_values(BaseRollupInputs<NT> inputs, size_t starting_insertion_value, size_t spacing);

std::tuple<BaseRollupInputs<NT>, abis::AppendOnlyTreeSnapshot<NT>, abis::AppendOnlyTreeSnapshot<NT>>
generate_nullifier_tree_testing_values(BaseRollupInputs<NT> inputs,
                                       std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH * 2> new_nullifiers,
                                       size_t spacing_prefill);

} // namespace aztec3::circuits::rollup::base::utils