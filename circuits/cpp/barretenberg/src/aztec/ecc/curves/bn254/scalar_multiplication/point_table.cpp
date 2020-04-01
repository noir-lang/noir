#include <srs/io.hpp>
#include "point_table.hpp"

#ifndef NO_MULTITHREADING
#include <omp.h>
#endif

namespace barretenberg {
namespace scalar_multiplication {

g1::affine_element* new_pippenger_point_table(uint8_t* points, size_t num_points)
{
    auto monomials = point_table_alloc<g1::affine_element>(num_points);

    monomials[0] = barretenberg::g1::affine_one;

    barretenberg::io::read_g1_elements_from_buffer(&monomials[1], (char*)points, num_points * 64);
    barretenberg::scalar_multiplication::generate_pippenger_point_table(monomials, monomials, num_points);

    return monomials;
}

g1::affine_element* new_pippenger_point_table_from_path(std::string const& path, size_t num_points)
{
    auto monomials = point_table_alloc<g1::affine_element>(num_points);

    barretenberg::io::read_transcript_g1(monomials, num_points, path);
    barretenberg::scalar_multiplication::generate_pippenger_point_table(monomials, monomials, num_points);

    return monomials;
}

}
}