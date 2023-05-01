#pragma once

#include "init.hpp"

namespace aztec3::circuits::rollup::merge {
BaseOrMergeRollupPublicInputs merge_rollup_circuit(DummyComposer& composer, MergeRollupInputs const& mergeRollupInputs);
}  // namespace aztec3::circuits::rollup::merge