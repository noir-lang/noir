#include "pippenger.hpp"
#include <srs/io.hpp>
namespace barretenberg {
namespace scalar_multiplication {

Pippenger::Pippenger(g1::affine_element* points, size_t num_points)
    : monomials_(points)
    , num_points_(num_points)
{
    monomials_[0] = barretenberg::g1::affine_one;
    io::byteswap(&monomials_[1], (num_points - 1) * 64);

    scalar_multiplication::generate_pippenger_point_table(monomials_, monomials_, num_points);
}

Pippenger::Pippenger(uint8_t const* points, size_t num_points, bool is_lagrange)
    : num_points_(num_points)
{
    monomials_ = point_table_alloc<g1::affine_element>(num_points);

    size_t index = 0;
    if (!is_lagrange) {
        monomials_[0] = barretenberg::g1::affine_one;
        index = 1;
    }

    barretenberg::io::read_g1_elements_from_buffer(&monomials_[index], (char*)points, (num_points - index) * 64);
    barretenberg::scalar_multiplication::generate_pippenger_point_table(monomials_, monomials_, num_points);
}

Pippenger::Pippenger(std::string const& path, size_t num_points, bool is_lagrange)
    : num_points_(num_points)
{
    monomials_ = point_table_alloc<g1::affine_element>(num_points);

    barretenberg::io::read_transcript_g1(monomials_, num_points, path, is_lagrange);
    barretenberg::scalar_multiplication::generate_pippenger_point_table(monomials_, monomials_, num_points);
}

g1::element Pippenger::pippenger_unsafe(fr* scalars, size_t from, size_t range)
{
    scalar_multiplication::pippenger_runtime_state state(range);
    return scalar_multiplication::pippenger_unsafe(scalars, monomials_ + from * 2, range, state);
}

Pippenger::~Pippenger()
{
    free(monomials_);
}

} // namespace scalar_multiplication
} // namespace barretenberg