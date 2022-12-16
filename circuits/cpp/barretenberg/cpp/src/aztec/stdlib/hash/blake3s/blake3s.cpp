#include "blake3s.hpp"
#include "blake3s_plookup.hpp"
#include <plonk/composer/turbo_composer.hpp>
#include <plonk/composer/ultra_composer.hpp>
#include <stdlib/primitives/uint/uint.hpp>
#include "../blake2s/blake_util.hpp"

namespace plonk {
namespace stdlib {

namespace blake3_internal {

/*
 * Constants and more.
 */
#define BLAKE3_VERSION_STRING "0.3.7"

// internal flags
enum blake3_flags {
    CHUNK_START = 1 << 0,
    CHUNK_END = 1 << 1,
    PARENT = 1 << 2,
    ROOT = 1 << 3,
    KEYED_HASH = 1 << 4,
    DERIVE_KEY_CONTEXT = 1 << 5,
    DERIVE_KEY_MATERIAL = 1 << 6,
};

// constants
enum blake3s_constant {
    BLAKE3_KEY_LEN = 32,
    BLAKE3_OUT_LEN = 32,
    BLAKE3_BLOCK_LEN = 64,
    BLAKE3_CHUNK_LEN = 1024,
    BLAKE3_MAX_DEPTH = 54,
    BLAKE3_STATE_SIZE = 16
};

static const uint32_t IV[8] = { 0x6A09E667UL, 0xBB67AE85UL, 0x3C6EF372UL, 0xA54FF53AUL,
                                0x510E527FUL, 0x9B05688CUL, 0x1F83D9ABUL, 0x5BE0CD19UL };

template <typename Composer> struct blake3_hasher {
    uint32<Composer> key[8];
    uint32<Composer> cv[8];
    byte_array<Composer> buf;
    uint8_t buf_len;
    uint8_t blocks_compressed;
    uint8_t flags;
    Composer* context;
};

/*
 * Core Blake3s functions. These are similar to that of Blake2s except for a few
 * constant parameters and fewer rounds.
 *
 */
template <typename Composer>
void compress_pre(uint32<Composer> state[BLAKE3_STATE_SIZE],
                  const uint32<Composer> cv[8],
                  const byte_array<Composer>& block,
                  uint8_t block_len,
                  uint8_t flags)
{
    uint32<Composer> block_words[BLAKE3_STATE_SIZE];
    for (size_t i = 0; i < BLAKE3_STATE_SIZE; ++i) {
        block_words[i] = uint32<Composer>(block.slice(i * 4, 4).reverse());
    }

    state[0] = cv[0];
    state[1] = cv[1];
    state[2] = cv[2];
    state[3] = cv[3];
    state[4] = cv[4];
    state[5] = cv[5];
    state[6] = cv[6];
    state[7] = cv[7];
    state[8] = IV[0];
    state[9] = IV[1];
    state[10] = IV[2];
    state[11] = IV[3];
    state[12] = uint32<Composer>(block.get_context(), 0);
    state[13] = uint32<Composer>(block.get_context(), 0);
    state[14] = uint32<Composer>(block.get_context(), block_len);
    state[15] = uint32<Composer>(block.get_context(), flags);

    blake_util::round_fn<Composer>(state, &block_words[0], 0, true);
    blake_util::round_fn<Composer>(state, &block_words[0], 1, true);
    blake_util::round_fn<Composer>(state, &block_words[0], 2, true);
    blake_util::round_fn<Composer>(state, &block_words[0], 3, true);
    blake_util::round_fn<Composer>(state, &block_words[0], 4, true);
    blake_util::round_fn<Composer>(state, &block_words[0], 5, true);
    blake_util::round_fn<Composer>(state, &block_words[0], 6, true);
}

template <typename Composer>
void blake3_compress_in_place(uint32<Composer> cv[8],
                              const byte_array<Composer>& block,
                              uint8_t block_len,
                              uint8_t flags)
{
    uint32<Composer> state[BLAKE3_STATE_SIZE];
    compress_pre<Composer>(state, cv, block, block_len, flags);
    cv[0] = state[0] ^ state[8];
    cv[1] = state[1] ^ state[9];
    cv[2] = state[2] ^ state[10];
    cv[3] = state[3] ^ state[11];
    cv[4] = state[4] ^ state[12];
    cv[5] = state[5] ^ state[13];
    cv[6] = state[6] ^ state[14];
    cv[7] = state[7] ^ state[15];
}

template <typename Composer>
void blake3_compress_xof(const uint32<Composer> cv[8],
                         const byte_array<Composer>& block,
                         uint8_t block_len,
                         uint8_t flags,
                         byte_array<Composer>& out)
{
    uint32<Composer> state[BLAKE3_STATE_SIZE];
    compress_pre<Composer>(state, cv, block, block_len, flags);

    out.write_at(static_cast<byte_array<Composer>>(state[0] ^ state[8]).reverse(), 0 * 4);
    out.write_at(static_cast<byte_array<Composer>>(state[1] ^ state[9]).reverse(), 1 * 4);
    out.write_at(static_cast<byte_array<Composer>>(state[2] ^ state[10]).reverse(), 2 * 4);
    out.write_at(static_cast<byte_array<Composer>>(state[3] ^ state[11]).reverse(), 3 * 4);
    out.write_at(static_cast<byte_array<Composer>>(state[4] ^ state[12]).reverse(), 4 * 4);
    out.write_at(static_cast<byte_array<Composer>>(state[5] ^ state[13]).reverse(), 5 * 4);
    out.write_at(static_cast<byte_array<Composer>>(state[6] ^ state[14]).reverse(), 6 * 4);
    out.write_at(static_cast<byte_array<Composer>>(state[7] ^ state[15]).reverse(), 7 * 4);

    out.write_at(static_cast<byte_array<Composer>>(state[8] ^ cv[0]).reverse(), 8 * 4);
    out.write_at(static_cast<byte_array<Composer>>(state[9] ^ cv[1]).reverse(), 9 * 4);
    out.write_at(static_cast<byte_array<Composer>>(state[10] ^ cv[2]).reverse(), 10 * 4);
    out.write_at(static_cast<byte_array<Composer>>(state[11] ^ cv[3]).reverse(), 11 * 4);
    out.write_at(static_cast<byte_array<Composer>>(state[12] ^ cv[4]).reverse(), 12 * 4);
    out.write_at(static_cast<byte_array<Composer>>(state[13] ^ cv[5]).reverse(), 13 * 4);
    out.write_at(static_cast<byte_array<Composer>>(state[14] ^ cv[6]).reverse(), 14 * 4);
    out.write_at(static_cast<byte_array<Composer>>(state[15] ^ cv[7]).reverse(), 15 * 4);
}

/*
 * Blake3s helper functions.
 *
 */
template <typename Composer> uint8_t maybe_start_flag(const blake3_hasher<Composer>* self)
{
    if (self->blocks_compressed == 0) {
        return CHUNK_START;
    } else {
        return 0;
    }
}

template <typename Composer> struct output_t {
    uint32<Composer> input_cv[8];
    byte_array<Composer> block;
    uint8_t block_len;
    uint8_t flags;
};

template <typename Composer>
output_t<Composer> make_output(const uint32<Composer> input_cv[8],
                               const byte_array<Composer>& block,
                               uint8_t block_len,
                               uint8_t flags)
{
    output_t<Composer> ret;
    for (size_t i = 0; i < (BLAKE3_OUT_LEN >> 2); ++i) {
        ret.input_cv[i] = input_cv[i];
    }
    ret.block = byte_array<Composer>(block.get_context(), BLAKE3_BLOCK_LEN);
    for (size_t i = 0; i < BLAKE3_BLOCK_LEN; i++) {
        ret.block.set_byte(i, block[i]);
    }
    ret.block_len = block_len;
    ret.flags = flags;
    return ret;
}

/*
 * Blake3s wrapper functions.
 *
 */
template <typename Composer> void blake3_hasher_init(blake3_hasher<Composer>* self)
{
    for (size_t i = 0; i < (BLAKE3_KEY_LEN >> 2); ++i) {
        self->key[i] = IV[i];
        self->cv[i] = IV[i];
    }
    self->buf = byte_array<Composer>(self->context, BLAKE3_BLOCK_LEN);
    for (size_t i = 0; i < BLAKE3_BLOCK_LEN; i++) {
        self->buf.set_byte(i, field_t<Composer>(self->context, 0));
    }
    self->buf_len = 0;
    self->blocks_compressed = 0;
    self->flags = 0;
}

template <typename Composer>
void blake3_hasher_update(blake3_hasher<Composer>* self, const byte_array<Composer>& input, size_t input_len)
{
    if (input_len == 0) {
        return;
    }

    size_t start_counter = 0;
    while (input_len > BLAKE3_BLOCK_LEN) {
        blake3_compress_in_place<Composer>(self->cv,
                                           input.slice(start_counter, BLAKE3_BLOCK_LEN),
                                           BLAKE3_BLOCK_LEN,
                                           self->flags | maybe_start_flag(self));
        self->blocks_compressed = static_cast<uint8_t>(self->blocks_compressed + 1);
        start_counter += BLAKE3_BLOCK_LEN;
        input_len -= BLAKE3_BLOCK_LEN;
    }

    size_t take = BLAKE3_BLOCK_LEN - ((size_t)self->buf_len);
    if (take > input_len) {
        take = input_len;
    }
    for (size_t i = 0; i < take; i++) {
        self->buf.set_byte(self->buf_len + i, input[i + start_counter]);
    }

    self->buf_len = static_cast<uint8_t>(self->buf_len + (uint8_t)take);
    input_len -= take;
}

template <typename Composer> void blake3_hasher_finalize(const blake3_hasher<Composer>* self, byte_array<Composer>& out)
{
    uint8_t block_flags = self->flags | maybe_start_flag(self) | CHUNK_END;
    output_t<Composer> output = make_output<Composer>(self->cv, self->buf, self->buf_len, block_flags);

    byte_array<Composer> wide_buf(out.get_context(), BLAKE3_BLOCK_LEN);
    blake3_compress_xof(output.input_cv, output.block, output.block_len, output.flags | ROOT, wide_buf);
    for (size_t i = 0; i < BLAKE3_OUT_LEN; i++) {
        out.set_byte(i, wide_buf[i]);
    }
    return;
}

} // namespace blake3_internal

using namespace blake3_internal;

template <typename Composer> byte_array<Composer> blake3s(const byte_array<Composer>& input)
{
    if constexpr (Composer::type == waffle::ComposerType::PLOOKUP) {
        return blake3s_plookup::blake3s<waffle::UltraComposer>(input);
    }

    blake3_hasher<Composer> hasher = {};
    hasher.context = input.get_context();
    blake3_hasher_init<Composer>(&hasher);
    blake3_hasher_update<Composer>(&hasher, input, input.size());
    byte_array<Composer> result(input.get_context(), BLAKE3_OUT_LEN);
    blake3_hasher_finalize<Composer>(&hasher, result);
    return result;
}

template byte_array<waffle::StandardComposer> blake3s(const byte_array<waffle::StandardComposer>& input);
template byte_array<waffle::TurboComposer> blake3s(const byte_array<waffle::TurboComposer>& input);
template byte_array<waffle::UltraComposer> blake3s(const byte_array<waffle::UltraComposer>& input);

} // namespace stdlib
} // namespace plonk
