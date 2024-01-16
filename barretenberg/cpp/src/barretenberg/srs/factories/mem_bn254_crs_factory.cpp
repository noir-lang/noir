#include "mem_bn254_crs_factory.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/ecc/curves/bn254/pairing.hpp"
#include "barretenberg/ecc/scalar_multiplication/point_table.hpp"
#include "barretenberg/ecc/scalar_multiplication/scalar_multiplication.hpp"
#include "barretenberg/srs/factories/mem_prover_crs.hpp"

namespace {

using namespace bb;
using namespace bb::srs::factories;

class MemVerifierCrs : public VerifierCrs<curve::BN254> {
  public:
    MemVerifierCrs(g2::affine_element const& g2_point)
        : g2_x(g2_point)
        , precomputed_g2_lines(
              static_cast<pairing::miller_lines*>(aligned_alloc(64, sizeof(bb::pairing::miller_lines) * 2)))
    {

        bb::pairing::precompute_miller_lines(bb::g2::one, precomputed_g2_lines[0]);
        bb::pairing::precompute_miller_lines(g2_x, precomputed_g2_lines[1]);
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

namespace bb::srs::factories {

MemBn254CrsFactory::MemBn254CrsFactory(std::vector<g1::affine_element> const& points,
                                       g2::affine_element const& g2_point)
    : prover_crs_(std::make_shared<MemProverCrs<curve::BN254>>(points))
    , verifier_crs_(std::make_shared<MemVerifierCrs>(g2_point))
{}

std::shared_ptr<bb::srs::factories::ProverCrs<curve::BN254>> MemBn254CrsFactory::get_prover_crs(size_t)
{
    return prover_crs_;
}

std::shared_ptr<bb::srs::factories::VerifierCrs<curve::BN254>> MemBn254CrsFactory::get_verifier_crs(size_t)
{
    return verifier_crs_;
}

} // namespace bb::srs::factories