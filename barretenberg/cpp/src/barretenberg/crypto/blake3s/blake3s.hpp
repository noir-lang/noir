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
#pragma once
#include <array>
#include <cstddef>
#include <cstdint>
#include <string>
#include <vector>

namespace blake3 {

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

using key_array = std::array<uint32_t, BLAKE3_KEY_LEN>;
using block_array = std::array<uint8_t, BLAKE3_BLOCK_LEN>;
using state_array = std::array<uint32_t, 16>;
using out_array = std::array<uint8_t, BLAKE3_OUT_LEN>;

static constexpr key_array IV = { 0x6A09E667UL, 0xBB67AE85UL, 0x3C6EF372UL, 0xA54FF53AUL,
                                  0x510E527FUL, 0x9B05688CUL, 0x1F83D9ABUL, 0x5BE0CD19UL };

static constexpr std::array<uint8_t, 16> MSG_SCHEDULE_0 = { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15 };
static constexpr std::array<uint8_t, 16> MSG_SCHEDULE_1 = { 2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8 };
static constexpr std::array<uint8_t, 16> MSG_SCHEDULE_2 = { 3, 4, 10, 12, 13, 2, 7, 14, 6, 5, 9, 0, 11, 15, 8, 1 };
static constexpr std::array<uint8_t, 16> MSG_SCHEDULE_3 = { 10, 7, 12, 9, 14, 3, 13, 15, 4, 0, 11, 2, 5, 8, 1, 6 };
static constexpr std::array<uint8_t, 16> MSG_SCHEDULE_4 = { 12, 13, 9, 11, 15, 10, 14, 8, 7, 2, 5, 3, 0, 1, 6, 4 };
static constexpr std::array<uint8_t, 16> MSG_SCHEDULE_5 = { 9, 14, 11, 5, 8, 12, 15, 1, 13, 3, 0, 10, 2, 6, 4, 7 };
static constexpr std::array<uint8_t, 16> MSG_SCHEDULE_6 = { 11, 15, 5, 0, 1, 9, 8, 6, 14, 10, 2, 12, 3, 4, 7, 13 };
static constexpr std::array<std::array<uint8_t, 16>, 7> MSG_SCHEDULE = {
    MSG_SCHEDULE_0, MSG_SCHEDULE_1, MSG_SCHEDULE_2, MSG_SCHEDULE_3, MSG_SCHEDULE_4, MSG_SCHEDULE_5, MSG_SCHEDULE_6,
};

struct blake3_hasher {
    key_array key;
    key_array cv;
    block_array buf;
    uint8_t buf_len = 0;
    uint8_t blocks_compressed = 0;
    uint8_t flags = 0;
};

inline const char* blake3_version()
{
    static const std::string version = "0.3.7";
    return version.c_str();
}

constexpr void blake3_hasher_init(blake3_hasher* self);
constexpr void blake3_hasher_update(blake3_hasher* self, const uint8_t* input, size_t input_len);
constexpr void blake3_hasher_finalize(const blake3_hasher* self, uint8_t* out);

constexpr void g(state_array& state, size_t a, size_t b, size_t c, size_t d, uint32_t x, uint32_t y);
constexpr void round_fn(state_array& state, const uint32_t* msg, size_t round);

constexpr void compress_pre(
    state_array& state, const key_array& cv, const uint8_t* block, uint8_t block_len, uint8_t flags);

constexpr void blake3_compress_in_place(key_array& cv, const uint8_t* block, uint8_t block_len, uint8_t flags);

constexpr void blake3_compress_xof(
    const key_array& cv, const uint8_t* block, uint8_t block_len, uint8_t flags, uint8_t* out);

constexpr std::array<uint8_t, BLAKE3_OUT_LEN> blake3s_constexpr(const uint8_t* input, size_t input_size);
inline std::vector<uint8_t> blake3s(std::vector<uint8_t> const& input);

} // namespace blake3

#include "blake3-impl.hpp"
