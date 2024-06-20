#include "barretenberg/vm/avm_trace/gadgets/avm_pedersen.hpp"
#include "barretenberg/crypto/pedersen_hash/pedersen.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"

namespace bb::avm_trace {

std::vector<AvmPedersenTraceBuilder::PedersenTraceEntry> AvmPedersenTraceBuilder::finalize()
{
    return std::move(pedersen_trace);
}

void AvmPedersenTraceBuilder::reset()
{
    pedersen_trace.clear();
}

FF AvmPedersenTraceBuilder::pedersen_hash(const std::vector<FF>& inputs, uint32_t offset, uint32_t clk)
{
    crypto::GeneratorContext<curve::Grumpkin> ctx;
    ctx.offset = offset;
    // Use the standard domain separator starting at ctx.offset
    FF output = crypto::pedersen_hash::hash(inputs, ctx);
    pedersen_trace.push_back({ clk, inputs, output });

    return output;
}

} // namespace bb::avm_trace
