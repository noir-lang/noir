//#include "pedersen.hpp"
//#include <common/serialize.hpp>
//#include <common/mem.hpp>
#include <stdint.h>
#include <stddef.h>

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void pedersen_encrypt(uint8_t const* inputs_buffer, uint8_t* output);
WASM_EXPORT void pedersen_compress_fields(uint8_t const* left, uint8_t const* right, uint8_t* result);
WASM_EXPORT void pedersen_compress(uint8_t const* inputs_buffer, uint8_t* output);

WASM_EXPORT void pedersen_compress_with_hash_index(uint8_t const* inputs_buffer, uint8_t* output, uint32_t hash_index);

WASM_EXPORT void pedersen_buffer_to_field(uint8_t const* data, size_t length, uint8_t* r);

// Given a buffer containing 64 byte leaves, return a new buffer containing the leaf hashes and all pairs of nodes that
// define a merkle tree.
WASM_EXPORT size_t pedersen_hash_to_tree(uint8_t const* data, size_t length, uint8_t** output);

}