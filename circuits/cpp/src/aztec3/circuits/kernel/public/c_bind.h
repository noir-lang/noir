#include <barretenberg/barretenberg.hpp>

#include <cstddef>
#include <cstdint>

WASM_EXPORT size_t public_kernel__init_proving_key(uint8_t const** pk_buf);
WASM_EXPORT size_t public_kernel__init_verification_key(uint8_t const* pk_buf, uint8_t const** vk_buf);
CBIND_DECL(public_kernel__sim);
