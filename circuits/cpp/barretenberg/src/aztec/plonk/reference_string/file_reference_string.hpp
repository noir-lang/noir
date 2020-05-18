/**
 * Create reference strings given a path to a directory of transcript files.
 */
#pragma once
#include "reference_string.hpp"
#include <cstddef>
#include <ecc/curves/bn254/g1.hpp>
#include <ecc/curves/bn254/g2.hpp>
#include <ecc/curves/bn254/scalar_multiplication/pippenger.hpp>

namespace barretenberg {
namespace pairing {
struct miller_lines;
}
} // namespace barretenberg

namespace waffle {

using namespace barretenberg;

class VerifierFileReferenceString : public VerifierReferenceString {
  public:
    VerifierFileReferenceString(std::string const& path);
    ~VerifierFileReferenceString();

    g2::affine_element get_g2x() const { return g2_x; }

    pairing::miller_lines const* get_precomputed_g2_lines() const { return precomputed_g2_lines; }

  private:
    g2::affine_element g2_x;
    pairing::miller_lines* precomputed_g2_lines;
};

class FileReferenceString : public ProverReferenceString {
  public:
    FileReferenceString(const size_t num_points, std::string const& path)
        : pippenger_(path, num_points)
    {}

    g1::affine_element* get_monomials() { return pippenger_.get_point_table(); }

  private:
    scalar_multiplication::Pippenger pippenger_;
};

class FileReferenceStringFactory : public ReferenceStringFactory {
  public:
    FileReferenceStringFactory(std::string const& path)
        : path_(path)
    {}

    FileReferenceStringFactory(FileReferenceStringFactory&& other) = default;

    std::shared_ptr<ProverReferenceString> get_prover_crs(size_t degree)
    {
        return std::make_shared<FileReferenceString>(degree, path_);
    }

    std::shared_ptr<VerifierReferenceString> get_verifier_crs()
    {
        return std::make_shared<VerifierFileReferenceString>(path_);
    }

  private:
    std::string path_;
};

} // namespace waffle
