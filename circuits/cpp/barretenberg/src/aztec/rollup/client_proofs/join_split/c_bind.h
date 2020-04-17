#include <cstdint>

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void join_split__init_proving_key();

WASM_EXPORT void join_split__init_verification_key(void* pippenger, uint8_t const* g2x);

WASM_EXPORT void join_split__encrypt_note(uint8_t* note_buffer, uint8_t* output);

WASM_EXPORT void* join_split__new_prover(uint8_t* join_split_buf, uint32_t buf_length);

WASM_EXPORT void join_split__delete_prover(void* prover);

WASM_EXPORT bool join_split__verify_proof(uint8_t* proof, uint32_t length);

}