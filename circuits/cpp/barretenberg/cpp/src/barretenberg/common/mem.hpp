#pragma once
#include "log.hpp"
#include "memory.h"
#include <memory>
#include <stdlib.h>

#define pad(size, alignment) (size - (size % alignment) + ((size % alignment) == 0 ? 0 : alignment))

#ifdef __APPLE__
inline void* aligned_alloc(size_t alignment, size_t size)
{
    void* t = 0;
    posix_memalign(&t, alignment, size);
    if (t == 0) {
        info("bad alloc of size: ", size);
        std::abort();
    }
    return t;
}

inline void aligned_free(void* mem)
{
    free(mem);
}
#endif

#if defined(__linux__) || defined(__wasm__)
inline void* protected_aligned_alloc(size_t alignment, size_t size)
{
    size += (size % alignment);
    void* t = 0;
    t = aligned_alloc(alignment, size);
    if (t == 0) {
        info("bad alloc of size: ", size);
        std::abort();
    }
    return t;
}

#define aligned_alloc protected_aligned_alloc

inline void aligned_free(void* mem)
{
    free(mem);
}
#endif

#ifdef _WIN32
inline void* aligned_alloc(size_t alignment, size_t size)
{
    return _aligned_malloc(size, alignment);
}

inline void aligned_free(void* mem)
{
    _aligned_free(mem);
}
#endif