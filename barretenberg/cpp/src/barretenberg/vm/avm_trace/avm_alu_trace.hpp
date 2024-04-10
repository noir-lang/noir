#pragma once

#include "avm_common.hpp"
#include "barretenberg/numeric/uint128/uint128.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include <array>
#include <unordered_map>

namespace bb::avm_trace {

class AvmAluTraceBuilder {

  public:
    struct AluTraceEntry {
        uint32_t alu_clk{};

        bool alu_op_add = false;
        bool alu_op_sub = false;
        bool alu_op_mul = false;
        bool alu_op_not = false;
        bool alu_op_eq = false;
        bool alu_op_lt = false;
        bool alu_op_lte = false;

        bool alu_ff_tag = false;
        bool alu_u8_tag = false;
        bool alu_u16_tag = false;
        bool alu_u32_tag = false;
        bool alu_u64_tag = false;
        bool alu_u128_tag = false;

        FF alu_ia{};
        FF alu_ib{};
        FF alu_ic{};

        bool alu_cf = false;

        uint8_t alu_u8_r0{};
        uint8_t alu_u8_r1{};

        std::array<uint16_t, 15> alu_u16_reg{};

        uint64_t alu_u64_r0{};

        FF alu_op_eq_diff_inv{};

        // Comparison Operation
        bool borrow = false;

        std::vector<FF> hi_lo_limbs{};
        bool p_a_borrow = false;
        bool p_b_borrow = false;
        uint8_t cmp_rng_ctr = 0;
        bool rng_chk_sel = false;
    };

    std::array<std::unordered_map<uint8_t, uint32_t>, 2> u8_range_chk_counters;
    std::array<std::unordered_map<uint16_t, uint32_t>, 15> u16_range_chk_counters;

    AvmAluTraceBuilder();
    void reset();
    std::vector<AluTraceEntry> finalize();

    FF op_add(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t clk);
    FF op_sub(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t clk);
    FF op_mul(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t clk);
    FF op_not(FF const& a, AvmMemoryTag in_tag, uint32_t clk);
    FF op_eq(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t clk);
    FF op_lt(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t clk);
    FF op_lte(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t clk);

  private:
    std::vector<AluTraceEntry> alu_trace;
    template <typename T> std::tuple<uint8_t, uint8_t, std::vector<uint16_t>> to_alu_slice_registers(T a);
    std::vector<AluTraceEntry> cmp_range_check_helper(AluTraceEntry row, std::vector<uint256_t> hi_lo_limbs);
    void count_range_checks(AluTraceEntry const& entry);
};
} // namespace bb::avm_trace
