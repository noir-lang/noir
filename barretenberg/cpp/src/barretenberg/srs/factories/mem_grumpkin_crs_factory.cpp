#include "mem_grumpkin_crs_factory.hpp"
#include "../io.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/ecc/scalar_multiplication/point_table.hpp"
#include "barretenberg/ecc/scalar_multiplication/scalar_multiplication.hpp"
#include "barretenberg/srs/factories/mem_prover_crs.hpp"

namespace {

using namespace bb::curve;
using namespace bb;
using namespace bb::srs::factories;

using Curve = curve::Grumpkin;

class MemVerifierCrs : public VerifierCrs<Grumpkin> {
  public:
    MemVerifierCrs(std::vector<Grumpkin::AffineElement> const& points)
        : num_points(points.size())
        , monomials_(scalar_multiplication::point_table_alloc<Grumpkin::AffineElement>(points.size()))
    {
        std::copy(points.begin(), points.end(), monomials_.get());
        scalar_multiplication::generate_pippenger_point_table<Grumpkin>(monomials_.get(), monomials_.get(), num_points);
    }

    virtual ~MemVerifierCrs() = default;
    Grumpkin::AffineElement* get_monomial_points() const override { return monomials_.get(); }
    size_t get_monomial_size() const override { return num_points; }
    Grumpkin::AffineElement get_first_g1() const override { return monomials_[0]; };

  private:
    size_t num_points;
    std::shared_ptr<Grumpkin::AffineElement[]> monomials_;
};

} // namespace

namespace bb::srs::factories {

MemGrumpkinCrsFactory::MemGrumpkinCrsFactory(std::vector<Grumpkin::AffineElement> const& points)
    : prover_crs_(std::make_shared<MemProverCrs<Grumpkin>>(points))
    , verifier_crs_(std::make_shared<MemVerifierCrs>(points))
{}

std::shared_ptr<bb::srs::factories::ProverCrs<Grumpkin>> MemGrumpkinCrsFactory::get_prover_crs(size_t)
{
    return prover_crs_;
}

std::shared_ptr<bb::srs::factories::VerifierCrs<Grumpkin>> MemGrumpkinCrsFactory::get_verifier_crs(size_t)
{
    return verifier_crs_;
}

} // namespace bb::srs::factories