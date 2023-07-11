#pragma once
#include <barretenberg/common/log.hpp>
#include <fstream>
#include <vector>

inline std::vector<uint8_t> read_file(const std::string& filename)
{
    std::ifstream file(filename, std::ios::binary);
    if (!file) {
        throw std::runtime_error(format("Unable to open file: ", filename));
    }
    std::vector<uint8_t> fileData((std::istreambuf_iterator<char>(file)), std::istreambuf_iterator<char>());
    return fileData;
}

inline void write_file(const std::string& filename, std::vector<uint8_t> const& data)
{
    std::ofstream file(filename, std::ios::binary);
    if (!file) {
        throw std::runtime_error("Failed to open data file for writing");
    }
    file.write((char*)data.data(), (std::streamsize)data.size());
    file.close();
}