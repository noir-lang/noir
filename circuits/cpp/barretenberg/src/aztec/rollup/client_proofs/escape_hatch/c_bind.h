#include <cstdint>

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void escape_hatch__init_proving_key(void* pippenger, const uint8_t* g2x);

WASM_EXPORT void escape_hatch__init_proving_key_from_buffer(uint8_t const* pk_buf);

WASM_EXPORT uint32_t escape_hatch__get_new_proving_key_data(uint8_t** output);

WASM_EXPORT void escape_hatch__init_verification_key(void* pippenger, uint8_t const* g2x);

WASM_EXPORT void escape_hatch__init_verification_key_from_buffer(uint8_t const* vk_buf, uint8_t const* g2x);

WASM_EXPORT uint32_t escape_hatch__get_new_verification_key_data(uint8_t** output);

WASM_EXPORT void escape_hatch__encrypt_note(uint8_t const* note_buffer, uint8_t* output);

WASM_EXPORT bool escape_hatch__decrypt_note(uint8_t const* encrypted_note_buf,
                                            uint8_t const* private_key_buf,
                                            uint8_t const* viewing_key_buf,
                                            uint8_t* output);

WASM_EXPORT void escape_hatch__sign_2_notes(uint8_t const* note_buffer, uint8_t* pk_buffer, uint8_t* output);

WASM_EXPORT void* escape_hatch__new_prover(uint8_t const* escape_hatch_buf);

WASM_EXPORT void escape_hatch__delete_prover(void* prover);

WASM_EXPORT bool escape_hatch__verify_proof(uint8_t* proof, uint32_t length);
}
