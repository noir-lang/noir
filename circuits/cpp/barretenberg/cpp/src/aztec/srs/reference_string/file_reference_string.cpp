#include "file_reference_string.hpp"
#include "../io.hpp"

#include <ecc/curves/bn254/pairing.hpp>

namespace waffle {

VerifierFileReferenceString::VerifierFileReferenceString(std::string const& path, bool is_lagrange)
    : precomputed_g2_lines(
          (barretenberg::pairing::miller_lines*)(aligned_alloc(64, sizeof(barretenberg::pairing::miller_lines) * 2)))
{

    barretenberg::io::read_transcript_g2(g2_x, path, is_lagrange);
    barretenberg::pairing::precompute_miller_lines(barretenberg::g2::one, precomputed_g2_lines[0]);
    barretenberg::pairing::precompute_miller_lines(g2_x, precomputed_g2_lines[1]);
}

VerifierFileReferenceString::~VerifierFileReferenceString()
{
    aligned_free(precomputed_g2_lines);
}

} // namespace waffle
