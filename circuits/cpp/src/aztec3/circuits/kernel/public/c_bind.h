#include <cstdint>
#include <cstddef>

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT size_t public_kernel__init_proving_key(uint8_t const** pk_buf);
WASM_EXPORT size_t public_kernel__init_verification_key(uint8_t const* pk_buf, uint8_t const** vk_buf);
WASM_EXPORT uint8_t* public_kernel__sim(uint8_t const* public_kernel_inputs_buf,
                                        size_t* public_kernel_public_inputs_size_out,
                                        uint8_t const** public_kernel_public_inputs_buf);
WASM_EXPORT uint8_t* public_kernel_no_previous_kernel__sim(uint8_t const* public_kernel_inputs_buf,
                                                           size_t* public_kernel_public_inputs_size_out,
                                                           uint8_t const** public_kernel_public_inputs_buf);
}