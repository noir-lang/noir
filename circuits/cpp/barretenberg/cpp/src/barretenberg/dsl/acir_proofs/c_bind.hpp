#include <cstdint>
#include <cstddef>

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT size_t acir_proofs_get_solidity_verifier(uint8_t const* g2x, uint8_t const* vk_buf, uint8_t** output_buf);
WASM_EXPORT uint32_t acir_proofs_get_exact_circuit_size(uint8_t const* constraint_system_buf);
WASM_EXPORT uint32_t acir_proofs_get_total_circuit_size(uint8_t const* constraint_system_buf);
// Construct composer using prover and verifier key buffers
WASM_EXPORT size_t acir_proofs_init_proving_key(uint8_t const* constraint_system_buf, uint8_t const** pk_buf);
WASM_EXPORT size_t acir_proofs_init_verification_key(void* pippenger,
                                                     uint8_t const* g2x,
                                                     uint8_t const* pk_buf,
                                                     uint8_t const** vk_buf);
WASM_EXPORT size_t acir_proofs_new_proof(void* pippenger,
                                         uint8_t const* g2x,
                                         uint8_t const* pk_buf,
                                         uint8_t const* constraint_system_buf,
                                         uint8_t const* witness_buf,
                                         uint8_t** proof_data_buf);
WASM_EXPORT bool acir_proofs_verify_proof(
    uint8_t const* g2x, uint8_t const* vk_buf, uint8_t const* constraint_system_buf, uint8_t* proof, uint32_t length);
}
