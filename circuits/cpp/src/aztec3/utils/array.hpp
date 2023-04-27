#pragma once
#include "./types/native_types.hpp"
#include "barretenberg/common/throw_or_abort.hpp"

/**
 * NOTE: see bberg's stdlib/primitives/field/array.hpp for the corresponding circuit implementations of these functions.
 */
namespace aztec3::utils {

using NT = types::NativeTypes;

/**
 * @brief Creates an array of zeros.
 *
 * @details This is necessary when a type (like fr) has a default constructor
 * that doesn't initialize members to zero.
 *
 * @tparam ELEMS_TYPE array element type
 * @tparam ARRAY_LEN
 * @return std::array<ELEMS_TYPE, ARRAY_LEN> the zero-initialized array
 */
template <typename ELEMS_TYPE, size_t ARRAY_LEN> std::array<ELEMS_TYPE, ARRAY_LEN> zero_array()
{
    std::array<ELEMS_TYPE, ARRAY_LEN> arr;
    arr.fill(ELEMS_TYPE(0)); // Assumes that integer type can be used here in initialization
    return arr;
}

/**
 * @brief Helper method to determine if a value is 'empty' based on what empty means for it's type
 * @tparam The type of the input value
 * @param The value being queried for 'emptiness'
 * @return Whether the value is 'empty'
 */
template <typename T> bool is_empty(T const& value)
{
    if constexpr (std::is_same<T, NT::fr>::value) {
        return value == NT::fr(0);
    } else {
        return value.is_empty();
    }
}

/**
 * @brief Helper method to determine the number of non 'empty' items in an array
 * @tparam The type of the value stored in the array
 * @tparam The size of the array
 * @param The array being evaluated for it's length
 * @return The number of non-empty items in the array
 */
template <typename T, size_t SIZE> size_t array_length(std::array<T, SIZE> const& arr)
{
    size_t length = 0;
    for (const auto& e : arr) {
        if (is_empty(e)) {
            break;
        }
        length++;
    }
    return length;
};

/**
 * @brief Helper method to return the last non-empty item in an array
 * @tparam The type of the value stored in the array
 * @tparam The size of the array
 * @param The array from which we are to return a value
 * @return The returned item
 */
template <size_t SIZE> NT::fr array_pop(std::array<NT::fr, SIZE> const& arr)
{
    for (size_t i = arr.max_size() - 1; i != (size_t)-1; i--) {
        if (!is_empty(arr[i])) {
            return arr[i];
        }
    }
    throw_or_abort("array_pop cannot pop from an empty array");
};

/**
 * @brief Helper method to push an item into the first empty slot in an array
 * @tparam The type of the value stored in the array
 * @tparam The size of the array
 * @param The array into which we want to store the value
 */
template <typename T, size_t SIZE> void array_push(std::array<T, SIZE>& arr, T const& value)
{
    for (size_t i = 0; i < arr.size(); ++i) {
        if (is_empty(arr[i])) {
            arr[i] = value;
            return;
        }
    }
    throw_or_abort("array_push cannot push to a full array");
};

/**
 * @brief Helper method to determine if an array contains all 'empty' items
 * @tparam The type of the value stored in the array
 * @tparam The size of the array
 * @param The array to evaluate for non-empty items
 */
template <typename T, size_t SIZE> NT::boolean is_array_empty(std::array<T, SIZE> const& arr)
{
    for (size_t i = 0; i < arr.size(); ++i) {
        if (!is_empty(arr[i]))
            return false;
    }
    return true;
};

/**
 * @brief Inserts the `source` array at the first 'empty' index of the `target` array.
 * Ensures that all values after the first 'empty' index are 'empty' too.
 * Fails if the `source` array is too large vs the remaining capacity of the `target` array.
 * @tparam The size of the `source` array
 * @tparam The size of the `target` array
 * @tparam The type of the value stored in the arrays
 * @param The `source` array
 * @param The `target` array
 */
template <size_t size_1, size_t size_2, typename T>
void push_array_to_array(std::array<T, size_1> const& source, std::array<T, size_2>& target)
{
    // Check if the `source` array is too large vs the remaining capacity of the `target` array
    size_t source_size = static_cast<size_t>(uint256_t(array_length(source)));
    size_t target_size = static_cast<size_t>(uint256_t(array_length(target)));
    ASSERT(source_size <= size_2 - target_size);

    // Ensure that there are no non-zero values in the `target` array after the first zero-valued index
    for (size_t i = target_size; i < size_2; i++) {
        ASSERT(is_empty(target[i]));
    }
    // Copy the non-zero elements of the `source` array to the `target` array at the first zero-valued index
    auto zero_index = target_size;
    for (size_t i = 0; i < size_1; i++) {
        if (!is_empty(source[i])) {
            target[zero_index] = source[i];
            zero_index++;
        }
    }
}

} // namespace aztec3::utils
