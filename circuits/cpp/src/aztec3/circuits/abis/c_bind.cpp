#include "c_bind.h"
#include "barretenberg/srs/reference_string/mem_reference_string.hpp"
#include "aztec3/circuits/abis/function_data.hpp"
#include "aztec3/circuits/abis/barretenberg/verifier_reference_string.hpp"
#include "private_circuit_public_inputs.hpp"
#include "tx_request.hpp"
#include "tx_context.hpp"
#include "function_leaf_preimage.hpp"
#include "base_rollup/base_rollup_inputs.hpp"
#include "private_kernel/previous_kernel_data.hpp"

#include <aztec3/constants.hpp>

#include <aztec3/utils/types/native_types.hpp>
#include <barretenberg/stdlib/merkle_tree/membership.hpp>
#include <barretenberg/crypto/keccak/keccak.hpp>
#include <barretenberg/common/serialize.hpp>

namespace {

using aztec3::circuits::abis::FunctionLeafPreimage;
using aztec3::circuits::abis::TxContext;
using aztec3::circuits::abis::TxRequest;
using NT = aztec3::utils::types::NativeTypes;

// Cbind helper functions
/**
 * @brief Compute an imperfect merkle tree's root from leaves.
 *
 * @details given a `uint8_t const*` buffer representing a merkle tree's leaves,
 * compute the corresponding tree's root and return the serialized results
 * in the `output` buffer. "Partial left tree" here means that the tree's leaves
 * are filled strictly from left to right, but there may be empty leaves on the right
 * end of the tree.
 *
 * @tparam TREE_HEIGHT height of the tree used to determine max leaves and used when computing root
 * @tparam LeafPreimage the preimage type with a `.hash()` function to generate empty/zero-leaves
 * @param leaves_buf a buffer of bytes representing the leaves of the tree, where each leaf is
 * assumed to be a field and is interpreted using `NT::fr::serialize_from_buffer(leaf_ptr)`
 * @param num_leaves the number of leaves in leaves_buf
 * @returns a field (`NT::fr`) containing the computed merkle tree root
 */
template <size_t TREE_HEIGHT, typename LeafPreimage>
NT::fr compute_root_of_partial_left_tree(uint8_t const* leaves_buf, uint8_t num_leaves)
{
    const size_t max_leaves = 2 << (TREE_HEIGHT - 1);
    // cant exceed max leaves
    ASSERT(num_leaves <= max_leaves);

    // initialize the vector of leaves to a complete-tree-sized vector of zero-leaves
    NT::fr zero_leaf = LeafPreimage().hash(); // hash of empty/0 preimage
    std::vector<NT::fr> leaves(max_leaves, zero_leaf);

    // Iterate over the input buffer, extracting each leaf and serializing it from buffer to field
    // Insert each leaf field into the vector
    // If num_leaves < perfect tree, remaining leaves will be `zero_leaf`
    for (size_t l = 0; l < num_leaves; l++) {
        // each iteration skips to over some number of `fr`s to get to the // next leaf
        uint8_t const* cur_leaf_ptr = leaves_buf + sizeof(NT::fr) * l;
        NT::fr leaf = NT::fr::serialize_from_buffer(cur_leaf_ptr);
        leaves[l] = leaf;
    }

    // compute the root of this complete tree, return
    return plonk::stdlib::merkle_tree::compute_tree_root_native(leaves);
}

} // namespace

#define WASM_EXPORT __attribute__((visibility("default")))

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

// TODO(AD): After Milestone 1, rewrite this with better injection mechanism.
WASM_EXPORT void abis__set_global_verifier_reference_string(uint8_t* data)
{
    auto vrs = std::make_shared<bonk::VerifierMemReferenceString>(data);
    serialize::set_global_verifier_reference_string(vrs);
}

/*** Serialization test helpers ***/
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
    return as_string_output<aztec3::circuits::abis::private_kernel::PreviousKernelData<NT>>(kernel_data_buf, size);
}
} // extern "C"
