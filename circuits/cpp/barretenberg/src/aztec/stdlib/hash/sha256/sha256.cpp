#include "sha256.hpp"
#include <plonk/composer/standard_composer.hpp>
#include <plonk/composer/turbo_composer.hpp>
#include <stdlib/primitives/bit_array/bit_array.hpp>

namespace plonk {
namespace stdlib {
namespace internal {
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

constexpr size_t get_num_blocks(const size_t num_bits)
{
    constexpr size_t extra_bits = 65UL;

    return ((num_bits + extra_bits) / 512UL) + ((num_bits + extra_bits) % 512UL > 0);
}
} // namespace internal

template <typename Composer> void prepare_constants(std::array<uint32<Composer>, 8>& input)
{
    input[0] = internal::init_constants[0];
    input[1] = internal::init_constants[1];
    input[2] = internal::init_constants[2];
    input[3] = internal::init_constants[3];
    input[4] = internal::init_constants[4];
    input[5] = internal::init_constants[5];
    input[6] = internal::init_constants[6];
    input[7] = internal::init_constants[7];
}

template <typename Composer>
std::array<uint32<Composer>, 8> sha256_block(const std::array<uint32<Composer>, 8>& h_init,
                                             const std::array<uint32<Composer>, 16>& input)
{
    typedef uint32<Composer> uint32;
    std::array<uint32, 64> w;

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
        uint32 s0 = w[i - 15].ror(7) ^ w[i - 15].ror(18) ^ (w[i - 15] >> 3);
        uint32 s1 = w[i - 2].ror(17) ^ w[i - 2].ror(19) ^ (w[i - 2] >> 10);
        w[i] = w[i - 16] + w[i - 7] + s0 + s1;
    }

    /**
     * Initialize round variables with previous block output
     **/
    uint32 a = h_init[0];
    uint32 b = h_init[1];
    uint32 c = h_init[2];
    uint32 d = h_init[3];
    uint32 e = h_init[4];
    uint32 f = h_init[5];
    uint32 g = h_init[6];
    uint32 h = h_init[7];

    /**
     * Apply SHA-256 compression function to the message schedule
     **/
    for (size_t i = 0; i < 64; ++i) {
        uint32 S1 = e.ror(6U) ^ e.ror(11U) ^ e.ror(25U);
        uint32 ch = (e & f) + (~e & g); // === (e & f) ^ (~e & g), `+` op is cheaper
        uint32 temp1 = h + S1 + ch + internal::round_constants[i] + w[i];
        uint32 S0 = a.ror(2U) ^ a.ror(13U) ^ a.ror(22U);
        uint32 T0 = (b & c);
        uint32 maj = (a & (b + c - (T0 + T0))) + T0; // === (a & b) ^ (a & c) ^ (b & c)
        uint32 temp2 = S0 + maj;

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
    std::array<uint32, 8> output;
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

template <typename Composer> byte_array<Composer> sha256_block(const byte_array<Composer>& input)
{
    typedef uint32<Composer> uint32;

    ASSERT(input.size() == 64);

    std::array<uint32, 8> hash;
    prepare_constants(hash);

    std::array<uint32, 16> hash_input;
    for (size_t i = 0; i < 16; ++i) {
        hash_input[i] = uint32(input.slice(i * 4, 4));
    }
    hash = sha256_block(hash, hash_input);

    byte_array<Composer> result(input.get_context());
    for (size_t i = 0; i < 8; ++i) {
        result.write(hash[i]);
    }

    return result;
}

template <typename Composer> bit_array<Composer> sha256(const bit_array<Composer>& input)
{
    typedef uint32<Composer> uint32;
    typedef bit_array<Composer> bit_array;

    size_t num_bits = input.size();
    size_t num_blocks = internal::get_num_blocks(num_bits);

    bit_array message_schedule = bit_array(input.get_context(), num_blocks * 512UL);

    // begin filling message schedule from most significant to least significant
    size_t offset = message_schedule.size() - input.size();

    for (size_t i = input.size() - 1; i < input.size(); --i) {
        size_t idx = offset + i;
        message_schedule[idx] = input[i];
    }
    message_schedule[offset - 1] = true;
    for (size_t i = 0; i < 32; ++i) {
        message_schedule[i] = static_cast<bool>((num_bits >> i) & 1);
    }

    std::array<uint32, 8> rolling_hash;
    prepare_constants(rolling_hash);
    for (size_t i = 0; i < num_blocks; ++i) {
        std::array<uint32, 16> hash_input;
        message_schedule.populate_uint32_array(i * 512, hash_input);
        rolling_hash = sha256_block(rolling_hash, hash_input);
    }
    return bit_array(rolling_hash);
}

template byte_array<waffle::TurboComposer> sha256_block(const byte_array<waffle::TurboComposer>& input);
template bit_array<waffle::StandardComposer> sha256(const bit_array<waffle::StandardComposer>& input);
template bit_array<waffle::TurboComposer> sha256(const bit_array<waffle::TurboComposer>& input);

} // namespace stdlib
} // namespace plonk
