#pragma once

#include "common/serialize.hpp"
#include <algorithm>
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
    // ensures truncated_key fits into k_prime
    static_assert(Hash::OUTPUT_SIZE <= B);
    constexpr uint8_t IPAD_CONST = 0x36;
    constexpr uint8_t OPAD_CONST = 0x5c;
    std::array<uint8_t, B> ipad;
    std::array<uint8_t, B> opad;
    ipad.fill(IPAD_CONST);
    opad.fill(OPAD_CONST);

    // initialize k_prime to 0x00,...,0x00
    // copy key or truncated key to start.
    // TODO: securely erase `k_prime`
    std::array<uint8_t, B> k_prime{};
    if (key.size() > B) {
        const auto truncated_key = Hash::hash(key);
        std::copy(truncated_key.begin(), truncated_key.end(), k_prime.begin());
    } else {
        std::copy(key.begin(), key.end(), k_prime.begin());
    }

    // TODO: securely erase `h1`
    std::array<uint8_t, B> h1;
    for (size_t i = 0; i < B; ++i) {
        h1[i] = k_prime[i] ^ opad[i];
    }

    // TODO: securely erase `h2`
    std::array<uint8_t, B> h2;
    for (size_t i = 0; i < B; ++i) {
        h2[i] = k_prime[i] ^ ipad[i];
    }

    // TODO: securely erase copy of `h2` in `message_buffer`,
    // ensure `message_buffer` is not re-allocated
    std::vector<uint8_t> message_buffer;
    std::copy(h2.begin(), h2.end(), std::back_inserter(message_buffer));
    std::copy(message.begin(), message.end(), std::back_inserter(message_buffer));

    const auto h3 = Hash::hash(message_buffer);

    // TODO: securely erase copy of `h1` in `hmac_buffer`,
    // ensure `hmac_buffer` is not re-allocated
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
 * @details We assume HASH_OUTPUT = 32. Reducing HMAC(key, message) modulo r would result in an unacceptable bias.
 * We hash input with `0` and `1` to produce 64 bytes of input data. This is then converted into a uin512_t,
 * which is taken modulo Fr::modulus to produce our field element, where the statistical bias is negligble in
 * the security parameter.
 *
 * @tparam Hash the hash function we're using
 * @tparam Fr field type
 * @tparam MessageContainer a byte container (std::vector<uint8_t>, std::array<uint8_t, ...>, std::string)
 * @tparam KeyContainer a byte container
 * @param message the input buffer
 * @param key key used to derive
 * @return Fr output field element as uint512_t( H(10...0 || HMAC(k,m)) || H(00...0 || HMAC(k,m)) ) % r
 */
template <typename Hash, typename Fr, typename MessageContainer, typename KeyContainer>
Fr get_unbiased_field_from_hmac(const MessageContainer& message,
                                const KeyContainer& key) requires(Hash::OUTPUT_SIZE == 32)
{
    // Strong assumption that works for now with our suite of Hashers
    static_assert(Hash::BLOCK_SIZE > Hash::OUTPUT_SIZE);
    constexpr size_t DOMAIN_SEPARATOR_SIZE = Hash::BLOCK_SIZE - Hash::OUTPUT_SIZE;

    // Domain separators whose size ensures we hash a block of the exact size expected by
    // the Hasher.
    constexpr std::array<uint8_t, DOMAIN_SEPARATOR_SIZE> KLO_DOMAIN_SEPARATOR{ 0x0 };
    constexpr std::array<uint8_t, DOMAIN_SEPARATOR_SIZE> KHI_DOMAIN_SEPARATOR{ 0x1 };

    auto input = hmac<Hash, MessageContainer, KeyContainer>(message, key);

    // klo = H(00...0 || input)
    std::vector<uint8_t> lo_buffer(KLO_DOMAIN_SEPARATOR.begin(), KLO_DOMAIN_SEPARATOR.end());
    std::copy(input.begin(), input.end(), std::back_inserter(lo_buffer));
    auto klo = Hash::hash(lo_buffer);

    // khi = H(10...0 || input)
    std::vector<uint8_t> hi_buffer(KHI_DOMAIN_SEPARATOR.begin(), KHI_DOMAIN_SEPARATOR.end());
    std::copy(input.begin(), input.end(), std::back_inserter(hi_buffer));
    auto khi = Hash::hash(hi_buffer);

    // full_buffer = khi || klo
    std::vector<uint8_t> full_buffer(khi.begin(), khi.end());
    std::copy(klo.begin(), klo.end(), std::back_inserter(full_buffer));

    auto field_as_u512 = from_buffer<numeric::uint512_t>(full_buffer);

    Fr result((field_as_u512 % Fr::modulus).lo);
    return result;
}
} // namespace crypto
