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
}