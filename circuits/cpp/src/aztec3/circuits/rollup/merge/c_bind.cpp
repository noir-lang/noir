#include "c_bind.h"

#include "index.hpp"

#include "aztec3/utils/dummy_circuit_builder.hpp"

#include <barretenberg/barretenberg.hpp>

namespace {
using NT = aztec3::utils::types::NativeTypes;
using DummyCircuitBuilder = aztec3::utils::DummyCircuitBuilder;
using aztec3::circuits::abis::MergeRollupInputs;
using aztec3::circuits::rollup::merge::merge_rollup_circuit;
}  // namespace

// WASM Cbinds

CBIND(merge_rollup__sim, [](MergeRollupInputs<NT> const& merge_rollup_inputs) {
    DummyCircuitBuilder builder = DummyCircuitBuilder("merge_rollup__sim");
    auto const& public_inputs = merge_rollup_circuit(builder, merge_rollup_inputs);
    return builder.result_or_error(public_inputs);
});