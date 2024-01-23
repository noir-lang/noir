#pragma once
#include "../uint128/uint128.hpp"
#include "../uint256/uint256.hpp"
#include "../uintx/uintx.hpp"
#include "unistd.h"
#include <cstdint>

namespace bb::numeric {

class RNG {
  public:
    virtual uint8_t get_random_uint8() = 0;

    virtual uint16_t get_random_uint16() = 0;

    virtual uint32_t get_random_uint32() = 0;

    virtual uint64_t get_random_uint64() = 0;

    virtual uint128_t get_random_uint128() = 0;

    virtual uint256_t get_random_uint256() = 0;

    virtual ~RNG() = default;
    RNG() noexcept = default;
    RNG(const RNG& other) = default;
    RNG(RNG&& other) = default;
    RNG& operator=(const RNG& other) = default;
    RNG& operator=(RNG&& other) = default;

    uint512_t get_random_uint512()
    {
        // Do not inline in constructor call. Evaluation order is important for cross-compiler consistency.
        auto lo = get_random_uint256();
        auto hi = get_random_uint256();
        return { lo, hi };
    }

    uint1024_t get_random_uint1024()
    {
        // Do not inline in constructor call. Evaluation order is important for cross-compiler consistency.
        auto lo = get_random_uint512();
        auto hi = get_random_uint512();
        return { lo, hi };
    }
};

RNG& get_debug_randomness(bool reset = false);
RNG& get_randomness();

} // namespace bb::numeric
