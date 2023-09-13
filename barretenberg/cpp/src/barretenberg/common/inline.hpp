#pragma once

#ifdef _WIN32
#define BBERG_INLINE __forceinline inline
#else
#define BBERG_INLINE __attribute__((always_inline)) inline
#endif
