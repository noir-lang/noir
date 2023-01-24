#include "mem_reference_string.hpp"
#include <common/streams.hpp>
#include <ecc/curves/bn254/pairing.hpp>
#include <srs/io.hpp>
#include <sstream>

#ifndef NO_MULTITHREADING
#include <omp.h>
#endif

namespace waffle {

VerifierMemReferenceString::VerifierMemReferenceString(uint8_t const* buffer)
{
    barretenberg::io::read_g2_elements_from_buffer(&g2_x, (char*)buffer, 128);

    precomputed_g2_lines =
        (barretenberg::pairing::miller_lines*)(aligned_alloc(64, sizeof(barretenberg::pairing::miller_lines) * 2));

    barretenberg::pairing::precompute_miller_lines(barretenberg::g2::one, precomputed_g2_lines[0]);
    barretenberg::pairing::precompute_miller_lines(g2_x, precomputed_g2_lines[1]);
}

VerifierMemReferenceString::~VerifierMemReferenceString()
{
    aligned_free(precomputed_g2_lines);
}

} // namespace waffle