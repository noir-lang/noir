#include "barretenberg/vm/avm_trace/gadgets/avm_slice_trace.hpp"

#include <cstddef>
#include <cstdint>

namespace bb::avm_trace {

void AvmSliceTraceBuilder::reset()
{
    slice_trace.clear();
    cd_lookup_counts.clear();
    ret_lookup_counts.clear();
}

std::vector<AvmSliceTraceBuilder::SliceTraceEntry> AvmSliceTraceBuilder::finalize()
{
    return std::move(slice_trace);
}

void AvmSliceTraceBuilder::create_calldata_copy_slice(std::vector<FF> const& calldata,
                                                      uint32_t clk,
                                                      uint8_t space_id,
                                                      uint32_t col_offset,
                                                      uint32_t copy_size,
                                                      uint32_t direct_dst_offset)
{
    create_slice(calldata, clk, space_id, col_offset, copy_size, direct_dst_offset, true);
}

void AvmSliceTraceBuilder::create_return_slice(
    std::vector<FF> const& returndata, uint32_t clk, uint8_t space_id, uint32_t direct_ret_offset, uint32_t ret_size)
{
    create_slice(returndata, clk, space_id, 0, ret_size, direct_ret_offset, false);
}

void AvmSliceTraceBuilder::create_slice(std::vector<FF> const& col_data,
                                        uint32_t clk,
                                        uint8_t space_id,
                                        uint32_t col_offset,
                                        uint32_t copy_size,
                                        uint32_t addr,
                                        bool rw)
{
    for (uint32_t i = 0; i < copy_size; i++) {
        slice_trace.push_back({
            .clk = clk,
            .space_id = space_id,
            .addr_ff = FF(addr + i),
            .val = col_data.at(col_offset + i),
            .col_offset = col_offset + i,
            .cnt = copy_size - i,
            .one_min_inv = FF(1) - FF(copy_size - i).invert(),
            .sel_start = i == 0,
            .sel_cd_cpy = rw,
            .sel_return = !rw,
        });

        rw ? cd_lookup_counts[col_offset + i]++ : ret_lookup_counts[col_offset + i]++;
    }

    // Last extra row for a slice operation. cnt is zero and we have to add extra dummy
    // values for addr and col_offset to satisfy the constraints: #[ADDR_INCREMENT] and #[COL_OFFSET_INCREMENT]
    // Alternatively, we would have to increase the degree of these two relations.
    // Note that addr = 2^32 would be a valid value here, therefore we do not wrap modulo 2^32.
    // col_offset is fine as the circuit trace cannot reach a size of 2^32.
    slice_trace.emplace_back(SliceTraceEntry{
        .clk = clk,
        .space_id = space_id,
        .addr_ff = FF(addr + copy_size - 1) + 1,
        .col_offset = col_offset + copy_size,
    });
}

} // namespace bb::avm_trace