#pragma once

#include "barretenberg/vm/avm_trace/avm_common.hpp"

#include <array>
#include <cstdint>
#include <vector>

namespace bb::avm_trace {

class AvmPoseidon2TraceBuilder {
  public:
    struct Poseidon2TraceEntry {
        uint32_t clk = 0;
        std::array<FF, 4> input;
        std::array<FF, 4> output;
    };

    AvmPoseidon2TraceBuilder() = default;
    void reset();
    // Finalize the trace
    std::vector<Poseidon2TraceEntry> finalize();

    std::array<FF, 4> poseidon2_permutation(const std::array<FF, 4>& input, uint32_t clk);

  private:
    std::vector<Poseidon2TraceEntry> poseidon2_trace;
};

} // namespace bb::avm_trace
