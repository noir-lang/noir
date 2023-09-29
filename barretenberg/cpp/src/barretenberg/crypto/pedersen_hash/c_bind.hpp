#pragma once
// TODO(@zac-wiliamson #2341 delete this file and rename c_bind_new to c_bind once we have migrated to new hash standard

#include "barretenberg/common/wasm_export.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"

extern "C" {

using namespace barretenberg;

WASM_EXPORT void pedersen_hash_init();

WASM_EXPORT void pedersen_hash_pair(fr::in_buf left, fr::in_buf right, fr::out_buf result);

WASM_EXPORT void pedersen_hash_multiple(fr::vec_in_buf inputs_buffer, fr::out_buf output);

WASM_EXPORT void pedersen_hash_multiple_with_hash_index(fr::vec_in_buf inputs_buffer,
                                                        uint32_t const* hash_index,
                                                        fr::out_buf output);

/**
 * Given a buffer containing 32 byte pedersen leaves, return a new buffer containing the leaves and all pairs of
 * nodes that define a merkle tree.
 * e.g.
 * input:  [1][2][3][4]
 * output: [1][2][3][4][compress(1,2)][compress(3,4)][compress(5,6)]
 */
WASM_EXPORT void pedersen_hash_to_tree(fr::vec_in_buf data, fr::vec_out_buf out);
}