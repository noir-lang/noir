#include "../bn254.hpp"
#include "barretenberg/ecc/scalar_multiplication/scalar_multiplication.hpp"
#include "barretenberg/ecc/scalar_multiplication/pippenger.hpp"
#include "barretenberg/common/mem.hpp"

using namespace barretenberg;

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void* bbmalloc(size_t size)
{
    auto ptr = aligned_alloc(64, size);
    return ptr;
}

WASM_EXPORT void bbfree(void* ptr)
{
    aligned_free(ptr);
}

WASM_EXPORT void* new_pippenger(uint8_t* points, size_t num_points)
{
    auto ptr = new scalar_multiplication::Pippenger<curve::BN254>(points, num_points);
    return ptr;
}

WASM_EXPORT void delete_pippenger(void* pippenger)
{
    delete reinterpret_cast<scalar_multiplication::Pippenger<curve::BN254>*>(pippenger);
}

WASM_EXPORT void pippenger_unsafe(void* pippenger_ptr, void* scalars_ptr, size_t from, size_t range, void* result_ptr)
{
    scalar_multiplication::pippenger_runtime_state<curve::BN254> state(range);
    auto pippenger = reinterpret_cast<scalar_multiplication::Pippenger<curve::BN254>*>(pippenger_ptr);
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
