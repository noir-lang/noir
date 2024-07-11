#pragma once
#include "../io.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/ecc/scalar_multiplication/point_table.hpp"
#include "barretenberg/ecc/scalar_multiplication/scalar_multiplication.hpp"
#include "crs_factory.hpp"
#include <cstddef>
#include <utility>

namespace bb::srs::factories {

/**
 * Create reference strings given a path to a directory of transcript files.
 */
template <typename Curve> class FileCrsFactory : public CrsFactory<Curve> {
  public:
    FileCrsFactory(std::string path, size_t initial_degree = 0);
    FileCrsFactory(FileCrsFactory&& other) = default;

    std::shared_ptr<bb::srs::factories::ProverCrs<Curve>> get_prover_crs(size_t degree) override;

    std::shared_ptr<bb::srs::factories::VerifierCrs<Curve>> get_verifier_crs(size_t degree = 0) override;

  private:
    std::string path_;
    size_t degree_;
    std::shared_ptr<bb::srs::factories::ProverCrs<Curve>> prover_crs_;
    std::shared_ptr<bb::srs::factories::VerifierCrs<Curve>> verifier_crs_;
};

template <typename Curve> class FileProverCrs : public ProverCrs<Curve> {
  public:
    /**
     * @brief Construct a prover CRS populated with a pippenger point table based on the SRS elements
     * @details Allocates space in monomials_ for 2 * num_points affine elements, populates the first num_points with
     * the raw SRS elements P_i, then overwrites the same memory with the 'pippenger point table' which contains the raw
     * elements P_i at even indices and the endomorphism point (\beta * P_i.x, -P_i.y) at odd indices.
     *
     * @param num_points
     * @param path
     */
    FileProverCrs(const size_t num_points, std::string const& path)
        : num_points(num_points)
    {
        monomials_ = scalar_multiplication::point_table_alloc<typename Curve::AffineElement>(num_points);

        srs::IO<Curve>::read_transcript_g1(monomials_.get(), num_points, path);
        scalar_multiplication::generate_pippenger_point_table<Curve>(monomials_.get(), monomials_.get(), num_points);
    };

    typename Curve::AffineElement* get_monomial_points() { return monomials_.get(); }

    [[nodiscard]] size_t get_monomial_size() const { return num_points; }

  private:
    size_t num_points;
    std::shared_ptr<typename Curve::AffineElement[]> monomials_;
};

template <typename Curve> class FileVerifierCrs : public VerifierCrs<Curve> {
  public:
    FileVerifierCrs(std::string const& path, const size_t num_points);
};

template <> class FileVerifierCrs<curve::BN254> : public VerifierCrs<curve::BN254> {
    using Curve = curve::BN254;

  public:
    FileVerifierCrs(std::string const& path, const size_t num_points = 0);
    virtual ~FileVerifierCrs();
    Curve::G2AffineElement get_g2x() const override { return g2_x; };
    pairing::miller_lines const* get_precomputed_g2_lines() const override { return precomputed_g2_lines; };
    Curve::AffineElement get_g1_identity() const override { return g1_identity; };

  private:
    Curve::AffineElement g1_identity;
    Curve::G2AffineElement g2_x;
    pairing::miller_lines* precomputed_g2_lines;
};

template <> class FileVerifierCrs<curve::Grumpkin> : public VerifierCrs<curve::Grumpkin> {
    using Curve = curve::Grumpkin;

  public:
    FileVerifierCrs(std::string const& path, const size_t num_points);
    virtual ~FileVerifierCrs() = default;
    Curve::AffineElement* get_monomial_points() const override;
    size_t get_monomial_size() const override;
    Curve::AffineElement get_g1_identity() const override { return g1_identity; };

  private:
    Curve::AffineElement g1_identity;
    size_t num_points;
    std::shared_ptr<Curve::AffineElement[]> monomials_;
};

} // namespace bb::srs::factories
