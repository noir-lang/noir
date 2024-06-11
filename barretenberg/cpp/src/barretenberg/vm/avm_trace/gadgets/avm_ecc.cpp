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

} // namespace bb::avm_trace
