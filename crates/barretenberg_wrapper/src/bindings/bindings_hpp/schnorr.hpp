#include <stdint.h>
#include <stddef.h>

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void compute_public_key(uint8_t const* private_key, uint8_t* public_key_buf);

WASM_EXPORT void construct_signature(
    uint8_t const* message, size_t msg_len, uint8_t const* private_key, uint8_t* s, uint8_t* e);

WASM_EXPORT bool verify_signature(
    uint8_t const* message, size_t msg_len, uint8_t const* pub_key, uint8_t const* sig_s, uint8_t const* sig_e);
}