#pragma once
/*
    BLAKE3 reference source code package - C implementations

    Intellectual property:

    The Rust code is copyright Jack O'Connor, 2019-2020.
    The C code is copyright Samuel Neves and Jack O'Connor, 2019-2020.
    The assembly code is copyright Samuel Neves, 2019-2020.

    This work is released into the public domain with CC0 1.0. Alternatively, it is licensed under the Apache
   License 2.0.

    - CC0 1.0 Universal : http://creativecommons.org/publicdomain/zero/1.0
    - Apache 2.0        : http://www.apache.org/licenses/LICENSE-2.0

    More information about the BLAKE3 hash function can be found at
    https://github.com/BLAKE3-team/BLAKE3.


    NOTE: We have modified the original code from the BLAKE3 reference C implementation.
    The following code works ONLY for inputs of size less than 1024 bytes. This kind of constraint
    on the input size greatly simplifies the code and helps us get rid of the recursive merkle-tree
    like operations on chunks (data of size 1024 bytes). This is because we would always be using BLAKE3
    hashing for inputs of size 32 bytes (or lesser) in barretenberg. The full C++ version of BLAKE3
    from the original authors is in the module `../crypto/blake3s_full`.

    Also, the length of the output in this specific implementation is fixed at 32 bytes which is the only
    version relevant to Barretenberg.
*/

#include <iostream>
#include <type_traits>

#include "blake3s.hpp"

