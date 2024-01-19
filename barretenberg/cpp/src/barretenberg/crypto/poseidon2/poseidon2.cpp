#include "poseidon2.hpp"

namespace bb::crypto {
/**
 * @brief Hashes a vector of field elements
 */
template <typename Params>
typename Poseidon2<Params>::FF Poseidon2<Params>::hash(const std::vector<typename Poseidon2<Params>::FF>& input)
{
    auto input_span = input;
    return Sponge::hash_fixed_length(input_span);
}

/**
 * @brief Hashes vector of bytes by chunking it into 31 byte field elements and calling hash()
 * @details Slice function cuts out the required number of bytes from the byte vector
 */
template <typename Params>
typename Poseidon2<Params>::FF Poseidon2<Params>::hash_buffer(const std::vector<uint8_t>& input)
{
    const size_t num_bytes = input.size();
    const size_t bytes_per_element = 31;
    size_t num_elements = static_cast<size_t>(num_bytes % bytes_per_element != 0) + (num_bytes / bytes_per_element);

    const auto slice = [](const std::vector<uint8_t>& data, const size_t start, const size_t slice_size) {
        uint256_t result(0);
        for (size_t i = 0; i < slice_size; ++i) {
            result = (result << uint256_t(8));
            result += uint256_t(data[i + start]);
        }
        return FF(result);
    };

    std::vector<FF> converted;
    for (size_t i = 0; i < num_elements - 1; ++i) {
        size_t bytes_to_slice = bytes_per_element;
        FF element = slice(input, i * bytes_per_element, bytes_to_slice);
        converted.emplace_back(element);
    }
    size_t bytes_to_slice = num_bytes - ((num_elements - 1) * bytes_per_element);
    FF element = slice(input, (num_elements - 1) * bytes_per_element, bytes_to_slice);
    converted.emplace_back(element);

    return hash(converted);
}

template class Poseidon2<Poseidon2Bn254ScalarFieldParams>;
} // namespace bb::crypto