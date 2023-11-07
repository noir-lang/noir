#pragma once
#include "./wasm_export.hpp"
#include <cstddef>

WASM_EXPORT void* bbmalloc(size_t size);

WASM_EXPORT void bbfree(void* ptr);
