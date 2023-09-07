#include "c_bind.h"

#include "call_stack_item.hpp"
#include "function_data.hpp"
#include "function_leaf_preimage.hpp"
#include "kernel_circuit_public_inputs.hpp"
#include "kernel_circuit_public_inputs_final.hpp"
#include "previous_kernel_data.hpp"
#include "private_circuit_public_inputs.hpp"
#include "tx_context.hpp"
#include "tx_request.hpp"
#include "private_kernel/private_kernel_inputs_inner.hpp"
#include "public_kernel/public_kernel_inputs.hpp"
#include "rollup/base/base_or_merge_rollup_public_inputs.hpp"
#include "rollup/base/base_rollup_inputs.hpp"
#include "rollup/root/root_rollup_inputs.hpp"
#include "rollup/root/root_rollup_public_inputs.hpp"

#include "aztec3/circuits/abis/combined_accumulated_data.hpp"
#include "aztec3/circuits/abis/final_accumulated_data.hpp"
#include "aztec3/circuits/abis/new_contract_data.hpp"
#include "aztec3/circuits/abis/packers.hpp"
#include "aztec3/circuits/abis/point.hpp"
#include "aztec3/circuits/abis/private_kernel/private_kernel_inputs_init.hpp"
#include "aztec3/circuits/abis/tx_request.hpp"
#include "aztec3/circuits/abis/types.hpp"
#include "aztec3/circuits/hash.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace {

using aztec3::circuits::compute_constructor_hash;
using aztec3::circuits::compute_contract_address;
using aztec3::circuits::compute_partial_address;
using aztec3::circuits::abis::CallStackItem;
using aztec3::circuits::abis::ConstantsPacker;
using aztec3::circuits::abis::FunctionData;
using aztec3::circuits::abis::FunctionLeafPreimage;
using aztec3::circuits::abis::GeneratorIndexPacker;
using aztec3::circuits::abis::NewContractData;
using aztec3::circuits::abis::Point;
using aztec3::circuits::abis::PrivateStateNoteGeneratorIndexPacker;
using aztec3::circuits::abis::PrivateStateTypePacker;
using aztec3::circuits::abis::StorageSlotGeneratorIndexPacker;
using aztec3::circuits::abis::TxContext;
using aztec3::circuits::abis::TxRequest;
using NT = aztec3::utils::types::NativeTypes;
using aztec3::circuits::abis::PrivateTypes;
using aztec3::circuits::abis::PublicTypes;

// Cbind helper functions

/**
 * @brief Fill in zero-leaves to get a full tree's bottom layer.
 *
 * @details Given the a vector of nonzero leaves starting at the left,
 * append zero leaves to that list until it represents a FULL set of leaves
 * for a tree of the given height.
 * **MODIFIES THE INPUT `leaves` REFERENCE!**
 *
 * @tparam TREE_HEIGHT height of the tree used to determine max leaves
 * @param leaves the nonzero leaves of the tree starting at the left
 * @param zero_leaf the leaf value to be used for any empty/unset leaves
 */
template <size_t TREE_HEIGHT> void rightfill_with_zeroleaves(std::vector<NT::fr>& leaves, NT::fr& zero_leaf)
{
    constexpr size_t max_leaves = 2 << (TREE_HEIGHT - 1);
    // input cant exceed max leaves
    // FIXME don't think asserts will show in wasm
    ASSERT(leaves.size() <= max_leaves);

    // fill in input vector with zero-leaves
    // to get a full bottom layer of the tree
    leaves.insert(leaves.end(), max_leaves - leaves.size(), zero_leaf);
}

}  // namespace

/** Copy this string to a bbmalloc'd buffer */
static const char* bbmalloc_copy_string(const char* data, size_t len)
{
    char* output_copy = static_cast<char*>(bbmalloc(len + 1));
    memcpy(output_copy, data, len + 1);
    return output_copy;
}

/**
 * For testing only. Take this object, write it to a buffer, then output it. */
template <typename T> static const char* as_string_output(uint8_t const* input_buf, uint32_t* size)
{
    using serialize::read;
    T obj;
    read(input_buf, obj);
    std::ostringstream stream;
    stream << obj;
    std::string const str = stream.str();
    *size = static_cast<uint32_t>(str.size());
    return bbmalloc_copy_string(str.c_str(), *size);
}

