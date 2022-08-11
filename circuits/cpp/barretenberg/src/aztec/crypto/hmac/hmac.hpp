#pragma once

#include <array>
#include <cstdint>
#include <string>
#include <vector>
#include <numeric/uintx/uintx.hpp>

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

/**
 * @brief Takes a size-HASH_OUTPUT buffer from HMAC and converts into a field element
 *
 * @details We assume HASH_OUTPUT = 32, which is insufficient entropy. We hash input with `0` and `1` to produce 64
 * bytes of input data. This is then converted into a uin512_t, which is taken modulo Fr::modulus to produce our field
 * element.
 *
 * @tparam Hash the hash function we're using
 * @tparam Fr field type
 * @param input the input buffer
 * @return Fr output field element
 */
template <typename Hash, typename Fr, typename MessageContainer, typename KeyContainer>
Fr get_unbiased_field_from_hmac(const MessageContainer& message, const KeyContainer& key)
{
    auto input = hmac<Hash, MessageContainer, KeyContainer>(message, key);

    std::vector<uint8_t> lo_buffer(input.begin(), input.end());
    lo_buffer.push_back(0);
    std::vector<uint8_t> hi_buffer(input.begin(), input.end());
    hi_buffer.push_back(1);

    auto klo = Hash::hash(lo_buffer);
    auto khi = Hash::hash(hi_buffer);

    std::vector<uint8_t> full_buffer(khi.begin(), khi.end());
    for (auto& v : klo) {
        full_buffer.push_back(v);
    }

    uint512_t field_as_u512;
    const uint8_t* ptr = &full_buffer[0];
    numeric::read(ptr, field_as_u512);

    Fr result((field_as_u512 % Fr::modulus).lo);
    return result;
}
} // namespace crypto