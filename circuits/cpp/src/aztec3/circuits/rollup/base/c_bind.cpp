#include "c_bind.h"

#include "index.hpp"
#include "init.hpp"

#include "aztec3/circuits/abis/rollup/base/base_or_merge_rollup_public_inputs.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/dummy_circuit_builder.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace {
using Builder = UltraCircuitBuilder;
using NT = aztec3::utils::types::NativeTypes;
using DummyCircuitBuilder = aztec3::utils::DummyCircuitBuilder;
using aztec3::circuits::abis::BaseRollupInputs;
using aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit;
}  // namespace

// WASM Cbinds
CBIND(base_rollup__sim, [](BaseRollupInputs<NT> const& base_rollup_inputs) {
    DummyCircuitBuilder builder = DummyCircuitBuilder("base_rollup__sim");
    auto const& public_inputs = base_rollup_circuit(builder, base_rollup_inputs);
    return builder.result_or_error(public_inputs);
});