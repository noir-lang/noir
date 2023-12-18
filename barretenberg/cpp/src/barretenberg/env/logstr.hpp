// To be provided by the environment.
// For a WASM build, this is provided by the JavaScript environment.
// For a native build, this is provided in this module.
#include "barretenberg/common/wasm_export.hpp"

WASM_IMPORT("logstr") void logstr(char const*);
