#include "barretenberg/vm/avm_trace/gadgets/avm_keccak.hpp"
#include "barretenberg/crypto/hashers/hashers.hpp"
#include "barretenberg/crypto/keccak/keccak.hpp"

#include <algorithm>
#include <cstdint>

namespace bb::avm_trace {

AvmKeccakTraceBuilder::AvmKeccakTraceBuilder()
{
    keccak_trace.reserve(AVM_TRACE_SIZE);
}

std::vector<AvmKeccakTraceBuilder::KeccakTraceEntry> AvmKeccakTraceBuilder::finalize()
{
    return std::move(keccak_trace);
}

void AvmKeccakTraceBuilder::reset()
{
    keccak_trace.clear();
}

std::array<uint64_t, 25> AvmKeccakTraceBuilder::keccakf1600(uint32_t clk, std::array<uint64_t, 25> input)
{
    // BB's Eth hash function uses C-style arrays, while we like to use std::array
    // We do a few conversions for here but maybe we will update later.
    uint64_t state[25] = {};
    std::copy(input.begin(), input.end(), state);
    std::vector<uint64_t> input_vector(input.begin(), input.end());
    // This function mutates state
    ethash_keccakf1600(state);
    std::array<uint64_t, 25> output = {};
    for (size_t i = 0; i < 25; i++) {
        output[i] = state[i];
    }
    std::vector<uint64_t> output_vector(output.begin(), output.end());
    keccak_trace.push_back(KeccakTraceEntry{
        .clk = clk,
        .input = input_vector,
        .output = output_vector,
        .input_size = 25,
        .output_size = 25,
    });
    return output;
}

std::array<uint8_t, 32> AvmKeccakTraceBuilder::keccak(uint32_t clk, std::vector<uint8_t> input, uint32_t size)
{
    // Pad input to a multiple of 8 bytes
    if (!input.empty()) {
        input.resize(8 * ((input.size() - 1) / 8 + 1));
    }

    // We treat the input vector as an array of 64-bit integers for the avm (even though keccak takes in bytes).
    std::vector<uint64_t> vector_input;
    for (size_t i = 0; i < input.size(); i += 8) {
        auto uint64 = from_buffer<uint64_t>(input, i);
        vector_input.push_back(uint64);
    }
    auto result = ethash_keccak256(&input[0], size);
    std::vector<uint64_t> output_vector = {};
    std::array<uint8_t, 32> output_bytes = {};
    // The result encodes each limb in LE, we need to swap them to BE
    // If we had C++23 we could use std::byteswap, but instead we write our own
    for (size_t i = 0; i < 4; i++) {
        std::vector<uint8_t> le_bytes = to_buffer(result.word64s[i]);
        // Reverse the bytes
        std::ranges::reverse(le_bytes);
        // Convert the bytes back to a uint64_t
        auto be_u64 = from_buffer<uint64_t>(le_bytes);
        output_vector.push_back(be_u64);
        // Copy the bytes to the output
        for (size_t j = 0; j < 8; j++) {
            output_bytes[i * 8 + j] = le_bytes[j];
        }
    }
    keccak_trace.push_back(KeccakTraceEntry{
        .clk = clk,
        .input = vector_input,
        .output = output_vector,
        .input_size = size,
        .output_size = 4,
    });
    return output_bytes;
}

} // namespace bb::avm_trace