/**
 * For testing only. Take this object, serialize it to a buffer, then output it. */
template <typename T> static const char* as_serialized_output(uint8_t const* input_buf, uint32_t* size)
{
    using serialize::read;
    T obj;
    serialize::read(input_buf, obj);
    std::vector<uint8_t> stream;
    serialize::write(stream, obj);
    *size = static_cast<uint32_t>(stream.size());
    return bbmalloc_copy_string(reinterpret_cast<char*>(stream.data()), *size);
}

// WASM Cbinds
/**
 * @brief Hashes a TX request. This is a WASM-export that can be called from Typescript.
 *
 * @details given a `uint8_t*` buffer representing a full TX request,
 * read it into a `TxRequest` object, hash it to a `fr`,
 * and serialize it to a `uint8_t*` output buffer
 *
 * @param tx_request_buf buffer of bytes containing all data needed to construct a TX request via `serialize::read()`
 * @param output buffer that will contain the output which will be the hashed `TxRequest`
 */
WASM_EXPORT void abis__hash_tx_request(uint8_t const* tx_request_buf, uint8_t* output)
{
    TxRequest<NT> tx_request;
    serialize::read(tx_request_buf, tx_request);
    // TODO(dbanks12) consider using write() and serialize::read() instead of
    // serialize to/from everywhere here and in test
    NT::fr::serialize_to_buffer(tx_request.hash(), output);
}

/**
 * @brief Generates a function's "selector" from its "signature" using keccak256.
 * This is a WASM-export that can be called from Typescript.
 *
 * @details given a `char const*` c-string representing a "function signature",
 * hash using keccak and return its first 4 bytes (the "function selector")
 * by copying them into the `output` buffer arg. This is a workalike of
 * Ethereum/solidity's function selector computation....
 * Ethereum function selector is computed as follows:
 * `uint8_t* hash = keccak256(const char* func_sig);`
 * where func_sig does NOT include the trailing null character
 * And the resulting cstring for "transfer(address,uint256)" is:
 * `0xa9059cbb`
 * The 0th to 3rd bytes make up the function selector like:
 * where 0xa9 is hash[0], 05 is hash[1], 9c is hash[2], and bb is hash[3]
 *
 * @param func_sig_cstr c-string representing the function signature string like "transfer(uint256,address)"
 * @param output buffer that will contain the output which will be 4-byte function selector
 */
WASM_EXPORT void abis__compute_function_selector(char const* func_sig_cstr, uint8_t* output)
{
    // hash the function signature using keccak256
    auto keccak_hash = ethash_keccak256(reinterpret_cast<uint8_t const*>(func_sig_cstr), strlen(func_sig_cstr));
    // get a pointer to the start of the hash bytes
    auto const* hash_bytes = reinterpret_cast<uint8_t const*>(&keccak_hash.word64s[0]);
    // get the correct number of bytes from the hash and copy into output buffer
    std::copy_n(hash_bytes, aztec3::FUNCTION_SELECTOR_NUM_BYTES, output);
}

/**
 * @brief Hash/compress verification key data.
 * This is a WASM-export that can be called from Typescript.
 *
 * @details Pedersen compress VK to use later when computing function leaf
 * or constructor hash. Return the serialized results in the `output` buffer.
 *
 * @param vk_data_buf buffer of bytes representing serialized verification_key_data
 * @param output buffer that will contain the output. The serialized vk_hash.
 */
WASM_EXPORT void abis__hash_vk(uint8_t const* vk_data_buf, uint8_t* output)
{
    NT::VKData vk_data;
    serialize::read(vk_data_buf, vk_data);

    NT::fr::serialize_to_buffer(vk_data.compress_native(aztec3::GeneratorIndex::VK), output);
}

/**
 * @brief Generates a function tree leaf from its preimage.
 * This is a WASM-export that can be called from Typescript.
 *
 * @details given a `uint8_t const*` buffer representing a function leaf's preimage,
 * construct a FunctionLeafPreimage instance, hash, and return the serialized results
 * in the `output` buffer.
 *
 * @param function_leaf_preimage_buf a buffer of bytes representing the function leaf's preimage
 * contents (`function_selector`, `is_private`, `vk_hash`, and `acir_hash`)
 * @param output buffer that will contain the output. The hashed and serialized function leaf.
 */
