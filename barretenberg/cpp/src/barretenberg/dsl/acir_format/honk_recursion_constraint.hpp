#pragma once
#include "barretenberg/dsl/acir_format/recursion_constraint.hpp"
#include "barretenberg/stdlib/primitives/bigfield/bigfield.hpp"
#include <vector>

namespace acir_format {
using Builder = bb::UltraCircuitBuilder;

using namespace bb;

AggregationObjectIndices create_honk_recursion_constraints(Builder& builder,
                                                           const RecursionConstraint& input,
                                                           AggregationObjectIndices input_aggregation_object,
                                                           bool has_valid_witness_assignments = false);

} // namespace acir_format
