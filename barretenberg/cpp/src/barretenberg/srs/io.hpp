#pragma once
#include "../ecc/curves/bn254/bn254.hpp"
#include "../ecc/curves/grumpkin/grumpkin.hpp"
#include <concepts>
#include <cstdint>
#include <fstream>
#include <string>
#include <sys/stat.h>

namespace bb::srs {
/**
 * @brief The manifest structure holds the header of a transcript file
 *
 * @details A transript file has the following structure:
 *
 * 00   | XX XX XX XX | This transcript file number
 * 04   | XX XX XX XX | The total number of transcripts
 * 08   | XX XX XX XX | The total number of G1 points (num_g1_points)
 * 0C   | XX XX XX XX | The total number of G2 points (num_g2_points)
 * 10   | XX XX XX XX | The number of G1 points in this file
 * 14   | XX XX XX XX | The number of G2 points in this file
 * 18   | XX XX XX XX | ‾\          ‾\
 *            ...         > G1 point  \
 * 34   | XX XX XX XX | _/             \
 *            ...                       >  num_g1_points * 0x20 bytes
 * YY   | XX XX XX XX | ‾\             /
 *            ...         > G1 point  /
 * YY   | XX XX XX XX | _/          _/
 * YY   | XX XX XX XX | ‾\          ‾\
 *            ...         > G2 point  \
 * YY   | XX XX XX XX | _/             \
 *            ...                       > num_g2_points * 0x40 bytes
 * YY   | XX XX XX XX | ‾\             /
 *            ...         > G2 point  /
 * YY   | XX XX XX XX | _/          _/
 *
 */
struct Manifest {
    uint32_t transcript_number;
    uint32_t total_transcripts;
    uint32_t total_g1_points;
    uint32_t total_g2_points;
    uint32_t num_g1_points;
    uint32_t num_g2_points;
    uint32_t start_from;
};

// Detect whether a curve has a G2AffineElement defined
template <typename Curve>
concept HasG2 = requires { typename Curve::G2AffineElement; };

// If Curve has a G2AffineElement type, check whether T is this type.
template <typename Curve, typename T>
concept GivingG2AffineElementType = std::same_as<T, typename Curve::G2AffineElement>;

// If Curve has a G2AffineElement type, check whether T is this type.
template <typename Curve, typename T>
concept GivingG1AffineElementType = std::same_as<T, typename Curve::AffineElement>;

template <typename Curve> class IO {
    using Fq = typename Curve::BaseField;
    using Fr = typename Curve::ScalarField;
    using Element = typename Curve::Element;
    using AffineElement = typename Curve::AffineElement;

    static constexpr size_t BLAKE2B_CHECKSUM_LENGTH = 64;

    static size_t get_transcript_size(const Manifest& manifest)
    {
        const size_t manifest_size = sizeof(Manifest);
        const size_t g1_buffer_size = sizeof(Fq) * 2 * manifest.num_g1_points;
        size_t result = manifest_size + g1_buffer_size + BLAKE2B_CHECKSUM_LENGTH;

        if constexpr (HasG2<Curve>) {
            const size_t g2_buffer_size = 2 * sizeof(Fq) * 2 * manifest.num_g2_points;
            info(g2_buffer_size);
            result += g2_buffer_size;
        }

        return result;
    }

    static void read_manifest(std::string const& filename, Manifest& manifest)
    {
        std::ifstream file;
        file.open(filename, std::ifstream::binary);
        file.read((char*)&manifest, sizeof(Manifest));
        if (!file) {
            ptrdiff_t read = file.gcount();
            throw_or_abort(format("Only read ", read, " bytes from file but expected ", sizeof(Manifest), "."));
        }
        file.close();

        manifest.transcript_number = ntohl(manifest.transcript_number);
        manifest.total_transcripts = ntohl(manifest.total_transcripts);
        manifest.total_g1_points = ntohl(manifest.total_g1_points);
        manifest.total_g2_points = ntohl(manifest.total_g2_points);
        manifest.num_g1_points = ntohl(manifest.num_g1_points);
        manifest.num_g2_points = ntohl(manifest.num_g2_points);
        manifest.start_from = ntohl(manifest.start_from);
    }

    static void write_buffer_to_file(std::string const& filename, char const* buffer, size_t buffer_size)
    {
        std::ofstream file;
        file.open(filename);
        file.write(&buffer[0], (int)(buffer_size));
        file.close();
    }

    static size_t get_file_size(std::string const& filename)
    {
        struct stat st;
        if (stat(filename.c_str(), &st) != 0) {
            return 0;
        }
        return (size_t)st.st_size;
    }

    static void read_file_into_buffer(
        char* buffer, size_t& size, std::string const& filename, size_t offset = 0, size_t amount = 0)
    {
        size = amount ? amount : get_file_size(filename) - offset;

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

    static std::string get_transcript_path(std::string const& dir, size_t num)
    {
        return format(dir, "/monomial/transcript", (num < 10) ? "0" : "", std::to_string(num), ".dat");
    };

    static bool is_file_exist(std::string const& fileName)
    {
        std::ifstream infile(fileName);
        return infile.good();
    }

    template <typename AffineElementType>
    static void write_elements_to_buffer(AffineElementType const* elements, char* buffer, size_t num_elements)
    {
        if constexpr (GivingG1AffineElementType<Curve, AffineElementType>) {
            uint64_t temp_x[4];
            uint64_t temp_y[4];
            Fq temp_x_g1;
            Fq temp_y_g1;

            if (is_little_endian()) {
                for (size_t i = 0; i < num_elements; ++i) {
                    size_t byte_position_1 = sizeof(Fq) * i * 2;
                    size_t byte_position_2 = sizeof(Fq) * (i * 2 + 1);

                    temp_x_g1 = elements[i].x.from_montgomery_form();
                    temp_y_g1 = elements[i].y.from_montgomery_form();

                    temp_x[0] = __builtin_bswap64(temp_x_g1.data[0]);
                    temp_x[1] = __builtin_bswap64(temp_x_g1.data[1]);
                    temp_x[2] = __builtin_bswap64(temp_x_g1.data[2]);
                    temp_x[3] = __builtin_bswap64(temp_x_g1.data[3]);
                    temp_y[0] = __builtin_bswap64(temp_y_g1.data[0]);
                    temp_y[1] = __builtin_bswap64(temp_y_g1.data[1]);
                    temp_y[2] = __builtin_bswap64(temp_y_g1.data[2]);
                    temp_y[3] = __builtin_bswap64(temp_y_g1.data[3]);

                    memcpy((void*)(buffer + byte_position_1), (void*)temp_x, sizeof(Fq));
                    memcpy((void*)(buffer + byte_position_2), (void*)temp_y, sizeof(Fq));
                }
            }
        } else if constexpr (GivingG2AffineElementType<Curve, AffineElementType>) {
            uint64_t temp_x[8];
            uint64_t temp_y[8];
            Fq temp_x_g2_1;
            Fq temp_x_g2_2;
            Fq temp_y_g2_1;
            Fq temp_y_g2_2;

            if (is_little_endian()) {
                for (size_t i = 0; i < num_elements; ++i) {
                    size_t byte_position_1 = sizeof(Fq) * (4 * i);
                    size_t byte_position_2 = sizeof(Fq) * (4 * i + 2);

                    temp_x_g2_1 = elements[i].x.c0.from_montgomery_form();
                    temp_x_g2_2 = elements[i].x.c1.from_montgomery_form();
                    temp_y_g2_1 = elements[i].y.c0.from_montgomery_form();
                    temp_y_g2_2 = elements[i].y.c1.from_montgomery_form();

                    temp_x[0] = __builtin_bswap64(temp_x_g2_1.data[0]);
                    temp_x[1] = __builtin_bswap64(temp_x_g2_1.data[1]);
                    temp_x[2] = __builtin_bswap64(temp_x_g2_1.data[2]);
                    temp_x[3] = __builtin_bswap64(temp_x_g2_1.data[3]);
                    temp_x[4] = __builtin_bswap64(temp_x_g2_2.data[0]);
                    temp_x[5] = __builtin_bswap64(temp_x_g2_2.data[1]);
                    temp_x[6] = __builtin_bswap64(temp_x_g2_2.data[2]);
                    temp_x[7] = __builtin_bswap64(temp_x_g2_2.data[3]);

                    temp_y[0] = __builtin_bswap64(temp_y_g2_1.data[0]);
                    temp_y[1] = __builtin_bswap64(temp_y_g2_1.data[1]);
                    temp_y[2] = __builtin_bswap64(temp_y_g2_1.data[2]);
                    temp_y[3] = __builtin_bswap64(temp_y_g2_1.data[3]);
                    temp_y[4] = __builtin_bswap64(temp_y_g2_2.data[0]);
                    temp_y[5] = __builtin_bswap64(temp_y_g2_2.data[1]);
                    temp_y[6] = __builtin_bswap64(temp_y_g2_2.data[2]);
                    temp_y[7] = __builtin_bswap64(temp_y_g2_2.data[3]);

                    memcpy((void*)(buffer + byte_position_1), (void*)temp_x, 2 * sizeof(Fq));
                    memcpy((void*)(buffer + byte_position_2), (void*)temp_y, 2 * sizeof(Fq));
                }
            }
        }
    }

  public:
    template <typename AffineElementType> static void byteswap(AffineElementType* elements, size_t elements_size)
    {
        if constexpr (GivingG1AffineElementType<Curve, AffineElementType>) {
            constexpr size_t bytes_per_element = sizeof(AffineElementType);
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
        } else if constexpr (GivingG2AffineElementType<Curve, AffineElementType>) {
            constexpr size_t bytes_per_element = sizeof(AffineElementType);
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
    }

    template <typename AffineElementType>
    static void read_affine_elements_from_buffer(AffineElementType* elements, char const* buffer, size_t buffer_size)
    {
        memcpy((void*)elements, (void*)buffer, buffer_size);
        byteswap<>(elements, buffer_size);
    }

    static void read_transcript_g1(AffineElement* monomials, size_t degree, std::string const& dir)
    {
        size_t num = 0;
        size_t num_read = 0;
        std::string path = get_transcript_path(dir, num);

        while (is_file_exist(path) && num_read < degree) {
            Manifest manifest;
            read_manifest(path, manifest);

            auto offset = sizeof(Manifest);
            const size_t num_to_read = std::min((size_t)manifest.num_g1_points, degree - num_read);
            const size_t g1_buffer_size = sizeof(Fq) * 2 * num_to_read;

            char* buffer = (char*)&monomials[num_read];
            size_t size = 0;

            // We must pass the size actually read to the second call, not the desired
            // g1_buffer_size as the file may have been smaller than this.
            read_file_into_buffer(buffer, size, path, offset, g1_buffer_size);
            srs::IO<Curve>::byteswap(&monomials[num_read], size);

            num_read += num_to_read;
            path = get_transcript_path(dir, ++num);
        }

        const bool monomial_srs_condition = num_read < degree;
        if (monomial_srs_condition) {
            throw_or_abort(
                format("Only read ",
                       num_read,
                       " points from ",
                       path,
                       ", but require ",
                       degree,
                       ". Is your srs large enough? Either run bootstrap.sh to download the transcript.dat "
                       "files to `srs_db/ignition/`, or you might need to download extra transcript.dat files "
                       "by editing `srs_db/download_ignition.sh` or in the case of grumpkin points, use "
                       " `grumpkin_srs_gen` (but be careful, as this suggests you've "
                       "just changed a circuit to exceed a new 'power of two' boundary)."));
        }
    }

    static void read_transcript_g2(auto& g2_x, std::string const& dir)
        requires HasG2<Curve>
    {
        const size_t g2_size = sizeof(typename Curve::G2BaseField) * 2;
        std::string path = format(dir, "/g2.dat");

        if (is_file_exist(path)) {
            char* buffer = (char*)&g2_x;
            size_t size = 0;

            // Again, size passed to second function should be size actually read
            read_file_into_buffer(buffer, size, path, 0, g2_size);
            byteswap(&g2_x, size);

            return;
        }

        // Get transcript starting at g0.dat
        path = get_transcript_path(dir, 0);

        Manifest manifest;
        read_manifest(path, manifest);

        const size_t g2_buffer_offset = sizeof(Fq) * 2 * manifest.num_g1_points;
        auto offset = sizeof(Manifest) + g2_buffer_offset;

        char* buffer = (char*)&g2_x;
        size_t size = 0;

        // Again, size passed to second function should be size actually read
        read_file_into_buffer(buffer, size, path, offset, g2_size);
        byteswap(&g2_x, size);
    }

    static void read_transcript(AffineElement* monomials, auto& g2_x, size_t degree, std::string const& path)
        requires HasG2<Curve>
    {
        read_transcript_g1(monomials, degree, path);
        read_transcript_g2(g2_x, path);
    }

    static void read_transcript(AffineElement* monomials, size_t degree, std::string const& path)
    {
        read_transcript_g1(monomials, degree, path);
    }

    // This function is a vestige of the Lagrange form transcript work, and it is not used anywhere.
    static void write_transcript(AffineElement const* g1_x,
                                 auto const* g2_x,
                                 Manifest const& manifest,
                                 std::string const& dir)
        requires HasG2<Curve>
    {
        const size_t num_g1_x = manifest.num_g1_points;
        const size_t num_g2_x = manifest.num_g2_points;
        const size_t transcript_num = manifest.transcript_number;
        const size_t manifest_size = sizeof(Manifest);
        const size_t g1_buffer_size = sizeof(Fq) * 2 * num_g1_x;
        const size_t g2_buffer_size = sizeof(Fq) * 4 * num_g2_x;
        const size_t transcript_size = manifest_size + g1_buffer_size + g2_buffer_size;
        std::string path = get_transcript_path(dir, transcript_num);
        std::vector<char> buffer(transcript_size);

        Manifest net_manifest;
        net_manifest.transcript_number = htonl(manifest.transcript_number);
        net_manifest.total_transcripts = htonl(manifest.total_transcripts);
        net_manifest.total_g1_points = htonl(manifest.total_g1_points);
        net_manifest.total_g2_points = htonl(manifest.total_g2_points);
        net_manifest.num_g1_points = htonl(manifest.num_g1_points);
        net_manifest.num_g2_points = htonl(manifest.num_g2_points);
        net_manifest.start_from = htonl(manifest.start_from);

        std::copy(&net_manifest, &net_manifest + 1, (Manifest*)&buffer[0]);

        write_g1_elements_to_buffer(g1_x, &buffer[manifest_size], num_g1_x);
        write_g2_elements_to_buffer(g2_x, &buffer[manifest_size + g1_buffer_size], num_g2_x);
        write_buffer_to_file(path, &buffer[0], transcript_size);
    }

    static void write_transcript(AffineElement const* g1_x, Manifest const& manifest, std::string const& dir)
    {
        const size_t num_g1_x = manifest.num_g1_points;
        const size_t transcript_num = manifest.transcript_number;
        const size_t manifest_size = sizeof(Manifest);
        const size_t g1_buffer_size = sizeof(Fq) * 2 * num_g1_x;
        const size_t transcript_size = manifest_size + g1_buffer_size;
        std::string path = get_transcript_path(dir, transcript_num);
        std::vector<char> buffer(transcript_size);

        Manifest net_manifest;
        net_manifest.transcript_number = htonl(manifest.transcript_number);
        net_manifest.total_transcripts = htonl(manifest.total_transcripts);
        net_manifest.total_g1_points = htonl(manifest.total_g1_points);
        net_manifest.total_g2_points = htonl(0);
        net_manifest.num_g1_points = htonl(manifest.num_g1_points);
        net_manifest.num_g2_points = htonl(0);
        net_manifest.start_from = htonl(manifest.start_from);

        std::copy(&net_manifest, &net_manifest + 1, (Manifest*)&buffer[0]);

        write_elements_to_buffer<AffineElement>(g1_x, &buffer[manifest_size], num_g1_x);
        write_buffer_to_file(path, &buffer[0], transcript_size);
    }
};

} // namespace bb::srs
