#pragma once

#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/vm/avm/generated/full_row.hpp"
#include "barretenberg/vm/avm/trace/common.hpp"
#include "barretenberg/vm/avm/trace/opcode.hpp"

#include <array>
#include <cstdint>
#include <optional>
#include <unordered_map>
#include <vector>

namespace bb::avm_trace {

class AvmAluTraceBuilder {

  public:
    struct AluTraceEntry {
        uint32_t alu_clk = 0;

        std::optional<OpCode> opcode = std::nullopt;
        std::optional<AvmMemoryTag> tag = std::nullopt;
        bool alu_op_cast_prev = false;

        FF alu_ia{};
        FF alu_ib{};
        FF alu_ic{};

        bool alu_cf = false;

        uint8_t alu_u8_r0 = 0;
        uint8_t alu_u8_r1 = 0;

        std::array<uint16_t, 15> alu_u16_reg{};

        FF alu_op_eq_diff_inv{};

        // Comparison Operation
        bool borrow = false;

        std::vector<FF> hi_lo_limbs{};
        bool p_a_borrow = false;
        bool p_b_borrow = false;
        uint8_t cmp_rng_ctr = 0;
        bool rng_chk_sel = false;

        // Shift Operations
        uint8_t mem_tag_bits = 0;
        uint8_t mem_tag_sub_shift = 0;
        bool shift_lt_bit_len = true;
        FF quot_div_rem_lo{};
        FF quot_div_rem_hi{};

        // Div Operations
        FF remainder{};
        FF divisor_lo{};
        FF divisor_hi{};
        FF quotient_lo{};
        FF quotient_hi{};
        FF partial_prod_lo{};
        FF partial_prod_hi{};
        bool div_u64_range_chk_sel = false;
        std::array<uint16_t, 8> div_u64_range_chk{};
    };

    std::array<std::unordered_map<uint8_t, uint32_t>, 2> u8_range_chk_counters;
    std::array<std::unordered_map<uint8_t, uint32_t>, 2> u8_pow_2_counters;
    std::array<std::unordered_map<uint16_t, uint32_t>, 15> u16_range_chk_counters;
    std::array<std::unordered_map<uint16_t, uint32_t>, 8> div_u64_range_chk_counters;

    AvmAluTraceBuilder() = default;
    size_t size() const { return alu_trace.size(); }
    void reset();
    void finalize(std::vector<AvmFullRow<FF>>& main_trace);

    // Compute - Arithmetic
    FF op_add(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t clk);
    FF op_sub(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t clk);
    FF op_mul(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t clk);
    FF op_div(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t clk);

    // Compute - Comparators
    FF op_eq(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t clk);
    FF op_lt(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t clk);
    FF op_lte(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t clk);

    // Compute - Bitwise
    FF op_not(FF const& a, AvmMemoryTag in_tag, uint32_t clk);
    FF op_shl(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t clk);
    FF op_shr(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t clk);

    // Compute - Type Conversions
    FF op_cast(FF const& a, AvmMemoryTag in_tag, uint32_t clk);

  private:
    std::vector<AluTraceEntry> alu_trace;
    bool range_checked_required = false;

    template <typename T> std::tuple<uint8_t, uint8_t, std::array<uint16_t, 15>> to_alu_slice_registers(T a);
    std::vector<AluTraceEntry> cmp_range_check_helper(AluTraceEntry row, std::vector<uint256_t> hi_lo_limbs);

    bool is_range_check_required() const;
    static bool is_alu_row_enabled(AvmAluTraceBuilder::AluTraceEntry const& r);
};

} // namespace bb::avm_trace
