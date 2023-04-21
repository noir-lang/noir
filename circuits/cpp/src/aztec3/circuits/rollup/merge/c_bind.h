#include <cstdint>
#include <cstddef>

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT uint8_t* merge_rollup__sim(uint8_t const* merge_rollup_inputs_buf,
                                       size_t* merge_rollup_public_inputs_size_out,
                                       uint8_t const** merge_rollup_public_inputs_buf);
}