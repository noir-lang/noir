#pragma once

#include "aztec3/circuits/abis/function_data.hpp"
#include "aztec3/circuits/abis/function_leaf_preimage.hpp"
#include "aztec3/circuits/abis/function_selector.hpp"
#include "aztec3/circuits/abis/global_variables.hpp"
#include "aztec3/circuits/abis/new_contract_data.hpp"
#include "aztec3/circuits/abis/point.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/circuit_errors.hpp"

#include <barretenberg/barretenberg.hpp>

#include <array>
#include <vector>

namespace aztec3::circuits {

using abis::FunctionData;
using abis::FunctionSelector;
using abis::Point;
using aztec3::circuits::abis::ContractLeafPreimage;
using aztec3::circuits::abis::FunctionLeafPreimage;
using MemoryStore = stdlib::merkle_tree::MemoryStore;
using MerkleTree = stdlib::merkle_tree::MerkleTree<MemoryStore>;

template <typename NCT> typename NCT::fr compute_var_args_hash(std::vector<typename NCT::fr> const& args)
{
    auto const MAX_ARGS = 32;
    if (args.size() > MAX_ARGS) {
        throw_or_abort("Too many arguments in call to compute_var_args_hash");
    }
    return NCT::hash(args, FUNCTION_ARGS);
}

template <typename NCT> typename NCT::fr compute_constructor_hash(FunctionData<NCT> const& function_data,
                                                                  typename NCT::fr const& args_hash,
                                                                  typename NCT::fr const& constructor_vk_hash)
{
    using fr = typename NCT::fr;

    fr const function_data_hash = function_data.hash();

    std::vector<fr> const inputs = {
        function_data_hash,
        args_hash,
        constructor_vk_hash,
    };

    return NCT::compress(inputs, aztec3::GeneratorIndex::CONSTRUCTOR);
}

template <typename NCT> typename NCT::fr compute_partial_address(typename NCT::fr const& contract_address_salt,
                                                                 typename NCT::fr const& function_tree_root,
                                                                 typename NCT::fr const& constructor_hash)
{
    std::vector<typename NCT::fr> const inputs = {
        typename NCT::fr(0), typename NCT::fr(0), contract_address_salt, function_tree_root, constructor_hash,
    };
    return NCT::hash(inputs, aztec3::GeneratorIndex::PARTIAL_ADDRESS);
}

template <typename NCT>
typename NCT::address compute_contract_address_from_partial(Point<NCT> const& point,
                                                            typename NCT::fr const& partial_address)
{
    std::vector<typename NCT::fr> const inputs = {
        point.x,
        point.y,
        partial_address,
    };
    return { NCT::hash(inputs, aztec3::GeneratorIndex::CONTRACT_ADDRESS) };
}

template <typename NCT> typename NCT::address compute_contract_address(Point<NCT> const& point,
                                                                       typename NCT::fr const& contract_address_salt,
                                                                       typename NCT::fr const& function_tree_root,
                                                                       typename NCT::fr const& constructor_hash)
{
    using fr = typename NCT::fr;

    const fr partial_address =
        compute_partial_address<NCT>(contract_address_salt, function_tree_root, constructor_hash);

    return compute_contract_address_from_partial(point, partial_address);
}

template <typename NCT> typename NCT::fr compute_commitment_nonce(typename NCT::fr const& first_nullifier,
                                                                  typename NCT::fr const& commitment_index)
{
    using fr = typename NCT::fr;

    std::vector<fr> const inputs = {
        first_nullifier,
        commitment_index,
    };

    return NCT::hash(inputs, aztec3::GeneratorIndex::COMMITMENT_NONCE);
}

template <typename NCT> typename NCT::fr silo_commitment(typename NCT::address const& contract_address,
                                                         typename NCT::fr const& inner_commitment)
{
    using fr = typename NCT::fr;

    std::vector<fr> const inputs = {
        contract_address.to_field(),
        inner_commitment,
    };

    return NCT::hash(inputs, aztec3::GeneratorIndex::SILOED_COMMITMENT);
}

template <typename NCT>
typename NCT::fr compute_unique_commitment(typename NCT::fr nonce, typename NCT::fr siloed_commitment)
{
    using fr = typename NCT::fr;

    std::vector<fr> const inputs = {
        nonce,
        siloed_commitment,
    };

    return NCT::hash(inputs, aztec3::GeneratorIndex::UNIQUE_COMMITMENT);
}

template <typename NCT>
typename NCT::fr silo_nullifier(typename NCT::address const& contract_address, typename NCT::fr nullifier)
{
    using fr = typename NCT::fr;

    std::vector<fr> const inputs = {
        contract_address.to_field(),
        nullifier,
    };

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1475): use hash here (everywhere?)
    return NCT::compress(inputs, aztec3::GeneratorIndex::OUTER_NULLIFIER);
}


