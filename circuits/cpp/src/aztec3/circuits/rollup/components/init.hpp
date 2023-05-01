
#pragma once

#include "aztec3/circuits/abis/append_only_tree_snapshot.hpp"
#include "aztec3/circuits/abis/rollup/base/base_or_merge_rollup_public_inputs.hpp"
#include "aztec3/circuits/abis/rollup/constant_rollup_data.hpp"
#include "aztec3/circuits/abis/rollup/merge/merge_rollup_inputs.hpp"
#include "aztec3/utils/dummy_composer.hpp"
#include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>
#include <aztec3/circuits/hash.hpp>
#include <aztec3/circuits/recursion/aggregator.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>
#include <aztec3/utils/types/native_types.hpp>

#include <barretenberg/crypto/sha256/sha256.hpp>

namespace aztec3::circuits::rollup::components {

using NT = aztec3::utils::types::NativeTypes;

// Params
using NT = aztec3::utils::types::NativeTypes;
using AggregationObject = aztec3::utils::types::NativeTypes::AggregationObject;
using BaseOrMergeRollupPublicInputs = aztec3::circuits::abis::BaseOrMergeRollupPublicInputs<NT>;
using AppendOnlySnapshot = abis::AppendOnlyTreeSnapshot<NT>;
using DummyComposer = aztec3::utils::DummyComposer;

}  // namespace aztec3::circuits::rollup::components