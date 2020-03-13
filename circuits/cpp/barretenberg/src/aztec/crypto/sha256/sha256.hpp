#pragma once

#include "stdint.h"
#include <vector>

namespace sha256 {
constexpr uint32_t ror(uint32_t val, uint32_t shift)
{
    return (val >> (shift & 31U)) | (val << (32U - (shift & 31U)));
}
std::vector<uint8_t> sha256_block(const std::vector<uint8_t>& input);
std::vector<uint8_t> sha256(const std::vector<uint8_t>& input);
} // namespace sha256