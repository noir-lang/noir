// TODO: Delete this cbind once funcs working in root cbind of ecc module.
#include "barretenberg/common/wasm_export.hpp"
#include "grumpkin.hpp"

// Silencing warnings about reserved identifiers. Fixing would break downstream code that calls our WASM API.
// NOLINTBEGIN(cert-dcl37-c, cert-dcl51-cpp, bugprone-reserved-identifier)
WASM_EXPORT void ecc_grumpkin__mul(uint8_t const* point_buf, uint8_t const* scalar_buf, uint8_t* result)
{
    using serialize::write;
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
    using serialize::write;
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
    bb::fr output = bb::fr::random_element();
    write(result, output);
}

WASM_EXPORT void ecc_grumpkin__reduce512_buffer_mod_circuit_modulus(uint8_t* input, uint8_t* result)
{
    auto bigint_input = from_buffer<uint512_t>(input);

    uint512_t barretenberg_modulus(bb::fr::modulus);

    uint512_t target_output = bigint_input % barretenberg_modulus;
    write(result, target_output.lo);
}

// NOLINTEND(cert-dcl37-c, cert-dcl51-cpp, bugprone-reserved-identifier)