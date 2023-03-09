#include "aes128.hpp"

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void aes__encrypt_buffer_cbc(uint8_t* in, uint8_t* iv, const uint8_t* key, const size_t length, uint8_t* r)
{
    crypto::aes128::encrypt_buffer_cbc(in, iv, key, length);
    for (size_t i = 0; i < length; ++i) {
        r[i] = in[i];
    }
}

WASM_EXPORT void aes__decrypt_buffer_cbc(uint8_t* in, uint8_t* iv, const uint8_t* key, const size_t length, uint8_t* r)
{
    crypto::aes128::decrypt_buffer_cbc(in, iv, key, length);
    for (size_t i = 0; i < length; ++i) {
        r[i] = in[i];
    }
}
}