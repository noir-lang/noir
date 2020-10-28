#include "pedersen.hpp"
#include <common/serialize.hpp>
#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void pedersen_compress_fields(uint8_t const* left, uint8_t const* right, uint8_t* result)
{
    auto lhs = barretenberg::fr::serialize_from_buffer(left);
    auto rhs = barretenberg::fr::serialize_from_buffer(right);
    auto r = crypto::pedersen::compress_native({ lhs, rhs });
    barretenberg::fr::serialize_to_buffer(r, result);
}

WASM_EXPORT void pedersen_compress(uint8_t const* inputs_buffer, uint8_t* output)
{
    std::vector<grumpkin::fq> to_compress;
    read(inputs_buffer, to_compress);
    auto r = crypto::pedersen::compress_native(to_compress);
    barretenberg::fr::serialize_to_buffer(r, output);
}

WASM_EXPORT void pedersen_compress_with_hash_index(uint8_t const* inputs_buffer, uint8_t* output, uint32_t hash_index)
{
    std::vector<grumpkin::fq> to_compress;
    read(inputs_buffer, to_compress);
    auto r = crypto::pedersen::compress_native(to_compress, hash_index);
    barretenberg::fr::serialize_to_buffer(r, output);
}

WASM_EXPORT void pedersen_buffer_to_field(uint8_t const* data, size_t length, uint8_t* r)
{
    std::vector<uint8_t> to_compress(data, data + length);
    
    std::vector<uint8_t> output = crypto::pedersen::compress_native(to_compress);
    auto result = barretenberg::fr::serialize_from_buffer(output.data());
    barretenberg::fr::serialize_to_buffer(result, r);
}
}