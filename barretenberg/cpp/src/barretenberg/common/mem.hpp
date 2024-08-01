#pragma once
#include "log.hpp"
#include "memory.h"
#include "tracy/Tracy.hpp"
#include "wasm_export.hpp"
#include <cstdlib>
#include <memory>

// This can be altered to capture stack traces, though more expensive
// This is the only reason we wrap TracyAlloc or TracyAllocS
#define TRACY_ALLOC(t, size) TracyAllocS(t, size, /*stack depth*/ 10)
#define TRACY_FREE(t) TracyFreeS(t, /*stack depth*/ 10)
// #define TRACY_ALLOC(t, size) TracyAlloc(t, size)
// #define TRACY_FREE(t) TracyFree(t)

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
    TRACY_ALLOC(t, size);
    return t;
}

inline void aligned_free(void* mem)
{
    TRACY_FREE(mem);
    free(mem);
}
#endif

#if defined(__linux__) || defined(__wasm__)
inline void* protected_aligned_alloc(size_t alignment, size_t size)
{
    size += (size % alignment);
    void* t = nullptr;
    // pad size to alignment
    if (size % alignment != 0) {
        size += alignment - (size % alignment);
    }
    // NOLINTNEXTLINE(cppcoreguidelines-owning-memory)
    t = aligned_alloc(alignment, size);
    if (t == nullptr) {
        info("bad alloc of size: ", size);
        std::abort();
    }
    TRACY_ALLOC(t, size);
    return t;
}

#define aligned_alloc protected_aligned_alloc

inline void aligned_free(void* mem)
{
    TRACY_FREE(mem);
    // NOLINTNEXTLINE(cppcoreguidelines-owning-memory, cppcoreguidelines-no-malloc)
    free(mem);
}
#endif

#ifdef _WIN32
inline void* aligned_alloc(size_t alignment, size_t size)
{
    void* t = _aligned_malloc(size, alignment);
    TRACY_ALLOC(t, size);
    return t;
}

inline void aligned_free(void* mem)
{
    TRACY_FREE(mem);
    _aligned_free(mem);
}
#endif

// inline void print_malloc_info()
// {
//     struct mallinfo minfo = mallinfo();

//     info("Total non-mmapped bytes (arena): ", minfo.arena);
//     info("Number of free chunks (ordblks): ", minfo.ordblks);
//     info("Number of fastbin blocks (smblks): ", minfo.smblks);
//     info("Number of mmapped regions (hblks): ", minfo.hblks);
//     info("Space allocated in mmapped regions (hblkhd): ", minfo.hblkhd);
//     info("Maximum total allocated space (usmblks): ", minfo.usmblks);
//     info("Space available in freed fastbin blocks (fsmblks): ", minfo.fsmblks);
//     info("Total allocated space (uordblks): ", minfo.uordblks);
//     info("Total free space (fordblks): ", minfo.fordblks);
//     info("Top-most, releasable space (keepcost): ", minfo.keepcost);
// }

inline void* tracy_malloc(size_t size)
{
    // NOLINTNEXTLINE(cppcoreguidelines-owning-memory, cppcoreguidelines-no-malloc)
    void* t = malloc(size);
    TRACY_ALLOC(t, size);
    return t;
}

inline void tracy_free(void* mem)
{
    TRACY_FREE(mem);
    // NOLINTNEXTLINE(cppcoreguidelines-owning-memory, cppcoreguidelines-no-malloc)
    free(mem);
}