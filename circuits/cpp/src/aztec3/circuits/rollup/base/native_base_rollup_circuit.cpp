#include "aztec3/constants.hpp"
#include "barretenberg/crypto/pedersen_hash/pedersen.hpp"
#include "barretenberg/crypto/sha256/sha256.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/stdlib/hash/pedersen/pedersen.hpp"
#include "barretenberg/stdlib/merkle_tree/membership.hpp"
#include "barretenberg/stdlib/merkle_tree/memory_tree.hpp"
#include "barretenberg/stdlib/merkle_tree/merkle_tree.hpp"
#include "init.hpp"
#include "aztec3/circuits/rollup/components/components.hpp"

#include <algorithm>
#include <array>
#include <aztec3/circuits/abis/rollup/base/base_rollup_inputs.hpp>
#include <aztec3/circuits/abis/rollup/base/base_or_merge_rollup_public_inputs.hpp>
#include <aztec3/circuits/abis/rollup/nullifier_leaf_preimage.hpp>
#include <cstdint>
#include <iostream>
#include <tuple>
#include <vector>

namespace aztec3::circuits::rollup::native_base_rollup {

const NT::fr EMPTY_COMMITMENTS_SUBTREE_ROOT = MerkleTree(PRIVATE_DATA_SUBTREE_DEPTH).root();
const NT::fr EMPTY_CONTRACTS_SUBTREE_ROOT = MerkleTree(CONTRACT_SUBTREE_DEPTH).root();
const NT::fr EMPTY_NULLIFIER_SUBTREE_ROOT = MerkleTree(NULLIFIER_SUBTREE_DEPTH).root();

// TODO: can we aggregate proofs if we do not have a working circuit impl

bool verify_kernel_proof(NT::Proof const& kernel_proof)
{
    (void)kernel_proof;
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
AggregationObject aggregate_proofs(BaseRollupInputs const& baseRollupInputs)
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

std::vector<NT::fr> calculate_contract_leaves(BaseRollupInputs const& baseRollupInputs)
{

    std::vector<NT::fr> contract_leaves;

    for (size_t i = 0; i < 2; i++) {

        auto new_contacts = baseRollupInputs.kernel_data[i].public_inputs.end.new_contracts;

        // loop over the new contracts
        // TODO: NOTE: we are currently assuming that there is only going to be one
        for (size_t j = 0; j < new_contacts.size(); j++) {

            NT::address contract_address = new_contacts[j].contract_address;
            NT::address portal_contract_address = new_contacts[j].portal_contract_address;
            NT::fr function_tree_root = new_contacts[j].function_tree_root;

            // Pedersen hash of the 3 fields (contract_address, portal_contract_address, function_tree_root)
            auto contract_leaf = crypto::pedersen_commitment::compress_native(
                { contract_address, portal_contract_address, function_tree_root }, GeneratorIndex::CONTRACT_LEAF);

            // When there is no contract deployment, we should insert a zero leaf into the tree and ignore the
            // member-ship check. This is to ensure that we don't hit "already deployed" errors when we are not
            // deploying contracts. e.g., when we are only calling functions on existing contracts.
            auto to_push = contract_address == NT::address(0) ? NT::fr(0) : contract_leaf;

            contract_leaves.push_back(to_push);
        }
    }

    return contract_leaves;
}

NT::fr calculate_contract_subtree(std::vector<NT::fr> contract_leaves)
{
    MerkleTree contracts_tree = MerkleTree(CONTRACT_SUBTREE_DEPTH);

    // Compute the merkle root of a contract subtree
    // Contracts subtree
    for (size_t i = 0; i < contract_leaves.size(); i++) {
        contracts_tree.update_element(i, contract_leaves[i]);
    }
    return contracts_tree.root();
}

NT::fr calculate_commitments_subtree(DummyComposer& composer, BaseRollupInputs const& baseRollupInputs)
{
    // Leaves that will be added to the new trees
    std::array<NT::fr, KERNEL_NEW_COMMITMENTS_LENGTH * 2> commitment_leaves;

    MerkleTree commitments_tree = MerkleTree(PRIVATE_DATA_SUBTREE_DEPTH);

    for (size_t i = 0; i < 2; i++) {

        auto new_commitments = baseRollupInputs.kernel_data[i].public_inputs.end.new_commitments;

        // Our commitments size MUST be 4 to calculate our subtrees correctly
        composer.do_assert(new_commitments.size() == 4, "New commitments in kernel data must be 4");

        for (size_t j = 0; j < new_commitments.size(); j++) {
            // todo: batch insert
            commitments_tree.update_element(i * KERNEL_NEW_COMMITMENTS_LENGTH + j, new_commitments[j]);
        }
    }

    // Commitments subtree
    return commitments_tree.root();
}

std::array<NT::fr, 2> calculate_calldata_hash(BaseRollupInputs const& baseRollupInputs,
                                              std::vector<NT::fr> const& contract_leaves)
{
    // Compute calldata hashes
    // 22 = (4 + 4 + 1 + 2) * 2 (2 kernels, 4 nullifiers per kernel, 4 commitments per kernel, 1 contract
    // deployments, 2 contracts data fields (size 2 for each) )
    std::array<NT::fr, 22> calldata_hash_inputs;

    for (size_t i = 0; i < 2; i++) {
        // Nullifiers
        auto new_nullifiers = baseRollupInputs.kernel_data[i].public_inputs.end.new_nullifiers;
        auto new_commitments = baseRollupInputs.kernel_data[i].public_inputs.end.new_commitments;
        for (size_t j = 0; j < KERNEL_NEW_COMMITMENTS_LENGTH; j++) {
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
void perform_historical_private_data_tree_membership_checks(DummyComposer& composer,
                                                            BaseRollupInputs const& baseRollupInputs)
{
    // For each of the historic_private_data_tree_membership_checks, we need to do an inclusion proof
    // against the historical root provided in the rollup constants
    auto historic_root = baseRollupInputs.constants.start_tree_of_historic_private_data_tree_roots_snapshot.root;

    for (size_t i = 0; i < 2; i++) {
        NT::fr leaf =
            baseRollupInputs.kernel_data[i]
                .public_inputs.constants.historic_tree_roots.private_historic_tree_roots.private_data_tree_root;
        abis::MembershipWitness<NT, PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT> historic_root_witness =
            baseRollupInputs.historic_private_data_tree_root_membership_witnesses[i];

        components::check_membership(
            composer, leaf, historic_root_witness.leaf_index, historic_root_witness.sibling_path, historic_root);
    }
}

void perform_historical_contract_data_tree_membership_checks(DummyComposer& composer,
                                                             BaseRollupInputs const& baseRollupInputs)
{
    auto historic_root = baseRollupInputs.constants.start_tree_of_historic_contract_tree_roots_snapshot.root;

    for (size_t i = 0; i < 2; i++) {
        NT::fr leaf = baseRollupInputs.kernel_data[i]
                          .public_inputs.constants.historic_tree_roots.private_historic_tree_roots.contract_tree_root;
        abis::MembershipWitness<NT, PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT> historic_root_witness =
            baseRollupInputs.historic_contract_tree_root_membership_witnesses[i];

        components::check_membership(
            composer, leaf, historic_root_witness.leaf_index, historic_root_witness.sibling_path, historic_root);
    }
}

// TODO: right now we are using the hash of NULLIFIER_LEAF{0,0,0} as the empty leaf, however this is an attack vector
// WE MUST after this hackathon change this to be 0, not the hash of some 0 values
NT::fr create_nullifier_subtree(std::array<NullifierLeaf, KERNEL_NEW_NULLIFIERS_LENGTH * 2> const& nullifier_leaves)
{
    // Build a merkle tree of the nullifiers
    MerkleTree nullifier_subtree = MerkleTree(NULLIFIER_SUBTREE_DEPTH);
    for (size_t i = 0; i < nullifier_leaves.size(); i++) {
        // check if the nullifier is zero, if so dont insert
        if (uint256_t(nullifier_leaves[i].value) == uint256_t(0)) {
            nullifier_subtree.update_element(i, fr::zero());
        } else {
            nullifier_subtree.update_element(i, nullifier_leaves[i].hash());
        }
    }

    return nullifier_subtree.root();
}

/**
 * @brief Check non membership of each of the generated nullifiers in the current tree
 *
 * @returns The end nullifier tree root
 */
AppendOnlySnapshot check_nullifier_tree_non_membership_and_insert_to_tree(DummyComposer& composer,
                                                                          BaseRollupInputs const& baseRollupInputs)
{
    // LADIES AND GENTLEMEN The P L A N ( is simple )
    // 1. Get the previous nullifier set setup
    // 2. Check for the first added nullifier that it doesnt exist
    // 3. Update the nullifier set
    // 4. Calculate a new root with the sibling path
    // 5. Use that for the next nullifier check.
    // 6. Iterate for all of em
    // 7. le bosh (profit)

    // BOYS AND GIRLS THE P L A N ( once the first plan is complete )
    // GENERATE OUR NEW NULLIFIER SUBTREE
    // 1. We need to point the new nullifiers to point to the index that the previous nullifier replaced
    // 2. If we receive the 0 nullifier leaf (where all values are 0, we skip insertion and leave a sparse subtree)

    // New nullifier subtree
    std::array<NullifierLeaf, KERNEL_NEW_NULLIFIERS_LENGTH * 2> nullifier_insertion_subtree;

    // This will update on each iteration
    auto current_nullifier_tree_root = baseRollupInputs.start_nullifier_tree_snapshot.root;

    // This will increase with every insertion
    auto start_insertion_index = baseRollupInputs.start_nullifier_tree_snapshot.next_available_leaf_index;
    auto new_index = start_insertion_index;

    // For each kernel circuit
    for (size_t i = 0; i < 2; i++) {

        auto new_nullifiers = baseRollupInputs.kernel_data[i].public_inputs.end.new_nullifiers;
        // For each of our nullifiers
        for (size_t j = 0; j < KERNEL_NEW_NULLIFIERS_LENGTH; j++) {

            // Witness containing index and path
            auto nullifier_index = 4 * i + j;

            auto witness = baseRollupInputs.low_nullifier_membership_witness[nullifier_index];
            // Preimage of the lo-index required for a non-membership proof
            auto low_nullifier_preimage = baseRollupInputs.low_nullifier_leaf_preimages[nullifier_index];
            // Newly created nullifier
            auto nullifier = new_nullifiers[j];

            // TODO: reason about this more strongly, can this cause issues?
            if (nullifier != 0) {

                // Create the nullifier leaf of the new nullifier to be inserted
                NullifierLeaf new_nullifier_leaf = {
                    .value = nullifier,
                    .nextIndex = low_nullifier_preimage.next_index,
                    .nextValue = low_nullifier_preimage.next_value,
                };

                // Assuming populated premier subtree
                if (low_nullifier_preimage.leaf_value == 0 && low_nullifier_preimage.next_value == 0) {
                    // check previous nullifier leaves
                    // TODO: this is a hack, and insecure, we need to fix this
                    bool matched = false;

                    for (size_t k = 0; k < nullifier_index && !matched; k++) {
                        if (nullifier_insertion_subtree[k].value == 0) {
                            continue;
                        }

                        if ((uint256_t(nullifier_insertion_subtree[k].value) < uint256_t(nullifier)) &&
                            (uint256_t(nullifier_insertion_subtree[k].nextValue) > uint256_t(nullifier) ||
                             nullifier_insertion_subtree[k].nextValue == 0)) {

                            matched = true;
                            // Update pointers
                            new_nullifier_leaf.nextIndex = nullifier_insertion_subtree[k].nextIndex;
                            new_nullifier_leaf.nextValue = nullifier_insertion_subtree[k].nextValue;

                            // Update child
                            nullifier_insertion_subtree[k].nextIndex = new_index;
                            nullifier_insertion_subtree[k].nextValue = nullifier;
                        }
                    }

                    // if not matched, our subtree will misformed - we must reject
                    composer.do_assert(matched, "Nullifier subtree is malformed");

                } else {
                    auto is_less_than_nullifier = uint256_t(low_nullifier_preimage.leaf_value) < uint256_t(nullifier);
                    auto is_next_greater_than = uint256_t(low_nullifier_preimage.next_value) > uint256_t(nullifier);

                    if (!(is_less_than_nullifier && is_next_greater_than)) {
                        if (low_nullifier_preimage.next_index != 0 && low_nullifier_preimage.next_value != 0) {
                            composer.do_assert(false, "Nullifier is not in the correct range");
                        }
                    }

                    // Recreate the original low nullifier from the preimage
                    NullifierLeaf original_low_nullifier = NullifierLeaf{
                        .value = low_nullifier_preimage.leaf_value,
                        .nextIndex = low_nullifier_preimage.next_index,
                        .nextValue = low_nullifier_preimage.next_value,
                    };

                    // perform membership check for the low nullifier against the original root
                    components::check_membership<NULLIFIER_TREE_HEIGHT>(composer,
                                                                        original_low_nullifier.hash(),
                                                                        witness.leaf_index,
                                                                        witness.sibling_path,
                                                                        current_nullifier_tree_root);

                    // Calculate the new value of the low_nullifier_leaf
                    NullifierLeaf updated_low_nullifier = NullifierLeaf{ .value = low_nullifier_preimage.leaf_value,
                                                                         .nextIndex = new_index,
                                                                         .nextValue = nullifier };

                    // We need another set of witness values for this
                    current_nullifier_tree_root = components::iterate_through_tree_via_sibling_path(
                        updated_low_nullifier.hash(), witness.leaf_index, witness.sibling_path);
                }

                nullifier_insertion_subtree[nullifier_index] = new_nullifier_leaf;
            } else {
                // 0 case
                NullifierLeaf new_nullifier_leaf = {
                    .value = 0,
                    .nextIndex = 0,
                    .nextValue = 0,
                };
                nullifier_insertion_subtree[nullifier_index] = new_nullifier_leaf;
            }

            // increment insertion index
            new_index = new_index + 1;
        }
    }

    // Create new nullifier subtree to insert into the whole nullifier tree
    auto nullifier_sibling_path = baseRollupInputs.new_nullifiers_subtree_sibling_path;
    auto nullifier_subtree_root = create_nullifier_subtree(nullifier_insertion_subtree);

    // Calculate the new root
    // We are inserting a subtree rather than a full tree here
    auto subtree_index = start_insertion_index >> (NULLIFIER_SUBTREE_DEPTH);
    auto new_root = components::iterate_through_tree_via_sibling_path(
        nullifier_subtree_root, subtree_index, nullifier_sibling_path);

    // Return the new state of the nullifier tree
    return {
        .root = new_root,
        .next_available_leaf_index = new_index,
    };
}

BaseOrMergeRollupPublicInputs base_rollup_circuit(DummyComposer& composer, BaseRollupInputs const& baseRollupInputs)
{
    // Verify the previous kernel proofs
    for (size_t i = 0; i < 2; i++) {
        NT::Proof proof = baseRollupInputs.kernel_data[i].proof;
        composer.do_assert(verify_kernel_proof(proof), "kernel proof verification failed");
    }

    // First we compute the contract tree leaves
    std::vector<NT::fr> contract_leaves = calculate_contract_leaves(baseRollupInputs);

    // Check contracts and commitments subtrees
    NT::fr contracts_tree_subroot = calculate_contract_subtree(contract_leaves);
    NT::fr commitments_tree_subroot = calculate_commitments_subtree(composer, baseRollupInputs);

    // Insert commitment subtrees:
    auto end_private_data_tree_snapshot =
        components::insert_subtree_to_snapshot_tree(composer,
                                                    baseRollupInputs.start_private_data_tree_snapshot,
                                                    baseRollupInputs.new_commitments_subtree_sibling_path,
                                                    EMPTY_COMMITMENTS_SUBTREE_ROOT,
                                                    commitments_tree_subroot,
                                                    PRIVATE_DATA_SUBTREE_DEPTH);

    // Insert contract subtrees:
    auto end_contract_tree_snapshot =
        components::insert_subtree_to_snapshot_tree(composer,
                                                    baseRollupInputs.start_contract_tree_snapshot,
                                                    baseRollupInputs.new_contracts_subtree_sibling_path,
                                                    EMPTY_CONTRACTS_SUBTREE_ROOT,
                                                    contracts_tree_subroot,
                                                    CONTRACT_SUBTREE_DEPTH);

    // Update nullifier tree and insert new subtree
    auto leafIndexNullifierSubtreeDepth =
        baseRollupInputs.start_nullifier_tree_snapshot.next_available_leaf_index >> NULLIFIER_SUBTREE_DEPTH;
    components::check_membership(composer,
                                 EMPTY_NULLIFIER_SUBTREE_ROOT,
                                 leafIndexNullifierSubtreeDepth,
                                 baseRollupInputs.new_nullifiers_subtree_sibling_path,
                                 baseRollupInputs.start_nullifier_tree_snapshot.root);
    AppendOnlySnapshot end_nullifier_tree_snapshot =
        check_nullifier_tree_non_membership_and_insert_to_tree(composer, baseRollupInputs);

    // Calculate the overall calldata hash
    std::array<NT::fr, 2> calldata_hash = calculate_calldata_hash(baseRollupInputs, contract_leaves);

    // Perform membership checks that the notes provided exist within the historic trees data
    perform_historical_private_data_tree_membership_checks(composer, baseRollupInputs);
    perform_historical_contract_data_tree_membership_checks(composer, baseRollupInputs);

    AggregationObject aggregation_object = aggregate_proofs(baseRollupInputs);

    BaseOrMergeRollupPublicInputs public_inputs = {
        .rollup_type = abis::BASE_ROLLUP_TYPE,
        .rollup_subtree_height = fr(0),
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