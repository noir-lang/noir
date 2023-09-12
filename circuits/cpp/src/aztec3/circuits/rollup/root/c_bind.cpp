#include "c_bind.h"

#include "index.hpp"
#include "init.hpp"

#include "aztec3/constants.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace {
using Builder = UltraCircuitBuilder;
using NT = aztec3::utils::types::NativeTypes;
using DummyCircuitBuilder = aztec3::utils::DummyCircuitBuilder;
using aztec3::circuits::rollup::native_root_rollup::root_rollup_circuit;
using aztec3::circuits::rollup::native_root_rollup::RootRollupInputs;
using aztec3::circuits::rollup::native_root_rollup::RootRollupPublicInputs;

}  // namespace

// WASM Cbinds
CBIND(root_rollup__sim, [](RootRollupInputs const& root_rollup_inputs) {
    DummyCircuitBuilder builder = DummyCircuitBuilder("root_rollup__sim");
    auto const& public_inputs = root_rollup_circuit(builder, root_rollup_inputs);
    return builder.result_or_error(public_inputs);
});