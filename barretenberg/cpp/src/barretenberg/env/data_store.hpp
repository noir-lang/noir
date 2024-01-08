// To be provided by the environment.
// For a WASM build, this is provided by the JavaScript environment.
// For a native build, this is provided in this module.
#include "barretenberg/common/wasm_export.hpp"
#include <cstddef>
#include <cstdint>

// Takes a copy of buf and saves it associated with key.
WASM_IMPORT("set_data") void set_data(char const* key, uint8_t const* buf, size_t length);

// Copies bytes of data associated with key into out_buf.
WASM_IMPORT("get_data") void get_data(char const* key, uint8_t* out_buf);