template <typename NCT> typename NCT::fr compute_block_hash(typename NCT::fr const& globals_hash,
                                                            typename NCT::fr const& private_data_tree_root,
                                                            typename NCT::fr const& nullifier_tree_root,
                                                            typename NCT::fr const& contract_tree_root,
                                                            typename NCT::fr const& l1_to_l2_data_tree_root,
                                                            typename NCT::fr const& public_data_tree_root)
{
    using fr = typename NCT::fr;

    std::vector<fr> const inputs = {
        globals_hash,       private_data_tree_root,  nullifier_tree_root,
        contract_tree_root, l1_to_l2_data_tree_root, public_data_tree_root,
    };

    return NCT::compress(inputs, aztec3::GeneratorIndex::BLOCK_HASH);
}

template <typename NCT>
typename NCT::fr compute_block_hash_with_globals(abis::GlobalVariables<NCT> const& globals,
                                                 typename NCT::fr const& private_data_tree_root,
                                                 typename NCT::fr const& nullifier_tree_root,
                                                 typename NCT::fr const& contract_tree_root,
                                                 typename NCT::fr const& l1_to_l2_data_tree_root,
                                                 typename NCT::fr const& public_data_tree_root)
{
    using fr = typename NCT::fr;

    std::vector<fr> const inputs = {
        globals.hash(),     private_data_tree_root,  nullifier_tree_root,
        contract_tree_root, l1_to_l2_data_tree_root, public_data_tree_root,
    };

    return NCT::compress(inputs, aztec3::GeneratorIndex::BLOCK_HASH);
}

template <typename NCT> typename NCT::fr compute_globals_hash(abis::GlobalVariables<NCT> const& globals)
{
    return globals.hash();
}

/**
 * @brief Calculate the Merkle tree root from the sibling path and leaf.
 *
 * @details The leaf is hashed with its sibling, and then the result is hashed
 * with the next sibling etc in the path. The last hash is the root.
 *
 * @tparam NCT Operate on NativeTypes or CircuitTypes
 * @tparam N The number of elements in the sibling path
 * @param leaf The leaf element of the Merkle tree
 * @param leaf_index The index of the leaf element in the Merkle tree
 * @param sibling_path The nodes representing the merkle siblings of the leaf, its parent,
 * the next parent, etc up to the sibling below the root
 * @return The computed Merkle tree root.
 *
 * TODO need to use conditional assigns instead of `ifs` for circuit version.
 *      see membership.hpp:check_subtree_membership (left/right/conditional_assign, etc)
 */
template <typename NCT, size_t N>
typename NCT::fr root_from_sibling_path(typename NCT::fr const& leaf,
                                        typename NCT::uint32 const& leaf_index,
                                        std::array<typename NCT::fr, N> const& sibling_path)
{
    auto node = leaf;
    for (size_t i = 0; i < N; i++) {
        if (leaf_index & (1 << i)) {
            node = NCT::merkle_hash(sibling_path[i], node);
        } else {
            node = NCT::merkle_hash(node, sibling_path[i]);
        }
    }
    return node;  // root
}

/**
 * @brief Calculate the Merkle tree root from the sibling path and leaf.
 *
 * @details The leaf is hashed with its sibling, and then the result is hashed
 * with the next sibling etc in the path. The last hash is the root.
 *
 * @tparam NCT Operate on NativeTypes or CircuitTypes
 * @tparam N The number of elements in the sibling path
 * @param leaf The leaf element of the Merkle tree
 * @param leaf_index The index of the leaf element in the Merkle tree
 * @param sibling_path The nodes representing the merkle siblings of the leaf, its parent,
 * the next parent, etc up to the sibling below the root
 * @return The computed Merkle tree root.
 *
 * TODO need to use conditional assigns instead of `ifs` for circuit version.
 *      see membership.hpp:check_subtree_membership (left/right/conditional_assign, etc)
 */
template <typename NCT, size_t N>
typename NCT::fr root_from_sibling_path(typename NCT::fr const& leaf,
                                        typename NCT::fr const& leaf_index,
                                        std::array<typename NCT::fr, N> const& sibling_path)
{
    auto node = leaf;
    uint256_t index = leaf_index;
    for (size_t i = 0; i < N; i++) {
        if (index & 1) {
            node = NCT::merkle_hash(sibling_path[i], node);
        } else {
            node = NCT::merkle_hash(node, sibling_path[i]);
        }
        index >>= uint256_t(1);
    }
    return node;  // root
}

