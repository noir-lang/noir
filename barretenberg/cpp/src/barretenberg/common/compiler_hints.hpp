#pragma once

#ifdef _WIN32
#define BBERG_INLINE __forceinline inline
#else
#define BBERG_INLINE __attribute__((always_inline)) inline
#endif

// TODO(AD): Other compilers
#if defined(__clang__)
#define BBERG_INSTRUMENT [[clang::xray_always_instrument]]
#define BBERG_NO_INSTRUMENT [[clang::xray_never_instrument]]
#define BBERG_NOINLINE [[clang::noinline]]
#else
#define BBERG_INSTRUMENT
#define BBERG_NO_INSTRUMENT
#define BBERG_NOINLINE
#endif