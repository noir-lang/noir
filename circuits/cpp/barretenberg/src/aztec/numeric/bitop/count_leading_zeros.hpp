#include <cstdint>
#include "../uint128/uint128.hpp"

namespace numeric {

template <typename T> inline size_t count_leading_zeros(T u);

template <> inline size_t count_leading_zeros<size_t>(size_t u)
{
    return (size_t)__builtin_clzl(u);
}

template <> inline size_t count_leading_zeros<uint128_t>(uint128_t u)
{
    uint64_t hi = static_cast<uint64_t>(u >> 64);
    if (hi) {
        return (size_t)__builtin_clzll(hi);
    } else {
        uint64_t lo = static_cast<uint64_t>(u);
        return (size_t)__builtin_clzll(lo) + 64;
    }
}

}