#include <cstdint>

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void init_keys(uint8_t const* monomials_buf, uint32_t monomials_buf_size, uint8_t const* g2x);

WASM_EXPORT void init_proving_key(uint8_t const* monomials_buf, uint32_t monomials_buf_size);

WASM_EXPORT void encrypt_note(uint8_t const* owner_buf, uint32_t value, uint8_t const* viewing_key_buf, uint8_t* output);

WASM_EXPORT void create_note_proof(uint8_t const* owner_buf,
                                   uint32_t value,
                                   uint8_t const* viewing_key_buf,
                                   uint8_t const* sig_s,
                                   uint8_t const* sig_e,
                                   uint8_t* proof_data_buf);

WASM_EXPORT bool verify_proof(uint8_t* proof, uint32_t length);

}