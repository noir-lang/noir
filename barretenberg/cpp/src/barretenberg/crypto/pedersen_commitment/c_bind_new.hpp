#pragma once
#include "barretenberg/common/wasm_export.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"

extern "C" {

using namespace barretenberg;

WASM_EXPORT void pedersen___init();

WASM_EXPORT void pedersen___commit(fr::vec_in_buf inputs_buffer, fr::out_buf output);
}