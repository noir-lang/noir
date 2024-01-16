#pragma once

#include "barretenberg/ecc/scalar_multiplication/point_table.hpp"
#include "barretenberg/ecc/scalar_multiplication/scalar_multiplication.hpp"
#include "barretenberg/srs/factories/crs_factory.hpp"

namespace bb::srs::factories {
// Common to both Grumpkin and Bn254, and generally curves regardless of pairing-friendliness
template <typename Curve> class MemProverCrs : public ProverCrs<Curve> {
  public:
    MemProverCrs(std::vector<typename Curve::AffineElement> const& points)
        : num_points(points.size())
        , monomials_(scalar_multiplication::point_table_alloc<typename Curve::AffineElement>(points.size()))
    {
        std::copy(points.begin(), points.end(), monomials_.get());
        scalar_multiplication::generate_pippenger_point_table<Curve>(monomials_.get(), monomials_.get(), num_points);
    }

    typename Curve::AffineElement* get_monomial_points() override { return monomials_.get(); }

    size_t get_monomial_size() const override { return num_points; }

  private:
    size_t num_points;
    std::shared_ptr<typename Curve::AffineElement[]> monomials_;
};

} // namespace bb::srs::factories