WASM_EXPORT void abis__compute_function_leaf(uint8_t const* function_leaf_preimage_buf, uint8_t* output)
{
    FunctionLeafPreimage<NT> leaf_preimage;
    serialize::read(function_leaf_preimage_buf, leaf_preimage);
    leaf_preimage.hash();
    NT::fr::serialize_to_buffer(leaf_preimage.hash(), output);
}

/**
 * @brief Compute a function tree root from its nonzero leaves.
 * This is a WASM-export that can be called from Typescript.
 *
 * @details given a serialized vector of nonzero function leaves,
 * compute the corresponding tree's root and return the
 * serialized results via `root_out` buffer.
 *
 * @param function_leaves_in input buffer representing a serialized vector of
 * nonzero function leaves where each leaf is an `fr` starting at the left of the tree
 * @param root_out buffer that will contain the serialized function tree root `fr`.
 */
WASM_EXPORT void abis__compute_function_tree_root(uint8_t const* function_leaves_in, uint8_t* root_out)
{
    std::vector<NT::fr> leaves;
    // fill in nonzero leaves to start
    read(function_leaves_in, leaves);
    // fill in zero leaves to complete tree
    NT::fr zero_leaf = FunctionLeafPreimage<NT>().hash();  // hash of empty/0 preimage
    rightfill_with_zeroleaves<aztec3::FUNCTION_TREE_HEIGHT>(leaves, zero_leaf);

    // compute the root of this complete tree, return
    NT::fr const root = plonk::stdlib::merkle_tree::compute_tree_root_native(leaves);

    // serialize and return root
    NT::fr::serialize_to_buffer(root, root_out);
}

/**
 * @brief Compute all of a function tree's nodes from its nonzero leaves.
 * This is a WASM-export that can be called from Typescript.
 *
 * @details given a serialized vector of nonzero function leaves,
 * compute ALL of the corresponding tree's nodes (including root) and return
 * the serialized results via `tree_nodes_out` buffer.
 *
 * @param function_leaves_in input buffer representing a serialized vector of
 * nonzero function leaves where each leaf is an `fr` starting at the left of the tree.
 * @param tree_nodes_out buffer that will contain the serialized function tree.
 * The 0th node is the bottom leftmost leaf. The last entry is the root.
 */
WASM_EXPORT void abis__compute_function_tree(uint8_t const* function_leaves_in, uint8_t* tree_nodes_out)
{
    std::vector<NT::fr> leaves;
    // fill in nonzero leaves to start
    read(function_leaves_in, leaves);
    // fill in zero leaves to complete tree
    NT::fr zero_leaf = FunctionLeafPreimage<NT>().hash();  // hash of empty/0 preimage
    rightfill_with_zeroleaves<aztec3::FUNCTION_TREE_HEIGHT>(leaves, zero_leaf);

    std::vector<NT::fr> const tree = plonk::stdlib::merkle_tree::compute_tree_native(leaves);

    // serialize and return tree
    write(tree_nodes_out, tree);
}

/**
 * @brief Hash some constructor info.
 * This is a WASM-export that can be called from Typescript.
 *
 * @details Hash constructor info to use later when deriving/generating contract address:
 * hash(function_signature_hash, args_hash, constructor_vk_hash)
 * Return the serialized results in the `output` buffer.
 *
 * @param function_data_buf function data struct but as a buffer of bytes
 * @param args_buf constructor args (array of fields) but as a buffer of bytes
 * @param constructor_vk_hash_buf constructor vk hashed to a field but as a buffer of bytes
 * @param output buffer that will contain the output. The serialized constructor_vk_hash.
 */
WASM_EXPORT void abis__hash_constructor(uint8_t const* function_data_buf,
                                        uint8_t const* args_hash_buf,
                                        uint8_t const* constructor_vk_hash_buf,
                                        uint8_t* output)
{
    FunctionData<NT> function_data;
    NT::fr args_hash;
    NT::fr constructor_vk_hash;

    serialize::read(function_data_buf, function_data);
    read(args_hash_buf, args_hash);
    read(constructor_vk_hash_buf, constructor_vk_hash);

    NT::fr const constructor_hash = compute_constructor_hash(function_data, args_hash, constructor_vk_hash);

    NT::fr::serialize_to_buffer(constructor_hash, output);
}

