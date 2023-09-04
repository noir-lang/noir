#pragma once
#include <barretenberg/common/log.hpp>
#include <fstream>
#include <vector>

inline std::vector<uint8_t> read_file(const std::string& filename)
{
    // Open the file in binary mode and move to the end.
    std::ifstream file(filename, std::ios::binary | std::ios::ate);
    if (!file) {
        throw std::runtime_error("Unable to open file: " + filename);
    }

    // Get the file size.
    std::streamsize size = file.tellg();
    if (size <= 0) {
        throw std::runtime_error("File is empty or there's an error reading it: " + filename);
    }

    // Create a vector with enough space for the file data.
    std::vector<uint8_t> fileData((size_t)size);

    // Go back to the start of the file and read all its contents.
    file.seekg(0, std::ios::beg);
    file.read(reinterpret_cast<char*>(fileData.data()), size);

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