/**
 * @brief Get the sibling path of an item in a given merkle tree
 *
 * WARNING: this function is for testing purposes only! leaf_index is an fr
 * in `MembershipWitness` but is a `size_t` here. This could lead to overflows
 * on `1 << i` if the tree is large enough.
 *
 * @tparam N height of tree (not including root)
 * @param tree merkle tree to operate on
 * @param leaf_index index of the leaf to get path for
 * @param subtree_depth_to_skip skip some number of bottom layers
 * @return std::array<fr, N> sibling path
 */
template <size_t N>
std::array<fr, N> get_sibling_path(MerkleTree& tree, size_t leaf_index, size_t const& subtree_depth_to_skip)
{
    std::array<fr, N> sibling_path;
    auto path = tree.get_hash_path(leaf_index);
    // slice out the skip
    leaf_index = leaf_index >> (subtree_depth_to_skip);
    for (size_t i = 0; i < N; i++) {
        if (leaf_index & (1 << i)) {
            sibling_path[i] = path[subtree_depth_to_skip + i].first;
        } else {
            sibling_path[i] = path[subtree_depth_to_skip + i].second;
        }
    }
    return sibling_path;
}

template <typename NCT, typename Builder, size_t SIZE>
void check_membership(Builder& builder,
                      typename NCT::fr const& value,
                      typename NCT::fr const& index,
                      std::array<typename NCT::fr, SIZE> const& sibling_path,
                      typename NCT::fr const& root,
                      std::string const& msg)
{
    const auto calculated_root = root_from_sibling_path<NCT>(value, index, sibling_path);
    builder.do_assert(calculated_root == root,
                      std::string("Membership check failed: ") + msg,
                      aztec3::utils::CircuitErrorCode::MEMBERSHIP_CHECK_FAILED);
}

/**
 * @brief Calculate the function tree root from the sibling path and leaf preimage.
 *
 * @tparam NCT (native or circuit)
 * @param selector in leaf preimage
 * @param is_internal in leaf preimage
 * @param is_private in leaf preimage
 * @param vk_hash in leaf preimage
 * @param acir_hash in leaf preimage
 * @param function_leaf_index leaf index in the function tree
 * @param function_leaf_sibling_path
 * @return NCT::fr
 */
template <typename NCT> typename NCT::fr function_tree_root_from_siblings(
    FunctionSelector<NCT> const& selector,
    typename NCT::boolean const& is_internal,
    typename NCT::boolean const& is_private,
    typename NCT::fr const& vk_hash,
    typename NCT::fr const& acir_hash,
    typename NCT::fr const& function_leaf_index,
    std::array<typename NCT::fr, FUNCTION_TREE_HEIGHT> const& function_leaf_sibling_path)
{
    const auto function_leaf_preimage = FunctionLeafPreimage<NCT>{
        .selector = selector,
        .is_internal = is_internal,
        .is_private = is_private,
        .vk_hash = vk_hash,
        .acir_hash = acir_hash,
    };

    const auto function_leaf = function_leaf_preimage.hash();

    auto function_tree_root =
        root_from_sibling_path<NCT>(function_leaf, function_leaf_index, function_leaf_sibling_path);
    return function_tree_root;
}

/**
 * @brief Calculate the contract tree root from the sibling path and leaf preimage.
 *
 * @tparam NCT (native or circuit)
 * @param function_tree_root in leaf preimage
 * @param storage_contract_address in leaf preimage
 * @param portal_contract_address in leaf preimage
 * @param contract_leaf_index leaf index in the function tree
 * @param contract_leaf_sibling_path
 * @return NCT::fr
 */
template <typename NCT> typename NCT::fr contract_tree_root_from_siblings(
    typename NCT::fr const& function_tree_root,
    typename NCT::address const& storage_contract_address,
    typename NCT::address const& portal_contract_address,
    typename NCT::fr const& contract_leaf_index,
    std::array<typename NCT::fr, CONTRACT_TREE_HEIGHT> const& contract_leaf_sibling_path)
{
    const ContractLeafPreimage<NCT> contract_leaf_preimage{ storage_contract_address,
                                                            portal_contract_address,
                                                            function_tree_root };

    const auto contract_leaf = contract_leaf_preimage.hash();

    const auto computed_contract_tree_root =
        root_from_sibling_path<NCT>(contract_leaf, contract_leaf_index, contract_leaf_sibling_path);
    return computed_contract_tree_root;
}

/**
 * @brief Compute sibling path for an empty tree.
 *
 * @tparam NCT (native or circuit)
 * @tparam TREE_HEIGHT
 * @param zero_leaf the leaf value that corresponds to a zero preimage
 * @return std::array<typename NCT::fr, TREE_HEIGHT>
 */