/**
 * @brief Compute a contract address
 * This is a WASM-export that can be called from Typescript.
 *
 * @details Computes a contract address by hashing the deployers public key along with the previously computed partial
 * address Return the serialized results in the `output` buffer.
 *
 * @param point_data_buf point data struct as a buffer of bytes
 * @param contract_address_salt_buf salt value for the contract address
 * @param function_tree_root_buf root value of the contract's function tree
 * @param constructor_hash_buf the hash of the contract constructor's verification key
 * @param output buffer that will contain the output. The serialized contract address.
 */
WASM_EXPORT void abis__compute_contract_address(uint8_t const* point_data_buf,
                                                uint8_t const* contract_address_salt_buf,
                                                uint8_t const* function_tree_root_buf,
                                                uint8_t const* constructor_hash_buf,
                                                uint8_t* output)
{
    Point<NT> deployer_public_key;
    NT::fr contract_address_salt;
    NT::fr function_tree_root;
    NT::fr constructor_hash;

    serialize::read(point_data_buf, deployer_public_key);
    read(contract_address_salt_buf, contract_address_salt);
    read(function_tree_root_buf, function_tree_root);
    read(constructor_hash_buf, constructor_hash);

    NT::fr const contract_address =
        compute_contract_address(deployer_public_key, contract_address_salt, function_tree_root, constructor_hash);

    NT::fr::serialize_to_buffer(contract_address, output);
}

/**
 * @brief Compute a contract address from deployer public key and partial address.
 * This is a WASM-export that can be called from Typescript.
 *
 * @details Computes a contract address by hashing the deployers public key along with the previously computed partial
 * address Return the serialized results in the `output` buffer.
 *
 * @param point_data_buf point data struct as a buffer of bytes
 * @param partial_address_data_buf partial address
 * @param output buffer that will contain the output. The serialized contract address.
 */
WASM_EXPORT void abis__compute_contract_address_from_partial(uint8_t const* point_data_buf,
                                                             uint8_t const* partial_address_data_buf,
                                                             uint8_t* output)
{
    Point<NT> deployer_public_key;
    NT::fr partial_address;

    serialize::read(point_data_buf, deployer_public_key);
    read(partial_address_data_buf, partial_address);

    NT::fr const contract_address =
        aztec3::circuits::compute_contract_address_from_partial(deployer_public_key, partial_address);

    NT::fr::serialize_to_buffer(contract_address, output);
}

/**
 * @brief Compute a partial address
 * This is a WASM-export that can be called from Typescript.
 *
 * @details Computes a partial address by hashing the salt, function tree root and constructor hash
 * Return the serialized results in the `output` buffer.
 *
 * @param contract_address_salt_buf salt value for the contract address
 * @param function_tree_root_buf root value of the contract's function tree
 * @param constructor_hash_buf the hash of the contract constructor's verification key
 * @param output buffer that will contain the output. The serialized contract address.
 * See the link bellow for more details:
 * https://github.com/AztecProtocol/aztec-packages/blob/master/docs/docs/concepts/foundation/accounts/keys.md#addresses-partial-addresses-and-public-keys
 */
WASM_EXPORT void abis__compute_partial_address(uint8_t const* contract_address_salt_buf,
                                               uint8_t const* function_tree_root_buf,
                                               uint8_t const* constructor_hash_buf,
                                               uint8_t* output)
{
    NT::fr contract_address_salt;
    NT::fr function_tree_root;
    NT::fr constructor_hash;

    read(contract_address_salt_buf, contract_address_salt);
    read(function_tree_root_buf, function_tree_root);
    read(constructor_hash_buf, constructor_hash);
    NT::fr const partial_address =
        compute_partial_address<NT>(contract_address_salt, function_tree_root, constructor_hash);

    NT::fr::serialize_to_buffer(partial_address, output);
}

/**
 * @brief Hash args for a function call.
 *
 * @param args_buf array of args (fields), with the length on the first position
 * @param output buffer that will contain the output
 */
