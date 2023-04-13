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
 * Gets the number of contiguous nonzero values of an array from the start.
 * Note: This assumes `0` always means 'not used', so be careful. As soon as we locate 0, we stop the counting.
 * If you actually want `0` to be counted, you'll need something else.
 */
template <size_t SIZE> NT::fr array_length(std::array<NT::fr, SIZE> const& arr)
{
    NT::fr length = 0;
    for (const auto& e : arr) {
        if (e == 0) {
            break;
        }
        length++;
    }
    return length;
};

/**
 * Note: doesn't remove the last element from the array; only returns it!
 * Note: this assumes `0` always means 'not used', so be careful. If you actually want `0` to be counted, you'll need
 * something else.
 * If it returns `0`, the array is considered 'empty'.
 */
template <size_t SIZE> NT::fr array_pop(std::array<NT::fr, SIZE> const& arr)
{
    for (size_t i = arr.max_size() - 1; i != (size_t)-1; i--) {
        if (arr[i] != 0) {
            return arr[i];
        }
    }
    throw_or_abort("array_pop cannot pop from an empty array");
};

/**
 * Note: this assumes `0` always means 'not used', so be careful. If you actually want `0` to be counted, you'll need
 * something else.
 */
template <size_t SIZE> void array_push(std::array<NT::fr, SIZE>& arr, NT::fr const& value)
{
    for (size_t i = 0; i < arr.size(); ++i) {
        if (arr[i] == 0) {
            arr[i] = value;
            return;
        }
    }
    throw_or_abort("array_push cannot push to a full array");
};

template <typename T, size_t SIZE> void array_push(std::array<T, SIZE>& arr, T const& value)
{
    for (size_t i = 0; i < arr.size(); ++i) {
        if (arr[i].is_empty()) {
            arr[i] = value;
            return;
        }
    }
    throw_or_abort("array_push cannot push to a full array");
};

/**
 * Note: this assumes `0` always means 'not used', so be careful. If you actually want `0` to be counted, you'll need
 * something else.
 */
template <size_t SIZE> NT::boolean is_array_empty(std::array<NT::fr, SIZE> const& arr)
{
    for (size_t i = 0; i < arr.size(); ++i) {
        if (arr[i] != 0)
            return false;
    }
    return true;
};

/**
 * Inserts the `source` array at the first zero-valued index of the `target` array.
 * Ensures that all values after the first zero-valued index are zeros too.
 * Fails if the `source` array is too large vs the remaining capacity of the `target` array.
 */
template <size_t size_1, size_t size_2>
void push_array_to_array(std::array<NT::fr, size_1> const& source, std::array<NT::fr, size_2>& target)
{
    // Check if the `source` array is too large vs the remaining capacity of the `target` array
    size_t source_size = static_cast<size_t>(uint256_t(array_length(source)));
    size_t target_size = static_cast<size_t>(uint256_t(array_length(target)));
    ASSERT(source_size <= size_2 - target_size);

    // Ensure that there are no non-zero values in the `target` array after the first zero-valued index
    for (size_t i = target_size; i < size_2; i++) {
        ASSERT(target[i] == NT::fr(0));
    }
    // Copy the non-zero elements of the `source` array to the `target` array at the first zero-valued index
    auto zero_index = target_size;
    for (size_t i = 0; i < size_1; i++) {
        if (source[i] != NT::fr(0)) {
            target[zero_index] = source[i];
            zero_index++;
        }
    }
}

} // namespace aztec3::utils
