#include "barretenberg/vm/avm_trace/gadgets/avm_poseidon2.hpp"
#include "barretenberg/crypto/poseidon2/poseidon2_permutation.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"

namespace bb::avm_trace {

std::vector<AvmPoseidon2TraceBuilder::Poseidon2TraceEntry> AvmPoseidon2TraceBuilder::finalize()
{
    return std::move(poseidon2_trace);
}

void AvmPoseidon2TraceBuilder::reset()
{
    poseidon2_trace.clear();
}

std::array<FF, 4> AvmPoseidon2TraceBuilder::poseidon2_permutation(const std::array<FF, 4>& input, uint32_t clk)
{
    std::array<FF, 4> output =
        crypto::Poseidon2Permutation<crypto::Poseidon2Bn254ScalarFieldParams>::permutation(input);

    poseidon2_trace.push_back(Poseidon2TraceEntry{ clk, input, output });

    return output;
}

} // namespace bb::avm_trace
