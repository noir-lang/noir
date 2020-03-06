#include "./blake2s.hpp"
#include "../../../composer/turbo_composer.hpp"
#include "../../uint32/uint32.hpp"

namespace plonk {
namespace stdlib {

namespace {
constexpr uint32_t blake2s_IV[8] = { 0x6A09E667UL, 0xBB67AE85UL, 0x3C6EF372UL, 0xA54FF53AUL,
                                     0x510E527FUL, 0x9B05688CUL, 0x1F83D9ABUL, 0x5BE0CD19UL };

constexpr uint32_t initial_H[8] = {
    0x6b08e647, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
};

constexpr uint8_t blake2s_sigma[10][16] = {
    { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15 }, { 14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3 },
    { 11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4 }, { 7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8 },
    { 9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13 }, { 2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9 },
    { 12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11 }, { 13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10 },
    { 6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5 }, { 10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0 },
};

enum blake2s_constant {
    BLAKE2S_BLOCKBYTES = 64,
    BLAKE2S_OUTBYTES = 32,
    BLAKE2S_KEYBYTES = 32,
    BLAKE2S_SALTBYTES = 8,
    BLAKE2S_PERSONALBYTES = 8
};

template <typename Composer> struct blake2s_state {
    uint32<Composer> h[8];
    uint32<Composer> t[2];
    uint32<Composer> f[2];
};

template <typename Composer> void blake2s_increment_counter(blake2s_state<Composer>& S, const uint32_t inc)
{
    S.t[0] = S.t[0] + inc;
    // TODO: Secure!? Think so as inc is known at "compile" time as it's derived from the msg length.
    S.t[1] = S.t[1] + ((S.t[0] < inc).get_value() ? 1 : 0);
}

#define G(r, i, a, b, c, d)                                                                                            \
    do {                                                                                                               \
        a = a + b + m[blake2s_sigma[r][2 * i + 0]];                                                                    \
        d = (d ^ a).ror(16);                                                                                           \
        c = c + d;                                                                                                     \
        b = (b ^ c).ror(12);                                                                                           \
        a = a + b + m[blake2s_sigma[r][2 * i + 1]];                                                                    \
        d = (d ^ a).ror(8);                                                                                            \
        c = c + d;                                                                                                     \
        b = (b ^ c).ror(7);                                                                                            \
    } while (0)

#define ROUND(r)                                                                                                       \
    do {                                                                                                               \
        G(r, 0, v[0], v[4], v[8], v[12]);                                                                              \
        G(r, 1, v[1], v[5], v[9], v[13]);                                                                              \
        G(r, 2, v[2], v[6], v[10], v[14]);                                                                             \
        G(r, 3, v[3], v[7], v[11], v[15]);                                                                             \
        G(r, 4, v[0], v[5], v[10], v[15]);                                                                             \
        G(r, 5, v[1], v[6], v[11], v[12]);                                                                             \
        G(r, 6, v[2], v[7], v[8], v[13]);                                                                              \
        G(r, 7, v[3], v[4], v[9], v[14]);                                                                              \
    } while (0)

template <typename Composer> void blake2s_compress(blake2s_state<Composer>& S, byte_array<Composer> const& in)
{
    uint32<Composer> m[16];
    uint32<Composer> v[16];

    for (size_t i = 0; i < 16; ++i) {
        m[i] = uint32<Composer>(in.slice(i * 4, 4).reverse());
    }

    for (size_t i = 0; i < 8; ++i) {
        v[i] = S.h[i];
    }

    v[8] = blake2s_IV[0];
    v[9] = blake2s_IV[1];
    v[10] = blake2s_IV[2];
    v[11] = blake2s_IV[3];
    v[12] = S.t[0] ^ blake2s_IV[4];
    v[13] = S.t[1] ^ blake2s_IV[5];
    v[14] = S.f[0] ^ blake2s_IV[6];
    v[15] = S.f[1] ^ blake2s_IV[7];

    ROUND(0);
    ROUND(1);
    ROUND(2);
    ROUND(3);
    ROUND(4);
    ROUND(5);
    ROUND(6);
    ROUND(7);
    ROUND(8);
    ROUND(9);

    for (size_t i = 0; i < 8; ++i) {
        S.h[i] = S.h[i] ^ v[i] ^ v[i + 8];
    }
}

#undef G
#undef ROUND

template <typename Composer> void blake2s(blake2s_state<Composer>& S, byte_array<Composer> const& in)
{
    size_t offset = 0;
    size_t size = in.size();

    while (size > BLAKE2S_BLOCKBYTES) {
        blake2s_increment_counter(S, BLAKE2S_BLOCKBYTES);
        blake2s_compress(S, in.slice(offset, BLAKE2S_BLOCKBYTES));
        offset += BLAKE2S_BLOCKBYTES;
        size -= BLAKE2S_BLOCKBYTES;
    }

    // Set last block.
    S.f[0] = (uint32_t)-1;

    byte_array<Composer> final(in.get_context());
    final.write(in.slice(offset)).write(byte_array<Composer>(in.get_context(), BLAKE2S_BLOCKBYTES - size));
    blake2s_increment_counter(S, (uint32_t)size);
    blake2s_compress(S, final);
}

} // namespace

template <typename Composer> byte_array<Composer> blake2s(const byte_array<Composer>& input)
{
    blake2s_state<Composer> S;

    for (size_t i = 0; i < 8; i++) {
        S.h[i] = initial_H[i];
    }

    blake2s(S, input);

    byte_array<Composer> result(input.get_context());
    for (auto h : S.h) {
        byte_array<Composer> v = h;
        result.write(v.reverse());
    }
    return result;
}

template byte_array<waffle::TurboComposer> blake2s(const byte_array<waffle::TurboComposer>& input);

} // namespace stdlib
} // namespace plonk