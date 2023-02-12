/**
 * Create reference strings given a Pippenger instance containing point table formatted monomials.
 * Does not take ownership of the given buffer.
 */
#pragma once
#include "mem_reference_string.hpp"

namespace barretenberg::pairing {
struct miller_lines;
} // namespace barretenberg::pairing

namespace waffle {

using namespace barretenberg;

class PippengerReferenceString : public ProverReferenceString {
  public:
    PippengerReferenceString(scalar_multiplication::Pippenger* pippenger,
                             scalar_multiplication::Pippenger* pippenger_lagrange)
        : pippenger_(pippenger)
        , pippenger_lagrange_(pippenger_lagrange)
    {}

    size_t get_monomial_size() const override { return pippenger_->get_num_points(); }
    size_t get_lagrange_size() const override { return pippenger_lagrange_->get_num_points(); }
    g1::affine_element* get_monomial_points() override { return pippenger_->get_point_table(); }
    g1::affine_element* get_lagrange_points() override { return pippenger_lagrange_->get_point_table(); }

  private:
    scalar_multiplication::Pippenger* pippenger_;
    scalar_multiplication::Pippenger* pippenger_lagrange_;
};

class PippengerReferenceStringFactory : public ReferenceStringFactory {
  public:
    PippengerReferenceStringFactory(scalar_multiplication::Pippenger* pippenger,
                                    scalar_multiplication::Pippenger* pippenger_lagrange,
                                    uint8_t const* g2x)
        : pippenger_(pippenger)
        , pippenger_lagrange_(pippenger_lagrange)
        , g2x_(g2x)
    {}

    PippengerReferenceStringFactory(PippengerReferenceStringFactory&& other) = default;

    std::shared_ptr<ProverReferenceString> get_prover_crs(size_t degree) override
    {
        ASSERT(degree <= pippenger_->get_num_points());
        ASSERT((1UL << numeric::get_msb(degree)) == pippenger_lagrange_->get_num_points());
        return std::make_shared<PippengerReferenceString>(pippenger_, pippenger_lagrange_);
    }

    std::shared_ptr<VerifierReferenceString> get_verifier_crs() override
    {
        return std::make_shared<VerifierMemReferenceString>(g2x_);
    }

  private:
    scalar_multiplication::Pippenger* pippenger_;
    scalar_multiplication::Pippenger* pippenger_lagrange_;
    uint8_t const* g2x_;
};

} // namespace waffle
