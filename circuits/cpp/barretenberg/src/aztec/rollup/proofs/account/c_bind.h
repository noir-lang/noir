#include <cstdint>

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void account__init_proving_key();

WASM_EXPORT void account__init_proving_key_from_buffer(uint8_t const* pk_buf);

WASM_EXPORT uint32_t account__get_new_proving_key_data(uint8_t** output);

WASM_EXPORT void account__init_verification_key(void* pippenger, uint8_t const* g2x);

WASM_EXPORT void account__init_verification_key_from_buffer(uint8_t const* vk_buf, uint8_t const* g2x);

WASM_EXPORT uint32_t account__get_new_verification_key_data(uint8_t** output);

WASM_EXPORT void account__compute_signing_data(uint8_t const* account_buf, uint8_t* output);

WASM_EXPORT void* account__new_prover(uint8_t const* account_buf);

WASM_EXPORT void account__delete_prover(void* prover);

WASM_EXPORT bool account__verify_proof(uint8_t* proof, uint32_t length);
}
