#pragma once
#include <barretenberg/common/log.hpp>
#include <cstdint>
#include <fstream>
#include <ios>
#include <vector>

inline size_t get_file_size(std::string const& filename)
{
    // Open the file in binary mode and move to the end.
    std::ifstream file(filename, std::ios::binary | std::ios::ate);
    if (!file) {
        return 0;
    }

    file.seekg(0, std::ios::end);
    return (size_t)file.tellg();
}

inline std::vector<uint8_t> read_file(const std::string& filename, size_t bytes = 0)
{
    // Get the file size.
    auto size = get_file_size(filename);
    if (size <= 0) {
        throw std::runtime_error("File is empty or there's an error reading it: " + filename);
    }

    auto to_read = bytes == 0 ? size : bytes;

    std::ifstream file(filename, std::ios::binary);
    if (!file) {
        throw std::runtime_error("Unable to open file: " + filename);
    }

    // Create a vector with enough space for the file data.
    std::vector<uint8_t> fileData(to_read);

    // Read all its contents.
    file.read(reinterpret_cast<char*>(fileData.data()), (std::streamsize)to_read);

    return fileData;
}

inline void write_file(const std::string& filename, std::vector<uint8_t> const& data)
{
    std::ofstream file(filename, std::ios::binary);
    if (!file) {
        throw std::runtime_error("Failed to open data file for writing: " + filename);
    }
    file.write((char*)data.data(), (std::streamsize)data.size());
    file.close();
}