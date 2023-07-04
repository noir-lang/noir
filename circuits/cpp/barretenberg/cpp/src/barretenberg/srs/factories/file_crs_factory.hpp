#pragma once
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "crs_factory.hpp"
#include <cstddef>
#include <utility>

namespace barretenberg::srs::factories {

template <typename Curve> class FileProverCrs : public ProverCrs<Curve> {
  public:
    FileProverCrs(const size_t num_points, std::string const& path);

    typename Curve::AffineElement* get_monomial_points() { return monomials_.get(); }

    size_t get_monomial_size() const { return num_points; }

  private:
    size_t num_points;
    std::shared_ptr<typename Curve::AffineElement[]> monomials_;
};

class FileVerifierCrs : public VerifierCrs {
  public:
    FileVerifierCrs(std::string const& path);

    ~FileVerifierCrs();

    g2::affine_element get_g2x() const override;

    pairing::miller_lines const* get_precomputed_g2_lines() const override;

  private:
    g2::affine_element g2_x;
    pairing::miller_lines* precomputed_g2_lines;
};

/**
 * Create reference strings given a path to a directory of transcript files.
 */
class FileCrsFactory : public CrsFactory {
  public:
    FileCrsFactory(std::string path, size_t initial_degree = 0);
    FileCrsFactory(FileCrsFactory&& other) = default;

    std::shared_ptr<barretenberg::srs::factories::ProverCrs<curve::BN254>> get_prover_crs(size_t degree) override;

    std::shared_ptr<barretenberg::srs::factories::VerifierCrs> get_verifier_crs() override;

  private:
    std::string path_;
    size_t degree_;
    std::shared_ptr<barretenberg::srs::factories::ProverCrs<curve::BN254>> prover_crs_;
    std::shared_ptr<barretenberg::srs::factories::VerifierCrs> verifier_crs_;
};

extern template class FileProverCrs<curve::BN254>;
extern template class FileProverCrs<curve::Grumpkin>;

} // namespace barretenberg::srs::factories
