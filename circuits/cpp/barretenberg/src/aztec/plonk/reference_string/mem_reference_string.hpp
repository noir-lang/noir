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

class VerifierMemReferenceString : public VerifierReferenceString {
  public:
    VerifierMemReferenceString(char const* buffer, size_t buffer_size);
    ~VerifierMemReferenceString();

    barretenberg::pairing::miller_lines const* get_precomputed_g2_lines() const { return precomputed_g2_lines; }

  private:
    barretenberg::pairing::miller_lines* precomputed_g2_lines;
};

class MemReferenceString : public ProverReferenceString {
  public:
    MemReferenceString(const size_t num_points, char const* buffer, size_t buffer_size);
    ~MemReferenceString();

    barretenberg::g1::affine_element* get_monomials() { return monomials; }

  private:
    barretenberg::g1::affine_element* monomials;
};

class MemReferenceStringFactory : public ReferenceStringFactory {
  public:
    MemReferenceStringFactory(char const* buffer, size_t size)
        : buffer_(buffer)
        , size_(size)
    {}

    std::shared_ptr<ProverReferenceString> get_prover_crs(size_t degree)
    {
        return std::make_shared<MemReferenceString>(degree, buffer_, size_);
    }

    std::shared_ptr<VerifierReferenceString> get_verifier_crs()
    {
        return std::make_shared<VerifierMemReferenceString>(buffer_, size_);
    }

  private:
    char const* buffer_;
    size_t size_;
};

} // namespace waffle
