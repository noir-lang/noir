#include <cstdint>

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void standard_example_init_keys(uint8_t* point_table, uint32_t num_points, uint8_t const* g2x);

WASM_EXPORT void standard_example_init_proving_key(uint8_t* point_table, uint32_t num_points);

WASM_EXPORT void* standard_example_new_prover();

WASM_EXPORT void standard_example_delete_prover(void* prover);

WASM_EXPORT bool standard_example_verify_proof(uint8_t* proof, uint32_t length);

}