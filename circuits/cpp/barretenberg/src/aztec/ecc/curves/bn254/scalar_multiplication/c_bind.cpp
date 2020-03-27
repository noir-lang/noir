#include "./scalar_multiplication.hpp"
#include <numeric/bitop/get_msb.hpp>
#include <srs/io.hpp>
#include <common/log.hpp>

using namespace barretenberg;

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void pippenger_unsafe(fr* scalars, uint8_t* points, const size_t num_points, g1::element* result)
{
    static g1::affine_element* point_tables[20] = {};

    size_t index = numeric::get_msb(num_points);
    g1::affine_element* monomials = point_tables[index];
    if (!monomials) {
        monomials = (barretenberg::g1::affine_element*)(aligned_alloc(
            64, sizeof(barretenberg::g1::affine_element) * (2 * num_points + 2)));

        monomials[0] = barretenberg::g1::affine_one;

        barretenberg::io::read_g1_elements_from_buffer(&monomials[1], (char*)points, num_points * 64);
        barretenberg::scalar_multiplication::generate_pippenger_point_table(monomials, monomials, num_points);
        point_tables[index] = monomials;
    }

    scalar_multiplication::unsafe_pippenger_runtime_state state(num_points);
    *result = scalar_multiplication::pippenger_unsafe(scalars, monomials, num_points, state);
}
}