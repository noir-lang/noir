#pragma once

#include "init.hpp"

// TODO: not needed right at this moment for native impl
#include <barretenberg/stdlib/types/types.hpp>

namespace aztec3::circuits::rollup::merge {
BaseOrMergeRollupPublicInputs merge_rollup_circuit(DummyComposer& composer, MergeRollupInputs const& mergeRollupInputs);
} // namespace aztec3::circuits::rollup::merge