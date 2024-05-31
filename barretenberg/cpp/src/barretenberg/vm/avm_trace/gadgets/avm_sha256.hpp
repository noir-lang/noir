#pragma once

#include "barretenberg/vm/avm_trace/avm_common.hpp"

#include <array>
#include <cstdint>
#include <vector>

namespace bb::avm_trace {
class AvmSha256TraceBuilder {
  public:
    struct Sha256TraceEntry {
        uint32_t clk = 0;
        std::array<uint32_t, 8> state{};
        std::array<uint32_t, 16> input{};
        std::array<uint32_t, 8> output{};
    };

    AvmSha256TraceBuilder();
    void reset();
    // Finalize the trace
    std::vector<Sha256TraceEntry> finalize();

    std::array<uint32_t, 8> sha256_compression(const std::array<uint32_t, 8>& h_init,
                                               const std::array<uint32_t, 16>& input,
                                               uint32_t clk);
    std::array<uint8_t, 32> sha256(const std::vector<uint8_t>& input, uint32_t clk);

  private:
    std::vector<Sha256TraceEntry> sha256_trace;
};

} // namespace bb::avm_trace
