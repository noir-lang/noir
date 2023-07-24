#include "../uint128/uint128.hpp"
#include "../uint256/uint256.hpp"
#include <cstdint>

namespace numeric {

/**
 * Returns the number of leading 0 bits for a given integer type.
 * Implemented in terms of intrinsics which will use instructions such as `bsr` or `lzcnt` for best performance.
 * Undefined behaviour when input is 0.
 */
template <typename T> constexpr inline size_t count_leading_zeros(T const& u);

template <> constexpr inline size_t count_leading_zeros<uint32_t>(uint32_t const& u)
{
    return (size_t)__builtin_clz(u);
}

template <> constexpr inline size_t count_leading_zeros<unsigned long>(unsigned long const& u)
{
    return (size_t)__builtin_clzl(u);
}

template <> constexpr inline size_t count_leading_zeros<unsigned long long>(unsigned long long const& u)
{
    return (size_t)__builtin_clzll(u);
}

template <> constexpr inline size_t count_leading_zeros<uint128_t>(uint128_t const& u)
{
    uint64_t hi = static_cast<uint64_t>(u >> 64);
    if (hi) {
        return (size_t)__builtin_clzll(hi);
    } else {
        uint64_t lo = static_cast<uint64_t>(u);
        return (size_t)__builtin_clzll(lo) + 64;
    }
}

template <> constexpr inline size_t count_leading_zeros<uint256_t>(uint256_t const& u)
{
    if (u.data[3]) {
        return count_leading_zeros(u.data[3]);
    }
    if (u.data[2]) {
        return count_leading_zeros(u.data[2]) + 64;
    }
    if (u.data[1]) {
        return count_leading_zeros(u.data[1]) + 128;
    }
    if (u.data[0]) {
        return count_leading_zeros(u.data[0]) + 192;
    }
    return 256;
}

} // namespace numeric