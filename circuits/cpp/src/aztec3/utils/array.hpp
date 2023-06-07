#pragma once
#include "./types/native_types.hpp"

#include "aztec3/utils/circuit_errors.hpp"

#include "barretenberg/proof_system/types/composer_type.hpp"
#include <barretenberg/barretenberg.hpp>

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
    arr.fill(ELEMS_TYPE(0));  // Assumes that integer type can be used here in initialization
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
 * @brief Helper method to generate an 'empty' value of a given type
 * @tparam The type of the value to return
 * @return The empty value
 */
template <typename T> T empty_value()
{
    if constexpr (std::is_same<T, NT::fr>::value) {
        return NT::fr(0);
    } else {
        return T();
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
template <typename T, size_t SIZE> T array_pop(std::array<T, SIZE>& arr)
{
    for (size_t i = arr.max_size() - 1; i != static_cast<size_t>(-1); i--) {
        if (!is_empty(arr[i])) {
            const auto temp = arr[i];
            arr[i] = empty_value<T>();
            return temp;
        }
    }
    throw_or_abort("array_pop cannot pop from an empty array");
};

/**
 * @brief Helper method to push an item into the first empty slot in an array
 * @tparam The type of the value stored in the array
 * @tparam The composer type
 * @tparam The size of the array
 * @param The array into which we want to store the value
 */
template <typename T, typename Composer, size_t SIZE>
void array_push(Composer& composer, std::array<T, SIZE>& arr, T const& value)
{
    for (size_t i = 0; i < arr.size(); ++i) {
        if (is_empty(arr[i])) {
            arr[i] = value;
            return;
        }
    }
    composer.do_assert(false, "array_push cannot push to a full array", CircuitErrorCode::ARRAY_OVERFLOW);
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
        if (!is_empty(arr[i])) {
            return false;
        }
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
template <size_t size_1, size_t size_2, typename T, typename Composer>
void push_array_to_array(Composer& composer, std::array<T, size_1> const& source, std::array<T, size_2>& target)
{
    // Check if the `source` array is too large vs the remaining capacity of the `target` array
    size_t const source_size = static_cast<size_t>(uint256_t(array_length(source)));
    size_t const target_size = static_cast<size_t>(uint256_t(array_length(target)));

    composer.do_assert(source_size <= size_2 - target_size,
                       "push_array_to_array cannot overflow the target",
                       CircuitErrorCode::ARRAY_OVERFLOW);

    // Ensure that there are no non-zero values in the `target` array after the first zero-valued index
    for (size_t i = target_size; i < size_2; i++) {
        composer.do_assert(is_empty(target[i]),
                           "push_array_to_array inserting new array into a non empty space",
                           CircuitErrorCode::ARRAY_OVERFLOW);
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

/**
 * @brief Verifies that the contents of 2 arrays are included within a third
 * Ensures that all values after the concatenated values are zero.
 * Fails if the `source` arrays combined are too large vs the size of the `target` array.
 * @tparam The size of the `source` 1 array
 * @tparam The size of the `source` 2 array
 * @tparam The size of the `target` array
 * @tparam The type of the value stored in the arrays
 * @param The first `source` array
 * @param The second `source` array
 * @param The `target` array
 * @return Whether the source arrays are indeed in the target
 */
template <size_t size_1, size_t size_2, size_t size_3, typename T>
bool source_arrays_are_in_target(std::array<T, size_1> const& source1,
                                 std::array<T, size_2> const& source2,
                                 std::array<T, size_3> const& target)
{
    // Check if the `source` arrays are too large vs the size of the `target` array
    size_t const source1_size = static_cast<size_t>(uint256_t(array_length(source1)));
    size_t const source2_size = static_cast<size_t>(uint256_t(array_length(source2)));
    ASSERT(source1_size + source2_size <= size_3);

    // first ensure that all non-empty items in the first source are in the target
    size_t target_index = 0;
    for (size_t i = 0; i < source1_size; ++i) {
        if (source1[i] != target[target_index]) {
            return false;
        }
        ++target_index;
    }

    // now ensure that all non-empty items in the second source are in the target
    for (size_t i = 0; i < source2_size; ++i) {
        if (source2[i] != target[target_index]) {
            return false;
        }
        ++target_index;
    }

    for (; target_index < size_3; ++target_index) {
        if (!is_empty(target[target_index])) {
            return false;
        }
    }
    return true;
}

}  // namespace aztec3::utils