WASM_EXPORT void abis__compute_var_args_hash(uint8_t const* args_buf, uint8_t* output)
{
    std::vector<NT::fr> args;
    read(args_buf, args);
    NT::fr const args_hash = aztec3::circuits::compute_var_args_hash<NT>(args);
    NT::fr::serialize_to_buffer(args_hash, output);
}

/**
 * @brief Generates a function tree leaf from its preimage.
 * This is a WASM-export that can be called from Typescript.
 *
 * @details given a `uint8_t const*` buffer representing a function leaf's prieimage,
 * construct a NewContractData instance, hash, and return the serialized results
 * in the `output` buffer.
 *
 * @param contract_leaf_preimage_buf a buffer of bytes representing the contract leaf's preimage
 * contents (`contract_address`, `portal_contract_address`, `function_tree_root`)
 * @param output buffer that will contain the output. The hashed and serialized contract leaf.
 */
WASM_EXPORT void abis__compute_contract_leaf(uint8_t const* contract_leaf_preimage_buf, uint8_t* output)
{
    NewContractData<NT> leaf_preimage;
    serialize::read(contract_leaf_preimage_buf, leaf_preimage);
    // as per the circuit implementation, if contract address == zero then return a zero leaf
    auto to_write = leaf_preimage.hash();
    NT::fr::serialize_to_buffer(to_write, output);
}

/**
 * @brief Generates a commitment nonce, which will be used to create a unique commitment.
 */
CBIND(abis__compute_commitment_nonce, aztec3::circuits::compute_commitment_nonce<NT>);

/**
 * @brief Generates a unique commitment using a commitment nonce.
 */
CBIND(abis__compute_unique_commitment, aztec3::circuits::compute_unique_commitment<NT>);

/**
 * @brief Generates a siloed commitment tree leaf from the contract and the commitment.
 */
CBIND(abis__silo_commitment, aztec3::circuits::silo_commitment<NT>);

/**
 * @brief Generates a siloed nullifier from the contract and the nullifier.
 */
CBIND(abis__silo_nullifier, aztec3::circuits::silo_nullifier<NT>);

/**
 * @brief Computes the block hash from the block information.
 * Globals is provided as a hash in this instance.
 */
CBIND(abis__compute_block_hash, aztec3::circuits::compute_block_hash<NT>);

/**
 * @brief Computes the block hash from the block information.
 * The entire globals object is provided in this instance, rather than a hash as in above.
 */
CBIND(abis__compute_block_hash_with_globals, aztec3::circuits::compute_block_hash_with_globals<NT>);

/**
 * @brief Computes the hash of the global variables
 */
CBIND(abis__compute_globals_hash, aztec3::circuits::compute_globals_hash<NT>);

/**
 * @brief Compute the value to be inserted into the public data tree
 */
CBIND(abis__compute_public_data_tree_value, aztec3::circuits::compute_public_data_tree_value<NT>);

/**
 * @brief Compute the index for inserting a value into the public data tree
 */
CBIND(abis__compute_public_data_tree_index, aztec3::circuits::compute_public_data_tree_index<NT>);

/**
 * @brief Generates a signed tx request hash from it's pre-image
 * This is a WASM-export that can be called from Typescript.
 *
 * @details given a `uint8_t const*` buffer representing a signed tx request's pre-image,
 * construct a TxRequest instance, hash, and return the serialized results
 * in the `output` buffer.
 *
 * @param tx_request_buf a buffer of bytes representing the signed tx request
 * @param output buffer that will contain the output. The hashed and serialized signed tx request.
 */
WASM_EXPORT void abis__compute_transaction_hash(uint8_t const* tx_request_buf, uint8_t* output)
{
    TxRequest<NT> tx_request_preimage;
    serialize::read(tx_request_buf, tx_request_preimage);
    auto to_write = tx_request_preimage.hash();
    NT::fr::serialize_to_buffer(to_write, output);
}

WASM_EXPORT void abis__compute_private_call_stack_item_hash(uint8_t const* call_stack_item_buf, uint8_t* output)
{
    CallStackItem<NT, PrivateTypes> call_stack_item;
    serialize::read(call_stack_item_buf, call_stack_item);
    NT::fr::serialize_to_buffer(call_stack_item.hash(), output);
}

