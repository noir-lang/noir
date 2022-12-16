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

#include <stddef.h>
#include <stdint.h>
#include <vector>

namespace blake3 {

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
    BLAKE3_MAX_DEPTH = 54
};

static const uint32_t IV[8] = { 0x6A09E667UL, 0xBB67AE85UL, 0x3C6EF372UL, 0xA54FF53AUL,
                                0x510E527FUL, 0x9B05688CUL, 0x1F83D9ABUL, 0x5BE0CD19UL };

static const uint8_t MSG_SCHEDULE[7][16] = {
    { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15 }, { 2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8 },
    { 3, 4, 10, 12, 13, 2, 7, 14, 6, 5, 9, 0, 11, 15, 8, 1 }, { 10, 7, 12, 9, 14, 3, 13, 15, 4, 0, 11, 2, 5, 8, 1, 6 },
    { 12, 13, 9, 11, 15, 10, 14, 8, 7, 2, 5, 3, 0, 1, 6, 4 }, { 9, 14, 11, 5, 8, 12, 15, 1, 13, 3, 0, 10, 2, 6, 4, 7 },
    { 11, 15, 5, 0, 1, 9, 8, 6, 14, 10, 2, 12, 3, 4, 7, 13 },
};

struct blake3_hasher {
    uint32_t key[8];
    uint32_t cv[8];
    uint8_t buf[BLAKE3_BLOCK_LEN];
    uint8_t buf_len = 0;
    uint8_t blocks_compressed = 0;
    uint8_t flags = 0;
};

const char* blake3_version(void);

void blake3_hasher_init(blake3_hasher* self);
void blake3_hasher_update(blake3_hasher* self, const uint8_t* input, size_t input_len);
void blake3_hasher_finalize(const blake3_hasher* self, uint8_t* out);

void g(uint32_t* state, size_t a, size_t b, size_t c, size_t d, uint32_t x, uint32_t y);
void round_fn(uint32_t state[16], const uint32_t* msg, size_t round);

void compress_pre(
    uint32_t state[16], const uint32_t cv[8], const uint8_t block[BLAKE3_BLOCK_LEN], uint8_t block_len, uint8_t flags);

void blake3_compress_in_place(uint32_t cv[8], const uint8_t block[BLAKE3_BLOCK_LEN], uint8_t block_len, uint8_t flags);

void blake3_compress_xof(
    const uint32_t cv[8], const uint8_t block[BLAKE3_BLOCK_LEN], uint8_t block_len, uint8_t flags, uint8_t out[64]);

std::vector<uint8_t> blake3s(std::vector<uint8_t> const& input);
} // namespace blake3
