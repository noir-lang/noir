#include "file_reference_string.hpp"
#include <ecc/curves/bn254/pairing.hpp>
#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include <srs/io.hpp>

#ifndef NO_MULTITHREADING
#include <omp.h>
#endif

namespace waffle {

VerifierFileReferenceString::VerifierFileReferenceString(std::string const& path)
{
    precomputed_g2_lines =
        (barretenberg::pairing::miller_lines*)(aligned_alloc(64, sizeof(barretenberg::pairing::miller_lines) * 2));
    barretenberg::io::read_transcript_g2(g2_x, path);
    barretenberg::pairing::precompute_miller_lines(barretenberg::g2::one, precomputed_g2_lines[0]);
    barretenberg::pairing::precompute_miller_lines(g2_x, precomputed_g2_lines[1]);
}

VerifierFileReferenceString::~VerifierFileReferenceString()
{
    aligned_free(precomputed_g2_lines);
}

FileReferenceString::FileReferenceString(const size_t num_points, std::string const& path)
{
#ifndef NO_MULTITHREADING
    const size_t num_threads = static_cast<size_t>(omp_get_max_threads());
#else
    const size_t num_threads = 1;
#endif
    const size_t prefetch_overflow = 16 * num_threads;

    monomials = (barretenberg::g1::affine_element*)(aligned_alloc(
        64, sizeof(barretenberg::g1::affine_element) * (2 * num_points + prefetch_overflow)));
    barretenberg::io::read_transcript_g1(monomials, num_points, path);
    barretenberg::scalar_multiplication::generate_pippenger_point_table(monomials, monomials, num_points);
}

FileReferenceString::~FileReferenceString()
{
    aligned_free(monomials);
}

} // namespace waffle