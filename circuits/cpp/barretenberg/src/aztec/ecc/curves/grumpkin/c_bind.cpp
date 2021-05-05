#include "grumpkin.hpp"

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void ecc_grumpkin__mul(uint8_t const* point_buf, uint8_t const* scalar_buf, uint8_t* result)
{
    auto point = from_buffer<grumpkin::g1::affine_element>(point_buf);
    auto scalar = from_buffer<grumpkin::fr>(scalar_buf);
    grumpkin::g1::affine_element r = point * scalar;
    write(result, r);
}

// multiplies a vector of points by a single scalar. Returns a vector of points (this is NOT a multi-exponentiation)
WASM_EXPORT void ecc_grumpkin__batch_mul(uint8_t const* point_buf,
                                         uint8_t const* scalar_buf,
                                         uint32_t num_points,
                                         uint8_t* result)
{
    std::vector<grumpkin::g1::affine_element> points;
    points.reserve(num_points);
    for (size_t i = 0; i < num_points; ++i) {
        points.emplace_back(from_buffer<grumpkin::g1::affine_element>(point_buf + (i * 64)));
    }
    auto scalar = from_buffer<grumpkin::fr>(scalar_buf);
    auto output = grumpkin::g1::element::batch_mul_with_endomorphism(points, scalar);
    for (size_t i = 0; i < num_points; ++i) {
        grumpkin::g1::affine_element r = output[i];
        uint8_t* result_ptr = result + (i * 64);
        write(result_ptr, r);
    }
}

WASM_EXPORT void ecc_grumpkin__get_random_scalar_mod_circuit_modulus(uint8_t* result)
{
    barretenberg::fr output = barretenberg::fr::random_element();
    write(result, output);
}

WASM_EXPORT void ecc_grumpkin__reduce512_buffer_mod_circuit_modulus(uint8_t* input, uint8_t* result)
{
    uint512_t bigint_input = from_buffer<uint512_t>(input);

    uint512_t barretenberg_modulus(barretenberg::fr::modulus);

    uint512_t target_output = bigint_input % barretenberg_modulus;
    write(result, target_output.lo);
}
}