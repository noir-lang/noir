#pragma once
#include "aztec3/circuits/rollup/base/nullifier_tree_testing_harness.hpp"
#include "index.hpp"
#include "init.hpp"

namespace aztec3::circuits::rollup::base::utils {

namespace {
using NT = aztec3::utils::types::NativeTypes;
using aztec3::circuits::abis::BaseRollupInputs;
using nullifier_tree_testing_values =
    std::tuple<BaseRollupInputs<NT>, abis::AppendOnlyTreeSnapshot<NT>, abis::AppendOnlyTreeSnapshot<NT>>;
} // namespace

BaseRollupInputs<NT> dummy_base_rollup_inputs();

NullifierMemoryTreeTestingHarness get_initial_nullifier_tree(size_t spacing);
abis::AppendOnlyTreeSnapshot<NT> get_snapshot_of_tree_state(NullifierMemoryTreeTestingHarness nullifier_tree);

nullifier_tree_testing_values generate_nullifier_tree_testing_values(BaseRollupInputs<NT> inputs,
                                                                     size_t starting_insertion_value,
                                                                     size_t spacing);

nullifier_tree_testing_values generate_nullifier_tree_testing_values(
    BaseRollupInputs<NT> inputs,
    std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH * 2> new_nullifiers,
    std::vector<fr> initial_values);

nullifier_tree_testing_values generate_nullifier_tree_testing_values(
    BaseRollupInputs<NT> inputs, std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH * 2> new_nullifiers, size_t spacing);

} // namespace aztec3::circuits::rollup::base::utils