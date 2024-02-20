#pragma once

#include "barretenberg/common/wasm_export.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"

WASM_EXPORT void pedersen_hash(bb::fr::vec_in_buf inputs_buffer, uint32_t const* hash_index, bb::fr::out_buf output);
WASM_EXPORT void pedersen_hashes(bb::fr::vec_in_buf inputs_buffer, uint32_t const* hash_index, bb::fr::out_buf output);
WASM_EXPORT void pedersen_hash_buffer(uint8_t const* input_buffer, uint32_t const* hash_index, bb::fr::out_buf output);