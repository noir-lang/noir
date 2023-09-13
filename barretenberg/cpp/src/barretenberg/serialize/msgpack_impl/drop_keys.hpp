#pragma once
#include <tuple>

namespace msgpack {
template <typename Tuple, std::size_t... Is> auto drop_keys_impl(Tuple&& tuple, std::index_sequence<Is...>)
{
    // Expand 0 to n/2 to 1 to n+1 (increments of 2)
    // Use it to filter the tuple
    return std::tie(std::get<Is * 2 + 1>(std::forward<Tuple>(tuple))...);
}

/**
 * @brief Drops every first value pairwise of a flat argument tuple, assuming that they are keys.
 */
template <typename... Args> auto drop_keys(std::tuple<Args...>&& tuple)
{
    static_assert(sizeof...(Args) % 2 == 0, "Tuple must contain an even number of elements");
    // Compile time sequence of integers from 0 to n/2
    auto compile_time_0_to_n_div_2 = std::make_index_sequence<sizeof...(Args) / 2>{};
    return drop_keys_impl(tuple, compile_time_0_to_n_div_2);
}
} // namespace msgpack
