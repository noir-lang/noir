#include <cstdint>

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void create_note__init_proving_key();

WASM_EXPORT void create_note__init_verification_key(void* pippenger, uint8_t const* g2x);

WASM_EXPORT void create_note__encrypt_note(uint8_t const* owner_buf, uint32_t value, uint8_t const* viewing_key_buf, uint8_t* output);

WASM_EXPORT void* create_note__new_prover(uint8_t const* owner_buf,
                                   uint32_t value,
                                   uint8_t const* viewing_key_buf,
                                   uint8_t const* sig_s,
                                   uint8_t const* sig_e);

WASM_EXPORT void create_note__delete_prover(void* prover);

WASM_EXPORT bool create_note__verify_proof(uint8_t* proof, uint32_t length);

}