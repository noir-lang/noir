
#pragma once

#include "aztec3/circuits/abis/append_only_tree_snapshot.hpp"
#include "aztec3/circuits/abis/rollup/root/root_rollup_inputs.hpp"
#include "aztec3/circuits/abis/rollup/constant_rollup_data.hpp"
#include "aztec3/circuits/abis/rollup/root/root_rollup_public_inputs.hpp"
#include <aztec3/circuits/recursion/aggregator.hpp>
#include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>

#include <barretenberg/crypto/sha256/sha256.hpp>
#include <barretenberg/stdlib/types/types.hpp>
#include <aztec3/utils/types/convert.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/native_types.hpp>

namespace aztec3::circuits::rollup::native_root_rollup {

using NT = aztec3::utils::types::NativeTypes;

// Params
using ConstantRollupData = abis::ConstantRollupData<NT>;
using RootRollupInputs = abis::RootRollupInputs<NT>;
using RootRollupPublicInputs = abis::RootRollupPublicInputs<NT>;

using Aggregator = aztec3::circuits::recursion::Aggregator;
using AggregationObject = utils::types::NativeTypes::AggregationObject;
using AppendOnlySnapshot = abis::AppendOnlyTreeSnapshot<NT>;

} // namespace aztec3::circuits::rollup::native_root_rollup