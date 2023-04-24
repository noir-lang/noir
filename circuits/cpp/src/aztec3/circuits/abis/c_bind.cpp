#include "c_bind.h"
#include "barretenberg/srs/reference_string/mem_reference_string.hpp"
#include "aztec3/circuits/abis/function_data.hpp"
#include "aztec3/circuits/abis/function_leaf_preimage.hpp"
#include "aztec3/circuits/abis/new_contract_data.hpp"
#include "private_circuit_public_inputs.hpp"
#include "tx_request.hpp"
#include "tx_context.hpp"
#include "function_data.hpp"
#include "function_leaf_preimage.hpp"
#include "rollup/base/base_rollup_inputs.hpp"
#include "rollup/base/base_or_merge_rollup_public_inputs.hpp"
#include "rollup/root/root_rollup_public_inputs.hpp"
#include "rollup/root/root_rollup_inputs.hpp"
#include "previous_kernel_data.hpp"
#include "private_kernel/private_inputs.hpp"
#include "kernel_circuit_public_inputs.hpp"
#include "public_kernel/public_kernel_inputs.hpp"
#include "public_kernel/public_kernel_inputs_no_previous_kernel.hpp"

#include <aztec3/circuits/hash.hpp>
#include <aztec3/constants.hpp>

#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/array.hpp>
#include <barretenberg/stdlib/merkle_tree/membership.hpp>
#include <barretenberg/crypto/keccak/keccak.hpp>

