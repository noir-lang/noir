/**
 * Create reference strings given a buffer containing network formatted g1 or g2 points.
 */
#pragma once
#include "reference_string.hpp"

namespace barretenberg {
namespace pairing {
struct miller_lines;
}
} // namespace barretenberg

namespace waffle {

using namespace barretenberg;

class VerifierMemReferenceString : public VerifierReferenceString {
  public:
    VerifierMemReferenceString(char const* g2x);
    ~VerifierMemReferenceString();

    g2::affine_element get_g2x() const { return g2_x; }

    pairing::miller_lines const* get_precomputed_g2_lines() const { return precomputed_g2_lines; }

  private:
    g2::affine_element g2_x;
    pairing::miller_lines* precomputed_g2_lines;
};

class MemReferenceString : public ProverReferenceString {
  public:
    MemReferenceString(const size_t num_points, char const* buffer);
    ~MemReferenceString();

    g1::affine_element* get_monomials() { return monomials_; }

  private:
    g1::affine_element* monomials_;
};

class MemReferenceStringFactory : public ReferenceStringFactory {
  public:
    MemReferenceStringFactory(char const* buffer, size_t num_points, char const* g2x)
        : buffer_(buffer)
        , num_points_(num_points)
        , g2x_(g2x)
    {}

    std::shared_ptr<ProverReferenceString> get_prover_crs(size_t degree)
    {
        ASSERT(degree <= num_points_);
        return std::make_shared<MemReferenceString>(degree, buffer_);
    }

    std::shared_ptr<VerifierReferenceString> get_verifier_crs()
    {
        return std::make_shared<VerifierMemReferenceString>(g2x_);
    }

  private:
    char const* buffer_;
    size_t num_points_;
    char const* g2x_;
};

} // namespace waffle
