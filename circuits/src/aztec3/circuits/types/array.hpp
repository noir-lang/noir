#pragma once

#include <stdlib/types/circuit_types.hpp>

namespace aztec3::circuits::types {

using plonk::stdlib::types::CircuitTypes;

/**
 * Gets the number of contiguous nonzero values of an array.
 * Note: this assumes `0` always means 'not used', so be careful. If you actually want `0` to be counted, you'll need
 * something else.
 */
// TODO: move to own file of helper functions.
template <typename Composer, size_t SIZE>
typename CircuitTypes<Composer>::fr array_length(std::array<typename CircuitTypes<Composer>::fr, SIZE> const& arr)
{
    typedef CircuitTypes<Composer> CT;
    typedef typename CT::fr fr;
    typedef typename CT::boolean boolean;

    fr length = 0;
    boolean hit_zero = false;
    for (const auto& e : arr) {
        hit_zero |= e == 0;
        const fr increment = !hit_zero;
        length += increment;
    }
    return length;
};

/**
 * Note: doesn't remove the last element from the array; only returns it!
 * Note: this assumes `0` always means 'not used', so be careful. If you actually want `0` to be counted, you'll need
 * something else.
 * If it returns `0`, the array is considered 'empty'.
 */
template <typename Composer, size_t SIZE>
typename CircuitTypes<Composer>::fr array_pop(std::array<typename CircuitTypes<Composer>::fr, SIZE> const& arr)
{
    typedef CircuitTypes<Composer> CT;
    typedef typename CT::fr fr;
    typedef typename CT::boolean boolean;

    fr popped_value;
    boolean already_popped = false;
    for (size_t i = arr.size() - 1; i != (size_t)-1; i--) {
        boolean is_non_zero = arr[i] != 0;
        popped_value = fr::conditional_assign(!already_popped && is_non_zero, arr[i], popped_value);

        already_popped |= is_non_zero;
    }
    return popped_value;
};

/**
 * Note: this assumes `0` always means 'not used', so be careful. If you actually want `0` to be counted, you'll need
 * something else.
 */
template <typename Composer, size_t SIZE>
void array_push(std::array<typename CircuitTypes<Composer>::fr, SIZE>& arr,
                typename CircuitTypes<Composer>::fr const& value)
{
    typedef CircuitTypes<Composer> CT;
    typedef typename CT::fr fr;
    typedef typename CT::boolean boolean;

    boolean already_pushed = false;
    for (size_t i = 0; i < arr.size(); ++i) {
        boolean is_zero = arr[i] == 0;
        arr[i] = fr::conditional_assign(!already_pushed && is_zero, value, arr[i]);

        already_pushed |= is_zero;
    }
};

template <typename Composer, size_t SIZE>
inline void array_push(std::array<std::optional<typename CircuitTypes<Composer>::fr>, SIZE>& arr,
                       typename CircuitTypes<Composer>::fr const& value)
{
    for (size_t i = 0; i < arr.size(); ++i) {
        if (arr[i] == std::nullopt) {
            arr[i] = value;
            return;
        }
    }
};

/**
 * Note: this assumes `0` always means 'not used', so be careful. If you actually want `0` to be counted, you'll need
 * something else.
 */
template <typename Composer, size_t SIZE>
typename CircuitTypes<Composer>::boolean is_array_empty(
    std::array<typename CircuitTypes<Composer>::fr, SIZE> const& arr)
{
    typedef CircuitTypes<Composer> CT;
    typedef typename CT::boolean boolean;

    boolean nonzero_found = false;
    for (size_t i = arr.size() - 1; i != (size_t)-1; i--) {
        boolean is_non_zero = arr[i] != 0;
        nonzero_found |= is_non_zero;
    }
    return !nonzero_found;
};

/**
 * Inserts the `source` array at the first zero-valued index of the `target` array.
 * Fails if the `source` array is too large vs the remaining capacity of the `target` array.
 */
template <typename Composer, size_t size_1, size_t size_2>
void push_array_to_array(std::array<typename CircuitTypes<Composer>::fr, size_1> const& source,
                         std::array<typename CircuitTypes<Composer>::fr, size_2>& target)
{
    typedef CircuitTypes<Composer> CT;
    typedef typename CT::fr fr;
    typedef typename CT::boolean boolean;

    fr target_length = array_length<Composer>(target);
    // fr source_length = array_length<Composer>(source);

    fr target_capacity = fr(target.size());
    // TODO: using safe_fr for an underflow check, do:
    // remaining_target_capacity = target_capacity.subtract(target_length + source_length);

    fr t_i = 0;
    fr next_index = target_length;
    for (const auto& s : source) {
        for (auto& t : target) {
            next_index.assert_not_equal(target_capacity, "Target array capacity exceeded");
            boolean at_index = t_i == next_index;
            t = fr::conditional_assign(at_index, s, t);
            next_index = fr::conditional_assign(at_index, next_index + 1, next_index);
            ++t_i;
        }
    }
}

} // namespace aztec3::circuits::types