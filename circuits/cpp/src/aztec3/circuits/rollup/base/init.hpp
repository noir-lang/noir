
#pragma once

#include "aztec3/circuits/abis/append_only_tree_snapshot.hpp"
#include "aztec3/circuits/abis/rollup/constant_rollup_data.hpp"
#include "aztec3/circuits/abis/rollup/base/base_rollup_inputs.hpp"
#include "aztec3/circuits/abis/rollup/base/base_or_merge_rollup_public_inputs.hpp"
#include "barretenberg/stdlib/merkle_tree/memory_tree.hpp"
#include "barretenberg/stdlib/merkle_tree/nullifier_tree/nullifier_memory_tree.hpp"
#include <aztec3/circuits/recursion/aggregator.hpp>
#include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>

#include <barretenberg/crypto/sha256/sha256.hpp>
#include <barretenberg/stdlib/types/types.hpp>
#include <barretenberg/stdlib/merkle_tree/nullifier_tree/nullifier_tree.hpp>
#include <barretenberg/stdlib/merkle_tree/nullifier_tree/nullifier_leaf.hpp>
#include <aztec3/utils/types/convert.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/native_types.hpp>

namespace aztec3::circuits::rollup::native_base_rollup {

using NT = aztec3::utils::types::NativeTypes;

// Params
using ConstantRollupData = abis::ConstantRollupData<NT>;
using BaseRollupInputs = abis::BaseRollupInputs<NT>;
using BaseOrMergeRollupPublicInputs = abis::BaseOrMergeRollupPublicInputs<NT>;

using Aggregator = aztec3::circuits::recursion::Aggregator;
using AggregationObject = utils::types::NativeTypes::AggregationObject;
using AppendOnlySnapshot = abis::AppendOnlyTreeSnapshot<NT>;

// Nullifier Tree Alias
using MerkleTree = stdlib::merkle_tree::MemoryTree;
using NullifierTree = stdlib::merkle_tree::NullifierMemoryTree;
using NullifierLeaf = stdlib::merkle_tree::nullifier_leaf;

} // namespace aztec3::circuits::rollup::native_base_rollup