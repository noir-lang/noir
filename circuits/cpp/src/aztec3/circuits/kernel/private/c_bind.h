#pragma once
#include <barretenberg/barretenberg.hpp>

#include <cstddef>
#include <cstdint>

WASM_EXPORT size_t private_kernel__init_proving_key(uint8_t const** pk_buf);
WASM_EXPORT size_t private_kernel__init_verification_key(uint8_t const* pk_buf, uint8_t const** vk_buf);
CBIND_DECL(private_kernel__dummy_previous_kernel);
CBIND_DECL(private_kernel__sim_init);
CBIND_DECL(private_kernel__sim_inner);
CBIND_DECL(private_kernel__sim_ordering);
WASM_EXPORT size_t private_kernel__prove(uint8_t const* tx_request_buf,
                                         uint8_t const* previous_kernel_buf,
                                         uint8_t const* private_call_buf,
                                         uint8_t const* pk_buf,
                                         bool first,
                                         uint8_t const** proof_data_buf);
WASM_EXPORT size_t private_kernel__verify_proof(uint8_t const* vk_buf, uint8_t const* proof, uint32_t length);
