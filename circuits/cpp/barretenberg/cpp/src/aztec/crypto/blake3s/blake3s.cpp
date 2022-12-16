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

#include <assert.h>
#include <stdbool.h>
#include <string.h>
#include <iostream>
#include <type_traits>

#include "blake3-impl.hpp"

namespace blake3 {

/*
 * Core Blake3s functions. These are similar to that of Blake2s except for a few
 * constant parameters and fewer rounds.
 *
 */
void g(uint32_t* state, size_t a, size_t b, size_t c, size_t d, uint32_t x, uint32_t y)
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

void round_fn(uint32_t state[16], const uint32_t* msg, size_t round)
{
    // Select the message schedule based on the round.
    const uint8_t* schedule = MSG_SCHEDULE[round];

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

void compress_pre(
    uint32_t state[16], const uint32_t cv[8], const uint8_t block[BLAKE3_BLOCK_LEN], uint8_t block_len, uint8_t flags)
{
    uint32_t block_words[16];
    block_words[0] = load32(block + 4 * 0);
    block_words[1] = load32(block + 4 * 1);
    block_words[2] = load32(block + 4 * 2);
    block_words[3] = load32(block + 4 * 3);
    block_words[4] = load32(block + 4 * 4);
    block_words[5] = load32(block + 4 * 5);
    block_words[6] = load32(block + 4 * 6);
    block_words[7] = load32(block + 4 * 7);
    block_words[8] = load32(block + 4 * 8);
    block_words[9] = load32(block + 4 * 9);
    block_words[10] = load32(block + 4 * 10);
    block_words[11] = load32(block + 4 * 11);
    block_words[12] = load32(block + 4 * 12);
    block_words[13] = load32(block + 4 * 13);
    block_words[14] = load32(block + 4 * 14);
    block_words[15] = load32(block + 4 * 15);

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
    state[14] = (uint32_t)block_len;
    state[15] = (uint32_t)flags;

    round_fn(state, &block_words[0], 0);
    round_fn(state, &block_words[0], 1);
    round_fn(state, &block_words[0], 2);
    round_fn(state, &block_words[0], 3);
    round_fn(state, &block_words[0], 4);
    round_fn(state, &block_words[0], 5);
    round_fn(state, &block_words[0], 6);
}

void blake3_compress_in_place(uint32_t cv[8], const uint8_t block[BLAKE3_BLOCK_LEN], uint8_t block_len, uint8_t flags)
{
    uint32_t state[16];
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

void blake3_compress_xof(
    const uint32_t cv[8], const uint8_t block[BLAKE3_BLOCK_LEN], uint8_t block_len, uint8_t flags, uint8_t out[64])
{
    uint32_t state[16];
    compress_pre(state, cv, block, block_len, flags);

    store32(&out[0 * 4], state[0] ^ state[8]);
    store32(&out[1 * 4], state[1] ^ state[9]);
    store32(&out[2 * 4], state[2] ^ state[10]);
    store32(&out[3 * 4], state[3] ^ state[11]);
    store32(&out[4 * 4], state[4] ^ state[12]);
    store32(&out[5 * 4], state[5] ^ state[13]);
    store32(&out[6 * 4], state[6] ^ state[14]);
    store32(&out[7 * 4], state[7] ^ state[15]);
    store32(&out[8 * 4], state[8] ^ cv[0]);
    store32(&out[9 * 4], state[9] ^ cv[1]);
    store32(&out[10 * 4], state[10] ^ cv[2]);
    store32(&out[11 * 4], state[11] ^ cv[3]);
    store32(&out[12 * 4], state[12] ^ cv[4]);
    store32(&out[13 * 4], state[13] ^ cv[5]);
    store32(&out[14 * 4], state[14] ^ cv[6]);
    store32(&out[15 * 4], state[15] ^ cv[7]);
}

const char* blake3_version(void)
{
    return BLAKE3_VERSION_STRING;
}

uint8_t maybe_start_flag(const blake3_hasher* self)
{
    if (self->blocks_compressed == 0) {
        return CHUNK_START;
    } else {
        return 0;
    }
}

typedef struct output_t__ {
    uint32_t input_cv[8];
    uint8_t block[BLAKE3_BLOCK_LEN];
    uint8_t block_len;
    uint8_t flags;
} output_t;

output_t make_output(const uint32_t input_cv[8],
                     const uint8_t block[BLAKE3_BLOCK_LEN],
                     uint8_t block_len,
                     uint8_t flags)
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

void blake3_hasher_init(blake3_hasher* self)
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

void blake3_hasher_update(blake3_hasher* self, const uint8_t* input, size_t input_len)
{
    if (input_len == 0) {
        return;
    }

    while (input_len > BLAKE3_BLOCK_LEN) {
        blake3_compress_in_place(self->cv, input, BLAKE3_BLOCK_LEN, self->flags | maybe_start_flag(self));

        // static_assert(std::is_same<decltype(self->blocks_compressed), uint8_t>::value, "blocks compressed type err");
        // // uint8_t foo = self->blocks_compressed;
        // // uint8_t bar(1U);
        // std::cout << "owauefheaiufhawuifh" << std::endl;
        // std::cout << "owauefheaiufhawuifh" << std::endl;
        // std::cout << "owauefheaiufhawuifh" << std::endl;
        // std::cout << "owauefheaiufhawuifh" << std::endl;

        // static_assert(std::is_same<decltype(foo), uint8_t>::value, "blocks compressed type err A ");
        // static_assert(std::is_same<decltype(bar), uint8_t>::value, "blocks compressed type err B");
        // foo = foo + bar;
        // std::cout << "owauefheaiufhawuifh" << std::endl;
        // std::cout << "owauefheaiufhawuifh" << std::endl;
        // std::cout << "owauefheaiufhawuifh" << std::endl;
        // std::cout << "owauefheaiufhawuifh" << std::endl;
        // std::cout << "owauefheaiufhawuifh" << std::endl;

        self->blocks_compressed = static_cast<uint8_t>(self->blocks_compressed + 1U);
        input += BLAKE3_BLOCK_LEN;
        input_len -= BLAKE3_BLOCK_LEN;
    }

    size_t take = BLAKE3_BLOCK_LEN - ((size_t)self->buf_len);
    if (take > input_len) {
        take = input_len;
    }
    uint8_t* dest = self->buf + ((size_t)self->buf_len);
    for (size_t i = 0; i < take; i++) {
        dest[i] = input[i];
    }

    self->buf_len = static_cast<uint8_t>(self->buf_len + static_cast<uint8_t>(take));
    input_len -= take;
}

void blake3_hasher_finalize(const blake3_hasher* self, uint8_t* out)
{
    uint8_t block_flags = self->flags | maybe_start_flag(self) | CHUNK_END;
    output_t output = make_output(self->cv, self->buf, self->buf_len, block_flags);

    uint8_t wide_buf[64];
    blake3_compress_xof(output.input_cv, output.block, output.block_len, output.flags | ROOT, wide_buf);
    for (size_t i = 0; i < BLAKE3_OUT_LEN; i++) {
        out[i] = wide_buf[i];
    }
    return;
}

std::vector<uint8_t> blake3s(std::vector<uint8_t> const& input)
{
    blake3_hasher hasher;
    blake3_hasher_init(&hasher);
    blake3_hasher_update(&hasher, (const uint8_t*)input.data(), input.size());

    std::vector<uint8_t> output(BLAKE3_OUT_LEN);
    blake3_hasher_finalize(&hasher, &output[0]);
    return output;
}

} // namespace blake3
