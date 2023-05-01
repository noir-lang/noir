#pragma once

#include "init.hpp"

#include "aztec3/utils/circuit_errors.hpp"

using aztec3::circuits::check_membership;
using aztec3::circuits::root_from_sibling_path;

namespace aztec3::circuits::rollup::components {
std::array<fr, 2> compute_calldata_hash(std::array<abis::PreviousRollupData<NT>, 2> previous_rollup_data);
void assert_prev_rollups_follow_on_from_each_other(DummyComposer& composer,
                                                   BaseOrMergeRollupPublicInputs const& left,
                                                   BaseOrMergeRollupPublicInputs const& right);
void assert_both_input_proofs_of_same_rollup_type(DummyComposer& composer,
                                                  BaseOrMergeRollupPublicInputs const& left,
                                                  BaseOrMergeRollupPublicInputs const& right);
NT::fr assert_both_input_proofs_of_same_height_and_return(DummyComposer& composer,
                                                          BaseOrMergeRollupPublicInputs const& left,
                                                          BaseOrMergeRollupPublicInputs const& right);
void assert_equal_constants(DummyComposer& composer,
                            BaseOrMergeRollupPublicInputs const& left,
                            BaseOrMergeRollupPublicInputs const& right);

AggregationObject aggregate_proofs(BaseOrMergeRollupPublicInputs const& left,
                                   BaseOrMergeRollupPublicInputs const& right);

template <size_t N> AppendOnlySnapshot insert_subtree_to_snapshot_tree(DummyComposer& composer,
                                                                       AppendOnlySnapshot snapshot,
                                                                       std::array<NT::fr, N> siblingPath,
                                                                       NT::fr emptySubtreeRoot,
                                                                       NT::fr subtreeRootToInsert,
                                                                       uint8_t subtreeDepth,
                                                                       std::string const& message)
{
    // TODO: Sanity check len of siblingPath > height of subtree
    // TODO: Ensure height of subtree is correct (eg 3 for commitments, 1 for contracts)
    auto leafIndexAtDepth = snapshot.next_available_leaf_index >> subtreeDepth;

    // Check that the current root is correct and that there is an empty subtree at the insertion location
    check_membership<NT>(composer, emptySubtreeRoot, leafIndexAtDepth, siblingPath, snapshot.root, message);

    // if index of leaf is x, index of its parent is x/2 or x >> 1. We need to find the parent `subtreeDepth` levels up.
    auto new_root = root_from_sibling_path<NT>(subtreeRootToInsert, leafIndexAtDepth, siblingPath);

    // 2^subtreeDepth is the number of leaves added. 2^x = 1 << x
    auto new_next_available_leaf_index = snapshot.next_available_leaf_index + (static_cast<uint8_t>(1) << subtreeDepth);

    AppendOnlySnapshot newTreeSnapshot = { .root = new_root,
                                           .next_available_leaf_index = new_next_available_leaf_index };
    return newTreeSnapshot;
}
}  // namespace aztec3::circuits::rollup::components