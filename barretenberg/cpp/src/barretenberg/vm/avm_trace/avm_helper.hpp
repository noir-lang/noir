#pragma once

#include "avm_common.hpp"

namespace bb::avm_trace {

void log_avm_trace(std::vector<Row> const& trace, size_t beg, size_t end, bool enable_selectors = false);

bool is_operand_indirect(uint8_t ind_value, uint8_t operand_idx);

} // namespace bb::avm_trace