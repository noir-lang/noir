#pragma once
#include <stddef.h>
#include <stdint.h>

namespace numeric {

constexpr inline uint64_t rotate64(const uint64_t value, const uint64_t rotation)
{
    return rotation ? (value >> rotation) + (value << (64 - rotation)) : value;
}

constexpr inline uint32_t rotate32(const uint32_t value, const uint32_t rotation)
{
    return rotation ? (value >> rotation) + (value << (32 - rotation)) : value;
}
} // namespace numeric