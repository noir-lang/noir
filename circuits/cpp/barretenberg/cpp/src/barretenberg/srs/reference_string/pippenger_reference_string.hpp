/**
 * Create reference strings given a Pippenger instance containing point table formatted monomials.
 * Does not take ownership of the given buffer.
 */
#pragma once
#include "mem_reference_string.hpp"

namespace barretenberg::pairing {
struct miller_lines;
} // namespace barretenberg::pairing

namespace proof_system {

using namespace barretenberg;

class PippengerReferenceString : public ProverReferenceString {
  public:
    // TODO(#473)?
    PippengerReferenceString(scalar_multiplication::Pippenger<curve::BN254>* pippenger)
        : pippenger_(pippenger)
    {}

    size_t get_monomial_size() const override { return pippenger_->get_num_points(); }
    g1::affine_element* get_monomial_points() override { return pippenger_->get_point_table(); }

  private:
    // TODO(#473)?
    scalar_multiplication::Pippenger<curve::BN254>* pippenger_;
};

class PippengerReferenceStringFactory : public ReferenceStringFactory {
  public:
    // TODO(#473)?
    PippengerReferenceStringFactory(scalar_multiplication::Pippenger<curve::BN254>* pippenger, uint8_t const* g2x)
        : pippenger_(pippenger)
        , g2x_(g2x)
    {}

    PippengerReferenceStringFactory(PippengerReferenceStringFactory&& other) = default;

    std::shared_ptr<ProverReferenceString> get_prover_crs(size_t degree) override
    {
        ASSERT(degree <= pippenger_->get_num_points());
        return std::make_shared<PippengerReferenceString>(pippenger_);
    }

    std::shared_ptr<VerifierReferenceString> get_verifier_crs() override
    {
        return std::make_shared<VerifierMemReferenceString>(g2x_);
    }

  private:
    // TODO(#473)?
    scalar_multiplication::Pippenger<curve::BN254>* pippenger_;
    uint8_t const* g2x_;
};

} // namespace proof_system
