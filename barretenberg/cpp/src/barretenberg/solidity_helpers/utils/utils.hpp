#pragma once
#include <bitset>
#include <iostream>

std::string bytes_to_hex_string(const std::vector<uint8_t>& input)
{
    static const char characters[] = "0123456789ABCDEF";

    // Zeroes out the buffer unnecessarily, can't be avoided for std::string.
    std::string ret(input.size() * 2, 0);

    // Hack... Against the rules but avoids copying the whole buffer.
    auto buf = const_cast<char*>(ret.data());

    for (const auto& oneInputByte : input) {
        *buf++ = characters[oneInputByte >> 4];
        *buf++ = characters[oneInputByte & 0x0F];
    }
    return ret;
}
