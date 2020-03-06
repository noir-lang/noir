#pragma once
#include "../../curves/bn254/fr.hpp"

namespace int_utils {
__extension__ using uint128_t = unsigned __int128;

// from http://supertech.csail.mit.edu/papers/debruijn.pdf
inline size_t get_msb(uint32_t v)
{
    static constexpr uint32_t MultiplyDeBruijnBitPosition[32] = { 0,  9,  1,  10, 13, 21, 2,  29, 11, 14, 16,
                                                                  18, 22, 25, 3,  30, 8,  12, 20, 28, 15, 17,
                                                                  24, 7,  19, 27, 23, 6,  26, 5,  4,  31 };

    v |= v >> 1; // first round down to one less than a power of 2
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;

    return MultiplyDeBruijnBitPosition[static_cast<uint32_t>(v * static_cast<uint32_t>(0x07C4ACDD)) >>
                                       static_cast<uint32_t>(27)];
}

inline size_t get_msb(uint128_t v)
{
    uint32_t lolo = static_cast<uint32_t>(v & static_cast<uint128_t>(0xffffffffULL));
    uint32_t lohi = static_cast<uint32_t>((v >> static_cast<uint128_t>(32ULL)) & static_cast<uint128_t>(0xffffffffULL));
    uint32_t hilo = static_cast<uint32_t>((v >> static_cast<uint128_t>(64ULL)) & static_cast<uint128_t>(0xffffffffULL));
    uint32_t hihi = static_cast<uint32_t>((v >> static_cast<uint128_t>(96ULL)) & static_cast<uint128_t>(0xffffffffULL));

    if (hihi > 0) {
        return (get_msb(hihi) + 96);
    }
    if (hilo > 0) {
        return (get_msb(hilo) + 64);
    }
    if (lohi > 0) {
        return (get_msb(lohi) + 32);
    }
    return get_msb(lolo);
}

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

template <typename T> inline T keep_n_lsb(T input, size_t num_bits)
{
    return num_bits >= sizeof(T) * 8 ? input : input & (((T)1 << num_bits) - 1);
}

inline uint128_t field_to_uint128(barretenberg::fr const& in)
{
    barretenberg::fr input = in.from_montgomery_form();
    uint128_t lo = input.data[0];
    uint128_t hi = input.data[1];
    return (hi << 64) | lo;
}

inline barretenberg::fr uint128_to_field(uint128_t input)
{
    return { { (uint64_t)input, (uint64_t)(input >> 64), 0, 0 } };
}

} // namespace int_utils