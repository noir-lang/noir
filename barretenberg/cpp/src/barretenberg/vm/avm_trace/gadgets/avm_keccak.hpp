#pragma once

#include "barretenberg/vm/avm_trace/avm_common.hpp"

#include <array>
#include <cstdint>
#include <vector>

namespace bb::avm_trace {

class AvmKeccakTraceBuilder {
  public:
    struct KeccakTraceEntry {
        uint32_t clk = 0;
        std::vector<uint64_t> input;
        std::vector<uint64_t> output;
        uint32_t input_size = 0;
        uint32_t output_size = 0;
    };

    AvmKeccakTraceBuilder() = default;
    void reset();
    // Finalize the trace
    std::vector<KeccakTraceEntry> finalize();

    std::array<uint64_t, 25> keccakf1600(uint32_t clk, std::array<uint64_t, 25> input);
    std::array<uint8_t, 32> keccak(uint32_t clk, std::vector<uint8_t> input, uint32_t size);

  private:
    std::vector<KeccakTraceEntry> keccak_trace;
};

} // namespace bb::avm_trace
