#include <barretenberg/common/serialize.hpp>
#include <barretenberg/common/wasm_export.hpp>
#include <barretenberg/ecc/curves/bn254/fr.hpp>
#include <cstddef>
#include <cstdint>

using namespace bb;

WASM_EXPORT void acir_get_circuit_sizes(uint8_t const* constraint_system_buf,
                                        uint32_t* exact,
                                        uint32_t* total,
                                        uint32_t* subgroup);

WASM_EXPORT void acir_new_acir_composer(uint32_t const* size_hint, out_ptr out);

WASM_EXPORT void acir_new_goblin_acir_composer(out_ptr out);

WASM_EXPORT void acir_delete_acir_composer(in_ptr acir_composer_ptr);

WASM_EXPORT void acir_create_circuit(in_ptr acir_composer_ptr,
                                     uint8_t const* constraint_system_buf,
                                     uint32_t const* size_hint);

WASM_EXPORT void acir_init_proving_key(in_ptr acir_composer_ptr, uint8_t const* constraint_system_buf);

/**
 * It would have been nice to just hold onto the constraint_system in the acir_composer, but we can't waste the
 * memory. Being able to reuse the underlying Composer would help as well. But, given the situation, we just have
 * to pass it in everytime.
 */
WASM_EXPORT void acir_create_proof(in_ptr acir_composer_ptr,
                                   uint8_t const* constraint_system_buf,
                                   uint8_t const* witness_buf,
                                   bool const* is_recursive,
                                   uint8_t** out);

/**
 * @brief Perform the goblin accumulate operation
 * @details Constructs a GUH proof and possibly handles transcript merge logic
 *
 */
WASM_EXPORT void acir_goblin_accumulate(in_ptr acir_composer_ptr,
                                        uint8_t const* constraint_system_buf,
                                        uint8_t const* witness_buf,
                                        uint8_t** out);

/**
 * @brief Construct a full goblin proof
 * @details Makes a call to accumulate to a final circuit before constructing a Goblin proof
 *
 */
WASM_EXPORT void acir_goblin_prove(in_ptr acir_composer_ptr,
                                   uint8_t const* constraint_system_buf,
                                   uint8_t const* witness_buf,
                                   uint8_t** out);

WASM_EXPORT void acir_load_verification_key(in_ptr acir_composer_ptr, uint8_t const* vk_buf);

WASM_EXPORT void acir_init_verification_key(in_ptr acir_composer_ptr);

WASM_EXPORT void acir_get_verification_key(in_ptr acir_composer_ptr, uint8_t** out);

WASM_EXPORT void acir_get_proving_key(in_ptr acir_composer_ptr, uint8_t const* acir_vec, uint8_t** out);

WASM_EXPORT void acir_verify_proof(in_ptr acir_composer_ptr,
                                   uint8_t const* proof_buf,
                                   bool const* is_recursive,
                                   bool* result);

/**
 * @brief Verifies a GUH proof produced during goblin accumulation
 *
 */
WASM_EXPORT void acir_goblin_verify_accumulator(in_ptr acir_composer_ptr, uint8_t const* proof_buf, bool* result);

/**
 * @brief Verifies a full goblin proof (and the GUH proof produced by accumulation)
 *
 */
WASM_EXPORT void acir_goblin_verify(in_ptr acir_composer_ptr, uint8_t const* proof_buf, bool* result);

WASM_EXPORT void acir_get_solidity_verifier(in_ptr acir_composer_ptr, out_str_buf out);

WASM_EXPORT void acir_serialize_proof_into_fields(in_ptr acir_composer_ptr,
                                                  uint8_t const* proof_buf,
                                                  uint32_t const* num_inner_public_inputs,
                                                  fr::vec_out_buf out);

WASM_EXPORT void acir_serialize_verification_key_into_fields(in_ptr acir_composer_ptr,
                                                             fr::vec_out_buf out_vkey,
                                                             fr::out_buf out_key_hash);