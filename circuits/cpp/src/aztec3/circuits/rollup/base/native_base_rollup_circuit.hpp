#pragma once

#include "init.hpp"

// TODO: not needed right at this moment for native impl
#include <aztec3/utils/types/convert.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/native_types.hpp>

#include <aztec3/circuits/abis/rollup/base/base_rollup_inputs.hpp>
#include <aztec3/circuits/abis/rollup/base/base_or_merge_rollup_public_inputs.hpp>
#include <aztec3/circuits/abis/rollup/nullifier_leaf_preimage.hpp>
#include <aztec3/circuits/abis/rollup/constant_rollup_data.hpp>

namespace aztec3::circuits::rollup::native_base_rollup {

BaseOrMergeRollupPublicInputs base_rollup_circuit(DummyComposer& composer, BaseRollupInputs const& baseRollupInputs);

} // namespace aztec3::circuits::rollup::native_base_rollup