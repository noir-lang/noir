#pragma once

#include <ecc/curves/grumpkin/grumpkin.hpp>

namespace crypto {
namespace pedersen {

inline std::vector<grumpkin::fq> convert_buffer_to_field(const std::vector<uint8_t>& input)
{
    const size_t num_bytes = input.size();
    const size_t bytes_per_element = 31;
    size_t num_elements = (num_bytes % bytes_per_element != 0) + (num_bytes / bytes_per_element);

    const auto slice = [](const std::vector<uint8_t>& data, const size_t start, const size_t slice_size) {
        uint256_t result(0);
        for (size_t i = 0; i < slice_size; ++i) {
            result = (result << uint256_t(8));
            result += uint256_t(data[i + start]);
        }
        return grumpkin::fq(result);
    };

    std::vector<grumpkin::fq> elements;
    for (size_t i = 0; i < num_elements; ++i) {
        size_t bytes_to_slice = 0;
        if (i == num_elements - 1) {
            bytes_to_slice = num_bytes - (i * bytes_per_element);
        } else {
            bytes_to_slice = bytes_per_element;
        }
        grumpkin::fq element = slice(input, i * bytes_per_element, bytes_to_slice);
        elements.emplace_back(element);
    }
    return elements;
}
} // namespace pedersen
} // namespace crypto