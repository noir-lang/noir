
#pragma once

#include "aztec3/circuits/abis/append_only_tree_snapshot.hpp"
#include "aztec3/circuits/abis/rollup/base/base_or_merge_rollup_public_inputs.hpp"
#include "aztec3/circuits/abis/rollup/base/base_rollup_inputs.hpp"
#include "aztec3/circuits/abis/rollup/constant_rollup_data.hpp"
#include "aztec3/circuits/abis/rollup/merge/merge_rollup_inputs.hpp"
#include "aztec3/circuits/abis/rollup/merge/previous_rollup_data.hpp"
#include "aztec3/circuits/abis/rollup/root/root_rollup_inputs.hpp"
#include "aztec3/circuits/rollup/base/native_base_rollup_circuit.hpp"
#include "aztec3/utils/dummy_composer.hpp"
#include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>
#include <aztec3/circuits/recursion/aggregator.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>
#include <aztec3/utils/types/native_types.hpp>

#include "barretenberg/stdlib/merkle_tree/memory_tree.hpp"
#include "barretenberg/stdlib/merkle_tree/nullifier_tree/nullifier_memory_tree.hpp"
#include <barretenberg/crypto/sha256/sha256.hpp>
#include <barretenberg/stdlib/merkle_tree/nullifier_tree/nullifier_leaf.hpp>
#include <barretenberg/stdlib/merkle_tree/nullifier_tree/nullifier_tree.hpp>

namespace aztec3::circuits::rollup::test_utils {

using NT = aztec3::utils::types::NativeTypes;

// Types
using ConstantRollupData = abis::ConstantRollupData<NT>;
using BaseRollupInputs = abis::BaseRollupInputs<NT>;
using BaseOrMergeRollupPublicInputs = abis::BaseOrMergeRollupPublicInputs<NT>;
using DummyComposer = aztec3::utils::DummyComposer;

using Aggregator = aztec3::circuits::recursion::Aggregator;
using AggregationObject = aztec3::utils::types::NativeTypes::AggregationObject;
using AppendOnlySnapshot = abis::AppendOnlyTreeSnapshot<NT>;

using NullifierLeafPreimage = aztec3::circuits::abis::NullifierLeafPreimage<NT>;

// Nullifier Tree Alias
using MerkleTree = stdlib::merkle_tree::MemoryTree;
using NullifierTree = stdlib::merkle_tree::NullifierMemoryTree;
using NullifierLeaf = stdlib::merkle_tree::nullifier_leaf;

using aztec3::circuits::abis::MembershipWitness;

}  // namespace aztec3::circuits::rollup::test_utils