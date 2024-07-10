#pragma once

#include <filesystem>

#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/avm_trace/avm_trace.hpp"

namespace bb::avm_trace {

void log_avm_trace(std::vector<Row> const& trace, size_t beg, size_t end, bool enable_selectors = false);
void dump_trace_as_csv(const std::vector<Row>& trace, const std::filesystem::path& filename);

bool is_operand_indirect(uint8_t ind_value, uint8_t operand_idx);

// Copy Public Input Columns
// There are 5 public input columns, one for inputs, one for returndata and 3 for the kernel outputs
// {value, side effect counter, metadata}. The verifier is generic, and so accepts vectors of these values
// rather than the fixed length arrays that are used during circuit building. This method copies each array
// into a vector to be used by the verifier.
std::vector<std::vector<FF>> copy_public_inputs_columns(VmPublicInputs const& public_inputs,
                                                        std::vector<FF> const& calldata,
                                                        std::vector<FF> const& returndata);

} // namespace bb::avm_trace