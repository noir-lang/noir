#include "aztec3/circuits/hash.hpp"

#include <barretenberg/barretenberg.hpp>

#include <algorithm>
#include <cstddef>
#include <cstdint>

WASM_EXPORT void abis__hash_tx_request(uint8_t const* tx_request_buf, uint8_t* output);

WASM_EXPORT void abis__compute_function_selector(char const* func_sig_cstr, uint8_t* output);

WASM_EXPORT void abis__compute_function_leaf(uint8_t const* function_leaf_preimage_buf, uint8_t* output);

WASM_EXPORT void abis__compute_function_tree_root(uint8_t const* function_leaves_buf, uint8_t* output);

WASM_EXPORT void abis__compute_function_tree(uint8_t const* function_leaves_buf, uint8_t* output);

WASM_EXPORT void abis__hash_vk(uint8_t const* vk_data_buf, uint8_t* output);

WASM_EXPORT void abis__hash_constructor(uint8_t const* func_data_buf,
                                        uint8_t const* args_buf,
                                        uint8_t const* constructor_vk_hash_buf,
                                        uint8_t* output);

WASM_EXPORT void abis__compute_contract_address(uint8_t const* point_data_buf,
                                                uint8_t const* contract_address_salt_buf,
                                                uint8_t const* function_tree_root_buf,
                                                uint8_t const* constructor_hash_buf,
                                                uint8_t* output);

WASM_EXPORT void abis__compute_partial_address(uint8_t const* contract_address_salt_buf,
                                               uint8_t const* function_tree_root_buf,
                                               uint8_t const* constructor_hash_buf,
                                               uint8_t* output);

CBIND_DECL(abis__compute_commitment_nonce);
CBIND_DECL(abis__compute_unique_commitment);
CBIND_DECL(abis__silo_commitment);
CBIND_DECL(abis__silo_nullifier);
CBIND_DECL(abis__compute_block_hash);
CBIND_DECL(abis__compute_block_hash_with_globals);
CBIND_DECL(abis__compute_globals_hash);

WASM_EXPORT void abis__compute_message_secret_hash(uint8_t const* secret, uint8_t* output);
WASM_EXPORT void abis__compute_contract_leaf(uint8_t const* contract_leaf_preimage_buf, uint8_t* output);
WASM_EXPORT void abis__compute_transaction_hash(uint8_t const* tx_request_buf, uint8_t* output);
WASM_EXPORT void abis__compute_call_stack_item_hash(uint8_t const* call_stack_item_buf, uint8_t* output);
WASM_EXPORT void abis__compute_var_args_hash(uint8_t const* args_buf, uint8_t* output);