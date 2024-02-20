#pragma once

#include "avm_common.hpp"

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

        std::array<uint16_t, 8> alu_u16_reg{};

        uint64_t alu_u64_r0{};

        FF alu_op_eq_diff_inv{};
    };

    AvmAluTraceBuilder();
    void reset();
    std::vector<AluTraceEntry> finalize();

    FF op_add(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t clk);
    FF op_sub(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t clk);
    FF op_mul(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t clk);
    FF op_not(FF const& a, AvmMemoryTag in_tag, uint32_t clk);
    FF op_eq(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t clk);

  private:
    std::vector<AluTraceEntry> alu_trace;
};
} // namespace bb::avm_trace
