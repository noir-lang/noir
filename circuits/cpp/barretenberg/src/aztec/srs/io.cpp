#include "io.hpp"
#include <common/mem.hpp>
#include <common/net.hpp>
#include <fstream>
#include <sys/stat.h>

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

constexpr size_t BLAKE2B_CHECKSUM_LENGTH = 64;

size_t get_transcript_size(const Manifest& manifest)
{
    const size_t manifest_size = sizeof(Manifest);
    const size_t g1_buffer_size = sizeof(fq) * 2 * manifest.num_g1_points;
    const size_t g2_buffer_size = sizeof(fq2) * 2 * manifest.num_g2_points;
    return manifest_size + g1_buffer_size + g2_buffer_size + BLAKE2B_CHECKSUM_LENGTH;
}

void read_manifest(std::string const& filename, Manifest& manifest)
{
    std::ifstream file;
    file.open(filename, std::ifstream::binary);
    file.read((char*)&manifest, sizeof(Manifest));
    file.close();

    manifest.transcript_number = ntohl(manifest.transcript_number);
    manifest.total_transcripts = ntohl(manifest.total_transcripts);
    manifest.total_g1_points = ntohl(manifest.total_g1_points);
    manifest.total_g2_points = ntohl(manifest.total_g2_points);
    manifest.num_g1_points = ntohl(manifest.num_g1_points);
    manifest.num_g2_points = ntohl(manifest.num_g2_points);
    manifest.start_from = ntohl(manifest.start_from);
}

void read_g1_elements_from_buffer(g1::affine_element* elements, char const* buffer, size_t buffer_size)
{
    constexpr size_t bytes_per_element = sizeof(g1::affine_element);
    size_t num_elements = buffer_size / bytes_per_element;

    memcpy((void*)elements, (void*)buffer, buffer_size);
    if (is_little_endian()) {
        for (size_t i = 0; i < num_elements; ++i) {
            elements[i].x.data[0] = __builtin_bswap64(elements[i].x.data[0]);
            elements[i].x.data[1] = __builtin_bswap64(elements[i].x.data[1]);
            elements[i].x.data[2] = __builtin_bswap64(elements[i].x.data[2]);
            elements[i].x.data[3] = __builtin_bswap64(elements[i].x.data[3]);
            elements[i].y.data[0] = __builtin_bswap64(elements[i].y.data[0]);
            elements[i].y.data[1] = __builtin_bswap64(elements[i].y.data[1]);
            elements[i].y.data[2] = __builtin_bswap64(elements[i].y.data[2]);
            elements[i].y.data[3] = __builtin_bswap64(elements[i].y.data[3]);
            elements[i].x.self_to_montgomery_form();
            elements[i].y.self_to_montgomery_form();
        }
    }
}

void read_g2_elements_from_buffer(g2::affine_element* elements, char const* buffer, size_t buffer_size)
{
    constexpr size_t bytes_per_element = sizeof(g2::affine_element);
    size_t num_elements = buffer_size / bytes_per_element;

    memcpy((void*)elements, (void*)buffer, buffer_size);

    if (is_little_endian()) {
        for (size_t i = 0; i < num_elements; ++i) {
            elements[i].x.c0.data[0] = __builtin_bswap64(elements[i].x.c0.data[0]);
            elements[i].x.c0.data[1] = __builtin_bswap64(elements[i].x.c0.data[1]);
            elements[i].x.c0.data[2] = __builtin_bswap64(elements[i].x.c0.data[2]);
            elements[i].x.c0.data[3] = __builtin_bswap64(elements[i].x.c0.data[3]);
            elements[i].y.c0.data[0] = __builtin_bswap64(elements[i].y.c0.data[0]);
            elements[i].y.c0.data[1] = __builtin_bswap64(elements[i].y.c0.data[1]);
            elements[i].y.c0.data[2] = __builtin_bswap64(elements[i].y.c0.data[2]);
            elements[i].y.c0.data[3] = __builtin_bswap64(elements[i].y.c0.data[3]);
            elements[i].x.c1.data[0] = __builtin_bswap64(elements[i].x.c1.data[0]);
            elements[i].x.c1.data[1] = __builtin_bswap64(elements[i].x.c1.data[1]);
            elements[i].x.c1.data[2] = __builtin_bswap64(elements[i].x.c1.data[2]);
            elements[i].x.c1.data[3] = __builtin_bswap64(elements[i].x.c1.data[3]);
            elements[i].y.c1.data[0] = __builtin_bswap64(elements[i].y.c1.data[0]);
            elements[i].y.c1.data[1] = __builtin_bswap64(elements[i].y.c1.data[1]);
            elements[i].y.c1.data[2] = __builtin_bswap64(elements[i].y.c1.data[2]);
            elements[i].y.c1.data[3] = __builtin_bswap64(elements[i].y.c1.data[3]);
            elements[i].x.c0.self_to_montgomery_form();
            elements[i].x.c1.self_to_montgomery_form();
            elements[i].y.c0.self_to_montgomery_form();
            elements[i].y.c1.self_to_montgomery_form();
        }
    }
}