WASM_EXPORT void abis__compute_public_call_stack_item_hash(uint8_t const* call_stack_item_buf, uint8_t* output)
{
    CallStackItem<NT, PublicTypes> call_stack_item;
    serialize::read(call_stack_item_buf, call_stack_item);
    NT::fr::serialize_to_buffer(get_call_stack_item_hash(call_stack_item), output);
}

/**
 * @brief Computes the hash of a message secret for use in l1 -> l2 messaging
 *
 * @param secret
 * @param output
 */
WASM_EXPORT void abis__compute_message_secret_hash(uint8_t const* secret, uint8_t* output)
{
    NT::fr message_secret;
    read(secret, message_secret);
    auto secret_hash = NT::hash({ message_secret }, aztec3::GeneratorIndex::L1_TO_L2_MESSAGE_SECRET);
    NT::fr::serialize_to_buffer(secret_hash, output);
}

/* Typescript test helpers that call as_string_output() to stress serialization.
 * Each of these take an object buffer, and a string size pointer.
 * They return a string pointer (to be bbfree'd) and write to the string size pointer. */
WASM_EXPORT const char* abis__test_roundtrip_serialize_tx_context(uint8_t const* tx_context_buf, uint32_t* size)
{
    return as_string_output<TxContext<NT>>(tx_context_buf, size);
}

WASM_EXPORT const char* abis__test_roundtrip_serialize_tx_request(uint8_t const* tx_request_buf, uint32_t* size)
{
    return as_string_output<TxRequest<NT>>(tx_request_buf, size);
}

WASM_EXPORT const char* abis__test_roundtrip_serialize_call_context(uint8_t const* call_context_buf, uint32_t* size)
{
    return as_string_output<aztec3::circuits::abis::CallContext<NT>>(call_context_buf, size);
}

WASM_EXPORT const char* abis__test_roundtrip_serialize_private_circuit_public_inputs(
    uint8_t const* private_circuits_public_inputs_buf, uint32_t* size)
{
    return as_string_output<aztec3::circuits::abis::PrivateCircuitPublicInputs<NT>>(private_circuits_public_inputs_buf,
                                                                                    size);
}

WASM_EXPORT const char* abis__test_roundtrip_serialize_public_circuit_public_inputs(
    uint8_t const* public_circuits_public_inputs_buf, uint32_t* size)
{
    return as_string_output<aztec3::circuits::abis::PublicCircuitPublicInputs<NT>>(public_circuits_public_inputs_buf,
                                                                                   size);
}

WASM_EXPORT const char* abis__test_roundtrip_serialize_function_data(uint8_t const* function_data_buf, uint32_t* size)
{
    return as_string_output<aztec3::circuits::abis::FunctionData<NT>>(function_data_buf, size);
}

WASM_EXPORT const char* abis__test_roundtrip_serialize_base_rollup_inputs(uint8_t const* rollup_inputs_buf,
                                                                          uint32_t* size)
{
    return as_string_output<aztec3::circuits::abis::BaseRollupInputs<NT>>(rollup_inputs_buf, size);
}

WASM_EXPORT const char* abis__test_roundtrip_serialize_previous_kernel_data(uint8_t const* kernel_data_buf,
                                                                            uint32_t* size)
{
    return as_string_output<aztec3::circuits::abis::PreviousKernelData<NT>>(kernel_data_buf, size);
}

WASM_EXPORT const char* abis__test_roundtrip_serialize_base_or_merge_rollup_public_inputs(
    uint8_t const* rollup_inputs_buf, uint32_t* size)
{
    return as_string_output<aztec3::circuits::abis::BaseOrMergeRollupPublicInputs<NT>>(rollup_inputs_buf, size);
}

WASM_EXPORT const char* abis__test_roundtrip_reserialize_base_or_merge_rollup_public_inputs(
    uint8_t const* rollup_inputs_buf, uint32_t* size)
{
    return as_serialized_output<aztec3::circuits::abis::BaseOrMergeRollupPublicInputs<NT>>(rollup_inputs_buf, size);
}

