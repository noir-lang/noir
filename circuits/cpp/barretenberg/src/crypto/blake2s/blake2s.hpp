/*
   BLAKE2 reference source code package - reference C implementations

   Copyright 2012, Samuel Neves <sneves@dei.uc.pt>.  You may use this under the
   terms of the CC0, the OpenSSL Licence, or the Apache Public License 2.0, at
   your option.  The terms of these licenses can be found at:

   - CC0 1.0 Universal : http://creativecommons.org/publicdomain/zero/1.0
   - OpenSSL license   : https://www.openssl.org/source/license.html
   - Apache 2.0        : http://www.apache.org/licenses/LICENSE-2.0

   More information about the BLAKE2 hash function can be found at
   https://blake2.net.
*/
#pragma once

#include <stdint.h>
#include <vector>

namespace blake2 {

#if defined(_MSC_VER)
#define BLAKE2_PACKED(x) __pragma(pack(push, 1)) x __pragma(pack(pop))
#else
#define BLAKE2_PACKED(x) x __attribute__((packed))
#endif

enum blake2s_constant {
    BLAKE2S_BLOCKBYTES = 64,
    BLAKE2S_OUTBYTES = 32,
    BLAKE2S_KEYBYTES = 32,
    BLAKE2S_SALTBYTES = 8,
    BLAKE2S_PERSONALBYTES = 8
};

typedef struct blake2s_state__ {
    uint32_t h[8];
    uint32_t t[2];
    uint32_t f[2];
    uint8_t buf[BLAKE2S_BLOCKBYTES];
    size_t buflen;
    size_t outlen;
    uint8_t last_node;
} blake2s_state;

BLAKE2_PACKED(struct blake2s_param__ {
    uint8_t digest_length; /* 1 */
    uint8_t key_length;    /* 2 */
    uint8_t fanout;        /* 3 */
    uint8_t depth;         /* 4 */
    uint32_t leaf_length;  /* 8 */
    uint32_t node_offset;  /* 12 */
    uint16_t xof_length;   /* 14 */
    uint8_t node_depth;    /* 15 */
    uint8_t inner_length;  /* 16 */
    /* uint8_t  reserved[0]; */
    uint8_t salt[BLAKE2S_SALTBYTES];         /* 24 */
    uint8_t personal[BLAKE2S_PERSONALBYTES]; /* 32 */
});

typedef struct blake2s_param__ blake2s_param;

/* Padded structs result in a compile-time error */
enum { BLAKE2_DUMMY_1 = 1 / (sizeof(blake2s_param) == BLAKE2S_OUTBYTES) };

int blake2s_init(blake2s_state* S, size_t outlen);
int blake2s_init_key(blake2s_state* S, size_t outlen, const void* key, size_t keylen);
int blake2s_init_param(blake2s_state* S, const blake2s_param* P);
int blake2s_update(blake2s_state* S, const void* in, size_t inlen);
int blake2s_final(blake2s_state* S, void* out, size_t outlen);

std::vector<uint8_t> blake2s(std::vector<uint8_t> const& input);

} // namespace blake2