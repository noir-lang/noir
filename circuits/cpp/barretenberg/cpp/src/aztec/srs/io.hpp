#pragma once
#include "../ecc/curves/bn254/g1.hpp"
#include "../ecc/curves/bn254/g2.hpp"
#include <cstdint>
#include <string>

namespace barretenberg {
namespace io {

struct Manifest {
    uint32_t transcript_number;
    uint32_t total_transcripts;
    uint32_t total_g1_points;
    uint32_t total_g2_points;
    uint32_t num_g1_points;
    uint32_t num_g2_points;
    uint32_t start_from;
};

void read_transcript_g1(g1::affine_element* monomials, size_t degree, std::string const& dir, bool is_lagrange = false);

void read_transcript_g2(g2::affine_element& g2_x, std::string const& dir, bool is_lagrange = false);

void read_transcript(g1::affine_element* monomials,
                     g2::affine_element& g2_x,
                     size_t degree,
                     std::string const& path,
                     bool is_lagrange = false);

void read_g1_elements_from_buffer(g1::affine_element* elements, char const* buffer, size_t buffer_size);
void byteswap(g1::affine_element* elements, size_t buffer_size);

void read_g2_elements_from_buffer(g2::affine_element* elements, char const* buffer, size_t buffer_size);
void byteswap(g2::affine_element* elements, size_t buffer_size);

void write_buffer_to_file(std::string const& filename, char const* buffer, size_t buffer_size);

void write_g1_elements_to_buffer(g1::affine_element const* elements, char* buffer, size_t num_elements);

void write_g2_elements_to_buffer(g2::affine_element const* elements, char* buffer, size_t num_elements);

void write_transcript(g1::affine_element const* g1_x,
                      g2::affine_element const* g2_x,
                      Manifest const& manifest,
                      std::string const& dir,
                      bool is_lagrange = false);

} // namespace io
} // namespace barretenberg