namespace {

using aztec3::circuits::compute_constructor_hash;
using aztec3::circuits::compute_contract_address;
using aztec3::circuits::abis::FunctionData;
using aztec3::circuits::abis::FunctionLeafPreimage;
using aztec3::circuits::abis::NewContractData;
using aztec3::circuits::abis::TxContext;
using aztec3::circuits::abis::TxRequest;
using NT = aztec3::utils::types::NativeTypes;

// Cbind helper functions

/**
 * @brief Fill in zero-leaves to get a full tree's bottom layer.
 *
 * @details Given the a vector of nonzero leaves starting at the left,
 * append zeroleaves to that list until it represents a FULL set of leaves
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

} // namespace

// Note: We don't have a simple way of calling the barretenberg c-bind.
// Mimick bbmalloc behaviour.
static void* bbmalloc(size_t size)
{
    auto ptr = aligned_alloc(64, size);
    return ptr;
}

/** Copy this string to a bbmalloc'd buffer */
static const char* bbmalloc_copy_string(const char* data, size_t len)
{
    char* output_copy = (char*)bbmalloc(len + 1);
    memcpy(output_copy, data, len + 1);
    return output_copy;
}

/**
 * For testing only. Take this object, write it to a buffer, then output it. */
template <typename T> static const char* as_string_output(uint8_t const* input_buf, uint32_t* size)
{
    T obj;
    read(input_buf, obj);
    std::ostringstream stream;
    stream << obj;
    std::string str = stream.str();
    *size = (uint32_t)str.size();
    return bbmalloc_copy_string(str.c_str(), *size);
}

/**
 * For testing only. Take this object, serialize it to a buffer, then output it. */
template <typename T> static const char* as_serialized_output(uint8_t const* input_buf, uint32_t* size)
{
    T obj;
    read(input_buf, obj);
    std::vector<uint8_t> stream;
    write(stream, obj);
    *size = (uint32_t)stream.size();
    return bbmalloc_copy_string((char*)stream.data(), *size);
}

#define WASM_EXPORT __attribute__((visibility("default")))
// WASM Cbinds
extern "C" {

/**
 * @brief Hashes a TX request. This is a WASM-export that can be called from Typescript.
 *
 * @details given a `uint8_t*` buffer representing a full TX request,
 * read it into a `TxRequest` object, hash it to a `fr`,
 * and serialize it to a `uint8_t*` output buffer
 *
 * @param tx_request_buf buffer of bytes containing all data needed to construct a TX request via `read()`
 * @param output buffer that will contain the output which will be the hashed `TxRequest`
 */
WASM_EXPORT void abis__hash_tx_request(uint8_t const* tx_request_buf, uint8_t* output)
{
    TxRequest<NT> tx_request;
    read(tx_request_buf, tx_request);
    // TODO consider using write() and read() instead of
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
    uint8_t const* hash_bytes = reinterpret_cast<uint8_t const*>(&keccak_hash.word64s[0]);
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
    read(vk_data_buf, vk_data);

    NT::fr::serialize_to_buffer(vk_data.compress_native(aztec3::GeneratorIndex::VK), output);
}

/**
 * @brief Generates a function tree leaf from its preimage.
 * This is a WASM-export that can be called from Typescript.
 *
 * @details given a `uint8_t const*` buffer representing a function leaf's prieimage,
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
    read(function_leaf_preimage_buf, leaf_preimage);
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
    NT::fr zero_leaf = FunctionLeafPreimage<NT>().hash(); // hash of empty/0 preimage
    rightfill_with_zeroleaves<aztec3::FUNCTION_TREE_HEIGHT>(leaves, zero_leaf);

    // compute the root of this complete tree, return
    NT::fr root = plonk::stdlib::merkle_tree::compute_tree_root_native(leaves);

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
    NT::fr zero_leaf = FunctionLeafPreimage<NT>().hash(); // hash of empty/0 preimage
    rightfill_with_zeroleaves<aztec3::FUNCTION_TREE_HEIGHT>(leaves, zero_leaf);

    std::vector<NT::fr> tree = plonk::stdlib::merkle_tree::compute_tree_native(leaves);

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
                                        uint8_t const* args_buf,
                                        uint8_t const* constructor_vk_hash_buf,
                                        uint8_t* output)
{
    FunctionData<NT> function_data;
    std::array<NT::fr, aztec3::ARGS_LENGTH> args;
    NT::fr constructor_vk_hash;

    read(function_data_buf, function_data);
    read(args_buf, args);
    read(constructor_vk_hash_buf, constructor_vk_hash);

    NT::fr constructor_hash = compute_constructor_hash(function_data, args, constructor_vk_hash);

    NT::fr::serialize_to_buffer(constructor_hash, output);
}

/**
 * @brief Generates a contract address from its components.
 * This is a WASM-export that can be called from Typescript.
 *
 * @details hash the inputs to generate a deterministic contract address:
 * hash(contract_address, contract_address_salt, function_tree_root, constructor_hash)
 * Return the serialized results in the `output` buffer.
 *
 * @param contract_address_salt_buf bytes buffer representing a field that lets a deployer have
 * some control over contract address
 * @param function_tree_root_buf bytes buffer representing a field that is the root of the
 * contract's function tree
 * @param constructor_hash_buf bytes buffer representing a field that is a hash of constructor info
 * @param output buffer that will contain the output contract address.
 */
WASM_EXPORT void abis__compute_contract_address(uint8_t const* deployer_address_buf,
                                                uint8_t const* contract_address_salt_buf,
                                                uint8_t const* function_tree_root_buf,
                                                uint8_t const* constructor_hash_buf,
                                                uint8_t* output)
{
    NT::address deployer_address;
    NT::fr contract_address_salt;
    NT::fr function_tree_root;
    NT::fr constructor_hash;

    read(deployer_address_buf, deployer_address);
    read(contract_address_salt_buf, contract_address_salt);
    read(function_tree_root_buf, function_tree_root);
    read(constructor_hash_buf, constructor_hash);

    NT::address contract_address =
        compute_contract_address<NT>(deployer_address, contract_address_salt, function_tree_root, constructor_hash);

    NT::fr::serialize_to_buffer(contract_address, output);
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
    read(contract_leaf_preimage_buf, leaf_preimage);
    // as per the circuit implementation, if contract address == zero then return a zero leaf
    auto to_write = leaf_preimage.contract_address == NT::address(0) ? NT::fr(0) : leaf_preimage.hash();
    NT::fr::serialize_to_buffer(to_write, output);
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

WASM_EXPORT const char* abis__test_roundtrip_serialize_private_kernel_inputs(uint8_t const* input, uint32_t* size)
{
    return as_string_output<aztec3::circuits::abis::private_kernel::PrivateInputs<NT>>(input, size);
}

WASM_EXPORT const char* abis__test_roundtrip_serialize_kernel_circuit_public_inputs(uint8_t const* input,
                                                                                    uint32_t* size)
{
    return as_string_output<aztec3::circuits::abis::KernelCircuitPublicInputs<NT>>(input, size);
}

WASM_EXPORT const char* abis__test_roundtrip_serialize_public_kernel_inputs(uint8_t const* input,
                                                                                    uint32_t* size)
{
    return as_string_output<aztec3::circuits::abis::public_kernel::PublicKernelInputs<NT>>(input, size);
}

WASM_EXPORT const char* abis__test_roundtrip_serialize_public_kernel_inputs_no_previous_kernel(uint8_t const* input,
                                                                                    uint32_t* size)
{
    return as_string_output<aztec3::circuits::abis::public_kernel::PublicKernelInputsNoKernelInput<NT>>(input, size);
}

WASM_EXPORT const char* abis__test_roundtrip_serialize_function_leaf_preimage(uint8_t const* function_leaf_preimage_buf,
                                                                              uint32_t* size)
{
    return as_string_output<aztec3::circuits::abis::FunctionLeafPreimage<NT>>(function_leaf_preimage_buf, size);
}

} // extern "C"
