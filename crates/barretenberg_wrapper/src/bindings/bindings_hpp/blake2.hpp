#include <stdint.h>
#include <stddef.h>
//#include <vector>
#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void blake2s_to_field(uint8_t const* data, size_t length, uint8_t* r);

}