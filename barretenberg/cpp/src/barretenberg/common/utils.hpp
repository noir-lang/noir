#pragma once

#include <cstdint>
#include <string>
#include <vector>

namespace bb::utils {

/**
 * @brief Routine to transform hexstring to vector of bytes.
 *
 * @param Hexadecimal string representation.
 * @return Vector of uint8_t values.
 */
std::vector<uint8_t> hex_to_bytes(const std::string& hex);

} // namespace bb::utils