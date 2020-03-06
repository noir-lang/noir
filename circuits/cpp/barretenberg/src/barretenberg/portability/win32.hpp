#pragma once

#ifdef _WIN32

#define PRIx64 "llx"
#define PRIu64 "llu"

#include "stddef.h"

inline void* aligned_alloc(size_t alignment, size_t size)
{
    return _aligned_malloc(size, alignment);
}

#define aligned_free _aligned_free

// TODO: WARNING! Using rand()! Should probably be called dontgetentropy()! Replace with native high entropy code.
inline int getentropy(void* buf, size_t size)
{
    for (size_t i = 0; i < size; ++i) {
        ((char*)buf)[i] = (char)rand();
    }
    return 0;
}

#endif