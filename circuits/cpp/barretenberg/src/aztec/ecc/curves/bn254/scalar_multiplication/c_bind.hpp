#pragma once
#include "../g1.hpp"

using namespace barretenberg;

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void* bbmalloc(size_t size);

WASM_EXPORT void bbfree(void* ptr);

WASM_EXPORT g1::affine_element* create_pippenger_point_table(uint8_t* points, size_t num_points);

WASM_EXPORT void pippenger_unsafe(
    fr* scalars, size_t from, size_t range, g1::affine_element* point_table, g1::element* result);

WASM_EXPORT void g1_sum(g1::element* points, const size_t num_points, g1::element* result);
}