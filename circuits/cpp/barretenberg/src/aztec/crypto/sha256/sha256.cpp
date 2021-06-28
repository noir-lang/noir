#include "./sha256.hpp"
#include <array>
#include <common/assert.hpp>
#include <common/net.hpp>
#include <memory.h>

namespace sha256 {

namespace {
constexpr uint32_t init_constants[8]{ 0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
                                      0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19 };

constexpr uint32_t round_constants[64]{
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2
};

constexpr uint32_t ror(uint32_t val, uint32_t shift)
{
    return (val >> (shift & 31U)) | (val << (32U - (shift & 31U)));
}

} // namespace

void prepare_constants(std::array<uint32_t, 8>& input)
{
    input[0] = init_constants[0];
    input[1] = init_constants[1];
    input[2] = init_constants[2];
    input[3] = init_constants[3];
    input[4] = init_constants[4];
    input[5] = init_constants[5];
    input[6] = init_constants[6];
    input[7] = init_constants[7];
}

std::array<uint32_t, 8> sha256_block(const std::array<uint32_t, 8>& h_init, const std::array<uint32_t, 16>& input)
{
    std::array<uint32_t, 64> w;

    /**
     * Fill first 16 words with the message schedule
     **/
    for (size_t i = 0; i < 16; ++i) {
        w[i] = input[i];
    }

    /**
     * Extend the input data into the remaining 48 words
     **/
    for (size_t i = 16; i < 64; ++i) {
        uint32_t s0 = ror(w[i - 15], 7) ^ ror(w[i - 15], 18) ^ (w[i - 15] >> 3);
        uint32_t s1 = ror(w[i - 2], 17) ^ ror(w[i - 2], 19) ^ (w[i - 2] >> 10);
        w[i] = w[i - 16] + w[i - 7] + s0 + s1;
    }

    /**
     * Initialize round variables with previous block output
     **/
    uint32_t a = h_init[0];
    uint32_t b = h_init[1];
    uint32_t c = h_init[2];
    uint32_t d = h_init[3];
    uint32_t e = h_init[4];
    uint32_t f = h_init[5];
    uint32_t g = h_init[6];
    uint32_t h = h_init[7];

    /**
     * Apply SHA-256 compression function to the message schedule
     **/
    for (size_t i = 0; i < 64; ++i) {
        uint32_t S1 = ror(e, 6U) ^ ror(e, 11U) ^ ror(e, 25U);
        uint32_t ch = (e & f) ^ (~e & g); // === (e & f) ^ (~e & g), `+` op is cheaper
        uint32_t temp1 = h + S1 + ch + round_constants[i] + w[i];
        uint32_t S0 = ror(a, 2U) ^ ror(a, 13U) ^ ror(a, 22U);
        uint32_t maj = (a & b) ^ (a & c) ^ (b & c); // (a & (b + c - (T0 * 2))) + T0; // === (a & b) ^ (a & c) ^ (b & c)
        uint32_t temp2 = S0 + maj;

        h = g;
        g = f;
        f = e;
        e = d + temp1;
        d = c;
        c = b;
        b = a;
        a = temp1 + temp2;
    }

    /**
     * Add into previous block output and return
     **/
    std::array<uint32_t, 8> output;
    output[0] = a + h_init[0];
    output[1] = b + h_init[1];
    output[2] = c + h_init[2];
    output[3] = d + h_init[3];
    output[4] = e + h_init[4];
    output[5] = f + h_init[5];
    output[6] = g + h_init[6];
    output[7] = h + h_init[7];
    return output;
}

hash sha256_block(const std::vector<uint8_t>& input)
{
    ASSERT(input.size() == 64);
    std::array<uint32_t, 8> result;
    prepare_constants(result);
    std::array<uint32_t, 16> hash_input;
    memcpy((void*)&hash_input[0], (void*)&input[0], 64);
    if (is_little_endian()) {
        for (size_t j = 0; j < hash_input.size(); ++j) {
            hash_input[j] = __builtin_bswap32(hash_input[j]);
        }
    }
    result = sha256_block(result, hash_input);

    hash output;
    memcpy((void*)&output[0], (void*)&result[0], 32);
    if (is_little_endian()) {
        uint32_t* output_uint32 = (uint32_t*)&output[0];
        for (size_t j = 0; j < 8; ++j) {
            output_uint32[j] = __builtin_bswap32(output_uint32[j]);
        }
    }

    return output;
}

hash sha256(const std::vector<uint8_t>& input)
{
    std::vector<uint8_t> message_schedule;

    std::copy(input.begin(), input.end(), std::back_inserter(message_schedule));
    uint64_t l = message_schedule.size() * 8;
    message_schedule.push_back(0x80);

    uint32_t num_zero_bytes = ((448U - (message_schedule.size() << 3U)) & 511U) >> 3U;

    for (size_t i = 0; i < num_zero_bytes; ++i) {
        message_schedule.push_back(0x00);
    }
    for (size_t i = 0; i < 8; ++i) {
        uint8_t byte = static_cast<uint8_t>(l >> (uint64_t)(56 - (i * 8)));
        message_schedule.push_back(byte);
    }
    std::array<uint32_t, 8> rolling_hash;
    prepare_constants(rolling_hash);
    const size_t num_blocks = message_schedule.size() / 64;
    for (size_t i = 0; i < num_blocks; ++i) {
        std::array<uint32_t, 16> hash_input;
        memcpy((void*)&hash_input[0], (void*)&message_schedule[i * 64], 64);
        if (is_little_endian()) {
            for (size_t j = 0; j < hash_input.size(); ++j) {
                hash_input[j] = __builtin_bswap32(hash_input[j]);
            }
        }
        rolling_hash = sha256_block(rolling_hash, hash_input);
    }

    hash output;
    memcpy((void*)&output[0], (void*)&rolling_hash[0], 32);
    if (is_little_endian()) {
        uint32_t* output_uint32 = (uint32_t*)&output[0];
        for (size_t j = 0; j < 8; ++j) {
            output_uint32[j] = __builtin_bswap32(output_uint32[j]);
        }
    }

    return output;
}
} // namespace sha256