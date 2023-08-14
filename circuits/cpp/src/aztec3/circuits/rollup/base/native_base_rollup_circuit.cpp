#include "init.hpp"

#include "aztec3/circuits/abis/membership_witness.hpp"
#include "aztec3/circuits/abis/public_data_read.hpp"
#include "aztec3/circuits/abis/public_data_update_request.hpp"
#include "aztec3/circuits/abis/rollup/base/base_or_merge_rollup_public_inputs.hpp"
#include "aztec3/circuits/abis/rollup/base/base_rollup_inputs.hpp"
#include "aztec3/circuits/hash.hpp"
#include "aztec3/circuits/rollup/components/components.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/circuit_errors.hpp"

#include <barretenberg/barretenberg.hpp>

#include <algorithm>
#include <array>
#include <cstdint>
#include <iostream>
#include <tuple>
#include <vector>


namespace aztec3::circuits::rollup::native_base_rollup {

// Used when calling library functions like `check_membership` which have their own generic error code.
// So we pad this in front of the error message to identify where the error originally came from.
const std::string BASE_CIRCUIT_ERROR_MESSAGE_BEGINNING = "base_rollup_circuit: ";

NT::fr calculate_empty_tree_root(const size_t depth)
{
    MemoryStore empty_tree_store;
    MerkleTree const empty_tree = MerkleTree(empty_tree_store, depth);
    return empty_tree.root();
}

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
        for (auto& leaf_preimage : new_contacts) {
            // When there is no contract deployment, we should insert a zero leaf into the tree and ignore the
            // member-ship check. This is to ensure that we don't hit "already deployed" errors when we are not
            // deploying contracts. e.g., when we are only calling functions on existing contracts.
            auto to_push = leaf_preimage.contract_address == NT::address(0) ? NT::fr(0) : leaf_preimage.hash();
            contract_leaves.push_back(to_push);
        }
    }

    return contract_leaves;
}

NT::fr calculate_contract_subtree(std::vector<NT::fr> contract_leaves)
{
    MemoryStore contracts_tree_store;
    MerkleTree contracts_tree(contracts_tree_store, CONTRACT_SUBTREE_HEIGHT);


    // Compute the merkle root of a contract subtree
    // Contracts subtree
    for (size_t i = 0; i < contract_leaves.size(); i++) {
        contracts_tree.update_element(i, contract_leaves[i]);
    }
    return contracts_tree.root();
}

NT::fr calculate_commitments_subtree(DummyBuilder& builder, BaseRollupInputs const& baseRollupInputs)
{
    MemoryStore commitments_tree_store;
    MerkleTree commitments_tree(commitments_tree_store, PRIVATE_DATA_SUBTREE_HEIGHT);


    for (size_t i = 0; i < 2; i++) {
        auto new_commitments = baseRollupInputs.kernel_data[i].public_inputs.end.new_commitments;

        // Our commitments size MUST be 4 to calculate our subtrees correctly
        builder.do_assert(new_commitments.size() == MAX_NEW_COMMITMENTS_PER_TX,
                          "New commitments in kernel data must be MAX_NEW_COMMITMENTS_PER_TX (see constants.hpp)",
                          CircuitErrorCode::BASE__INCORRECT_NUM_OF_NEW_COMMITMENTS);

        for (size_t j = 0; j < new_commitments.size(); j++) {
            // todo: batch insert
            commitments_tree.update_element(i * MAX_NEW_COMMITMENTS_PER_TX + j, new_commitments[j]);
        }
    }

    // Commitments subtree
    return commitments_tree.root();
}

/**
 * @brief Check all of the provided commitments against the historical tree roots
 *
 * @param constantBaseRollupData
 * @param baseRollupInputs
 */