namespace blake3 {

/*
 * Core Blake3s functions. These are similar to that of Blake2s except for a few
 * constant parameters and fewer rounds.
 *
 */
constexpr void g(state_array& state, size_t a, size_t b, size_t c, size_t d, uint32_t x, uint32_t y)
{
    state[a] = state[a] + state[b] + x;
    state[d] = rotr32(state[d] ^ state[a], 16);
    state[c] = state[c] + state[d];
    state[b] = rotr32(state[b] ^ state[c], 12);
    state[a] = state[a] + state[b] + y;
    state[d] = rotr32(state[d] ^ state[a], 8);
    state[c] = state[c] + state[d];
    state[b] = rotr32(state[b] ^ state[c], 7);
}

constexpr void round_fn(state_array& state, const uint32_t* msg, size_t round)
{
    // Select the message schedule based on the round.
    const auto schedule = MSG_SCHEDULE[round];

    // Mix the columns.
    g(state, 0, 4, 8, 12, msg[schedule[0]], msg[schedule[1]]);
    g(state, 1, 5, 9, 13, msg[schedule[2]], msg[schedule[3]]);
    g(state, 2, 6, 10, 14, msg[schedule[4]], msg[schedule[5]]);
    g(state, 3, 7, 11, 15, msg[schedule[6]], msg[schedule[7]]);

    // Mix the rows.
    g(state, 0, 5, 10, 15, msg[schedule[8]], msg[schedule[9]]);
    g(state, 1, 6, 11, 12, msg[schedule[10]], msg[schedule[11]]);
    g(state, 2, 7, 8, 13, msg[schedule[12]], msg[schedule[13]]);
    g(state, 3, 4, 9, 14, msg[schedule[14]], msg[schedule[15]]);
}

constexpr void compress_pre(
    state_array& state, const key_array& cv, const uint8_t* block, uint8_t block_len, uint8_t flags)
{
    std::array<uint32_t, 16> block_words;
    block_words[0] = load32(&block[0]);
    block_words[1] = load32(&block[4]);
    block_words[2] = load32(&block[8]);
    block_words[3] = load32(&block[12]);
    block_words[4] = load32(&block[16]);
    block_words[5] = load32(&block[20]);
    block_words[6] = load32(&block[24]);
    block_words[7] = load32(&block[28]);
    block_words[8] = load32(&block[32]);
    block_words[9] = load32(&block[36]);
    block_words[10] = load32(&block[40]);
    block_words[11] = load32(&block[44]);
    block_words[12] = load32(&block[48]);
    block_words[13] = load32(&block[52]);
    block_words[14] = load32(&block[56]);
    block_words[15] = load32(&block[60]);

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
    state[12] = 0;
    state[13] = 0;
    state[14] = static_cast<uint32_t>(block_len);
    state[15] = static_cast<uint32_t>(flags);

    round_fn(state, &block_words[0], 0);
    round_fn(state, &block_words[0], 1);
    round_fn(state, &block_words[0], 2);
    round_fn(state, &block_words[0], 3);
    round_fn(state, &block_words[0], 4);
    round_fn(state, &block_words[0], 5);
    round_fn(state, &block_words[0], 6);
}

constexpr void blake3_compress_in_place(key_array& cv, const uint8_t* block, uint8_t block_len, uint8_t flags)
{
    state_array state;
    compress_pre(state, cv, block, block_len, flags);
    cv[0] = state[0] ^ state[8];
    cv[1] = state[1] ^ state[9];
    cv[2] = state[2] ^ state[10];
    cv[3] = state[3] ^ state[11];
    cv[4] = state[4] ^ state[12];
    cv[5] = state[5] ^ state[13];
    cv[6] = state[6] ^ state[14];
    cv[7] = state[7] ^ state[15];
}

constexpr void blake3_compress_xof(
    const key_array& cv, const uint8_t* block, uint8_t block_len, uint8_t flags, uint8_t* out)
{
    state_array state;
    compress_pre(state, cv, block, block_len, flags);

    store32(&out[0], state[0] ^ state[8]);
    store32(&out[4], state[1] ^ state[9]);
    store32(&out[8], state[2] ^ state[10]);
    store32(&out[12], state[3] ^ state[11]);
    store32(&out[16], state[4] ^ state[12]);
    store32(&out[20], state[5] ^ state[13]);
    store32(&out[24], state[6] ^ state[14]);
    store32(&out[28], state[7] ^ state[15]);
    store32(&out[32], state[8] ^ cv[0]);
    store32(&out[36], state[9] ^ cv[1]);
    store32(&out[40], state[10] ^ cv[2]);
    store32(&out[44], state[11] ^ cv[3]);
    store32(&out[48], state[12] ^ cv[4]);
    store32(&out[52], state[13] ^ cv[5]);
    store32(&out[56], state[14] ^ cv[6]);
    store32(&out[60], state[15] ^ cv[7]);
}

constexpr uint8_t maybe_start_flag(const blake3_hasher* self)
{
    if (self->blocks_compressed == 0) {
        return CHUNK_START;
    }
    return 0;
}

struct output_t {
    key_array input_cv = {};
    block_array block = {};
    uint8_t block_len = 0;
    uint8_t flags = 0;
};

constexpr output_t make_output(const key_array& input_cv, const uint8_t* block, uint8_t block_len, uint8_t flags)
{
    output_t ret;
    for (size_t i = 0; i < (BLAKE3_OUT_LEN >> 2); ++i) {
        ret.input_cv[i] = input_cv[i];
    }
    for (size_t i = 0; i < BLAKE3_BLOCK_LEN; i++) {
        ret.block[i] = block[i];
    }
    ret.block_len = block_len;
    ret.flags = flags;
    return ret;
}

constexpr void blake3_hasher_init(blake3_hasher* self)
{
    for (size_t i = 0; i < (BLAKE3_KEY_LEN >> 2); ++i) {
        self->key[i] = IV[i];
        self->cv[i] = IV[i];
    }
    for (size_t i = 0; i < BLAKE3_BLOCK_LEN; i++) {
        self->buf[i] = 0;
    }
    self->buf_len = 0;
    self->blocks_compressed = 0;
    self->flags = 0;
}

constexpr void blake3_hasher_update(blake3_hasher* self, const uint8_t* input, size_t input_len)
{
    if (input_len == 0) {
        return;
    }

    while (input_len > BLAKE3_BLOCK_LEN) {
        blake3_compress_in_place(self->cv, input, BLAKE3_BLOCK_LEN, self->flags | maybe_start_flag(self));

        self->blocks_compressed = static_cast<uint8_t>(self->blocks_compressed + 1U);
        input += BLAKE3_BLOCK_LEN;
        input_len -= BLAKE3_BLOCK_LEN;
    }

    size_t take = BLAKE3_BLOCK_LEN - (static_cast<size_t>(self->buf_len));
    if (take > input_len) {
        take = input_len;
    }
    uint8_t* dest = &self->buf[0] + (static_cast<size_t>(self->buf_len));
    for (size_t i = 0; i < take; i++) {
        dest[i] = input[i];
    }

    self->buf_len = static_cast<uint8_t>(self->buf_len + static_cast<uint8_t>(take));
    input_len -= take;
}

constexpr void blake3_hasher_finalize(const blake3_hasher* self, uint8_t* out)
{
    uint8_t block_flags = self->flags | maybe_start_flag(self) | CHUNK_END;
    output_t output = make_output(self->cv, &self->buf[0], self->buf_len, block_flags);

    block_array wide_buf;
    blake3_compress_xof(output.input_cv, &output.block[0], output.block_len, output.flags | ROOT, &wide_buf[0]);
    for (size_t i = 0; i < BLAKE3_OUT_LEN; i++) {
        out[i] = wide_buf[i];
    }
}

std::vector<uint8_t> blake3s(std::vector<uint8_t> const& input)
{
    blake3_hasher hasher;
    blake3_hasher_init(&hasher);
    blake3_hasher_update(&hasher, static_cast<const uint8_t*>(input.data()), input.size());

    std::vector<uint8_t> output(BLAKE3_OUT_LEN);
    blake3_hasher_finalize(&hasher, &output[0]);
    return output;
}

constexpr std::array<uint8_t, BLAKE3_OUT_LEN> blake3s_constexpr(const uint8_t* input, const size_t input_size)
{
    blake3_hasher hasher;
    blake3_hasher_init(&hasher);
    blake3_hasher_update(&hasher, input, input_size);

    std::array<uint8_t, BLAKE3_OUT_LEN> output;
    blake3_hasher_finalize(&hasher, &output[0]);
    return output;
}

} // namespace blake3
