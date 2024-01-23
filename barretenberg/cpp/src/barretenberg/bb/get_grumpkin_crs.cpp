#include "get_grumpkin_crs.hpp"

namespace {
std::vector<uint8_t> download_grumpkin_g1_data(size_t num_points)
{
    size_t g1_start = 28;
    size_t g1_end = g1_start + num_points * 64 - 1;

    std::string url = "https://aztec-ignition.s3.amazonaws.com/TEST%20GRUMPKIN/monomial/transcript00.dat";

    // IMPORTANT: this currently uses a shell, DO NOT let user-controlled strings here.
    std::string command =
        "curl -s -H \"Range: bytes=" + std::to_string(g1_start) + "-" + std::to_string(g1_end) + "\" '" + url + "'";

    auto data = exec_pipe(command);
    // Header + num_points * sizeof point.
    if (data.size() < g1_end - g1_start) {
        throw std::runtime_error("Failed to download grumpkin g1 data.");
    }

    return data;
}
} // namespace

namespace bb {
std::vector<curve::Grumpkin::AffineElement> get_grumpkin_g1_data(const std::filesystem::path& path, size_t num_points)
{
    std::filesystem::create_directories(path);
    std::ifstream size_file(path / "grumpkin_size");
    size_t size = 0;
    if (size_file) {
        size_file >> size;
        size_file.close();
    }
    if (size >= num_points) {
        auto file = path / "grumpkin_g1.dat";
        vinfo("using cached crs at: ", file);
        auto data = read_file(file, 28 + num_points * 64);
        auto points = std::vector<curve::Grumpkin::AffineElement>(num_points);
        auto size_of_points_in_bytes = num_points * 64;
        srs::IO<curve::Grumpkin>::read_affine_elements_from_buffer(
            points.data(), (char*)data.data(), size_of_points_in_bytes);
        return points;
    }

    vinfo("downloading grumpkin crs...");
    auto data = download_grumpkin_g1_data(num_points);
    write_file(path / "grumpkin_g1.dat", data);

    std::ofstream new_size_file(path / "grumpkin_size");
    if (!new_size_file) {
        throw std::runtime_error("Failed to open size file for writing");
    }
    new_size_file << num_points;
    new_size_file.close();

    auto points = std::vector<curve::Grumpkin::AffineElement>(num_points);
    srs::IO<curve::Grumpkin>::read_affine_elements_from_buffer(points.data(), (char*)data.data(), data.size());
    return points;
}
} // namespace bb
