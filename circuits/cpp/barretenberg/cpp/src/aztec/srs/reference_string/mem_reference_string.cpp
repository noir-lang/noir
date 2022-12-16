#include "mem_reference_string.hpp"
#include "../io.hpp"

#include <common/streams.hpp>
#include <ecc/curves/bn254/pairing.hpp>

#include <sstream>

namespace waffle {

VerifierMemReferenceString::VerifierMemReferenceString(uint8_t const* g2x)
    : precomputed_g2_lines(
          (barretenberg::pairing::miller_lines*)(aligned_alloc(64, sizeof(barretenberg::pairing::miller_lines) * 2)))
{
    barretenberg::io::read_g2_elements_from_buffer(&g2_x, (char*)g2x, 128);

    barretenberg::pairing::precompute_miller_lines(barretenberg::g2::one, precomputed_g2_lines[0]);
    barretenberg::pairing::precompute_miller_lines(g2_x, precomputed_g2_lines[1]);
}

VerifierMemReferenceString::~VerifierMemReferenceString()
{
    aligned_free(precomputed_g2_lines);
}

} // namespace waffle