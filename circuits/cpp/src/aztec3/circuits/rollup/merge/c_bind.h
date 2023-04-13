#include <cstdint>
#include <cstddef>

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT size_t merge_rollup__sim(uint8_t const* merge_rollup_inputs_buf,
                                    uint8_t const** merge_rollup_public_inputs_buf);
}