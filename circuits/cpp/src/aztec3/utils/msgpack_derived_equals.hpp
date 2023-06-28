#pragma once
#include <tuple>

namespace aztec3::utils {
// Auxiliary function for comparison of elements from tuples
template <typename Tuple1, typename Tuple2, std::size_t... I>
auto _compare_msgpack_tuples(const Tuple1& t1, const Tuple2& t2, std::index_sequence<I...>)  // NOLINT
{
    // Compare every 2nd value from our key1, value1, key2, value2 list
    return ((std::get<I * 2 + 1>(t1) == std::get<I * 2 + 1>(t2)) && ...);
}

// Function to check equality of msgpack objects based on their values.
// Normally, you should instead use operator==() = default;
// BoolLike represents a type like the boolean<Builder> DSL type
template <typename BoolLike, typename T> BoolLike msgpack_derived_equals(const T& obj1, const T& obj2)
{
    BoolLike are_equal;
    // De-serialize objects to alternating key-value tuples and compare the values
    const_cast<T&>(obj1).msgpack([&](auto&... args1) {      // NOLINT
        const_cast<T&>(obj2).msgpack([&](auto&... args2) {  // NOLINT
            auto tuple1 = std::tie(args1...);
            auto tuple2 = std::tie(args2...);
            are_equal = _compare_msgpack_tuples(tuple1, tuple2, std::make_index_sequence<sizeof...(args1) / 2>{});
        });
    });
    return are_equal;
}
}  // namespace aztec3::utils
