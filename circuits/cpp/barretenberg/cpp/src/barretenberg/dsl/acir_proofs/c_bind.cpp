#include "c_bind.hpp"
#include "acir_proofs.hpp"
#include <cstdint>

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT size_t acir_proofs_get_solidity_verifier(uint8_t const* g2x, uint8_t const* vk_buf, uint8_t** output_buf)
{
    return acir_proofs::get_solidity_verifier(g2x, vk_buf, output_buf);
}

// Get the exact circuit size for the constraint system.
WASM_EXPORT uint32_t acir_proofs_get_exact_circuit_size(uint8_t const* constraint_system_buf)
{
    return acir_proofs::get_exact_circuit_size(constraint_system_buf);
}

WASM_EXPORT uint32_t acir_proofs_get_total_circuit_size(uint8_t const* constraint_system_buf)
{
    return acir_proofs::get_total_circuit_size(constraint_system_buf);
}

WASM_EXPORT size_t acir_proofs_init_proving_key(uint8_t const* constraint_system_buf, uint8_t const** pk_buf)
{
    return acir_proofs::init_proving_key(constraint_system_buf, pk_buf);
}

WASM_EXPORT size_t acir_proofs_init_verification_key(void* pippenger,
                                                     uint8_t const* g2x,
                                                     uint8_t const* pk_buf,
                                                     uint8_t const** vk_buf)
{
    return acir_proofs::init_verification_key(pippenger, g2x, pk_buf, vk_buf);
}

WASM_EXPORT size_t acir_proofs_new_proof(void* pippenger,
                                         uint8_t const* g2x,
                                         uint8_t const* pk_buf,
                                         uint8_t const* constraint_system_buf,
                                         uint8_t const* witness_buf,
                                         uint8_t** proof_data_buf)
{
    return acir_proofs::new_proof(pippenger, g2x, pk_buf, constraint_system_buf, witness_buf, proof_data_buf);
}

WASM_EXPORT bool acir_proofs_verify_proof(
    uint8_t const* g2x, uint8_t const* vk_buf, uint8_t const* constraint_system_buf, uint8_t* proof, uint32_t length)
{
    return acir_proofs::verify_proof(g2x, vk_buf, constraint_system_buf, proof, length);
}
}
