#pragma once

#if defined(__linux__) || defined(__wasm__)
#include <arpa/inet.h>
#include <endian.h>
#define ntohll be64toh
#define htonll htobe64
#endif

inline bool is_little_endian()
{
    constexpr int num = 42;
    return (*(char*)&num == 42);
}