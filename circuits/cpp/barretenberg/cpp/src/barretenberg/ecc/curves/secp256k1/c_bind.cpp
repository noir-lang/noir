#include "secp256k1.hpp"

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void ecc_secp256k1__mul(uint8_t const* point_buf, uint8_t const* scalar_buf, uint8_t* result)
{
    auto point = from_buffer<secp256k1::g1::affine_element>(point_buf);
    auto scalar = from_buffer<secp256k1::fr>(scalar_buf);
    secp256k1::g1::affine_element r = point * scalar;
    write(result, r);
}

WASM_EXPORT void ecc_secp256k1__get_random_scalar_mod_circuit_modulus(uint8_t* result)
{
    barretenberg::fr output = barretenberg::fr::random_element();
    write(result, output);
}

WASM_EXPORT void ecc_secp256k1__reduce512_buffer_mod_circuit_modulus(uint8_t* input, uint8_t* result)
{
    uint512_t bigint_input = from_buffer<uint512_t>(input);

    uint512_t barretenberg_modulus(barretenberg::fr::modulus);

    uint512_t target_output = bigint_input % barretenberg_modulus;
    write(result, target_output.lo);
}
}