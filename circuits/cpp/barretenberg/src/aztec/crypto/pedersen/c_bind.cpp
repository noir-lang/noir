#include "pedersen.hpp"

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void pedersen_compress_fields(uint8_t const* left, uint8_t const* right, uint8_t* result)
{
    auto lhs = barretenberg::fr::serialize_from_buffer(left);
    auto rhs = barretenberg::fr::serialize_from_buffer(right);
    auto r = crypto::pedersen::compress_native({ lhs, rhs });
    barretenberg::fr::serialize_to_buffer(r, result);
}

}