void perform_historical_blocks_tree_membership_checks(DummyBuilder& builder, BaseRollupInputs const& baseRollupInputs)
{
    // For each of the historic_private_data_tree_membership_checks, we need to do an inclusion proof
    // against the historical root provided in the rollup constants
    auto historic_root = baseRollupInputs.constants.start_historic_blocks_tree_roots_snapshot.root;

    for (size_t i = 0; i < 2; i++) {
        // Rebuild the block hash
        auto historic_block = baseRollupInputs.kernel_data[i].public_inputs.constants.block_data;

        auto private_data_tree_root = historic_block.private_data_tree_root;
        auto nullifier_tree_root = historic_block.nullifier_tree_root;
        auto contract_tree_root = historic_block.contract_tree_root;
        auto l1_to_l2_data_tree_root = historic_block.l1_to_l2_messages_tree_root;
        auto public_data_tree_root = historic_block.public_data_tree_root;

        auto previous_block_hash = compute_block_hash<NT>(historic_block.global_variables_hash,
                                                          private_data_tree_root,
                                                          nullifier_tree_root,
                                                          contract_tree_root,
                                                          l1_to_l2_data_tree_root,
                                                          public_data_tree_root);

        abis::MembershipWitness<NT, HISTORIC_BLOCKS_TREE_HEIGHT> const historic_root_witness =
            baseRollupInputs.historic_blocks_tree_root_membership_witnesses[i];

        check_membership<NT>(
            builder,
            previous_block_hash,
            historic_root_witness.leaf_index,
            historic_root_witness.sibling_path,
            historic_root,
            format(BASE_CIRCUIT_ERROR_MESSAGE_BEGINNING,
                   "historical root is in rollup constants but not in historic block tree roots at kernel input ",
                   i,
                   " to this base rollup circuit"));
    }
}

NT::fr create_nullifier_subtree(
    std::array<NullifierLeafPreimage, MAX_NEW_NULLIFIERS_PER_TX * 2> const& nullifier_leaves)
{
    // Build a merkle tree of the nullifiers
    MemoryStore nullifier_subtree_store;
    MerkleTree nullifier_subtree(nullifier_subtree_store, NULLIFIER_SUBTREE_HEIGHT);
    for (size_t i = 0; i < nullifier_leaves.size(); i++) {
        // hash() checks if nullifier is empty (and if so returns 0)
        nullifier_subtree.update_element(i, nullifier_leaves[i].hash());
    }

    return nullifier_subtree.root();
}

/**
 * @brief Check non membership of each of the generated nullifiers in the current tree
 *
 * @returns The end nullifier tree root
 */
