#include "io.hpp"
#include <common/mem.hpp>
#include <common/net.hpp>
#include <common/throw_or_abort.hpp>
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
    memcpy((void*)elements, (void*)buffer, buffer_size);
    byteswap(elements, buffer_size);
}

void byteswap(g1::affine_element* elements, size_t elements_size)
{
    constexpr size_t bytes_per_element = sizeof(g1::affine_element);
    size_t num_elements = elements_size / bytes_per_element;

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
    memcpy((void*)elements, (void*)buffer, buffer_size);
    byteswap(elements, buffer_size);
}

void byteswap(g2::affine_element* elements, size_t elements_size)
{
    constexpr size_t bytes_per_element = sizeof(g2::affine_element);
    size_t num_elements = elements_size / bytes_per_element;

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

void read_file_into_buffer(
    char* buffer, size_t& size, std::string const& filename, size_t offset = 0, size_t amount = 0)
{
    size = amount ? amount : get_file_size(filename);

    std::ifstream file;
    file.open(filename, std::ifstream::binary);
    file.seekg((int)offset);

    // Read the desired size, but return the actual size read
    file.read(buffer, (int)size);
    if (!file) {
        ptrdiff_t read = file.gcount();
        throw_or_abort(format("Only read ", read, " bytes from file but expected ", size, "."));
    }

    file.close();
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

    info("Reading transcript g1 from ", path);

    while (is_file_exist(path) && num_read < degree) {
        Manifest manifest;
        read_manifest(path, manifest);

        auto offset = sizeof(Manifest);
        const size_t num_to_read = std::min((size_t)manifest.num_g1_points, degree - num_read);
        const size_t g1_buffer_size = sizeof(fq) * 2 * num_to_read;

        char* buffer = (char*)&monomials[num_read];
        size_t size = 0;

        // We must pass the size actually read to the second call, not the desired
        // g1_buffer_size as the file may have been smaller than this.
        read_file_into_buffer(buffer, size, path, offset, g1_buffer_size);
        byteswap(&monomials[num_read], size);

        num_read += num_to_read;
        path = get_transcript_path(dir, ++num);
    }

    if (num_read < degree) {
        throw_or_abort(format("Only read ", num_read, " points but require ", degree, ". Is your srs large enough?"));
    }
}

void read_transcript_g2(g2::affine_element& g2_x, std::string const& dir)
{

    const size_t g2_size = sizeof(fq2) * 2;
    std::string path = dir + "/g2.dat";

    if (is_file_exist(path)) {
        info("Reading transcript g2 from ", path);

        char* buffer = (char*)&g2_x;
        size_t size = 0;

        // Again, size passed to second function should be size actually read
        read_file_into_buffer(buffer, size, path, 0, g2_size);
        byteswap(&g2_x, size);

        return;
    }

    // Get transcript starting at g0.dat
    path = get_transcript_path(dir, 0);

    info("Reading transcript g2 from ", path);

    Manifest manifest;
    read_manifest(path, manifest);

    const size_t g2_buffer_offset = sizeof(fq) * 2 * manifest.num_g1_points;
    auto offset = sizeof(Manifest) + g2_buffer_offset;

    char* buffer = (char*)&g2_x;
    size_t size = 0;

    // Again, size passed to second function should be size actually read
    read_file_into_buffer(buffer, size, path, offset, g2_size);
    byteswap(&g2_x, size);
}

void read_transcript(g1::affine_element* monomials, g2::affine_element& g2_x, size_t degree, std::string const& path)
{
    read_transcript_g1(monomials, degree, path);
    read_transcript_g2(g2_x, path);
}

} // namespace io
} // namespace barretenberg
