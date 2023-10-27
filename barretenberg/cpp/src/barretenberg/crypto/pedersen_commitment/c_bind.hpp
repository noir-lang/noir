#pragma once
#include "barretenberg/common/mem.hpp"
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/common/streams.hpp"
#include "barretenberg/common/timer.hpp"

WASM_EXPORT void pedersen__commit(uint8_t const* inputs_buffer, uint8_t* output);
