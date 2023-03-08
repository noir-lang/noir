#include <common/serialize.hpp>
#include <common/timer.hpp>
#include <common/mem.hpp>
#include <common/streams.hpp>
#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void pedersen__init();

WASM_EXPORT void pedersen__compress_fields(uint8_t const* left, uint8_t const* right, uint8_t* result);

WASM_EXPORT void pedersen__compress(uint8_t const* inputs_buffer, uint8_t* output);

WASM_EXPORT void pedersen__compress_with_hash_index(uint8_t const* inputs_buffer, uint8_t* output, uint32_t hash_index);

WASM_EXPORT void pedersen__commit(uint8_t const* inputs_buffer, uint8_t* output);

WASM_EXPORT void pedersen__buffer_to_field(uint8_t const* data, size_t length, uint8_t* r);

/**
 * Given a buffer containing 32 byte pedersen leaves, return a new buffer containing the leaves and all pairs of
 * nodes that define a merkle tree.
 * e.g.
 * input:  [1][2][3][4]
 * output: [1][2][3][4][compress(1,2)][compress(3,4)][compress(5,6)]
 */
WASM_EXPORT uint8_t* pedersen__hash_to_tree(uint8_t const* data);
}
