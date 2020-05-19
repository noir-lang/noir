#include "c_bind.hpp"
#include "./scalar_multiplication.hpp"
#include "pippenger.hpp"
#include <common/mem.hpp>
#include <srs/io.hpp>

using namespace barretenberg;

extern "C" {

WASM_EXPORT void* bbmalloc(size_t size)
{
    return aligned_alloc(64, size);
}

WASM_EXPORT void bbfree(void* ptr)
{
    aligned_free(ptr);
}

WASM_EXPORT void* new_pippenger(uint8_t* points, size_t num_points)
{
    return new scalar_multiplication::Pippenger(points, num_points);
}

WASM_EXPORT void delete_pippenger(void* pippenger)
{
    delete reinterpret_cast<scalar_multiplication::Pippenger*>(pippenger);
}

WASM_EXPORT void pippenger_unsafe(void* pippenger_ptr, void* scalars_ptr, size_t from, size_t range, void* result_ptr)
{
    scalar_multiplication::pippenger_runtime_state state(range);
    auto pippenger = reinterpret_cast<scalar_multiplication::Pippenger*>(pippenger_ptr);
    auto scalars = reinterpret_cast<fr*>(scalars_ptr);
    auto result = reinterpret_cast<g1::element*>(result_ptr);
    *result = pippenger->pippenger_unsafe(scalars, from, range);
}

WASM_EXPORT void g1_sum(void* points_ptr, const size_t num_points, void* result_ptr)
{
    auto points = reinterpret_cast<g1::element*>(points_ptr);
    auto result = reinterpret_cast<g1::element*>(result_ptr);
    result->self_set_infinity();
    *result = std::accumulate(points, points + num_points, *result);
}
}