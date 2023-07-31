#include "file_crs_factory.hpp"
#include "../io.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/ecc/curves/bn254/pairing.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/ecc/scalar_multiplication/point_table.hpp"
#include "barretenberg/ecc/scalar_multiplication/scalar_multiplication.hpp"

namespace barretenberg::srs::factories {

FileVerifierCrs<curve::BN254>::FileVerifierCrs(std::string const& path, const size_t)
    : precomputed_g2_lines(
          (barretenberg::pairing::miller_lines*)(aligned_alloc(64, sizeof(barretenberg::pairing::miller_lines) * 2)))
{
    using Curve = curve::BN254;
    auto point_buf = scalar_multiplication::point_table_alloc<Curve::AffineElement>(1);
    srs::IO<Curve>::read_transcript_g1(point_buf.get(), 1, path);
    srs::IO<curve::BN254>::read_transcript_g2(g2_x, path);
    barretenberg::pairing::precompute_miller_lines(barretenberg::g2::one, precomputed_g2_lines[0]);
    barretenberg::pairing::precompute_miller_lines(g2_x, precomputed_g2_lines[1]);
    first_g1 = point_buf[0];
}

FileVerifierCrs<curve::BN254>::~FileVerifierCrs()
{
    aligned_free(precomputed_g2_lines);
}

FileVerifierCrs<curve::Grumpkin>::FileVerifierCrs(std::string const& path, const size_t num_points)
    : num_points(num_points)
{
    using Curve = curve::Grumpkin;
    monomials_ = scalar_multiplication::point_table_alloc<Curve::AffineElement>(num_points);
    srs::IO<Curve>::read_transcript_g1(monomials_.get(), num_points, path);
    scalar_multiplication::generate_pippenger_point_table<Curve>(monomials_.get(), monomials_.get(), num_points);
    first_g1 = monomials_[0];
};

curve::Grumpkin::AffineElement* FileVerifierCrs<curve::Grumpkin>::get_monomial_points() const
{
    return monomials_.get();
}

size_t FileVerifierCrs<curve::Grumpkin>::get_monomial_size() const
{
    return num_points;
}

template <typename Curve>
FileCrsFactory<Curve>::FileCrsFactory(std::string path, size_t initial_degree)
    : path_(std::move(path))
    , degree_(initial_degree)
{}

template <typename Curve>
std::shared_ptr<barretenberg::srs::factories::ProverCrs<Curve>> FileCrsFactory<Curve>::get_prover_crs(size_t degree)
{
    if (degree != degree_ || !prover_crs_) {
        prover_crs_ = std::make_shared<FileProverCrs<Curve>>(degree, path_);
        degree_ = degree;
    }
    return prover_crs_;
}

template <typename Curve>
std::shared_ptr<barretenberg::srs::factories::VerifierCrs<Curve>> FileCrsFactory<Curve>::get_verifier_crs(size_t degree)
{
    if (degree != degree_ || !verifier_crs_) {
        verifier_crs_ = std::make_shared<FileVerifierCrs<Curve>>(path_, degree);
        degree_ = degree;
    }
    return verifier_crs_;
}

template class FileProverCrs<curve::BN254>;
template class FileProverCrs<curve::Grumpkin>;
template class FileCrsFactory<curve::BN254>;
template class FileCrsFactory<curve::Grumpkin>;

} // namespace barretenberg::srs::factories
