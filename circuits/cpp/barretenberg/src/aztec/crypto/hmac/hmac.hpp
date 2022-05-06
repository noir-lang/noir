#pragma once

#include <array>
#include <cstdint>
#include <string>
#include <vector>

namespace crypto {
/**
 * @brief Compute an HMAC given a secret key and a message
 *
 * @tparam Hash hasher being used
 * @tparam MessageContainer a byte container (std::vector<uint8_t>, std::array<uint8_t, ...>, std::string)
 * @tparam KeyContainer a byte container
 * @param message the message!
 * @param key the key!
 * @return std::array<uint8_t, Hash::OUTPUT_SIZE> the HMAC output!
 */
template <typename Hash, typename MessageContainer, typename KeyContainer>
std::array<uint8_t, Hash::OUTPUT_SIZE> hmac(const MessageContainer& message, const KeyContainer& key)
{
    constexpr size_t B = Hash::BLOCK_SIZE;
    std::array<uint8_t, B> ipad;
    std::array<uint8_t, B> opad;
    for (size_t i = 0; i < B; ++i) {
        opad[i] = 0x5c;
        ipad[i] = 0x36;
    }

    std::array<uint8_t, B> k_prime;
    if (key.size() > B) {
        const auto truncated_key = Hash::hash(key);
        std::copy(truncated_key.begin(), truncated_key.end(), k_prime.begin());
        for (size_t i = Hash::OUTPUT_SIZE; i < B; ++i) {
            k_prime[i] = 0x00;
        }
    } else {
        std::copy(key.begin(), key.end(), k_prime.begin());
        for (size_t i = key.size(); i < B; ++i) {
            k_prime[i] = 0x00;
        }
    }

    // std::cout << uint8_to_hex_string(&k_prime[0], B) << std::endl;
    std::array<uint8_t, B> h1;
    for (size_t i = 0; i < B; ++i) {
        h1[i] = k_prime[i] ^ opad[i];
    }

    std::array<uint8_t, B> h2;
    for (size_t i = 0; i < B; ++i) {
        h2[i] = k_prime[i] ^ ipad[i];
    }

    std::vector<uint8_t> message_buffer;
    std::copy(h2.begin(), h2.end(), std::back_inserter(message_buffer));
    std::copy(message.begin(), message.end(), std::back_inserter(message_buffer));

    const auto h3 = Hash::hash(message_buffer);

    std::vector<uint8_t> hmac_buffer;
    std::copy(h1.begin(), h1.end(), std::back_inserter(hmac_buffer));
    std::copy(h3.begin(), h3.end(), std::back_inserter(hmac_buffer));

    const auto hmac_key = Hash::hash(hmac_buffer);

    std::array<uint8_t, Hash::OUTPUT_SIZE> result;
    std::copy(hmac_key.begin(), hmac_key.end(), result.begin());
    return result;
}

} // namespace crypto