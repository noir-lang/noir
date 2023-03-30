#pragma once
#include "index.hpp"
#include "init.hpp"

namespace aztec3::circuits::rollup::base::utils {

namespace {
using NT = aztec3::utils::types::NativeTypes;
using aztec3::circuits::abis::BaseRollupInputs;
using aztec3::circuits::abis::PreviousRollupData;
} // namespace

BaseRollupInputs<NT> dummy_base_rollup_inputs_with_vk_proof();
PreviousRollupData<NT> dummy_previous_rollup_with_vk_proof();

} // namespace aztec3::circuits::rollup::base::utils