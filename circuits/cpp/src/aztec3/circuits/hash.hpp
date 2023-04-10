#include <aztec3/circuits/abis/function_data.hpp>
#include <aztec3/constants.hpp>

namespace aztec3::circuits {

using abis::FunctionData;

template <typename NCT> typename NCT::fr compute_args_hash(std::array<typename NCT::fr, ARGS_LENGTH> args)
{
    return NCT::compress(args, CONSTRUCTOR_ARGS);
}

template <typename NCT>
typename NCT::fr compute_constructor_hash(FunctionData<NCT> function_data,
                                          std::array<typename NCT::fr, ARGS_LENGTH> args,
                                          typename NCT::fr constructor_vk_hash)
{
    using fr = typename NCT::fr;

    fr function_data_hash = function_data.hash();
    fr args_hash = compute_args_hash<NCT>(args);

    std::vector<fr> inputs = {
        function_data_hash,
        args_hash,
        constructor_vk_hash,
    };

    return NCT::compress(inputs, aztec3::GeneratorIndex::CONSTRUCTOR);
}

template <typename NCT>
typename NCT::address compute_contract_address(typename NCT::address deployer_address,
                                               typename NCT::fr contract_address_salt,
                                               typename NCT::fr function_tree_root,
                                               typename NCT::fr constructor_hash)
{
    using fr = typename NCT::fr;
    using address = typename NCT::address;

    std::vector<fr> inputs = {
        deployer_address.to_field(),
        contract_address_salt,
        function_tree_root,
        constructor_hash,
    };

    return address(NCT::compress(inputs, aztec3::GeneratorIndex::CONTRACT_ADDRESS));
}

template <typename NCT>
typename NCT::fr add_contract_address_to_commitment(typename NCT::address contract_address, typename NCT::fr commitment)
{
    using fr = typename NCT::fr;

    std::vector<fr> inputs = {
        contract_address.to_field(),
        commitment,
    };

    return NCT::compress(inputs, aztec3::GeneratorIndex::OUTER_COMMITMENT);
}

template <typename NCT>
typename NCT::fr add_contract_address_to_nullifier(typename NCT::address contract_address, typename NCT::fr nullifier)
{
    using fr = typename NCT::fr;

    std::vector<fr> inputs = {
        contract_address.to_field(),
        nullifier,
    };

    return NCT::compress(inputs, aztec3::GeneratorIndex::OUTER_NULLIFIER);
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
 * @param leafIndex The index of the leaf element in the Merkle tree
 * @param siblingPath The nodes representing the merkle siblings of the leaf, its parent,
 * the next parent, etc up to the sibling below the root
 * @return The computed Merkle tree root.
 */
template <typename NCT, size_t N>
typename NCT::fr root_from_sibling_path(typename NCT::fr leaf,
                                        typename NCT::uint32 leafIndex,
                                        std::array<typename NCT::fr, N> siblingPath)
{
    for (size_t i = 0; i < siblingPath.size(); i++) {
        if (leafIndex & (1 << i)) {
            leaf = NCT::merkle_hash(siblingPath[i], leaf);
        } else {
            leaf = NCT::merkle_hash(leaf, siblingPath[i]);
        }
    }
    return leaf; // root
}

} // namespace aztec3::circuits