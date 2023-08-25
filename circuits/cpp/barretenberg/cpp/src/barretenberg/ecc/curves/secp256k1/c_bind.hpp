#include "secp256k1.hpp"

// Silencing warnings about reserved identifiers. Fixing would break downstream code that calls our WASM API.
// NOLINTBEGIN(cert-dcl37-c, cert-dcl51-cpp, bugprone-reserved-identifier)
WASM_EXPORT void ecc_secp256k1__mul(uint8_t const* point_buf, uint8_t const* scalar_buf, uint8_t* result);

WASM_EXPORT void ecc_secp256k1__get_random_scalar_mod_circuit_modulus(uint8_t* result);

WASM_EXPORT void ecc_secp256k1__reduce512_buffer_mod_circuit_modulus(uint8_t* input, uint8_t* result);
// NOLINTEND(cert-dcl37-c, cert-dcl51-cpp, bugprone-reserved-identifier)
