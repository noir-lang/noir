#include "mem_crs_factory.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/ecc/curves/bn254/pairing.hpp"
#include "barretenberg/ecc/scalar_multiplication/point_table.hpp"
#include "barretenberg/ecc/scalar_multiplication/scalar_multiplication.hpp"

namespace {

using namespace barretenberg;
using namespace barretenberg::srs::factories;

class MemProverCrs : public ProverCrs<curve::BN254> {
  public:
    MemProverCrs(std::vector<g1::affine_element> const& points)
        : num_points(points.size())
    {
        monomials_ = scalar_multiplication::point_table_alloc<g1::affine_element>(num_points);
        std::copy(points.begin(), points.end(), monomials_.get());
        scalar_multiplication::generate_pippenger_point_table<curve::BN254>(
            monomials_.get(), monomials_.get(), num_points);
    }

    g1::affine_element* get_monomial_points() override { return monomials_.get(); }

    size_t get_monomial_size() const override { return num_points; }

  private:
    size_t num_points;
    std::shared_ptr<g1::affine_element[]> monomials_;
};

class MemVerifierCrs : public VerifierCrs<curve::BN254> {
  public:
    MemVerifierCrs(g2::affine_element const& g2_point)
        : g2_x(g2_point)
        , precomputed_g2_lines(
              static_cast<pairing::miller_lines*>(aligned_alloc(64, sizeof(barretenberg::pairing::miller_lines) * 2)))
    {

        barretenberg::pairing::precompute_miller_lines(barretenberg::g2::one, precomputed_g2_lines[0]);
        barretenberg::pairing::precompute_miller_lines(g2_x, precomputed_g2_lines[1]);
    }

    virtual ~MemVerifierCrs() { aligned_free(precomputed_g2_lines); }

    g2::affine_element get_g2x() const { return g2_x; }

    pairing::miller_lines const* get_precomputed_g2_lines() const { return precomputed_g2_lines; }
    g1::affine_element get_first_g1() const { return first_g1x; };

  private:
    g1::affine_element first_g1x;
    g2::affine_element g2_x;
    pairing::miller_lines* precomputed_g2_lines;
};

} // namespace

namespace barretenberg::srs::factories {

MemCrsFactory::MemCrsFactory(std::vector<g1::affine_element> const& points, g2::affine_element const g2_point)
    : prover_crs_(std::make_shared<MemProverCrs>(points))
    , verifier_crs_(std::make_shared<MemVerifierCrs>(g2_point))
{}

std::shared_ptr<barretenberg::srs::factories::ProverCrs<curve::BN254>> MemCrsFactory::get_prover_crs(size_t)
{
    return prover_crs_;
}

std::shared_ptr<barretenberg::srs::factories::VerifierCrs<curve::BN254>> MemCrsFactory::get_verifier_crs(size_t)
{
    return verifier_crs_;
}

} // namespace barretenberg::srs::factories