#pragma once
#include "barretenberg/dsl/acir_format/recursion_constraint.hpp"
#include "barretenberg/stdlib/primitives/bigfield/bigfield.hpp"
#include <vector>

namespace acir_format {
using Builder = bb::UltraCircuitBuilder;

using namespace bb;

// In Honk, the proof starts with circuit_size, num_public_inputs, and pub_input_offset. We use this offset to keep
// track of where the public inputs start.
static constexpr size_t HONK_RECURSION_PUBLIC_INPUT_OFFSET = 3;

AggregationObjectIndices create_honk_recursion_constraints(Builder& builder,
                                                           const RecursionConstraint& input,
                                                           AggregationObjectIndices input_aggregation_object,
                                                           bool has_valid_witness_assignments = false);

} // namespace acir_format
