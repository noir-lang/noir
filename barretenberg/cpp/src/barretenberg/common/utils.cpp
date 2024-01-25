#include "./utils.hpp"

namespace bb::utils {

std::vector<uint8_t> hex_to_bytes(const std::string& hex)
{
    std::vector<uint8_t> bytes;

    for (unsigned int i = 0; i < hex.length(); i += 2) {
        std::string byteString = hex.substr(i, 2);
        bytes.push_back(static_cast<uint8_t>(strtol(byteString.c_str(), nullptr, 16)));
    }

    return bytes;
}

} // namespace bb::utils