AppendOnlySnapshot check_nullifier_tree_non_membership_and_insert_to_tree(DummyBuilder& builder,
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
    std::array<NullifierLeafPreimage, MAX_NEW_NULLIFIERS_PER_TX * 2> nullifier_insertion_subtree;

    // This will update on each iteration
    auto current_nullifier_tree_root = baseRollupInputs.start_nullifier_tree_snapshot.root;

    // This will increase with every insertion
    auto start_insertion_index = baseRollupInputs.start_nullifier_tree_snapshot.next_available_leaf_index;
    auto new_index = start_insertion_index;

    // For each kernel circuit
    for (size_t i = 0; i < 2; i++) {
        auto new_nullifiers = baseRollupInputs.kernel_data[i].public_inputs.end.new_nullifiers;
        // For each of our nullifiers
        for (size_t j = 0; j < MAX_NEW_NULLIFIERS_PER_TX; j++) {
            // Witness containing index and path
            auto nullifier_index = i * MAX_NEW_NULLIFIERS_PER_TX + j;

            auto witness = baseRollupInputs.low_nullifier_membership_witness[nullifier_index];
            // Preimage of the lo-index required for a non-membership proof
            auto low_nullifier_preimage = baseRollupInputs.low_nullifier_leaf_preimages[nullifier_index];
            // Newly created nullifier
            auto nullifier = new_nullifiers[j];

            // TODO(maddiaa): reason about this more strongly, can this cause issues?
            if (nullifier != 0) {
                // Create the nullifier leaf of the new nullifier to be inserted
                NullifierLeafPreimage new_nullifier_leaf = {
                    .leaf_value = nullifier,
                    .next_value = low_nullifier_preimage.next_value,
                    .next_index = low_nullifier_preimage.next_index,
                };

                // Assuming populated premier subtree
                if (low_nullifier_preimage.is_empty()) {
                    // check previous nullifier leaves
                    bool matched = false;

                    for (size_t k = 0; k < nullifier_index && !matched; k++) {
                        if (nullifier_insertion_subtree[k].is_empty()) {
                            continue;
                        }

                        if ((uint256_t(nullifier_insertion_subtree[k].leaf_value) < uint256_t(nullifier)) &&
                            (uint256_t(nullifier_insertion_subtree[k].next_value) > uint256_t(nullifier) ||
                             nullifier_insertion_subtree[k].next_value == 0)) {
                            matched = true;
                            // Update pointers
                            new_nullifier_leaf.next_index = nullifier_insertion_subtree[k].next_index;
                            new_nullifier_leaf.next_value = nullifier_insertion_subtree[k].next_value;

                            // Update child
                            nullifier_insertion_subtree[k].next_index = new_index;
                            nullifier_insertion_subtree[k].next_value = nullifier;
                        }
                    }

                    // if not matched, our subtree will misformed - we must reject
                    builder.do_assert(
                        matched, "Nullifier subtree is malformed", CircuitErrorCode::BASE__INVALID_NULLIFIER_SUBTREE);

                } else {
                    auto is_less_than_nullifier = uint256_t(low_nullifier_preimage.leaf_value) < uint256_t(nullifier);
                    auto is_next_greater_than = uint256_t(low_nullifier_preimage.next_value) > uint256_t(nullifier);

                    if (!(is_less_than_nullifier && is_next_greater_than)) {
                        if (low_nullifier_preimage.next_index != 0 && low_nullifier_preimage.next_value != 0) {
                            builder.do_assert(false,
                                              format("Nullifier is not in the correct range. \n  ",
                                                     "is_less_than_nullifier ",
                                                     is_less_than_nullifier,
                                                     "\n is_next_greater_than ",
                                                     is_next_greater_than,
                                                     "\n low_nullifier_preimage.leaf_value ",
                                                     low_nullifier_preimage.leaf_value,
                                                     "\n low_nullifier_preimage.next_index ",
                                                     low_nullifier_preimage.next_index,
                                                     "\n low_nullifier_preimage.next_value ",
                                                     low_nullifier_preimage.next_value),
                                              CircuitErrorCode::BASE__INVALID_NULLIFIER_RANGE);
                        }
                    }

                    // Recreate the original low nullifier from the preimage
                    auto const original_low_nullifier = NullifierLeafPreimage{
                        .leaf_value = low_nullifier_preimage.leaf_value,
                        .next_value = low_nullifier_preimage.next_value,
                        .next_index = low_nullifier_preimage.next_index,
                    };

                    // perform membership check for the low nullifier against the original root
                    check_membership<NT, DummyBuilder, NULLIFIER_TREE_HEIGHT>(
                        builder,
                        original_low_nullifier.hash(),
                        witness.leaf_index,
                        witness.sibling_path,
                        current_nullifier_tree_root,
                        format(BASE_CIRCUIT_ERROR_MESSAGE_BEGINNING, "low nullifier not in nullifier tree"));

                    // Calculate the new value of the low_nullifier_leaf
                    auto const updated_low_nullifier =
                        NullifierLeafPreimage{ .leaf_value = low_nullifier_preimage.leaf_value,
                                               .next_value = nullifier,
                                               .next_index = new_index };

                    // We need another set of witness values for this
                    current_nullifier_tree_root = root_from_sibling_path<NT>(
                        updated_low_nullifier.hash(), witness.leaf_index, witness.sibling_path);
                }

                nullifier_insertion_subtree[nullifier_index] = new_nullifier_leaf;
            } else {
                // 0 case
                NullifierLeafPreimage const new_nullifier_leaf = { .leaf_value = 0, .next_value = 0, .next_index = 0 };
                nullifier_insertion_subtree[nullifier_index] = new_nullifier_leaf;
            }

            // increment insertion index
            new_index = new_index + 1;
        }
    }

    // Check that the new subtree is to be inserted at the next location, and is empty currently
    const auto empty_nullifier_subtree_root = components::calculate_empty_tree_root(NULLIFIER_SUBTREE_HEIGHT);
    auto leafIndexNullifierSubtreeDepth =
        baseRollupInputs.start_nullifier_tree_snapshot.next_available_leaf_index >> NULLIFIER_SUBTREE_HEIGHT;
    check_membership<NT>(
        builder,
        empty_nullifier_subtree_root,
        leafIndexNullifierSubtreeDepth,
        baseRollupInputs.new_nullifiers_subtree_sibling_path,
        current_nullifier_tree_root,
        format(BASE_CIRCUIT_ERROR_MESSAGE_BEGINNING,
               "nullifier tree not empty at location where the new nullifier subtree would be inserted"));

    // Create new nullifier subtree to insert into the whole nullifier tree
    auto nullifier_sibling_path = baseRollupInputs.new_nullifiers_subtree_sibling_path;
    auto nullifier_subtree_root = create_nullifier_subtree(nullifier_insertion_subtree);

    // Calculate the new root
    // We are inserting a subtree rather than a full tree here
    auto subtree_index = start_insertion_index >> (NULLIFIER_SUBTREE_HEIGHT);
    auto new_root = root_from_sibling_path<NT>(nullifier_subtree_root, subtree_index, nullifier_sibling_path);

    // Return the new state of the nullifier tree
    return {
        .root = new_root,
        .next_available_leaf_index = new_index,
    };
}

