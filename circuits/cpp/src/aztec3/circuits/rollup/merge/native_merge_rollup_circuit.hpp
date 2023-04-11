#pragma once

#include "init.hpp"

// TODO: not needed right at this moment for native impl
#include <barretenberg/stdlib/types/types.hpp>
#include <aztec3/utils/types/convert.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/native_types.hpp>

#include <aztec3/circuits/abis/rollup/merge/merge_rollup_inputs.hpp>

namespace aztec3::circuits::rollup::native_merge_rollup {

BaseOrMergeRollupPublicInputs merge_rollup_circuit(DummyComposer& composer, MergeRollupInputs const& mergeRollupInputs);

std::array<fr, 2> compute_calldata_hash(std::array<abis::PreviousRollupData<NT>, 2> const& previous_rollup_data);
void assert_prev_rollups_follow_on_from_each_other(DummyComposer& composer,
                                                   BaseOrMergeRollupPublicInputs const& left,
                                                   BaseOrMergeRollupPublicInputs const& right);
void assert_both_input_proofs_of_same_rollup_type(DummyComposer& composer,
                                                  BaseOrMergeRollupPublicInputs const& left,
                                                  BaseOrMergeRollupPublicInputs const& right);
NT::fr assert_both_input_proofs_of_same_height_and_return(DummyComposer& composer,
                                                          BaseOrMergeRollupPublicInputs const& left,
                                                          BaseOrMergeRollupPublicInputs const& right);
void assert_equal_constants(DummyComposer& composer,
                            BaseOrMergeRollupPublicInputs const& left,
                            BaseOrMergeRollupPublicInputs const& right);
AggregationObject aggregate_proofs(BaseOrMergeRollupPublicInputs const& left,
                                   BaseOrMergeRollupPublicInputs const& right);
} // namespace aztec3::circuits::rollup::native_merge_rollup