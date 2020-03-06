#pragma once 

#include "../../curves/bn254/fr.hpp"
#include "../../curves/bn254/g1.hpp"

namespace waffle {
namespace transcript_helpers {
inline std::vector<uint8_t> convert_field_element(const barretenberg::fr& ele)
{
    std::vector<uint8_t> buffer(sizeof(barretenberg::fr));
    barretenberg::fr::serialize_to_buffer(ele, &buffer[0]);
    return buffer;
}

inline std::vector<uint8_t> convert_field_elements(const std::vector<barretenberg::fr>& ele)
{
    std::vector<uint8_t> buffer(sizeof(barretenberg::fr) * ele.size());  
    for (size_t i = 0; i < ele.size(); ++i)
    {
        barretenberg::fr::serialize_to_buffer(ele[i], &buffer[i * sizeof(barretenberg::fr)]);
    }
    return buffer;
}

inline std::vector<uint8_t> convert_g1_element(const barretenberg::g1::affine_element& ele)
{
    std::vector<uint8_t> buffer(sizeof(barretenberg::g1::affine_element));
    barretenberg::g1::affine_element::serialize_to_buffer(ele, &buffer[0]);
    return buffer;
}

inline std::vector<barretenberg::fr> read_field_elements(const std::vector<uint8_t>& buffer)
{
    const size_t num_elements = buffer.size() / sizeof(barretenberg::fr);
    std::vector<barretenberg::fr> elements;
    for (size_t i = 0; i < num_elements; ++i)
    {
        elements.push_back(barretenberg::fr::serialize_from_buffer(&buffer[i * sizeof(barretenberg::fr)]));
    }
    return elements;
}
} // namespace transcript_helpers
} // namespace waffle