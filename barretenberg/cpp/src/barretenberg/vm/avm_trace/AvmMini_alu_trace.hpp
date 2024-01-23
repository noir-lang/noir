#pragma once

#include "AvmMini_common.hpp"

namespace avm_trace {

class AvmMiniAluTraceBuilder {

  public:
    struct AluTraceEntry {
        uint32_t alu_clk{};

        bool alu_op_add = false;
        bool alu_op_sub = false;
        bool alu_op_mul = false;

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
    };

    AvmMiniAluTraceBuilder();
    void reset();
    std::vector<AluTraceEntry> finalize();

    FF add(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t clk);
    FF sub(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t clk);
    FF mul(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t clk);

  private:
    std::vector<AluTraceEntry> alu_trace;
};
} // namespace avm_trace