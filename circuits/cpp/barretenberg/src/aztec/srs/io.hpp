#pragma once
#include <cstdint>
#include <string>
#include <ecc/curves/bn254/g1.hpp>
#include <ecc/curves/bn254/g2.hpp>

namespace barretenberg {
namespace io {

void read_transcript_g1(g1::affine_element* monomials, size_t degree, std::string const& dir);

void read_transcript_g2(g2::affine_element& g2_x, std::string const& dir);

void read_transcript(g1::affine_element* monomials, g2::affine_element& g2_x, size_t degree, std::string const& path);

} // namespace io
} // namespace barretenberg
