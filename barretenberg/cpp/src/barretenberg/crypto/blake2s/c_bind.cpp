#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "blake2s.hpp"
#include <barretenberg/common/wasm_export.hpp>

using namespace bb;

extern "C" {

WASM_EXPORT void blake2s(uint8_t const* data, out_buf32 out)
{
    std::vector<uint8_t> inputv;
    read(data, inputv);
    auto output = blake2::blake2s(inputv);
    std::copy(output.begin(), output.end(), out);
}

WASM_EXPORT void blake2s_to_field(uint8_t const* data, size_t length, uint8_t* r)
{
    std::vector<uint8_t> inputv(data, data + length);
    auto output = blake2::blake2s(inputv);
    auto result = bb::fr::serialize_from_buffer(output.data());
    bb::fr::serialize_to_buffer(result, r);
}

// Underscore to not conflict with old cbind. Remove the above when right.
WASM_EXPORT void blake2s_to_field_(uint8_t const* data, fr::out_buf r)
{
    std::vector<uint8_t> inputv;
    read(data, inputv);
    auto output = blake2::blake2s(inputv);
    auto result = bb::fr::serialize_from_buffer(output.data());
    bb::fr::serialize_to_buffer(result, r);
}
}
