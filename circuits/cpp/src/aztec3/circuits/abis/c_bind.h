#include <cstdint>

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void abis__hash_tx_request(uint8_t const* tx_request_buf, uint8_t* output);

WASM_EXPORT void abis__compute_function_selector(char const* func_sig_cstr, uint8_t* output);

WASM_EXPORT void abis__compute_function_leaf(uint8_t const* function_leaf_preimage_buf, uint8_t* output);

WASM_EXPORT void abis__compute_function_tree_root(uint8_t const* function_leaves_buf,
                                                  uint8_t* output);

WASM_EXPORT void abis__compute_function_tree(uint8_t const* function_leaves_buf,
                                             uint8_t* output);

WASM_EXPORT void abis__hash_vk(uint8_t const* vk_data_buf, uint8_t* output);

WASM_EXPORT void abis__hash_constructor(uint8_t const* func_data_buf,
                                        uint8_t const* args_buf,
                                        uint8_t const* constructor_vk_hash_buf,
                                        uint8_t* output);

WASM_EXPORT void abis__compute_contract_address(uint8_t const* deployer_address_buf,
                                                uint8_t const* contract_address_salt_buf,
                                                uint8_t const* function_tree_root_buf,
                                                uint8_t const* constructor_hash_buf,
                                                uint8_t* output);

WASM_EXPORT void abis__compute_contract_leaf(uint8_t const* contract_leaf_preimage_buf, uint8_t* output);

WASM_EXPORT void abis__compute_call_stack_item_hash(uint8_t const* call_stack_item_buf, uint8_t* output);

}
