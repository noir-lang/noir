
#pragma once

#include "barretenberg/vm/avm/generated/relations/cmp.hpp"
#include "barretenberg/vm/avm/trace/common.hpp"
#include "barretenberg/vm/avm/trace/gadgets/range_check.hpp"
#include <cstdint>

enum class CmpOp { EQ, GT };

namespace bb::avm_trace {
class AvmCmpBuilder {
  public:
    struct CmpEvent {
        uint64_t clk;
        FF input_a;
        FF input_b;
        EventEmitter emitter;
        CmpOp op;
    };

    struct CmpEntry {
        uint64_t clk;
        FF input_a;
        FF input_b;
        FF result;
        FF op_eq_diff_inv;
        bool is_gt;
        bool is_eq;
        std::tuple<FF, FF> a_limbs;
        std::tuple<FF, FF> b_limbs;
        // Tuple of lo, hi and borrow
        std::tuple<FF, FF, FF> p_sub_a_limbs;
        std::tuple<FF, FF, FF> p_sub_b_limbs;
        std::tuple<FF, FF, FF> gt_result_limbs;
    };

    // Useful since we need to 'unroll' cmp entries over multiple rows
    struct CmpRow {
        FF clk;
        FF result;
        FF op_eq_diff_inv;
        FF op_gt;
        FF op_eq;
        FF a_lo;
        FF a_hi;
        FF b_lo;
        FF b_hi;
        FF p_sub_a_lo;
        FF p_sub_a_hi;
        FF p_a_borrow;
        FF p_sub_b_lo;
        FF p_sub_b_hi;
        FF p_b_borrow;
        FF res_lo;
        FF res_hi;
        FF borrow;
        FF input_a;
        FF input_b;
        FF sel_cmp;
        FF cmp_rng_ctr;
        FF range_chk_clk;
        FF sel_rng_chk;
        FF shift_sel;
    };

    AvmRangeCheckBuilder range_check_builder;

    bool constrained_eq(FF a, FF b, uint64_t clk, EventEmitter e);
    // Constrains a > b
    bool constrained_gt(FF a, FF b, uint64_t clk, EventEmitter e);

    uint32_t get_cmp_trace_size() const;

    // Turns cmp events into real entries
    std::vector<CmpEntry> finalize();

    std::vector<CmpRow> into_canonical(std::vector<CmpEntry> const& entries) const;

    template <typename DestRow> void merge_into(DestRow& row, const CmpRow& entry)
    {
        row.cmp_clk = entry.clk;
        row.cmp_result = entry.result;
        row.cmp_op_eq_diff_inv = entry.op_eq_diff_inv;
        row.cmp_op_gt = entry.op_gt;
        row.cmp_op_eq = entry.op_eq;

        row.cmp_a_lo = entry.a_lo;
        row.cmp_a_hi = entry.a_hi;
        row.cmp_b_lo = entry.b_lo;
        row.cmp_b_hi = entry.b_hi;

        row.cmp_p_sub_a_lo = entry.p_sub_a_lo;
        row.cmp_p_sub_a_hi = entry.p_sub_a_hi;
        row.cmp_p_a_borrow = entry.p_a_borrow;

        row.cmp_p_sub_b_lo = entry.p_sub_b_lo;
        row.cmp_p_sub_b_hi = entry.p_sub_b_hi;
        row.cmp_p_b_borrow = entry.p_b_borrow;

        row.cmp_res_lo = entry.res_lo;
        row.cmp_res_hi = entry.res_hi;
        row.cmp_borrow = entry.borrow;

        row.cmp_input_a = entry.input_a;
        row.cmp_input_b = entry.input_b;
        row.cmp_result = entry.result;
        row.cmp_sel_cmp = entry.sel_cmp;

        row.cmp_cmp_rng_ctr = entry.cmp_rng_ctr;
        row.cmp_range_chk_clk = entry.range_chk_clk;
        row.cmp_sel_rng_chk = entry.sel_rng_chk;
        row.cmp_shift_sel = entry.shift_sel;
    }

  private:
    std::vector<CmpEvent> cmp_events;
};
} // namespace bb::avm_trace
