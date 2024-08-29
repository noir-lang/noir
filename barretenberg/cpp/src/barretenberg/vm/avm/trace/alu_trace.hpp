#pragma once

#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/vm/avm/generated/full_row.hpp"
#include "barretenberg/vm/avm/trace/common.hpp"
#include "barretenberg/vm/avm/trace/gadgets/cmp.hpp"
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

        // Registers / Inputs
        FF alu_ia{};
        FF alu_ib{};
        FF alu_ic{};
        // Input Limbs
        FF alu_a_lo{};
        FF alu_a_hi{};
        FF alu_b_lo{};
        FF alu_b_hi{};
        FF alu_c_lo{};
        FF alu_c_hi{};

        // Partial Products for Integer Multiplication
        FF partial_prod_lo{};
        FF partial_prod_hi{};

        // Carry Flag
        bool alu_cf = false;

        // Shift Operations
        uint8_t mem_tag_bits = 0;
        uint8_t mem_tag_sub_shift = 0;
        bool zero_shift = true;

        // Div Operations
        FF remainder{};

        // Range Gadget - we don't need to track the output since has to be 1
        FF range_check_input{};
        FF range_check_num_bits{};
        bool range_check_sel{};

        // Comparison gadget
        FF cmp_input_a{};
        FF cmp_input_b{};
        FF cmp_result{};
        bool cmp_op_is_gt = false;
        bool cmp_op_is_eq = false;
    };

    std::array<std::unordered_map<uint8_t, uint32_t>, 2> u8_range_chk_counters;
    std::array<std::unordered_map<uint8_t, uint32_t>, 2> u8_pow_2_counters;

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

    AvmCmpBuilder cmp_builder;

  private:
    std::vector<AluTraceEntry> alu_trace;
    bool range_checked_required = false;

    bool is_range_check_required() const;
    static bool is_alu_row_enabled(AvmAluTraceBuilder::AluTraceEntry const& r);
};

} // namespace bb::avm_trace