template <typename NCT, size_t TREE_HEIGHT>
std::array<typename NCT::fr, TREE_HEIGHT> compute_empty_sibling_path(typename NCT::fr const& zero_leaf)
{
    std::array<typename NCT::fr, TREE_HEIGHT> sibling_path = { zero_leaf };
    for (size_t i = 1; i < TREE_HEIGHT; i++) {
        // hash previous sibling with itself to get node above
        sibling_path[i] = NCT::merkle_hash(sibling_path[i - 1], sibling_path[i - 1]);
    }
    return sibling_path;
}

/**
 * @brief Compute the value to be inserted into the public data tree
 * @param value The value to be inserted into the public data tree
 * @return The hash value required for insertion into the public data tree
 */
template <typename NCT> typename NCT::fr compute_public_data_tree_value(typename NCT::fr const& value)
{
    // as it's a public value, it doesn't require hashing.
    // leaving this function here in case we decide to change this.
    return value;
}

/**
 * @brief Compute the index for inserting a value into the public data tree
 * @param contract_address The address of the contract to which the inserted element belongs
 * @param storage_slot The storage slot to which the inserted element belongs
 * @return The index for insertion into the public data tree
 */
template <typename NCT> typename NCT::fr compute_public_data_tree_index(typename NCT::fr const& contract_address,
                                                                        typename NCT::fr const& storage_slot)
{
    return NCT::compress({ contract_address, storage_slot }, GeneratorIndex::PUBLIC_LEAF_INDEX);
}

template <typename NCT> typename NCT::fr compute_l2_to_l1_hash(typename NCT::address const& contract_address,
                                                               typename NCT::fr const& rollup_version_id,
                                                               typename NCT::fr const& portal_contract_address,
                                                               typename NCT::fr const& chain_id,
                                                               typename NCT::fr const& content)
{
    using fr = typename NCT::fr;

    std::vector<fr> const inputs = {
        contract_address.to_field(), rollup_version_id, portal_contract_address, chain_id, content,
    };

    constexpr auto const num_bytes = 5 * 32;
    std::array<uint8_t, num_bytes> calldata_hash_inputs_bytes;
    // Convert all into a buffer, then copy into the array, then hash
    for (size_t i = 0; i < inputs.size(); i++) {
        auto as_bytes = inputs[i].to_buffer();

        auto offset = i * 32;
        std::copy(as_bytes.begin(), as_bytes.end(), calldata_hash_inputs_bytes.begin() + offset);
    }

    std::vector<uint8_t> const calldata_hash_inputs_bytes_vec(calldata_hash_inputs_bytes.begin(),
                                                              calldata_hash_inputs_bytes.end());

    // @todo @LHerskind NOTE sha to field!
    return sha256::sha256_to_field(calldata_hash_inputs_bytes_vec);
}

/**
 * @brief Computes sha256 hash of 2 input hashes stored in 4 fields.
 * @param hashes 4 fields containing 2 hashes [high, low, high, low].
 * @return Resulting sha256 hash stored in 2 fields.
 */
template <typename NCT> std::array<typename NCT::fr, NUM_FIELDS_PER_SHA256> accumulate_sha256(
    std::array<typename NCT::fr, NUM_FIELDS_PER_SHA256 * 2> const& hashes)
{
    using fr = typename NCT::fr;

    // Generate a 512 bit input from right and left 256 bit hashes
    constexpr auto num_bytes = 2 * 32;
    std::array<uint8_t, num_bytes> hash_input_bytes;
    for (size_t i = 0; i < 4; i++) {
        auto half = hashes[i].to_buffer();
        for (size_t j = 0; j < 16; j++) {
            hash_input_bytes[i * 16 + j] = half[16 + j];
        }
    }

    // Compute the sha256
    std::vector<uint8_t> const hash_input_bytes_vec(hash_input_bytes.begin(), hash_input_bytes.end());
    auto h = sha256::sha256(hash_input_bytes_vec);

    // Split the hash into two fields, a high and a low
    std::array<uint8_t, 32> buf_1;
    std::array<uint8_t, 32> buf_2;
    for (uint8_t i = 0; i < 16; i++) {
        buf_1[i] = 0;
        buf_1[16 + i] = h[i];
        buf_2[i] = 0;
        buf_2[16 + i] = h[i + 16];
    }
    auto high = fr::serialize_from_buffer(buf_1.data());
    auto low = fr::serialize_from_buffer(buf_2.data());

    return { high, low };
}

}  // namespace aztec3::circuits
