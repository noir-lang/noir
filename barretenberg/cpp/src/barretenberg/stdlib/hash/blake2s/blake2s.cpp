#include "blake2s.hpp"
#include "barretenberg/stdlib/primitives/uint/uint.hpp"
#include "blake2s_plookup.hpp"
#include "blake_util.hpp"

namespace bb::stdlib {

namespace {
constexpr uint32_t blake2s_IV[8] = { 0x6A09E667UL, 0xBB67AE85UL, 0x3C6EF372UL, 0xA54FF53AUL,
                                     0x510E527FUL, 0x9B05688CUL, 0x1F83D9ABUL, 0x5BE0CD19UL };

constexpr uint32_t initial_H[8] = {
    0x6b08e647, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
};

enum blake2s_constant {
    BLAKE2S_BLOCKBYTES = 64,
    BLAKE2S_OUTBYTES = 32,
    BLAKE2S_KEYBYTES = 32,
    BLAKE2S_SALTBYTES = 8,
    BLAKE2S_PERSONALBYTES = 8
};

template <typename Builder> struct blake2s_state {
    uint32<Builder> h[8];
    uint32<Builder> t[2];
    uint32<Builder> f[2];
};

template <typename Builder> void blake2s_increment_counter(blake2s_state<Builder>& S, const uint32_t inc)
{
    S.t[0] = S.t[0] + inc;
    const bool to_inc = S.t[0].get_value() < inc;
    S.t[1] = S.t[1] + (to_inc ? 1 : 0);
    // We assert that t[0] and t[1] are circuit constants to ensure the incerementing depends only on the circuit and
    // not on witness values
    ASSERT(S.t[0].is_constant());
    ASSERT(S.t[1].is_constant());
}

template <typename Builder> void blake2s_compress(blake2s_state<Builder>& S, byte_array<Builder> const& in)
{
    uint32<Builder> m[16];
    uint32<Builder> v[16];

    for (size_t i = 0; i < 16; ++i) {
        m[i] = uint32<Builder>(in.slice(i * 4, 4).reverse());
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

    blake_util::round_fn<Builder>(v, m, 0);
    blake_util::round_fn<Builder>(v, m, 1);
    blake_util::round_fn<Builder>(v, m, 2);
    blake_util::round_fn<Builder>(v, m, 3);
    blake_util::round_fn<Builder>(v, m, 4);
    blake_util::round_fn<Builder>(v, m, 5);
    blake_util::round_fn<Builder>(v, m, 6);
    blake_util::round_fn<Builder>(v, m, 7);
    blake_util::round_fn<Builder>(v, m, 8);
    blake_util::round_fn<Builder>(v, m, 9);

    // ROUND(0);
    // ROUND(1);
    // ROUND(2);
    // ROUND(3);
    // ROUND(4);
    // ROUND(5);
    // ROUND(6);
    // ROUND(7);
    // ROUND(8);
    // ROUND(9);

    for (size_t i = 0; i < 8; ++i) {
        S.h[i] = S.h[i] ^ v[i] ^ v[i + 8];
    }
}

#undef G
#undef ROUND

template <typename Builder> void blake2s(blake2s_state<Builder>& S, byte_array<Builder> const& in)
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

    byte_array<Builder> final(in.get_context());
    final.write(in.slice(offset)).write(byte_array<Builder>(in.get_context(), BLAKE2S_BLOCKBYTES - size));
    blake2s_increment_counter(S, (uint32_t)size);
    blake2s_compress(S, final);
}

} // namespace

template <typename Builder> byte_array<Builder> blake2s(const byte_array<Builder>& input)
{
    if constexpr (HasPlookup<Builder>) {
        return blake2s_plookup::blake2s<Builder>(input);
    }

    blake2s_state<Builder> S;

    for (size_t i = 0; i < 8; i++) {
        S.h[i] = initial_H[i];
    }

    blake2s(S, input);

    byte_array<Builder> result(input.get_context());
    for (auto h : S.h) {
        byte_array<Builder> v = static_cast<byte_array<Builder>>(h);
        result.write(v.reverse());
    }
    return result;
}

template byte_array<bb::StandardCircuitBuilder> blake2s(const byte_array<bb::StandardCircuitBuilder>& input);
template byte_array<bb::UltraCircuitBuilder> blake2s(const byte_array<bb::UltraCircuitBuilder>& input);
template byte_array<bb::GoblinUltraCircuitBuilder> blake2s(const byte_array<bb::GoblinUltraCircuitBuilder>& input);

} // namespace bb::stdlib
