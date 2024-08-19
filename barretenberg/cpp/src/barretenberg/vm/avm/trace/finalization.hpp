#pragma once

#include "barretenberg/common/assert.hpp"
#include "barretenberg/vm/avm/generated/full_row.hpp"
#include "barretenberg/vm/avm/trace/common.hpp"

#include <cstdint>
#include <vector>

namespace bb::avm_trace {

/**
 * @brief Iterates over the main trace and an event trace and performs an action.
 * @details This function iterates on the main trace and an event trace and applies:
 * - `func_map` when the main trace clk matches the event trace clk.
 * - `func_all` for all rows that are EXECUTION trace rows.
 * This function assumes that the clks in the traces are monotonically increasing.
 */
template <typename S, typename M, typename A>
void iterate_with_actions(const S& src, std::vector<AvmFullRow<FF>>& main_trace, M&& func_map, A&& func_all)
{
    size_t src_idx = 0;
    size_t dst_idx = 0;
    while (src_idx < src.size() && dst_idx < main_trace.size()) {
        if (FF(src.at(src_idx).clk) == main_trace.at(dst_idx).main_clk) {
            func_map(src_idx, dst_idx);
            ++src_idx;
        }
        ++dst_idx;
    }

    for (size_t dst_idx = 0; dst_idx < main_trace.size(); ++dst_idx) {
        if (main_trace.at(dst_idx).main_sel_execution_row == 1) {
            func_all(dst_idx);
        }
    }
}

} // namespace bb::avm_trace