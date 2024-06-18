#include "barretenberg/vm/avm_trace/gadgets/avm_ecc.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"

namespace bb::avm_trace {
using element = grumpkin::g1::affine_element;

AvmEccTraceBuilder::AvmEccTraceBuilder()
{
    ecc_trace.reserve(AVM_TRACE_SIZE);
}

std::vector<AvmEccTraceBuilder::EccTraceEntry> AvmEccTraceBuilder::finalize()
{
    return std::move(ecc_trace);
}

void AvmEccTraceBuilder::reset()
{
    ecc_trace.clear();
}

element AvmEccTraceBuilder::embedded_curve_add(element lhs, element rhs, uint32_t clk)
{
    element result = lhs + rhs;
    std::tuple<FF, FF, bool> p1 = { lhs.x, lhs.y, lhs.is_point_at_infinity() };
    std::tuple<FF, FF, bool> p2 = { rhs.x, rhs.y, rhs.is_point_at_infinity() };
    std::tuple<FF, FF, bool> result_tuple = { result.x, result.y, result.is_point_at_infinity() };
    ecc_trace.push_back({ clk, p1, p2, result_tuple });

    return result;
}

element AvmEccTraceBuilder::variable_msm(const std::vector<element>& points,
                                         const std::vector<grumpkin::fr>& scalars,
                                         uint32_t clk)
{
    // Replace this with pippenger if/when we have the time
    auto result = grumpkin::g1::affine_point_at_infinity;
    for (size_t i = 0; i < points.size(); ++i) {
        result = result + points[i] * scalars[i];
    }

    std::tuple<FF, FF, bool> result_tuple = { result.x, result.y, result.is_point_at_infinity() };

    ecc_trace.push_back({ .clk = clk, .result = result_tuple });

    return result;
}

} // namespace bb::avm_trace
