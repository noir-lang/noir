///adapted from aztec2-internal/barretenberg/src/aztec/dsl/standard_format/c_bind.h
#include <cstdint>
#include <stddef.h>
#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {
WASM_EXPORT uint32_t composer__get_circuit_size(uint8_t const* constraint_system_buf);

WASM_EXPORT void* composer__new_prover(void* pippenger,
                                                uint8_t const* g2x,
                                                uint8_t const* constraint_system_buf,
                                                uint8_t const* witness_buf);

WASM_EXPORT void composer__delete_prover(void* prover);

WASM_EXPORT bool composer__verify_proof(
    void* pippenger, uint8_t const* g2x, uint8_t const* constraint_system_buf, uint8_t* proof, uint32_t length);

WASM_EXPORT bool composer__verify_proof_with_public_inputs(void* pippenger,
                                                                    uint8_t const* g2x,
                                                                    uint8_t const* constraint_system_buf,
                                                                    uint8_t const* public_inputs_buf,
                                                                    uint8_t* proof,
                                                                    uint32_t length);

WASM_EXPORT size_t composer__new_proof(void* pippenger,
                                       uint8_t const* g2x,
                                       uint8_t const* constraint_system_buf,
                                       uint8_t const* witness_buf,
                                       uint8_t** proof_data_buf);

WASM_EXPORT uint32_t composer__smart_contract(void* pippenger,
                                              uint8_t const* g2x,
                                              uint8_t const* constraint_system_buf,
                                              uint8_t** output_buf);
}


