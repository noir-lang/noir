#pragma once
#include "init.hpp"

namespace aztec3::circuits::rollup::merge {

MergeRollupInputs dummy_merge_rollup_inputs();
std::array<PreviousRollupData, 2> previous_rollup_datas();

} // namespace aztec3::circuits::rollup::merge