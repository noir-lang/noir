#pragma once

#include <array>

namespace bb {
/**
 * @brief Concatenates multiple std::array objects into a single array.
 *
 * This function template takes a variadic number of std::array objects and concatenates
 * their elements into a single std::array. The size of the resulting array is the sum of the sizes
 * of the input arrays.
 *
 * @tparam T The type of elements stored in the arrays.
 * @tparam Ns The sizes of the input arrays. This is a variadic template parameter pack representing
 *            the sizes of each input array.
 * @param arrays Variadic number of std::array objects to concatenate. Each array can have a
 *               different size but must contain the same type of elements.
 * @return std::array<T, (Ns + ...)> A new std::array containing all elements from the input arrays
 *         concatenated in the order they were passed.
 *
 * Example usage:
 *   std::array<int, 2> a = {1, 2};
 *   std::array<int, 3> b = {3, 4, 5};
 *   auto result = concatenate(a, b);  // result is std::array<int, 5>{1, 2, 3, 4, 5}
 */
template <typename T, std::size_t... Ns> std::array<T, (Ns + ...)> concatenate(const std::array<T, Ns>&... arrays)
{
    std::array<T, (Ns + ...)> result;

    std::size_t offset = 0;
    auto copy_into = [&](const auto& array) {
        std::copy(array.begin(), array.end(), result.begin() + offset);
        offset += array.size();
    };

    (copy_into(arrays), ...);

    return result;
}

} // namespace bb