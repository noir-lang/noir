#pragma once
#ifdef __clang__
#define WASM_EXPORT extern "C" __attribute__((visibility("default"))) __attribute__((annotate("wasm_export")))
#define ASYNC_WASM_EXPORT                                                                                              \
    extern "C" __attribute__((visibility("default"))) __attribute__((annotate("async_wasm_export")))
#else
#define WASM_EXPORT extern "C" __attribute__((visibility("default")))
#define ASYNC_WASM_EXPORT extern "C" __attribute__((visibility("default")))
#endif

#ifdef __wasm__
// Allow linker to not link this
#define WASM_IMPORT(name) extern "C" __attribute__((import_module("env"), import_name(name)))
#else
#define WASM_IMPORT(name) extern "C"
#endif