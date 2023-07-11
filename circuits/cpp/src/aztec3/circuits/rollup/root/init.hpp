
#pragma once

#include "aztec3/circuits/abis/append_only_tree_snapshot.hpp"
#include "aztec3/circuits/abis/rollup/constant_rollup_data.hpp"
#include "aztec3/circuits/abis/rollup/root/root_rollup_inputs.hpp"
#include "aztec3/circuits/abis/rollup/root/root_rollup_public_inputs.hpp"
#include "aztec3/circuits/recursion/aggregator.hpp"
#include "aztec3/utils/dummy_circuit_builder.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::rollup::native_root_rollup {

using NT = aztec3::utils::types::NativeTypes;
using DummyBuilder = aztec3::utils::DummyCircuitBuilder;

// Params
using ConstantRollupData = abis::ConstantRollupData<NT>;
using RootRollupInputs = abis::RootRollupInputs<NT>;
using RootRollupPublicInputs = abis::RootRollupPublicInputs<NT>;

using Aggregator = aztec3::circuits::recursion::Aggregator;

using MemoryStore = stdlib::merkle_tree::MemoryStore;
using MerkleTree = stdlib::merkle_tree::MerkleTree<MemoryStore>;

}  // namespace aztec3::circuits::rollup::native_root_rollup
