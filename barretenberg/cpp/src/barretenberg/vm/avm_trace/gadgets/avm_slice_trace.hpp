#pragma once

#include "barretenberg/vm/avm_trace/avm_common.hpp"

#include <cstdint>

namespace bb::avm_trace {

class AvmSliceTraceBuilder {

  public:
    // Keeps track of the number of times a calldata/returndata value is copied.
    // column offset -> count
    std::unordered_map<uint32_t, uint32_t> cd_lookup_counts;
    std::unordered_map<uint32_t, uint32_t> ret_lookup_counts;

    struct SliceTraceEntry {
        uint32_t clk = 0;
        uint8_t space_id = 0;
        FF addr_ff = 0; // Should normally be uint32_t but the last witness addr of a calldatacopy/return operation
                        // row might be FF(2^32).
        FF val{};
        uint32_t col_offset = 0;
        uint32_t cnt = 0;
        FF one_min_inv{};

        bool sel_start = false;
        bool sel_cd_cpy = false;
        bool sel_return = false;
    };

    AvmSliceTraceBuilder() = default;

    void reset();
    std::vector<SliceTraceEntry> finalize();

    void create_calldata_copy_slice(std::vector<FF> const& calldata,
                                    uint32_t clk,
                                    uint8_t space_id,
                                    uint32_t col_offset,
                                    uint32_t copy_size,
                                    uint32_t direct_dst_offset);
    void create_return_slice(std::vector<FF> const& returndata,
                             uint32_t clk,
                             uint8_t space_id,
                             uint32_t direct_ret_offset,
                             uint32_t ret_size);

  private:
    std::vector<SliceTraceEntry> slice_trace;
    void create_slice(std::vector<FF> const& col_data,
                      uint32_t clk,
                      uint8_t space_id,
                      uint32_t col_offset,
                      uint32_t copy_size,
                      uint32_t addr,
                      bool rw);
};

template <typename DestRow> void merge_into(DestRow& dest, AvmSliceTraceBuilder::SliceTraceEntry const& src)
{
    dest.slice_clk = src.clk;
    dest.slice_space_id = src.space_id;
    dest.slice_addr = src.addr_ff;
    dest.slice_val = src.val;
    dest.slice_col_offset = src.col_offset;
    dest.slice_cnt = src.cnt;
    dest.slice_one_min_inv = src.one_min_inv;
    dest.slice_sel_start = static_cast<uint32_t>(src.sel_start);
    dest.slice_sel_cd_cpy = static_cast<uint32_t>(src.sel_cd_cpy);
    dest.slice_sel_return = static_cast<uint32_t>(src.sel_return);
    dest.slice_sel_mem_active = static_cast<uint32_t>(src.sel_return || src.sel_cd_cpy);
}

} // namespace bb::avm_trace
