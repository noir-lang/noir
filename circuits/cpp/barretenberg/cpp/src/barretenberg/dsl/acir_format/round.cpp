#include "round.hpp"

namespace acir_format {

// Rounds a number to the nearest multiple of 8
uint32_t round_to_nearest_mul_8(uint32_t num_bits)
{
    uint32_t remainder = num_bits % 8;
    if (remainder == 0) {
        return num_bits;
    }

    return num_bits + 8 - remainder;
}

// Rounds the number of bits to the nearest byte
uint32_t round_to_nearest_byte(uint32_t num_bits)
{
    return round_to_nearest_mul_8(num_bits) / 8;
}

} // namespace acir_format
