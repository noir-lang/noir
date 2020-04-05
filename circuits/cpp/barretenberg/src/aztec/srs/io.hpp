#pragma once
#include "../ecc/curves/bn254/g1.hpp"
#include "../ecc/curves/bn254/g2.hpp"
#include <cstdint>
#include <string>

namespace barretenberg {
namespace io {

void read_transcript_g1(g1::affine_element* monomials, size_t degree, std::string const& dir);

void read_transcript_g2(g2::affine_element& g2_x, std::string const& dir);

void read_transcript(g1::affine_element* monomials, g2::affine_element& g2_x, size_t degree, std::string const& path);

void read_g1_elements_from_buffer(g1::affine_element* elements, char const* buffer, size_t buffer_size);

void read_g2_elements_from_buffer(g2::affine_element* elements, char const* buffer, size_t buffer_size);

} // namespace io
} // namespace barretenberg
