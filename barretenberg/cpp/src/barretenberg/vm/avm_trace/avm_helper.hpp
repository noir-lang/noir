#pragma once

#include "avm_common.hpp"

namespace avm_trace {

void log_avm_trace(std::vector<Row> const& trace, size_t beg, size_t end);

} // namespace avm_trace