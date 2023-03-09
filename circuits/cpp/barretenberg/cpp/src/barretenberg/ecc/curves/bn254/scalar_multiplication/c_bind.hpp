#include <cstddef>
#include "../g1.hpp"

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void* bbmalloc(size_t size);

WASM_EXPORT void bbfree(void* ptr);

WASM_EXPORT void* new_pippenger(uint8_t* points, size_t num_points);

WASM_EXPORT void delete_pippenger(void* pippenger);

WASM_EXPORT void pippenger_unsafe(void* pippenger_ptr, void* scalars_ptr, size_t from, size_t range, void* result_ptr);
WASM_EXPORT void g1_sum(void* points_ptr, size_t num_points, void* result_ptr);
}
