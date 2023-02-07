#include <cstdint>

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT uint32_t join_split__get_new_proving_key_data(uint8_t** output);

}
