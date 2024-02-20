#pragma once

#include "barretenberg/common/wasm_export.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"

WASM_EXPORT void poseidon_hash(bb::fr::vec_in_buf inputs_buffer, bb::fr::out_buf output);
WASM_EXPORT void poseidon_hashes(bb::fr::vec_in_buf inputs_buffer, bb::fr::out_buf output);