#pragma once
#include <vector>

namespace bb {
/**
 * @brief Concatenates multiple std::vector objects into a single std::vector.
 *
 * @tparam T The type of elements in the std::vector.
 * @param vectors The std::vector objects to be concatenated.
 * @return std::vector object containing all elements from the input vectors.
 */
template <typename T> std::vector<T> concatenate(const std::vector<T>& vector, const auto&... vectors)
{
    std::vector<T> concatenated;
    // Reserve our final space
    concatenated.reserve(vector.size() + (vectors.size() + ...));

    auto append = [&](const auto& vec) { std::copy(vec.begin(), vec.end(), std::back_inserter(concatenated)); };

    append(vector);
    // Unpack and append each std::vector's elements to concatenated
    (append(vectors), ...);

    return concatenated;
}
} // namespace bb