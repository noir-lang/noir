#include "./types/native_types.hpp"
#include "barretenberg/common/throw_or_abort.hpp"

/**
 * NOTE: see bberg's stdlib/primitives/field/array.hpp for the corresponding circuit implementations of these functions.
 */
namespace aztec3::utils {

using NT = types::NativeTypes;

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
 * Fails if the `source` array is too large vs the remaining capacity of the `target` array.
 * TODO: this is an awful mess and should be improved!
 */
template <size_t size_1, size_t size_2>
void push_array_to_array(std::array<NT::fr, size_1> const& source, std::array<NT::fr, size_2>& target)
{
    // TODO: inefficient to get length this way within this function. Probably best to inline the checks that we need
    // into the below loops directly.
    NT::fr target_length = array_length(target);
    NT::fr target_capacity = NT::fr(target.size());
    const NT::fr overflow_capacity = target_capacity + 1;

    // ASSERT(uint256_t(target_capacity.get_value()) + 1 >
    //        uint256_t(target_length.get_value()) + uint256_t(source_length.get_value()));

    NT::fr j_ct = 0; // circuit-type index for the inner loop
    NT::fr next_target_index = target_length;
    for (size_t i = 0; i < source.size(); ++i) {
        auto& s = source[i];

        // Triangular loop:
        for (size_t j = i; j < target.size() - source.size() + i + 1; ++j) {
            auto& t = target[j];

            NT::boolean at_next_index = j_ct == next_target_index;

            t = at_next_index ? s : t;

            j_ct++;
        }

        next_target_index++;

        ASSERT(next_target_index != overflow_capacity); //"push_array_to_array target array capacity exceeded"

        j_ct = i + 1;
    }
}

} // namespace aztec3::utils
