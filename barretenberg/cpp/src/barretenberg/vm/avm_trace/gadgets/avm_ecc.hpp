#pragma once

#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/ecc/groups/affine_element.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"

namespace bb::avm_trace {
class AvmEccTraceBuilder {
  public:
    struct EccTraceEntry {
        uint32_t clk = 0;
        std::tuple<FF, FF, bool> p1 = { FF(0), FF(0), true }; // x, y, is_infinity
        std::tuple<FF, FF, bool> p2 = { FF(0), FF(0), true };
        std::tuple<FF, FF, bool> result = { FF(0), FF(0), true };
    };

    AvmEccTraceBuilder();
    void reset();
    // Finalize the trace
    std::vector<EccTraceEntry> finalize();
    grumpkin::g1::affine_element embedded_curve_add(grumpkin::g1::affine_element lhs,
                                                    grumpkin::g1::affine_element rhs,
                                                    uint32_t clk);
    grumpkin::g1::affine_element variable_msm(const std::vector<grumpkin::g1::affine_element>& points,
                                              const std::vector<grumpkin::fr>& scalars,
                                              uint32_t clk);

  private:
    std::vector<EccTraceEntry> ecc_trace;
};

} // namespace bb::avm_trace
