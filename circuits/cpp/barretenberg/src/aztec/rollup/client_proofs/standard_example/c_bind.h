#include <cstdint>

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void standard_example__init_proving_key();

WASM_EXPORT void standard_example__init_verification_key(void* pippenger_ptr, uint8_t const* g2x);

WASM_EXPORT void* standard_example__new_prover();

WASM_EXPORT void standard_example__delete_prover(void* prover);

WASM_EXPORT bool standard_example__verify_proof(uint8_t* proof, uint32_t length);

}