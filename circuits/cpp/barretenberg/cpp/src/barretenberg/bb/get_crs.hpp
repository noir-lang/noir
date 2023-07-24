#pragma once
#include "exec_pipe.hpp"
#include "file_io.hpp"
#include "vinfo.hpp"
#include <barretenberg/ecc/curves/bn254/g1.hpp>
#include <barretenberg/srs/io.hpp>
#include <filesystem>
#include <fstream>
#include <ios>

inline std::vector<uint8_t> download_g1_data(size_t num_points)
{
    size_t g1_start = 28;
    size_t g1_end = g1_start + num_points * 64 - 1;
    std::string command = "curl -s -H \"Range: bytes=" + std::to_string(g1_start) + "-" + std::to_string(g1_end) +
                          "\" 'https://aztec-ignition.s3.amazonaws.com/MAIN%20IGNITION/monomial/transcript00.dat'";

    return exec_pipe(command);
}

inline std::vector<uint8_t> download_g2_data()
{
    size_t g2_start = 28 + 5040000 * 64;
    size_t g2_end = g2_start + 128 - 1;
    std::string command = "curl -s -H \"Range: bytes=" + std::to_string(g2_start) + "-" + std::to_string(g2_end) +
                          "\" 'https://aztec-ignition.s3.amazonaws.com/MAIN%20IGNITION/sealed/transcript00.dat'";

    return exec_pipe(command);
}

inline std::vector<barretenberg::g1::affine_element> get_g1_data(const std::filesystem::path& path, size_t num_points)
{
    std::filesystem::create_directories(path);

    std::ifstream size_file(path / "size");
    size_t size = 0;
    if (size_file) {
        size_file >> size;
        size_file.close();
    }
    if (size >= num_points) {
        vinfo("using cached crs at: ", path);
        auto data = read_file(path / "g1.dat");
        auto points = std::vector<barretenberg::g1::affine_element>(num_points);
        barretenberg::srs::IO<curve::BN254>::read_affine_elements_from_buffer(
            points.data(), (char*)data.data(), num_points * 64);
        return points;
    }

    std::ofstream new_size_file(path / "size");
    if (!new_size_file) {
        throw std::runtime_error("Failed to open size file for writing");
    }
    new_size_file << num_points;
    new_size_file.close();

    vinfo("downloading crs...");
    auto data = download_g1_data(num_points);

    write_file(path / "g1.dat", data);

    auto points = std::vector<barretenberg::g1::affine_element>(num_points);
    barretenberg::srs::IO<curve::BN254>::read_affine_elements_from_buffer(
        points.data(), (char*)data.data(), data.size());
    return points;
}

inline barretenberg::g2::affine_element get_g2_data(const std::filesystem::path& path)
{
    std::filesystem::create_directories(path);

    try {
        auto data = read_file(path / "g2.dat");
        barretenberg::g2::affine_element g2_point;
        barretenberg::srs::IO<curve::BN254>::read_affine_elements_from_buffer(&g2_point, (char*)data.data(), 128);
        return g2_point;
    } catch (std::exception&) {
        auto data = download_g2_data();
        write_file(path / "g2.dat", data);
        barretenberg::g2::affine_element g2_point;
        barretenberg::srs::IO<curve::BN254>::read_affine_elements_from_buffer(&g2_point, (char*)data.data(), 128);
        return g2_point;
    }
}