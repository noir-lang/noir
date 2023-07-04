#include "./serialize.hpp"
#include "./wasm_export.hpp"
#include <cstddef>

WASM_EXPORT void test_threads(uint32_t const* threads, uint32_t const* iterations, uint32_t* out);

WASM_EXPORT void common_init_slab_allocator(uint32_t const* circuit_size);