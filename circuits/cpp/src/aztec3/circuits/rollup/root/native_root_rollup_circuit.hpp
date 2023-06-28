#pragma once

#include "init.hpp"

// TODO: not needed right at this moment for native impl
#include "aztec3/circuits/abis/rollup/root/root_rollup_inputs.hpp"
#include "aztec3/circuits/abis/rollup/root/root_rollup_public_inputs.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

namespace aztec3::circuits::rollup::native_root_rollup {

RootRollupPublicInputs root_rollup_circuit(DummyBuilder& builder, RootRollupInputs const& rootRollupInputs);

}  // namespace aztec3::circuits::rollup::native_root_rollup