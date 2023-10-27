#pragma once

#include "barretenberg/common/wasm_export.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"

extern "C" {

using namespace barretenberg;

WASM_EXPORT void pedersen_hash_with_hash_index(fr::vec_in_buf inputs_buffer,
                                               uint32_t const* hash_index,
                                               fr::out_buf output);
}