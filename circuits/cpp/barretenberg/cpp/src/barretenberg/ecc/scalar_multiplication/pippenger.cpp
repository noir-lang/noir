#include "pippenger.hpp"
#include "barretenberg/srs/io.hpp"
namespace barretenberg {
namespace scalar_multiplication {

template <typename Curve>
Pippenger<Curve>::Pippenger(typename Curve::AffineElement* points, size_t num_points)
    : monomials_(points)
    , num_points_(num_points)
{
    srs::IO<Curve>::byteswap(&monomials_[0], num_points * 64);
    scalar_multiplication::generate_pippenger_point_table<Curve>(monomials_, monomials_, num_points);
}

template <typename Curve>
Pippenger<Curve>::Pippenger(uint8_t const* points, size_t num_points)
    : num_points_(num_points)
{
    monomials_ = point_table_alloc<AffineElement>(num_points);

    srs::IO<Curve>::read_affine_elements_from_buffer(&monomials_[0], (char*)points, num_points * 64);
    scalar_multiplication::generate_pippenger_point_table<Curve>(monomials_, monomials_, num_points);
}

template <typename Curve>
Pippenger<Curve>::Pippenger(std::string const& path, size_t num_points)
    : num_points_(num_points)
{
    monomials_ = point_table_alloc<AffineElement>(num_points);

    srs::IO<Curve>::read_transcript_g1(monomials_, num_points, path);
    scalar_multiplication::generate_pippenger_point_table<Curve>(monomials_, monomials_, num_points);
}

template <typename Curve>
typename Curve::Element Pippenger<Curve>::pippenger_unsafe(ScalarField* scalars, size_t from, size_t range)
{
    scalar_multiplication::pippenger_runtime_state<Curve> state(range);
    return scalar_multiplication::pippenger_unsafe<Curve>(scalars, monomials_ + from * 2, range, state);
}

template <typename Curve> Pippenger<Curve>::~Pippenger()
{
    free(monomials_);
}

template class Pippenger<curve::BN254>;
template class Pippenger<curve::Grumpkin>;

} // namespace scalar_multiplication
} // namespace barretenberg