WASM_EXPORT const char* abis__test_roundtrip_serialize_root_rollup_inputs(uint8_t const* rollup_inputs_buf,
                                                                          uint32_t* size)
{
    return as_string_output<aztec3::circuits::abis::RootRollupInputs<NT>>(rollup_inputs_buf, size);
}

WASM_EXPORT const char* abis__test_roundtrip_serialize_root_rollup_public_inputs(uint8_t const* rollup_inputs_buf,
                                                                                 uint32_t* size)
{
    return as_string_output<aztec3::circuits::abis::RootRollupPublicInputs<NT>>(rollup_inputs_buf, size);
}

WASM_EXPORT const char* abis__test_roundtrip_reserialize_root_rollup_public_inputs(uint8_t const* rollup_inputs_buf,
                                                                                   uint32_t* size)
{
    return as_serialized_output<aztec3::circuits::abis::RootRollupPublicInputs<NT>>(rollup_inputs_buf, size);
}

WASM_EXPORT const char* abis__test_roundtrip_serialize_combined_accumulated_data(uint8_t const* input, uint32_t* size)
{
    return as_string_output<aztec3::circuits::abis::CombinedAccumulatedData<NT>>(input, size);
}

WASM_EXPORT const char* abis__test_roundtrip_serialize_final_accumulated_data(uint8_t const* input, uint32_t* size)
{
    return as_string_output<aztec3::circuits::abis::FinalAccumulatedData<NT>>(input, size);
}

WASM_EXPORT const char* abis__test_roundtrip_serialize_signature(uint8_t const* input, uint32_t* size)
{
    return as_string_output<NT::schnorr_signature>(input, size);
}

WASM_EXPORT const char* abis__test_roundtrip_serialize_private_kernel_inputs_inner(uint8_t const* input, uint32_t* size)
{
    return as_string_output<aztec3::circuits::abis::private_kernel::PrivateKernelInputsInner<NT>>(input, size);
}

WASM_EXPORT const char* abis__test_roundtrip_serialize_private_kernel_inputs_init(uint8_t const* input, uint32_t* size)
{
    return as_string_output<aztec3::circuits::abis::private_kernel::PrivateKernelInputsInit<NT>>(input, size);
}

WASM_EXPORT const char* abis__test_roundtrip_serialize_kernel_circuit_public_inputs(uint8_t const* input,
                                                                                    uint32_t* size)
{
    return as_string_output<aztec3::circuits::abis::KernelCircuitPublicInputs<NT>>(input, size);
}

WASM_EXPORT const char* abis__test_roundtrip_serialize_kernel_circuit_public_inputs_final(uint8_t const* input,
                                                                                          uint32_t* size)
{
    return as_string_output<aztec3::circuits::abis::KernelCircuitPublicInputsFinal<NT>>(input, size);
}

WASM_EXPORT const char* abis__test_roundtrip_serialize_public_kernel_inputs(uint8_t const* input, uint32_t* size)
{
    return as_string_output<aztec3::circuits::abis::public_kernel::PublicKernelInputs<NT>>(input, size);
}

WASM_EXPORT const char* abis__test_roundtrip_serialize_function_leaf_preimage(uint8_t const* function_leaf_preimage_buf,
                                                                              uint32_t* size)
{
    return as_string_output<aztec3::circuits::abis::FunctionLeafPreimage<NT>>(function_leaf_preimage_buf, size);
}


// When we return a packer from packers.hpp, we call its msgpack_pack method (as that is what is used
// internally in msgpack) and thus can get a JSON-like object of all our constants in Typescript. We explicitly do not
// want a schema here as our ConstantsPacker is not meant to be used in a Typescript function. (if it were, it would
// need to implement msgpack_schema, but as we handle it specially not much value).
CBIND_NOSCHEMA(get_circuit_constants, [] { return ConstantsPacker{}; });
CBIND_NOSCHEMA(get_circuit_generator_index, [] { return GeneratorIndexPacker{}; });
CBIND_NOSCHEMA(get_circuit_private_state_note_generator_index, [] { return PrivateStateNoteGeneratorIndexPacker{}; });
CBIND_NOSCHEMA(get_circuit_storage_slot_generator_index, [] { return StorageSlotGeneratorIndexPacker{}; });
CBIND_NOSCHEMA(get_circuit_private_state_type, [] { return PrivateStateTypePacker{}; });
