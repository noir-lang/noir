#include "./bbmalloc.hpp"
#include "./slab_allocator.hpp"

WASM_EXPORT void* bbmalloc(size_t size)
{
    return bb::get_mem_slab_raw(size);
}

WASM_EXPORT void bbfree(void* ptr)
{
    bb::free_mem_slab_raw(ptr);
}
