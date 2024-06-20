#pragma once

#include "barretenberg/numeric/uint128/uint128.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"

#include <cstdint>
#include <unordered_map>
#include <vector>

namespace bb::avm_trace {

class AvmConversionTraceBuilder {
  public:
    struct ConversionTraceEntry {
        uint32_t conversion_clk = 0;
        bool to_radix_le_sel = false;
        FF input{};
        uint32_t radix = 0;
        uint32_t num_limbs = 0;
        std::vector<uint8_t> limbs;
    };

    AvmConversionTraceBuilder() = default;
    void reset();
    // Finalize the trace
    std::vector<ConversionTraceEntry> finalize();

    std::vector<uint8_t> op_to_radix_le(FF const& a, uint32_t radix, uint32_t num_limbs, uint32_t clk);

  private:
    std::vector<ConversionTraceEntry> conversion_trace;
};

} // namespace bb::avm_trace