size_t get_file_size(std::string const& filename)
{
    struct stat st;
    if (stat(filename.c_str(), &st) != 0) {
        return 0;
    }
    return (size_t)st.st_size;
}

std::vector<char> read_file_into_buffer(std::string const& filename, size_t offset = 0, size_t size = 0)
{
    size_t file_size = size ? size : get_file_size(filename);
    std::vector<char> buffer(file_size);
    std::ifstream file;
    file.open(filename, std::ifstream::binary);
    file.seekg((int)offset);
    file.read(&buffer[0], (int)buffer.size());
    file.close();
    return buffer;
}

std::string get_transcript_path(std::string const& dir, size_t num)
{
    return dir + "/transcript" + (num < 10 ? "0" : "") + std::to_string(num) + ".dat";
};

bool is_file_exist(std::string const& fileName)
{
    std::ifstream infile(fileName);
    return infile.good();
}

void read_transcript_g1(g1::affine_element* monomials, size_t degree, std::string const& dir)
{
    // read g1 elements at second array position - first point is the basic generator
    monomials[0] = g1::affine_one;

    size_t num = 0;
    size_t num_read = 1;
    std::string path = get_transcript_path(dir, num);

    while (is_file_exist(path) && num_read < degree) {
        Manifest manifest;
        read_manifest(path, manifest);

        auto offset = sizeof(Manifest);
        const size_t num_to_read = std::min((size_t)manifest.num_g1_points, degree - num_read);
        const size_t g1_buffer_size = sizeof(fq) * 2 * num_to_read;

        auto buffer = read_file_into_buffer(path, offset, g1_buffer_size);

        read_g1_elements_from_buffer(&monomials[num_read], buffer.data(), g1_buffer_size);

        num_read += num_to_read;
        path = get_transcript_path(dir, ++num);
    }

    if (num == 0) {
#ifdef __wasm__
        std::abort();
#else
        throw std::runtime_error("No input files found.");
#endif
    }
}

void read_transcript_g2(g2::affine_element& g2_x, std::string const& dir)
{
    std::string path = get_transcript_path(dir, 0);
    Manifest manifest;
    read_manifest(path, manifest);

    const size_t g2_buffer_offset = sizeof(fq) * 2 * manifest.num_g1_points;
    const size_t g2_size = sizeof(fq2) * 2;
    auto offset = sizeof(Manifest) + g2_buffer_offset;

    auto buffer = read_file_into_buffer(path, offset, g2_size);

    read_g2_elements_from_buffer(&g2_x, buffer.data(), g2_size);
}

void read_transcript(g1::affine_element* monomials, g2::affine_element& g2_x, size_t degree, std::string const& path)
{
    read_transcript_g1(monomials, degree, path);
    read_transcript_g2(g2_x, path);
}

} // namespace io
} // namespace barretenberg
