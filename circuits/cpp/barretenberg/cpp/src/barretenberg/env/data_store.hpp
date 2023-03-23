#include <stddef.h>

// To be provided by the environment.
// For a WASM build, this is provided by the JavaScript environment.
// For a native build, this is provided in this module.
extern "C" void* get_data(char const* key, size_t* length_out);
extern "C" void set_data(char const* key, void* addr, size_t length);
