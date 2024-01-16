#include "barretenberg/common/wasm_export.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "blake3s.hpp"

WASM_EXPORT void blake3s_to_field(uint8_t const* data, size_t length, uint8_t* r)
{
    std::vector<uint8_t> inputv(data, data + length);
    std::vector<uint8_t> output = blake3::blake3s(inputv);
    auto result = bb::fr::serialize_from_buffer(output.data());
    bb::fr::serialize_to_buffer(result, r);
}
