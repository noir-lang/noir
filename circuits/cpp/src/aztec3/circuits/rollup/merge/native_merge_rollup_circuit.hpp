#pragma once

#include "init.hpp"

// TODO: not needed right at this moment for native impl
#include <barretenberg/stdlib/types/types.hpp>
#include <aztec3/utils/types/convert.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/native_types.hpp>

#include <aztec3/circuits/abis/rollup/merge/merge_rollup_inputs.hpp>

namespace aztec3::circuits::rollup::native_merge_rollup {

MergeRollupPublicInputs merge_rollup_circuit(MergeRollupInputs mergeRollupInputs);

} // namespace aztec3::circuits::rollup::native_merge_rollup