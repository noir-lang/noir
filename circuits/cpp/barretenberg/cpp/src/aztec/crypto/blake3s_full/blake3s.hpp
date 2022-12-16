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
*/

#include <stddef.h>
#include <stdint.h>
#include <vector>

namespace blake3_full {

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

// modes
enum mode { HASH_MODE = 0, KEYED_HASH_MODE = 1, DERIVE_KEY_MODE = 2 };

static const uint32_t IV[8] = { 0x6A09E667UL, 0xBB67AE85UL, 0x3C6EF372UL, 0xA54FF53AUL,
                                0x510E527FUL, 0x9B05688CUL, 0x1F83D9ABUL, 0x5BE0CD19UL };

static const uint8_t MSG_SCHEDULE[7][16] = {
    { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15 }, { 2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8 },
    { 3, 4, 10, 12, 13, 2, 7, 14, 6, 5, 9, 0, 11, 15, 8, 1 }, { 10, 7, 12, 9, 14, 3, 13, 15, 4, 0, 11, 2, 5, 8, 1, 6 },
    { 12, 13, 9, 11, 15, 10, 14, 8, 7, 2, 5, 3, 0, 1, 6, 4 }, { 9, 14, 11, 5, 8, 12, 15, 1, 13, 3, 0, 10, 2, 6, 4, 7 },
    { 11, 15, 5, 0, 1, 9, 8, 6, 14, 10, 2, 12, 3, 4, 7, 13 },
};

// This struct is a private implementation detail. It has to be here because
// it's part of blake3_hasher below.
typedef struct blake3_chunk_state__ {
    uint32_t cv[8];
    uint64_t chunk_counter;
    uint8_t buf[BLAKE3_BLOCK_LEN];
    uint8_t buf_len;
    uint8_t blocks_compressed;
    uint8_t flags;
} blake3_chunk_state;

typedef struct blake3_hasher__ {
    uint32_t key[8];
    blake3_chunk_state chunk;
    uint8_t cv_stack_len;
    // The stack size is MAX_DEPTH + 1 because we do lazy merging. For example,
    // with 7 chunks, we have 3 entries in the stack. Adding an 8th chunk
    // requires a 4th entry, rather than merging everything down to 1, because we
    // don't know whether more input is coming. This is different from how the
    // reference implementation does things.
    uint8_t cv_stack[(BLAKE3_MAX_DEPTH + 1) * BLAKE3_OUT_LEN];
} blake3_hasher;

const char* blake3_version(void);
void blake3_hasher_init(blake3_hasher* self);
void blake3_hasher_init_keyed(blake3_hasher* self, const uint8_t key[BLAKE3_KEY_LEN]);

void blake3_hasher_init_derive_key(blake3_hasher* self, const char* context);
void blake3_hasher_init_derive_key_raw(blake3_hasher* self, const void* context, size_t context_len);

void blake3_hasher_update(blake3_hasher* self, const void* input, size_t input_len);
void blake3_hasher_finalize(const blake3_hasher* self, uint8_t* out, size_t out_len);
void blake3_hasher_finalize_seek(const blake3_hasher* self, uint64_t seek, uint8_t* out, size_t out_len);

void g(uint32_t* state, size_t a, size_t b, size_t c, size_t d, uint32_t x, uint32_t y);
void round_fn(uint32_t state[16], const uint32_t* msg, size_t round);

void compress_pre(uint32_t state[16],
                  const uint32_t cv[8],
                  const uint8_t block[BLAKE3_BLOCK_LEN],
                  uint8_t block_len,
                  uint64_t counter,
                  uint8_t flags);

void blake3_compress_in_place(
    uint32_t cv[8], const uint8_t block[BLAKE3_BLOCK_LEN], uint8_t block_len, uint64_t counter, uint8_t flags);

void blake3_compress_xof(const uint32_t cv[8],
                         const uint8_t block[BLAKE3_BLOCK_LEN],
                         uint8_t block_len,
                         uint64_t counter,
                         uint8_t flags,
                         uint8_t out[64]);

void blak3s_hash_one(const uint8_t* input,
                     size_t blocks,
                     const uint32_t key[8],
                     uint64_t counter,
                     uint8_t flags,
                     uint8_t flags_start,
                     uint8_t flags_end,
                     uint8_t out[BLAKE3_OUT_LEN]);

void blake3_hash_many(const uint8_t* const* inputs,
                      size_t num_inputs,
                      size_t blocks,
                      const uint32_t key[8],
                      uint64_t counter,
                      bool increment_counter,
                      uint8_t flags,
                      uint8_t flags_start,
                      uint8_t flags_end,
                      uint8_t* out);

std::vector<uint8_t> blake3s(std::vector<uint8_t> const& input,
                             const mode mode_id = HASH_MODE,
                             const uint8_t key[BLAKE3_KEY_LEN] = nullptr,
                             const char* context = nullptr);

} // namespace blake3_full
