#pragma once
#include "barretenberg/common/wasm_export.hpp"
#include <cstdint>

WASM_IMPORT("env_hardware_concurrency") uint32_t env_hardware_concurrency();