#include <stddef.h>

// To be provided by the environment.
// For barretenberg.wasm, this is provided by the JavaScript environment.
// For anything other than barretenberg.wasm, this is provided in this module.
extern "C" void* get_data(char const* key, size_t* length_out);
extern "C" void set_data(char const* key, void* addr, size_t length);