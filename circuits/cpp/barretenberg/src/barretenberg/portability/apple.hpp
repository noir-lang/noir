#pragma once

#ifdef __APPLE__

#include "stdint.h"
#include <sys/random.h>

inline void* aligned_alloc(size_t alignment, size_t size)
{
    void* t = 0;
    posix_memalign(&t, alignment, size);
    return t;
}

#define aligned_free free

#endif