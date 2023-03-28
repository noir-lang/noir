#include "aztec3/constants.hpp"
#include "barretenberg/crypto/pedersen_hash/pedersen.hpp"
#include "barretenberg/crypto/sha256/sha256.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/stdlib/hash/pedersen/pedersen.hpp"
#include "barretenberg/stdlib/merkle_tree/membership.hpp"
#include "barretenberg/stdlib/merkle_tree/memory_tree.hpp"
#include "barretenberg/stdlib/merkle_tree/merkle_tree.hpp"
#include "init.hpp"

#include <algorithm>
#include <array>
#include <aztec3/circuits/abis/rollup/base/base_rollup_inputs.hpp>
#include <aztec3/circuits/abis/rollup/base/base_rollup_public_inputs.hpp>
#include <aztec3/circuits/abis/rollup/nullifier_leaf_preimage.hpp>
#include <cstdint>
#include <iostream>
#include <tuple>
#include <vector>

namespace aztec3::circuits::rollup::native_base_rollup {

const uint8_t COMMITMENTS_SUBTREE_DEPTH = 3;
const uint8_t CONTRACTS_SUBTREE_DEPTH = 1;
const NT::fr EMPTY_COMMITMENTS_SUBTREE_ROOT = stdlib::merkle_tree::MemoryTree(COMMITMENTS_SUBTREE_DEPTH).root();
const NT::fr EMPTY_CONTRACTS_SUBTREE_ROOT = stdlib::merkle_tree::MemoryTree(CONTRACTS_SUBTREE_DEPTH).root();

// TODO: can we aggregate proofs if we do not have a working circuit impl

// TODO: change the public inputs array - we wont be using this?

// Access Native types through NT namespace

bool verify_kernel_proof(NT::Proof kernel_proof)
{
    std::cout << kernel_proof << std::endl; // REMOVE_ME
    return true;
}

/**
 * @brief Create an aggregation object for the proofs that are provided
 *          - We add points P0 for each of our proofs
 *          - We add points P1 for each of our proofs
 *          - We concat our public inputs
 *
 * @param baseRollupInputs
 * @return AggregationObject
 */
AggregationObject aggregate_proofs(BaseRollupInputs baseRollupInputs)
{

    // TODO: NOTE: for now we simply return the aggregation object from the first proof
    return baseRollupInputs.kernel_data[0].public_inputs.end.aggregation_object;
}

/** TODO: implement
 * @brief Get the prover contribution hash object
 *
 * @return NT::fr
 */
NT::fr get_prover_contribution_hash()
{
    return NT::fr(0);
}

std::vector<NT::fr> calculate_contract_leaves(BaseRollupInputs baseRollupInputs)
{

    std::vector<NT::fr> contract_leaves;

    for (size_t i = 0; i < 2; i++) {

        auto new_contacts = baseRollupInputs.kernel_data[i].public_inputs.end.new_contracts;

        // loop over the new contracts
        // TODO: NOTE: we are currently assuming that there is only going to be one
        for (size_t j = 0; j < new_contacts.size(); j++) {

            NT::address contract_address = new_contacts[j].contract_address;
            // TODO: UPDATE protal_contract_address is listed as a 20 byte address in the ABI
            NT::fr portal_contract_address = new_contacts[j].portal_contract_address;
            NT::fr function_tree_root = new_contacts[j].function_tree_root;

            // Pedersen hash of the 3 fields (contract_address, portal_contract_address, function_tree_root)
            auto contract_leaf =
                crypto::pedersen_hash::hash_multiple({ contract_address, portal_contract_address, function_tree_root });

            // When there is no contract deployment, we should insert a zero leaf into the tree and ignore the
            // member-ship check. This is to ensure that we don't hit "already deployed" errors when we are not
            // deploying contracts. e.g., when we are only calling functions on existing contracts.
            auto to_push = contract_address == NT::address(0) ? NT::fr(0) : contract_leaf;

            contract_leaves.push_back(to_push);
        }
    }

    return contract_leaves;
}

template <size_t N>
NT::fr iterate_through_tree_via_sibling_path(NT::fr leaf, NT::uint32 leafIndex, std::array<NT::fr, N> siblingPath)
{
    for (size_t i = 0; i < siblingPath.size(); i++) {
        if (leafIndex & (1 << i)) {
            leaf = crypto::pedersen_hash::hash_multiple({ leaf, siblingPath[i] });
        } else {
            leaf = crypto::pedersen_hash::hash_multiple({ siblingPath[i], leaf });
        }
    }
    return leaf;
}

template <size_t N>
void check_membership(NT::fr leaf, NT::uint32 leafIndex, std::array<NT::fr, N> siblingPath, NT::fr root)
{
    auto calculatedRoot = iterate_through_tree_via_sibling_path(leaf, leafIndex, siblingPath);
    if (calculatedRoot != root) {
        // throw std::runtime_error("Merkle membership check failed");
    }
}

template <size_t N>
AppendOnlySnapshot insert_subtree_to_snapshot_tree(std::array<NT::fr, N> siblingPath,
                                                   NT::uint32 nextAvailableLeafIndex,
                                                   NT::fr subtreeRootToInsert,
                                                   uint8_t subtreeDepth)
{
    // TODO: Sanity check len of siblingPath > height of subtree
    // TODO: Ensure height of subtree is correct (eg 3 for commitments, 1 for contracts)
    auto leafIndexAtDepth = nextAvailableLeafIndex >> subtreeDepth;
    auto new_root = iterate_through_tree_via_sibling_path(subtreeRootToInsert, leafIndexAtDepth, siblingPath);
    // 2^subtreeDepth is the number of leaves added. 2^x = 1 << x
    auto new_next_available_leaf_index = nextAvailableLeafIndex + (uint8_t(1) << subtreeDepth);

    AppendOnlySnapshot newTreeSnapshot = { .root = new_root,
                                           .next_available_leaf_index = new_next_available_leaf_index };
    return newTreeSnapshot;
}

std::array<NT::fr, 3> calculate_new_subtrees(BaseRollupInputs baseRollupInputs, std::vector<NT::fr> contract_leaves)
{
    // Leaves that will be added to the new trees
    std::array<NT::fr, KERNEL_NEW_COMMITMENTS_LENGTH * 2> commitment_leaves; // TODO: use constant
    std::array<NT::fr, KERNEL_NEW_NULLIFIERS_LENGTH * 2> nullifier_leaves;   // TODO: use constant

    // TODO: we have at size two for now, but we will need

    stdlib::merkle_tree::MemoryTree contracts_tree = stdlib::merkle_tree::MemoryTree(CONTRACTS_SUBTREE_DEPTH);
    stdlib::merkle_tree::MemoryTree commitments_tree = stdlib::merkle_tree::MemoryTree(COMMITMENTS_SUBTREE_DEPTH);
    stdlib::merkle_tree::MemoryTree nullifier_tree = stdlib::merkle_tree::MemoryTree(COMMITMENTS_SUBTREE_DEPTH);

    for (size_t i = 0; i < 2; i++) {

        auto new_commitments = baseRollupInputs.kernel_data[i].public_inputs.end.new_commitments;

        // Our commitments size MUST be 4 to calculate our subtrees correctly
        assert(new_commitments.size() == 4);

        for (size_t j = 0; j < new_commitments.size(); j++) {
            // todo: batch insert
            commitments_tree.update_element(i * KERNEL_NEW_COMMITMENTS_LENGTH + j, new_commitments[j]);
        }

        // Nullifiers
        auto new_nullifiers = baseRollupInputs.kernel_data[i].public_inputs.end.new_nullifiers;
        assert(new_nullifiers.size() == 4);
        for (size_t j = 0; j < new_nullifiers.size(); j++) {
            nullifier_tree.update_element(i * KERNEL_NEW_NULLIFIERS_LENGTH + j, new_nullifiers[j]);
        }
    }

    // Compute the merkle root of a contract subtree
    // TODO: consolidate what the tree depth should be
    // TODO: cleanup lmao
    // Contracts subtree
    for (size_t i = 0; i < contract_leaves.size(); i++) {
        contracts_tree.update_element(i, contract_leaves[i]);
    }
    NT::fr contracts_tree_subroot = contracts_tree.root();

    // Commitments subtree
    NT::fr commitments_tree_subroot = commitments_tree.root();

    NT::fr nullifiers_tree_subroot =
        nullifier_tree.root(); // TODO: nullifier tree will be a different tree impl - indexed merkle tree

    return std::array<NT::fr, 3>{ contracts_tree_subroot, commitments_tree_subroot, nullifiers_tree_subroot };
}

std::array<NT::fr, 2> calculate_calldata_hash(BaseRollupInputs baseRollupInputs, std::vector<NT::fr> contract_leaves)
{
    // Compute calldata hashes
    // 22 = (4 + 4 + 1 + 2) * 2 (2 kernels, 4 nullifiers per kernel, 4 commitments per kernel, 1 contract
    // deployments, 2 contracts data fields (size 2 for each) )
    std::array<NT::fr, 22> calldata_hash_inputs;

    for (size_t i = 0; i < 2; i++) {
        // Nullifiers
        auto new_nullifiers = baseRollupInputs.kernel_data[i].public_inputs.end.new_nullifiers;
        auto new_commitments = baseRollupInputs.kernel_data[i].public_inputs.end.new_commitments;
        for (size_t j = 0; j < KERNEL_NEW_COMMITMENTS_LENGTH; j++) { // TODO: const
            calldata_hash_inputs[i * KERNEL_NEW_COMMITMENTS_LENGTH + j] = new_nullifiers[j];
            calldata_hash_inputs[(KERNEL_NEW_NULLIFIERS_LENGTH * 2) + i * KERNEL_NEW_NULLIFIERS_LENGTH + j] =
                new_commitments[j];
        }

        // yuck - TODO: is contract_leaves fixed size?
        calldata_hash_inputs[16 + i] = contract_leaves[i];

        auto new_contracts = baseRollupInputs.kernel_data[i].public_inputs.end.new_contracts;

        // TODO: this assumes that there is only one contract deployment
        calldata_hash_inputs[18 + i] = new_contracts[0].contract_address;
        calldata_hash_inputs[20 + i] = new_contracts[0].portal_contract_address;
    }

    // FIXME
    // Calculate sha256 hash of calldata; TODO: work out typing here
    // 22 * 32 = 22 fields, each 32 bytes
    std::array<uint8_t, 22 * 32> calldata_hash_inputs_bytes;
    // Convert all into a buffer, then copy into the array, then hash
    for (size_t i = 0; i < calldata_hash_inputs.size(); i++) {
        auto as_bytes = calldata_hash_inputs[i].to_buffer();

        auto offset = i * 32;
        std::copy(as_bytes.begin(), as_bytes.end(), calldata_hash_inputs_bytes.begin() + offset);
    }
    // TODO: double check this gpt code
    std::vector<uint8_t> calldata_hash_inputs_bytes_vec(calldata_hash_inputs_bytes.begin(),
                                                        calldata_hash_inputs_bytes.end());

    auto h = sha256::sha256(calldata_hash_inputs_bytes_vec);

    // Split the hash into two fields, a high and a low
    std::array<uint8_t, 32> buf_1, buf_2;
    for (uint8_t i = 0; i < 16; i++) {
        buf_1[i] = 0;
        buf_1[16 + i] = h[i];
        buf_2[i] = 0;
        buf_2[16 + i] = h[i + 16];
    }
    auto high = fr::serialize_from_buffer(buf_1.data());
    auto low = fr::serialize_from_buffer(buf_2.data());

    return std::array<NT::fr, 2>{ high, low };
}

/**
 * @brief Check all of the provided commitments against the historical tree roots
 *
 * @param constantBaseRollupData
 * @param baseRollupInputs
 */
void perform_historical_private_data_tree_membership_checks(BaseRollupInputs baseRollupInputs)
{
    // For each of the historic_private_data_tree_membership_checks, we need to do an inclusion proof
    // against the historical root provided in the rollup constants
    auto historic_root = baseRollupInputs.constants.start_tree_of_historic_private_data_tree_roots_snapshot.root;

    for (size_t i = 0; i < 2; i++) {
        NT::fr leaf = baseRollupInputs.kernel_data[i].public_inputs.constants.old_tree_roots.private_data_tree_root;
        abis::MembershipWitness<NT, PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT> historic_root_witness =
            baseRollupInputs.historic_private_data_tree_root_membership_witnesses[i];

        check_membership(leaf, historic_root_witness.leaf_index, historic_root_witness.sibling_path, historic_root);
    }
}

void perform_historical_contract_data_tree_membership_checks(BaseRollupInputs baseRollupInputs)
{
    auto historic_root = baseRollupInputs.constants.start_tree_of_historic_contract_tree_roots_snapshot.root;

    for (size_t i = 0; i < 2; i++) {
        NT::fr leaf = baseRollupInputs.kernel_data[i].public_inputs.constants.old_tree_roots.contract_tree_root;
        abis::MembershipWitness<NT, PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT> historic_root_witness =
            baseRollupInputs.historic_contract_tree_root_membership_witnesses[i];

        check_membership(leaf, historic_root_witness.leaf_index, historic_root_witness.sibling_path, historic_root);
    }
}

/**
 * @brief Check non membership of each of the generated nullifiers in the current tree
 *
 * @returns The end nullifier tree root
 */
AppendOnlySnapshot check_nullifier_tree_non_membership_and_insert_to_tree(BaseRollupInputs baseRollupInputs,
                                                                          NT::fr nullifier_tree_subtree_root)
{
    // LADIES AND GENTLEMEN The P L A N ( is simple )
    // 1. Get the previous nullifier set setup
    // 2. Check for the first added nullifier that it doesnt exist
    // 3. Update the nullifier set
    // 4. Calculate a new root with the sibling path
    // 5. Use that for the next nullifier check.
    // 6. Iterate for all of em
    // 7. le bosh (profit)

    // This will update on each iteration
    auto current_nullifier_tree_root = baseRollupInputs.start_nullifier_tree_snapshot.root;

    // This will increase with every insertion
    auto new_index = baseRollupInputs.start_nullifier_tree_snapshot.next_available_leaf_index;

    for (size_t i = 0; i < 2; i++) {
        auto new_nullifiers = baseRollupInputs.kernel_data[i].public_inputs.end.new_nullifiers;
        for (size_t j = 0; j < KERNEL_NEW_NULLIFIERS_LENGTH; j++) {

            // Witness containing index and path
            auto nullifier_index = 4 * i + j;
            auto witness = baseRollupInputs.low_nullifier_membership_witness[nullifier_index];
            // Preimage of the lo-index required for a non-membership proof
            auto low_nullifier_preimage = baseRollupInputs.low_nullifier_leaf_preimages[nullifier_index];
            // Newly created nullifier
            auto nullifier = new_nullifiers[j];

            // assert that the low_nullifier provided is the correct one by performing two range checks
            auto is_less_than_nullifier = low_nullifier_preimage.leaf_value < nullifier;
            auto is_next_greater_than = low_nullifier_preimage.next_value > nullifier;
            if (!(is_less_than_nullifier && is_next_greater_than)) {
                if (low_nullifier_preimage.next_index != 0 && low_nullifier_preimage.next_value != 0) {
                    // throw std::runtime_error("Low nullifier preimage is incorrect");
                }
            }

            // Calculate the leaf hash
            auto updated_leaf = crypto::pedersen_hash::hash_multiple({ low_nullifier_preimage.leaf_value,
                                                                       low_nullifier_preimage.next_index,
                                                                       low_nullifier_preimage.next_value });

            // perform (non) membership check for each of the provided paths
            check_membership<NULLIFIER_TREE_HEIGHT>(
                updated_leaf, witness.leaf_index, witness.sibling_path, current_nullifier_tree_root);

            // against the new nullifier root, calculate the new leaf hash of the new low_nullifier_preimage
            auto new_leaf =
                crypto::pedersen_hash::hash_multiple({ low_nullifier_preimage.leaf_value, new_index, nullifier });

            // increase new index for next insertion
            new_index = new_index + 1;

            // Use the existing sibling path to calculate the new root
            current_nullifier_tree_root =
                iterate_through_tree_via_sibling_path(new_leaf, witness.leaf_index, witness.sibling_path);
        }
    }

    // Check the subtree insertions for the new nullifiers
    auto nullifier_sibling_path = baseRollupInputs.new_nullifiers_subtree_sibling_path;

    // Calculate the new root
    // We are inserting a subtree rather than a full tree here
    auto subtree_index = new_index >> 3;
    auto new_root =
        iterate_through_tree_via_sibling_path(nullifier_tree_subtree_root, subtree_index, nullifier_sibling_path);

    // Return the new state of the nullifier tree
    return {
        .root = new_root,
        .next_available_leaf_index = new_index,
    };
}

// Important types:
//   - BaseRollupPublicInputs - where we want to put our return values
//
// TODO: replace auto
BaseRollupPublicInputs base_rollup_circuit(BaseRollupInputs baseRollupInputs)
{

    // First we compute the contract tree leaves

    // Verify the previous kernel proofs
    for (size_t i = 0; i < 2; i++) {
        NT::Proof proof = baseRollupInputs.kernel_data[i].proof;
        assert(verify_kernel_proof(proof));
    }

    std::vector<NT::fr> contract_leaves = calculate_contract_leaves(baseRollupInputs);

    // Perform merkle membership check with the provided sibling path up to the root
    // Note - the subtree hasn't been created (i.e. it is empty) so you check that the sibling path corresponds to an
    // empty tree

    // check for commitments/private_data
    // next_available_leaf_index is at the leaf level. We need at the subtree level (say height 3). So divide by 8.
    // (if leaf is at index x, its parent is at index floor(x/2))
    auto leafIndexAtSubtreeDepth = baseRollupInputs.start_private_data_tree_snapshot.next_available_leaf_index /
                                   (NT::uint32(1) << COMMITMENTS_SUBTREE_DEPTH);
    check_membership(EMPTY_COMMITMENTS_SUBTREE_ROOT,
                     leafIndexAtSubtreeDepth,
                     baseRollupInputs.new_commitments_subtree_sibling_path,
                     baseRollupInputs.start_private_data_tree_snapshot.root);

    // check for contracts
    leafIndexAtSubtreeDepth = baseRollupInputs.start_contract_tree_snapshot.next_available_leaf_index /
                              (NT::uint32(1) << CONTRACTS_SUBTREE_DEPTH);
    check_membership(EMPTY_CONTRACTS_SUBTREE_ROOT,
                     leafIndexAtSubtreeDepth,
                     baseRollupInputs.new_contracts_subtree_sibling_path,
                     baseRollupInputs.start_contract_tree_snapshot.root);

    std::array<NT::fr, 3> new_subtrees = calculate_new_subtrees(baseRollupInputs, contract_leaves);
    NT::fr contracts_tree_subroot = new_subtrees[0];
    NT::fr commitments_tree_subroot = new_subtrees[1];
    NT::fr nullifiers_tree_subroot = new_subtrees[2];

    // Insert subtrees to the tree:
    auto end_private_data_tree_snapshot =
        insert_subtree_to_snapshot_tree(baseRollupInputs.new_commitments_subtree_sibling_path,
                                        baseRollupInputs.start_private_data_tree_snapshot.next_available_leaf_index,
                                        commitments_tree_subroot,
                                        COMMITMENTS_SUBTREE_DEPTH);

    auto end_contract_tree_snapshot =
        insert_subtree_to_snapshot_tree(baseRollupInputs.new_contracts_subtree_sibling_path,
                                        baseRollupInputs.start_contract_tree_snapshot.next_available_leaf_index,
                                        contracts_tree_subroot,
                                        CONTRACTS_SUBTREE_DEPTH);

    // TODO: Nullifiers tree insertion

    // Calculate the overall calldata hash
    std::array<NT::fr, 2> calldata_hash = calculate_calldata_hash(baseRollupInputs, contract_leaves);

    // Perform membership checks that the notes provided exist within the historic trees data
    perform_historical_private_data_tree_membership_checks(baseRollupInputs);
    perform_historical_contract_data_tree_membership_checks(baseRollupInputs);

    AppendOnlySnapshot end_nullifier_tree_snapshot =
        check_nullifier_tree_non_membership_and_insert_to_tree(baseRollupInputs, nullifiers_tree_subroot);

    AggregationObject aggregation_object = aggregate_proofs(baseRollupInputs);

    BaseRollupPublicInputs public_inputs = {
        .end_aggregation_object = aggregation_object,
        .constants = baseRollupInputs.constants,
        .start_private_data_tree_snapshot = baseRollupInputs.start_private_data_tree_snapshot,
        .end_private_data_tree_snapshot = end_private_data_tree_snapshot,
        .start_nullifier_tree_snapshot = baseRollupInputs.start_nullifier_tree_snapshot,
        .end_nullifier_tree_snapshot = end_nullifier_tree_snapshot,
        .start_contract_tree_snapshot = baseRollupInputs.start_contract_tree_snapshot,
        .end_contract_tree_snapshot = end_contract_tree_snapshot,
        .calldata_hash = calldata_hash,
    };
    return public_inputs;
}

} // namespace aztec3::circuits::rollup::native_base_rollup