#pragma once
#include "aztec3/circuits/abis/rollup/base/base_or_merge_rollup_public_inputs.hpp"
#include "index.hpp"
#include "init.hpp"

namespace aztec3::circuits::rollup::merge::utils {

namespace {
using NT = aztec3::utils::types::NativeTypes;
using aztec3::circuits::abis::MergeRollupInputs;
using aztec3::circuits::abis::PreviousRollupData;
} // namespace

MergeRollupInputs<NT> dummy_merge_rollup_inputs_with_vk_proof();
std::array<PreviousRollupData<NT>, 2> previous_rollups_with_vk_proof_that_follow_on();

} // namespace aztec3::circuits::rollup::merge::utils