fr insert_public_data_update_requests(
    DummyBuilder& builder,
    fr tree_root,
    std::array<abis::PublicDataUpdateRequest<NT>, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX> const&
        public_data_update_requests,
    size_t witnesses_offset,
    std::array<std::array<fr, PUBLIC_DATA_TREE_HEIGHT>, 2 * MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX> const& witnesses)
{
    auto root = tree_root;

    for (size_t i = 0; i < MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX; ++i) {
        const auto& state_write = public_data_update_requests[i];
        const auto& witness = witnesses[i + witnesses_offset];

        if (state_write.is_empty()) {
            continue;
        }

        check_membership<NT>(
            builder,
            state_write.old_value,
            state_write.leaf_index,
            witness,
            root,
            format(BASE_CIRCUIT_ERROR_MESSAGE_BEGINNING, "validate_public_data_update_requests index ", i));

        root = root_from_sibling_path<NT>(state_write.new_value, state_write.leaf_index, witness);
    }

    return root;
}

void validate_public_data_reads(
    DummyBuilder& builder,
    fr tree_root,
    std::array<abis::PublicDataRead<NT>, MAX_PUBLIC_DATA_READS_PER_TX> const& public_data_reads,
    size_t witnesses_offset,
    std::array<std::array<fr, PUBLIC_DATA_TREE_HEIGHT>, 2 * MAX_PUBLIC_DATA_READS_PER_TX> const& witnesses)
{
    for (size_t i = 0; i < MAX_PUBLIC_DATA_READS_PER_TX; ++i) {
        const auto& public_data_read = public_data_reads[i];
        const auto& witness = witnesses[i + witnesses_offset];

        if (public_data_read.is_empty()) {
            continue;
        }

        check_membership<NT>(
            builder,
            public_data_read.value,
            public_data_read.leaf_index,
            witness,
            tree_root,
            format(BASE_CIRCUIT_ERROR_MESSAGE_BEGINNING, "validate_public_data_reads index ", i + witnesses_offset));
    }
};

fr validate_and_process_public_state(DummyBuilder& builder, BaseRollupInputs const& baseRollupInputs)
{
    // Process public data reads and public data update requests for left input
    validate_public_data_reads(builder,
                               baseRollupInputs.start_public_data_tree_root,
                               baseRollupInputs.kernel_data[0].public_inputs.end.public_data_reads,
                               0,
                               baseRollupInputs.new_public_data_reads_sibling_paths);

    auto mid_public_data_tree_root = insert_public_data_update_requests(
        builder,
        baseRollupInputs.start_public_data_tree_root,
        baseRollupInputs.kernel_data[0].public_inputs.end.public_data_update_requests,
        0,
        baseRollupInputs.new_public_data_update_requests_sibling_paths);

    // Process public data reads and public data update requests for right input using the resulting tree root from the
    // left one
    validate_public_data_reads(builder,
                               mid_public_data_tree_root,
                               baseRollupInputs.kernel_data[1].public_inputs.end.public_data_reads,
                               MAX_PUBLIC_DATA_READS_PER_TX,
                               baseRollupInputs.new_public_data_reads_sibling_paths);

    auto end_public_data_tree_root = insert_public_data_update_requests(
        builder,
        mid_public_data_tree_root,
        baseRollupInputs.kernel_data[1].public_inputs.end.public_data_update_requests,
        MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
        baseRollupInputs.new_public_data_update_requests_sibling_paths);

    return end_public_data_tree_root;
}

