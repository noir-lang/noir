#pragma once

#include "barretenberg/vm/avm_trace/avm_common.hpp"

#include <cstdint>
#include <vector>

namespace bb::avm_trace {

class AvmPedersenTraceBuilder {
  public:
    struct PedersenTraceEntry {
        uint32_t clk = 0;
        std::vector<FF> input;
        FF output;
    };

    AvmPedersenTraceBuilder() = default;
    void reset();
    // Finalize the trace
    std::vector<PedersenTraceEntry> finalize();

    // Note that this version of pedersen_hash is defined over Grumpkin (we could make it generic later if we wanted to
    // also support BBJubJub) The inputs are Fr_BN254, and the output is Fq_Grumpkin (which is also Fr_BN254 since
    // Grumpkin and BN254 form a 2-cycle).
    FF pedersen_hash(const std::vector<FF>& inputs, uint32_t offset, uint32_t clk);

  private:
    std::vector<PedersenTraceEntry> pedersen_trace;
};

} // namespace bb::avm_trace
