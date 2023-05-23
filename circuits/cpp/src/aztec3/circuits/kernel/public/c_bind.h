#include <cstdint>
#include <cstddef>
#include <barretenberg/serialize/cbind_fwd.hpp>

WASM_EXPORT size_t public_kernel__init_proving_key(uint8_t const** pk_buf);
WASM_EXPORT size_t public_kernel__init_verification_key(uint8_t const* pk_buf, uint8_t const** vk_buf);
CBIND_DECL(public_kernel__sim);
WASM_EXPORT uint8_t* public_kernel_no_previous_kernel__sim(uint8_t const* public_kernel_inputs_buf,
                                                           size_t* public_kernel_public_inputs_size_out,
                                                           uint8_t const** public_kernel_public_inputs_buf);