BaseOrMergeRollupPublicInputs base_rollup_circuit(DummyBuilder& builder, BaseRollupInputs const& baseRollupInputs)
{
    // Verify the previous kernel proofs
    for (size_t i = 0; i < 2; i++) {
        NT::Proof const proof = baseRollupInputs.kernel_data[i].proof;
        builder.do_assert(verify_kernel_proof(proof),
                          "kernel proof verification failed",
                          CircuitErrorCode::BASE__KERNEL_PROOF_VERIFICATION_FAILED);
    }

    // Verify the kernel chain_id and versions
    for (size_t i = 0; i < 2; i++) {
        builder.do_assert(baseRollupInputs.kernel_data[i].public_inputs.constants.tx_context.chain_id ==
                              baseRollupInputs.constants.global_variables.chain_id,
                          "kernel chain_id does not match the rollup chain_id",
                          CircuitErrorCode::BASE__INVALID_CHAIN_ID);
        builder.do_assert(baseRollupInputs.kernel_data[i].public_inputs.constants.tx_context.version ==
                              baseRollupInputs.constants.global_variables.version,
                          "kernel version does not match the rollup version",
                          CircuitErrorCode::BASE__INVALID_VERSION);
    }

    // First we compute the contract tree leaves
    std::vector<NT::fr> const contract_leaves = calculate_contract_leaves(baseRollupInputs);

    // Check contracts and commitments subtrees
    NT::fr const contracts_tree_subroot = calculate_contract_subtree(contract_leaves);
    NT::fr const commitments_tree_subroot = calculate_commitments_subtree(builder, baseRollupInputs);

    // Insert commitment subtrees:
    const auto empty_commitments_subtree_root = components::calculate_empty_tree_root(PRIVATE_DATA_SUBTREE_HEIGHT);
    auto end_private_data_tree_snapshot = components::insert_subtree_to_snapshot_tree(
        builder,
        baseRollupInputs.start_private_data_tree_snapshot,
        baseRollupInputs.new_commitments_subtree_sibling_path,
        empty_commitments_subtree_root,
        commitments_tree_subroot,
        PRIVATE_DATA_SUBTREE_HEIGHT,
        format(BASE_CIRCUIT_ERROR_MESSAGE_BEGINNING,
               "private data tree not empty at location where the new commitment subtree would be inserted"));
    // Insert contract subtrees:
    const auto empty_contracts_subtree_root = components::calculate_empty_tree_root(CONTRACT_SUBTREE_HEIGHT);
    auto end_contract_tree_snapshot = components::insert_subtree_to_snapshot_tree(
        builder,
        baseRollupInputs.start_contract_tree_snapshot,
        baseRollupInputs.new_contracts_subtree_sibling_path,
        empty_contracts_subtree_root,
        contracts_tree_subroot,
        CONTRACT_SUBTREE_HEIGHT,
        format(BASE_CIRCUIT_ERROR_MESSAGE_BEGINNING,
               "contract tree not empty at location where the new contract subtree would be inserted"));

    // Insert nullifiers:
    AppendOnlySnapshot const end_nullifier_tree_snapshot =
        check_nullifier_tree_non_membership_and_insert_to_tree(builder, baseRollupInputs);

    // Validate public public data reads and public data update requests, and update public data tree
    fr const end_public_data_tree_root = validate_and_process_public_state(builder, baseRollupInputs);

    // Calculate the overall calldata hash
    std::array<NT::fr, NUM_FIELDS_PER_SHA256> const calldata_hash =
        components::compute_kernels_calldata_hash(baseRollupInputs.kernel_data);

    // Perform membership checks that the notes provided exist within the historic trees data
    perform_historical_blocks_tree_membership_checks(builder, baseRollupInputs);

    AggregationObject const aggregation_object = aggregate_proofs(baseRollupInputs);

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
        .start_public_data_tree_root = baseRollupInputs.start_public_data_tree_root,
        .end_public_data_tree_root = end_public_data_tree_root,
        .calldata_hash = calldata_hash,
    };
    return public_inputs;
}

}  // namespace aztec3::circuits::rollup::native_base_rollup
