#include <cstdint>

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void notes__sign_4_notes(uint8_t* pk_buffer,
                                     uint8_t const* output_owner_buffer,
                                     uint8_t const* note_buffer,
                                     uint8_t* output);

WASM_EXPORT void notes__encrypt_note(uint8_t const* note_buffer, uint8_t* output);

WASM_EXPORT bool notes__decrypt_note(uint8_t const* encrypted_note_buf,
                                     uint8_t const* private_key_buf,
                                     uint8_t const* viewing_key_buf,
                                     uint8_t* output);
}
