#pragma once

#include "AvmMini_common.hpp"

namespace avm_trace {

void log_avmMini_trace(std::vector<Row> const& trace, size_t beg, size_t end);

} // namespace avm_trace