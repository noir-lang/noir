#include "blake2s.hpp"
#include <ecc/curves/bn254/fr.hpp>

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void blake2s_to_field(uint8_t const* data, size_t length, uint8_t* r)
{
    std::vector<uint8_t> inputv(data, data + length);
    std::vector<uint8_t> output = blake2::blake2s(inputv);
    auto result = barretenberg::fr::serialize_from_buffer(output.data());
    barretenberg::fr::serialize_to_buffer(result, r);
}

}