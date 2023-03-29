#include <cstdint>
#include <cstddef>

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT size_t root_rollup__init_proving_key(uint8_t const** pk_buf);
WASM_EXPORT size_t root_rollup__init_verification_key(uint8_t const* pk_buf, uint8_t const** vk_buf);
WASM_EXPORT size_t root_rollup__sim(uint8_t const* root_rollup_inputs_buf,
                                    uint8_t const** root_rollup_public_inputs_buf);
WASM_EXPORT size_t root_rollup__verify_proof(uint8_t const* vk_buf,
                                             uint8_t const* proof,
                                             uint32_t length);

}