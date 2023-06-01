#pragma once

#include <barretenberg/barretenberg.hpp>

#include <cstddef>
#include <cstdint>

extern "C" {

WASM_EXPORT uint8_t* merge_rollup__sim(uint8_t const* merge_rollup_inputs_buf,
                                       size_t* merge_rollup_public_inputs_size_out,
                                       uint8_t const** merge_rollup_public_inputs_buf);
}