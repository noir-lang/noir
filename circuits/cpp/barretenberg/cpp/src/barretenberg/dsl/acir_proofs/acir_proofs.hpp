#include <cstdint>
#include <cstddef>

namespace acir_proofs {

size_t get_solidity_verifier(uint8_t const* g2x, uint8_t const* vk_buf, uint8_t** output_buf);
uint32_t get_exact_circuit_size(uint8_t const* constraint_system_buf);
uint32_t get_total_circuit_size(uint8_t const* constraint_system_buf);
size_t init_proving_key(uint8_t const* constraint_system_buf, uint8_t const** pk_buf);
size_t init_verification_key(void* pippenger, uint8_t const* g2x, uint8_t const* pk_buf, uint8_t const** vk_buf);
size_t new_proof(void* pippenger,
                 uint8_t const* g2x,
                 uint8_t const* pk_buf,
                 uint8_t const* constraint_system_buf,
                 uint8_t const* witness_buf,
                 uint8_t** proof_data_buf);
bool verify_proof(
    uint8_t const* g2x, uint8_t const* vk_buf, uint8_t const* constraint_system_buf, uint8_t* proof, uint32_t length);

} // namespace acir_proofs
