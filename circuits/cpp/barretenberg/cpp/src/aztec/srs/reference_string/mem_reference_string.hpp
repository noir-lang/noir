/**
 * Create reference strings given a buffer containing network formatted g1 or g2 points.
 */
#pragma once

#include "reference_string.hpp"

#include <ecc/curves/bn254/scalar_multiplication/pippenger.hpp>

namespace barretenberg::pairing {
struct miller_lines;
} // namespace barretenberg::pairing

namespace waffle {

using namespace barretenberg;

class VerifierMemReferenceString : public VerifierReferenceString {
  public:
    VerifierMemReferenceString(uint8_t const* g2x);
    ~VerifierMemReferenceString() override;

    g2::affine_element get_g2x() const override { return g2_x; }

    pairing::miller_lines const* get_precomputed_g2_lines() const override { return precomputed_g2_lines; }

  private:
    g2::affine_element g2_x;
    pairing::miller_lines* precomputed_g2_lines;
};

} // namespace waffle
