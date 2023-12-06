use crate::abis::nullifier_leaf_preimage::NullifierLeafPreimage;
use crate::abis::append_only_tree_snapshot::AppendOnlyTreeSnapshot;
use crate::abis::constant_rollup_data::ConstantRollupData;
use crate::abis::base_or_merge_rollup_public_inputs::{BaseOrMergeRollupPublicInputs, BASE_ROLLUP_TYPE};
use crate::merkle_tree::{calculate_subtree, calculate_empty_tree_root};
use crate::components;
use dep::types::utils::uint256::U256;
use dep::types::abis::public_data_update_request::PublicDataUpdateRequest;
use dep::types::abis::public_data_read::PublicDataRead;
use dep::types::mocked::{AggregationObject, Proof};
use dep::types::constants::{
    MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP,
    NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH,
    NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH,
    CONTRACT_SUBTREE_SIBLING_PATH_LENGTH,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_BASE_ROLLUP,
    MAX_PUBLIC_DATA_READS_PER_BASE_ROLLUP,
    PUBLIC_DATA_TREE_HEIGHT,
    KERNELS_PER_BASE_ROLLUP,
    MAX_NEW_CONTRACTS_PER_TX,
    NOTE_HASH_SUBTREE_HEIGHT,
    CONTRACT_SUBTREE_HEIGHT,
    NUM_FIELDS_PER_SHA256,
    MAX_NEW_COMMITMENTS_PER_TX,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
    MAX_PUBLIC_DATA_READS_PER_TX,
    MAX_NEW_NULLIFIERS_PER_TX,
    NUM_ENCRYPTED_LOGS_HASHES_PER_TX,
    MAX_NEW_L2_TO_L1_MSGS_PER_TX,
    NUM_UNENCRYPTED_LOGS_HASHES_PER_TX,
    NULLIFIER_SUBTREE_HEIGHT,
    NULLIFIER_TREE_HEIGHT,
};
use dep::types::abis::previous_kernel_data::PreviousKernelData;
use dep::types::abis::membership_witness::{NullifierMembershipWitness, MembershipWitness};
use dep::types::abis::membership_witness::ArchiveRootMembershipWitness;

struct BaseRollupInputs {
    kernel_data: [PreviousKernelData; KERNELS_PER_BASE_ROLLUP],
    start_note_hash_tree_snapshot: AppendOnlyTreeSnapshot,
    start_nullifier_tree_snapshot: AppendOnlyTreeSnapshot,
    start_contract_tree_snapshot: AppendOnlyTreeSnapshot,
    start_public_data_tree_root: Field,
    archive_snapshot: AppendOnlyTreeSnapshot,

    sorted_new_nullifiers: [Field; MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP],
    sorted_new_nullifiers_indexes: [u32; MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP],
    low_nullifier_leaf_preimages: [NullifierLeafPreimage; MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP],
    low_nullifier_membership_witness: [NullifierMembershipWitness; MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP],

    // For inserting the new subtrees into their respective trees:
    // Note: the insertion leaf index can be derived from the above snapshots' `next_available_leaf_index` values.
    new_commitments_subtree_sibling_path: [Field; NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH],
    new_nullifiers_subtree_sibling_path: [Field; NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH],
    new_contracts_subtree_sibling_path: [Field; CONTRACT_SUBTREE_SIBLING_PATH_LENGTH],
    new_public_data_update_requests_sibling_paths: [[Field; PUBLIC_DATA_TREE_HEIGHT]; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_BASE_ROLLUP],
    new_public_data_reads_sibling_paths: [[Field; PUBLIC_DATA_TREE_HEIGHT]; MAX_PUBLIC_DATA_READS_PER_BASE_ROLLUP],
    
    archive_root_membership_witnesses: [ArchiveRootMembershipWitness; KERNELS_PER_BASE_ROLLUP],
    
    constants: ConstantRollupData,
}

impl BaseRollupInputs {
    pub fn base_rollup_circuit(self) -> BaseOrMergeRollupPublicInputs {        
        // Verify the previous kernel proofs
        for i in 0..KERNELS_PER_BASE_ROLLUP {
            let proof = self.kernel_data[i].proof;
            assert(verify_kernel_proof(proof), "kernel proof verification failed");
        };

        // Verify the kernel chain_id and versions
        for i in 0..KERNELS_PER_BASE_ROLLUP {
            assert(self.kernel_data[i].public_inputs.constants.tx_context.chain_id ==
                   self.constants.global_variables.chain_id, "kernel chain_id does not match the rollup chain_id");
            assert(self.kernel_data[i].public_inputs.constants.tx_context.version ==
                              self.constants.global_variables.version, "kernel version does not match the rollup version");
        };

        // First we compute the contract tree leaves
        let contract_leaves = self.calculate_contract_leaves();
        let contracts_tree_subroot = self.calculate_contract_subtree(contract_leaves);

        let commitments_tree_subroot = self.calculate_commitments_subtree();

        let empty_commitments_subtree_root = calculate_empty_tree_root(NOTE_HASH_SUBTREE_HEIGHT);

        let end_note_hash_tree_snapshot = components::insert_subtree_to_snapshot_tree(
            self.start_note_hash_tree_snapshot,
            self.new_commitments_subtree_sibling_path,
            empty_commitments_subtree_root,
            commitments_tree_subroot,
            NOTE_HASH_SUBTREE_HEIGHT as u8,
        );

        // Insert contract subtrees:
        let empty_contracts_subtree_root = calculate_empty_tree_root(CONTRACT_SUBTREE_HEIGHT);
        let end_contract_tree_snapshot = components::insert_subtree_to_snapshot_tree(
            self.start_contract_tree_snapshot,
            self.new_contracts_subtree_sibling_path,
            empty_contracts_subtree_root,
            contracts_tree_subroot,
            CONTRACT_SUBTREE_HEIGHT as u8,
        );

        // Insert nullifiers:
        let end_nullifier_tree_snapshot = self.check_nullifier_tree_non_membership_and_insert_to_tree();

        // Validate public public data reads and public data update requests, and update public data tree
        let end_public_data_tree_root = self.validate_and_process_public_state();

        // Calculate the overall calldata hash
        let calldata_hash = BaseRollupInputs::components_compute_kernel_calldata_hash(self.kernel_data);
        
        // Perform membership checks that the notes provided exist within the historical trees data
        self.perform_archive_membership_checks();

        let aggregation_object = self.aggregate_proofs();

        BaseOrMergeRollupPublicInputs {
            rollup_type : BASE_ROLLUP_TYPE,
            rollup_subtree_height : 0,
            end_aggregation_object : aggregation_object,
            constants : self.constants,
            start_note_hash_tree_snapshot : self.start_note_hash_tree_snapshot,
            end_note_hash_tree_snapshot : end_note_hash_tree_snapshot,
            start_nullifier_tree_snapshot : self.start_nullifier_tree_snapshot,
            end_nullifier_tree_snapshot : end_nullifier_tree_snapshot,
            start_contract_tree_snapshot : self.start_contract_tree_snapshot,
            end_contract_tree_snapshot : end_contract_tree_snapshot,
            start_public_data_tree_root : self.start_public_data_tree_root,
            end_public_data_tree_root : end_public_data_tree_root,
            calldata_hash : calldata_hash,
        }
    }

    fn calculate_contract_leaves(self) -> [Field; NUM_CONTRACT_LEAVES] {
        let mut contract_leaves = [0; NUM_CONTRACT_LEAVES];
        for i in 0..KERNELS_PER_BASE_ROLLUP {
            let new_contracts = self.kernel_data[i].public_inputs.end.new_contracts;

            // loop over the new contracts
            for j in 0..new_contracts.len() {
                let leaf_preimage = new_contracts[j];
                // When there is no contract deployment, we should insert a zero leaf into the tree and ignore the
                // member-ship check. This is to ensure that we don't hit "already deployed" errors when we are not
                // deploying contracts. e.g., when we are only calling functions on existing contracts.
                let to_push = if leaf_preimage.contract_address.to_field() == 0  {
                    0
                } else {    
                    leaf_preimage.hash()
                };
                contract_leaves[i*new_contracts.len() + j] = to_push;
            }
        }

        contract_leaves
    }

    // TODO(Kev): This should say calculate_contract_subtree_root
    // Cpp code says calculate_contract_subtree, so I'm leaving it as is for now
    fn calculate_contract_subtree(self, leaves : [Field; NUM_CONTRACT_LEAVES]) -> Field {
        calculate_subtree(leaves)
    }

    // TODO(Kev): This should say calculate_commitments_subtree_root
    // Cpp code says calculate_commitments_subtree, so I'm leaving it as is for now
    fn calculate_commitments_subtree(self) -> Field {
        let mut commitment_tree_leaves = [0; NOTE_HASH_SUBTREE_WIDTH];
        
        for i in 0..KERNELS_PER_BASE_ROLLUP {
            let new_commitments = self.kernel_data[i].public_inputs.end.new_commitments;

            // Our commitments size MUST be 4 to calculate our subtrees correctly
            assert(new_commitments.len() == MAX_NEW_COMMITMENTS_PER_TX, "New commitments in kernel data must be MAX_NEW_COMMITMENTS_PER_TX (see constants.hpp)");

            for j in 0..MAX_NEW_COMMITMENTS_PER_TX {
                // TODO(Maddiaa): batch insert
                commitment_tree_leaves[i * MAX_NEW_COMMITMENTS_PER_TX + j] =  new_commitments[j];
            }
        }

        calculate_subtree(commitment_tree_leaves)
    }

    fn check_nullifier_tree_non_membership_and_insert_to_tree(self) -> AppendOnlyTreeSnapshot {
        let mut new_nullifiers = [0; MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP];

        for i in 0..2 {
            for j in 0..MAX_NEW_NULLIFIERS_PER_TX {
                new_nullifiers[i * MAX_NEW_NULLIFIERS_PER_TX + j] = self.kernel_data[i].public_inputs.end.new_nullifiers[j];
            }
        };

        crate::indexed_tree::batch_insert(
            self.start_nullifier_tree_snapshot,
            new_nullifiers,
            self.sorted_new_nullifiers,
            self.sorted_new_nullifiers_indexes,
            self.new_nullifiers_subtree_sibling_path,
            self.low_nullifier_leaf_preimages,
            self.low_nullifier_membership_witness.map(|witness: NullifierMembershipWitness| {
                MembershipWitness {
                    leaf_index: witness.leaf_index,
                    sibling_path: witness.sibling_path,
                }
            }),
            |a: Field, b: Field| {a == b}, // Nullifier equals
            |nullifier: Field| {nullifier == 0}, // Nullifier is zero
            |leaf: NullifierLeafPreimage| {leaf.hash()}, // Hash leaf
            |low_leaf: NullifierLeafPreimage, nullifier: Field| { // Is valid low leaf
                let is_less_than_nullifier = full_field_less_than(low_leaf.leaf_value, nullifier);
                let is_next_greater_than = full_field_less_than(nullifier, low_leaf.next_value);

                (!low_leaf.is_empty()) & is_less_than_nullifier & (
                    is_next_greater_than | 
                    ((low_leaf.next_index == 0) & (low_leaf.next_value == 0))
                )
            },
            |low_leaf: NullifierLeafPreimage, nullifier: Field, nullifier_index: u32| { // Update low leaf
                NullifierLeafPreimage{
                    leaf_value : low_leaf.leaf_value,
                    next_value : nullifier,
                    next_index : nullifier_index, 
                }
            },
            |nullifier: Field, low_leaf: NullifierLeafPreimage| { // Build insertion leaf
                NullifierLeafPreimage {
                    leaf_value : nullifier,
                    next_value : low_leaf.next_value,
                    next_index : low_leaf.next_index,
                }
            },
            [0; NULLIFIER_SUBTREE_HEIGHT],
            [0; NULLIFIER_TREE_HEIGHT],
        )
    }

    fn create_nullifier_subtree<N>(self, leaves: [NullifierLeafPreimage; N]) -> Field {
        calculate_subtree(leaves.map(|leaf:NullifierLeafPreimage| leaf.hash()))
    }

    fn validate_and_process_public_state(self) -> Field {
        // TODO(#2521) - data read validation should happen against the current state of the tx and not the start state.
        // Blocks all interesting usecases that read and write to the same public state in the same tx.
        // https://aztecprotocol.slack.com/archives/C02M7VC7TN0/p1695809629015719?thread_ts=1695653252.007339&cid=C02M7VC7TN0


        // Process public data reads and public data update requests for left input
        // validate_public_data_reads(
        //                            self.start_public_data_tree_root,
        //                            self.kernel_data[0].public_inputs.end.public_data_reads,
        //                            0,
        //                            self.new_public_data_reads_sibling_paths);

        let mid_public_data_tree_root = insert_public_data_update_requests(
            self.start_public_data_tree_root,
            self.kernel_data[0].public_inputs.end.public_data_update_requests,
            0,
            self.new_public_data_update_requests_sibling_paths
        );


        // TODO(#2521) - data read validation should happen against the current state of the tx and not the start state.
        // Blocks all interesting usecases that read and write to the same public state in the same tx.
        // https://aztecprotocol.slack.com/archives/C02M7VC7TN0/p1695809629015719?thread_ts=1695653252.007339&cid=C02M7VC7TN0


        // Process public data reads and public data update requests for right input using the resulting tree root from the
        // left one
        // validate_public_data_reads(
        //                            mid_public_data_tree_root,
        //                            baseRollupInputs.kernel_data[1].public_inputs.end.public_data_reads,
        //                            MAX_PUBLIC_DATA_READS_PER_TX,
        //                            baseRollupInputs.new_public_data_reads_sibling_paths);

        let end_public_data_tree_root = insert_public_data_update_requests(
            mid_public_data_tree_root,
            self.kernel_data[1].public_inputs.end.public_data_update_requests,
            MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
            self.new_public_data_update_requests_sibling_paths
        );

        end_public_data_tree_root
    }

    // Computes the calldata hash for a base rollup
    // TODO(Kev): move this into components module
    // TODO(Alvaro): This is too slow for brillig without the array optimization
    fn components_compute_kernel_calldata_hash(kernel_data : [PreviousKernelData; KERNELS_PER_BASE_ROLLUP]) -> [Field; NUM_FIELDS_PER_SHA256]{
        // Compute calldata hashes
        // Consist of 2 kernels
        // 2 * MAX_NEW_COMMITMENTS_PER_TX fields for commitments
        // 2 * MAX_NEW_NULLIFIERS_PER_TX fields for nullifiers
        // 8 public data update requests (4 per kernel) -> 16 fields
        // 4 l2 -> l1 messages (2 per kernel) -> 4 fields
        // 2 contract deployments (1 per kernel) -> 6 fields
        // 2 encrypted logs hashes (1 per kernel) -> 4 fields --> 2 sha256 hashes --> 64 bytes
        // 2 unencrypted logs hashes (1 per kernel) -> 4 fields --> 2 sha256 hashes --> 64 bytes
        let mut calldata_hash_inputs = [0; CALLDATA_HASH_INPUT_SIZE];

        for i in 0..KERNELS_PER_BASE_ROLLUP {
            let new_commitments = kernel_data[i].public_inputs.end.new_commitments;
            let new_nullifiers = kernel_data[i].public_inputs.end.new_nullifiers;
            let public_data_update_requests = kernel_data[i].public_inputs.end.public_data_update_requests;
            let newL2ToL1msgs = kernel_data[i].public_inputs.end.new_l2_to_l1_msgs;
            let encryptedLogsHash = kernel_data[i].public_inputs.end.encrypted_logs_hash;
            let unencryptedLogsHash = kernel_data[i].public_inputs.end.unencrypted_logs_hash;

            let mut offset = 0;

            for j in 0..MAX_NEW_COMMITMENTS_PER_TX {
                calldata_hash_inputs[offset + i * MAX_NEW_COMMITMENTS_PER_TX + j] = new_commitments[j];
            }
            offset += MAX_NEW_COMMITMENTS_PER_TX * 2;

            for j in 0..MAX_NEW_NULLIFIERS_PER_TX {
                calldata_hash_inputs[offset + i * MAX_NEW_NULLIFIERS_PER_TX + j] = new_nullifiers[j];
            }
            offset += MAX_NEW_NULLIFIERS_PER_TX * 2;

            for j in 0..MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX {
                calldata_hash_inputs[offset + i * MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX * 2 + j * 2] =
                    public_data_update_requests[j].leaf_index;
                calldata_hash_inputs[offset + i * MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX * 2 + j * 2 + 1] =
                    public_data_update_requests[j].new_value;
            }
            offset += MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX * 2 * 2;

            for j in 0..MAX_NEW_L2_TO_L1_MSGS_PER_TX {
                calldata_hash_inputs[offset + i * MAX_NEW_L2_TO_L1_MSGS_PER_TX + j] = newL2ToL1msgs[j];
            }
            offset += MAX_NEW_L2_TO_L1_MSGS_PER_TX * 2;

            let contract_leaf = kernel_data[i].public_inputs.end.new_contracts[0];
            calldata_hash_inputs[offset + i] = contract_leaf.hash();

            offset += MAX_NEW_CONTRACTS_PER_TX * 2;

            let new_contracts = kernel_data[i].public_inputs.end.new_contracts;
            calldata_hash_inputs[offset + i * 2] = new_contracts[0].contract_address.to_field();
            calldata_hash_inputs[offset + i * 2 + 1] = new_contracts[0].portal_contract_address.to_field();

            offset += MAX_NEW_CONTRACTS_PER_TX * 2 * 2;

            for j in 0..NUM_FIELDS_PER_SHA256 {
                calldata_hash_inputs[offset + i * 2 + j] = encryptedLogsHash[j];
            }

            offset += NUM_ENCRYPTED_LOGS_HASHES_PER_TX * NUM_FIELDS_PER_SHA256 * 2;

            for j in 0..NUM_FIELDS_PER_SHA256 {
                calldata_hash_inputs[offset + i * 2 + j] = unencryptedLogsHash[j];
            }
        }

        let mut hash_input_flattened = [0; CALL_DATA_HASH_FULL_FIELDS * 32 + CALL_DATA_HASH_LOG_FIELDS * 16];
        for offset in 0..CALL_DATA_HASH_FULL_FIELDS {
            let input_as_bytes = calldata_hash_inputs[offset].to_be_bytes(32);
            for byte_index in 0..32 {
                hash_input_flattened[offset * 32 + byte_index] = input_as_bytes[byte_index];
            }
        }

        for log_field_index in 0..CALL_DATA_HASH_LOG_FIELDS {
            let input_as_bytes = calldata_hash_inputs[CALL_DATA_HASH_FULL_FIELDS + log_field_index].to_be_bytes(16);
            for byte_index in 0..16 {
                hash_input_flattened[CALL_DATA_HASH_FULL_FIELDS * 32 + log_field_index * 16 + byte_index] = input_as_bytes[byte_index];
            }
        }

        let sha_digest = dep::std::hash::sha256(hash_input_flattened);
        U256::from_bytes32(sha_digest).to_u128_limbs()
    }

    // Check that the block header used by each kernel is a member of the blocks tree --> since the block header
    // contains roots of all the trees this is sufficient to verify that the tree roots used by kernels are correct
    fn perform_archive_membership_checks(self) {
        // For each of the block header (their block hashes), we need to do an inclusion proof
        // against the blocks tree root from the beginning of a rollup provided in the rollup constants
        let archive_root = self.constants.archive_snapshot.root;

        for i in 0..KERNELS_PER_BASE_ROLLUP {
            // Rebuild the block hash
            let block_header = self.kernel_data[i].public_inputs.constants.block_header;
            let previous_block_hash = block_header.block.hash();

            let previous_block_hash_witness = self.archive_root_membership_witnesses[i];

            // Now check that the previous block hash is in the blocks tree from the beginning of the rollup
            components::assert_check_membership(
                previous_block_hash,
                previous_block_hash_witness.leaf_index,
                previous_block_hash_witness.sibling_path,
                archive_root
            );
        }
    }

    // TODO(Kev): This aggregate_proof method is duplicated in a lot of places
    fn aggregate_proofs(self) -> AggregationObject {
        // TODO: for now we simply return the aggregation object from the first proof
        self.kernel_data[0].public_inputs.end.aggregation_object
    }
}

fn verify_kernel_proof(proof: Proof) -> bool {
    // TODO: Just return true as we are mocking out the proof verification
    // and aggregation.
    // TODO(Kev): It may make sense to move all of these methods into a 
    // separate module.
    true
}

fn insert_public_data_update_requests(
    tree_root: Field,
    public_data_update_requests: [PublicDataUpdateRequest;MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
    witnesses_offset: Field,
    witnesses: [[Field; PUBLIC_DATA_TREE_HEIGHT]; 2 * MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX]
) -> Field {
    let mut root = tree_root;

    for i in 0..MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX {
        let state_write = public_data_update_requests[i];
        let witness = witnesses[i + witnesses_offset];

        if (!state_write.is_empty()) {
            components::assert_check_membership(state_write.old_value, state_write.leaf_index, witness, root);
            root = components::root_from_sibling_path(state_write.new_value, state_write.leaf_index, witness);
        }
    }

    root
}

fn validate_public_data_reads(
    tree_root: Field,
    public_data_reads: [PublicDataRead; MAX_PUBLIC_DATA_READS_PER_TX],
    witnesses_offset: Field,
    witnesses: [[Field; PUBLIC_DATA_TREE_HEIGHT]; 2 * MAX_PUBLIC_DATA_READS_PER_TX]
) {
    for i in 0..MAX_PUBLIC_DATA_READS_PER_TX {
        let public_data_read = public_data_reads[i];
        let witness = witnesses[i + witnesses_offset];

        if (!public_data_read.is_empty()) {
            components::assert_check_membership(
                public_data_read.value,
                public_data_read.leaf_index,
                witness,
                tree_root
            );
        }
    }
}

global NUM_CONTRACT_LEAVES = 2;
#[test]
fn consistent_num_contract_leaves() {
    assert(NUM_CONTRACT_LEAVES == MAX_NEW_CONTRACTS_PER_TX * 2, "num contract leaves incorrect, see calculate_contract_leaves to see how it is computed");
}

global NOTE_HASH_SUBTREE_WIDTH = 128;
#[test]
fn consistent_not_hash_subtree_width() {
    assert_eq(NOTE_HASH_SUBTREE_WIDTH, 2.pow_32(NOTE_HASH_SUBTREE_HEIGHT) as u32, "note hash subtree width is incorrect");
}

global CALLDATA_HASH_INPUT_SIZE = 338;
#[test]
fn consistent_calldata_hash_input_size() {
    let expected_size = (MAX_NEW_COMMITMENTS_PER_TX
        + MAX_NEW_NULLIFIERS_PER_TX
        + MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX * 2
        + MAX_NEW_L2_TO_L1_MSGS_PER_TX
        + MAX_NEW_CONTRACTS_PER_TX * 3
        + NUM_ENCRYPTED_LOGS_HASHES_PER_TX * NUM_FIELDS_PER_SHA256
        + NUM_UNENCRYPTED_LOGS_HASHES_PER_TX * NUM_FIELDS_PER_SHA256)
        * 2;
    assert(CALLDATA_HASH_INPUT_SIZE == expected_size, "calldata hash input size is incorrect");
}

global CALL_DATA_HASH_LOG_FIELDS = 8;
#[test]
fn consistent_call_data_hash_log_fields() {
    assert_eq(CALL_DATA_HASH_LOG_FIELDS, NUM_ENCRYPTED_LOGS_HASHES_PER_TX * NUM_FIELDS_PER_SHA256 * 2
        + NUM_UNENCRYPTED_LOGS_HASHES_PER_TX * NUM_FIELDS_PER_SHA256 * 2, "calldata hash log fields is incorrect");
}

global CALL_DATA_HASH_FULL_FIELDS = 330;
#[test]
fn consistent_call_data_hash_full_fields() {
    assert_eq(CALL_DATA_HASH_FULL_FIELDS, CALLDATA_HASH_INPUT_SIZE - CALL_DATA_HASH_LOG_FIELDS, "calldata hash log fields is incorrect");
}

// TODO to radix returns u8, so we cannot use bigger radixes. It'd be ideal to use a radix of the maximum range-constrained integer noir supports
pub fn full_field_less_than(lhs: Field, rhs: Field) -> bool {
    dep::std::eddsa::lt_bytes32(lhs, rhs)
}

pub fn full_field_greater_than(lhs: Field, rhs: Field) -> bool {
    dep::std::eddsa::lt_bytes32(rhs, lhs)
}

#[test]
fn test_u256_less_than() {
    assert(full_field_less_than(1, 1000));
    assert(!full_field_less_than(1000, 1000));
    assert(!full_field_less_than(1000, 1));
    assert(full_field_less_than(0, 0 - 1));
    assert(!full_field_less_than(0 - 1, 0));
}

#[test]
fn test_u256_greater_than() {
    assert(full_field_greater_than(1000, 1));
    assert(!full_field_greater_than(1000, 1000));
    assert(!full_field_greater_than(1, 1000));
    assert(!full_field_greater_than(0, 0 - 1));
    assert(full_field_greater_than(0 - 1, 0));
}

mod tests {
    use crate::{
        base::base_rollup_inputs::{
            CALL_DATA_HASH_FULL_FIELDS,
            CALL_DATA_HASH_LOG_FIELDS,
            NOTE_HASH_SUBTREE_WIDTH, 
            NUM_CONTRACT_LEAVES, 
            BaseRollupInputs,
            full_field_less_than,
        },
        merkle_tree::{calculate_subtree, calculate_empty_tree_root},
        abis::append_only_tree_snapshot::AppendOnlyTreeSnapshot,
        abis::base_or_merge_rollup_public_inputs::BaseOrMergeRollupPublicInputs,
        abis::nullifier_leaf_preimage::NullifierLeafPreimage,
        abis::constant_rollup_data::ConstantRollupData,
        tests::merkle_tree_utils::{NonEmptyMerkleTree, compute_zero_hashes},
        components,
    };
    use dep::types::constants::{
        CONTRACT_SUBTREE_SIBLING_PATH_LENGTH,
        CONTRACT_TREE_HEIGHT,
        CONTRACT_SUBTREE_HEIGHT,
        ARCHIVE_HEIGHT,
        KERNELS_PER_BASE_ROLLUP,
        MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP,
        MAX_PUBLIC_DATA_READS_PER_BASE_ROLLUP,
        MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_BASE_ROLLUP,
        NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH,
        NOTE_HASH_TREE_HEIGHT,
        NOTE_HASH_SUBTREE_HEIGHT,
        NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH,
        NULLIFIER_TREE_HEIGHT,
        NULLIFIER_SUBTREE_HEIGHT,
        PUBLIC_DATA_TREE_HEIGHT,
        NUM_FIELDS_PER_SHA256,
    };
    use dep::types::{
        abis::membership_witness::ArchiveRootMembershipWitness,
        abis::membership_witness::NullifierMembershipWitness,
        abis::new_contract_data::NewContractData,
        abis::public_data_read::PublicDataRead,
        abis::public_data_update_request::PublicDataUpdateRequest,
        abis::previous_kernel_data::PreviousKernelData,
        tests::previous_kernel_data_builder::PreviousKernelDataBuilder,
        address::{Address, EthAddress},
        utils::bounded_vec::BoundedVec,
        utils::uint256::U256,
    };
    use dep::std::option::Option;

    struct NullifierInsertion {
        existing_index: u64,
        value: Field,
    }


    struct SortedNullifierTuple {
        value: Field,
        original_index: u32,
    }

    global MAX_NEW_NULLIFIERS_PER_TEST = 4;

    struct BaseRollupInputsBuilder {
        kernel_data: [PreviousKernelDataBuilder; KERNELS_PER_BASE_ROLLUP],
        pre_existing_notes: [Field; NOTE_HASH_SUBTREE_WIDTH],
        pre_existing_nullifiers: [NullifierLeafPreimage; MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP],
        pre_existing_contracts: [Field; NUM_CONTRACT_LEAVES],
        pre_existing_public_data: [Field; MAX_PUBLIC_DATA_READS_PER_BASE_ROLLUP],
        pre_existing_blocks: [Field; KERNELS_PER_BASE_ROLLUP],
        public_data_reads: BoundedVec<u64, 2>,
        public_data_writes: BoundedVec<(u64, Field), 2>,
        new_nullifiers: BoundedVec<NullifierInsertion, MAX_NEW_NULLIFIERS_PER_TEST>,
        constants: ConstantRollupData,
    }

    fn test_compute_empty_root<N>(size: [Field; N]) -> Field {
        compute_zero_hashes(size)[N - 1]
    }

    impl BaseRollupInputsBuilder {
        fn new() -> Self {
            let mut inputs: BaseRollupInputsBuilder = dep::std::unsafe::zeroed();
            inputs.constants.global_variables.chain_id = 1;
            inputs.constants.global_variables.version = 0;

            inputs.kernel_data = inputs.kernel_data.map(|_| {
                let mut builder = PreviousKernelDataBuilder::new();
                let _nullifier = builder.end.new_nullifiers.pop();
                builder.is_public()
            });

            inputs.pre_existing_blocks = inputs.kernel_data.map(|builder: PreviousKernelDataBuilder|{
                builder.block_header.block.hash()
            });

            inputs
        }

        fn extract_subtree_sibling_path<FULL_HEIGHT, SIBLING_PATH_LENGTH>(path: [Field; FULL_HEIGHT], mut sibling_path: [Field; SIBLING_PATH_LENGTH]) -> [Field; SIBLING_PATH_LENGTH] {
            let subtree_height = FULL_HEIGHT - SIBLING_PATH_LENGTH;
            for i in subtree_height..FULL_HEIGHT {
                sibling_path[i - subtree_height] = path[i];
            }
            sibling_path
        }

        fn update_nullifier_tree_with_new_leaves(
            mut self, 
            nullifier_tree: &mut NonEmptyMerkleTree<MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP, NULLIFIER_TREE_HEIGHT, NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH, NULLIFIER_SUBTREE_HEIGHT>, 
            kernel_data: &mut [PreviousKernelData; KERNELS_PER_BASE_ROLLUP], 
            start_nullifier_tree_snapshot: AppendOnlyTreeSnapshot
        ) -> (
            [NullifierLeafPreimage; MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP], 
            [NullifierMembershipWitness; MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP],
            [Field; MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP],
            [u32; MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP],
        ) {
            let mut low_nullifier_leaf_preimages: [NullifierLeafPreimage; MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP] = dep::std::unsafe::zeroed();
            let mut low_nullifier_membership_witness: [NullifierMembershipWitness; MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP] = dep::std::unsafe::zeroed();

            let mut sorted_new_nullifier_tuples = [SortedNullifierTuple {
                value: 0,
                original_index: 0,
            }; MAX_NEW_NULLIFIERS_PER_TEST];


            for i in 0..MAX_NEW_NULLIFIERS_PER_TEST {
                sorted_new_nullifier_tuples[i] = SortedNullifierTuple {
                    value: self.new_nullifiers.get_unchecked(i).value,
                    original_index: i as u32,
                };
            }
            sorted_new_nullifier_tuples = sorted_new_nullifier_tuples.sort_via(|a: SortedNullifierTuple, b: SortedNullifierTuple| {full_field_less_than(b.value, a.value)});

            let mut sorted_new_nullifiers = [0; MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP];
            let mut sorted_new_nullifiers_indexes = [0; MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP];

            for i in 0..MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP {
                if (i as u32) < (MAX_NEW_NULLIFIERS_PER_TEST as u32) {
                    sorted_new_nullifiers[i] = sorted_new_nullifier_tuples[i].value;
                    sorted_new_nullifiers_indexes[i] = sorted_new_nullifier_tuples[i].original_index;
                } else {
                    sorted_new_nullifiers[i] = 0;
                    sorted_new_nullifiers_indexes[i] = i as u32;
                }
            }

            let mut pre_existing_nullifiers = self.pre_existing_nullifiers;

            for i in 0..MAX_NEW_NULLIFIERS_PER_TEST {
                if (i as u64) < (self.new_nullifiers.len() as u64) {
                    let sorted_tuple = sorted_new_nullifier_tuples[i];
                    let new_nullifier = sorted_tuple.value;
                    let original_index = sorted_tuple.original_index;

                    let low_index = self.new_nullifiers.get_unchecked(original_index as Field).existing_index;

                    kernel_data[0].public_inputs.end.new_nullifiers[original_index] = new_nullifier;

                    let mut low_preimage = pre_existing_nullifiers[low_index];
                    low_nullifier_leaf_preimages[i] = low_preimage;
                    low_nullifier_membership_witness[i] = NullifierMembershipWitness {
                        leaf_index: low_index as Field,
                        sibling_path: nullifier_tree.get_sibling_path(low_index as Field)
                    };

                    low_preimage.next_value = new_nullifier;
                    low_preimage.next_index = start_nullifier_tree_snapshot.next_available_leaf_index + original_index;
                    pre_existing_nullifiers[low_index] = low_preimage;

                    nullifier_tree.update_leaf(low_index, low_preimage.hash());
                }
            }

            (low_nullifier_leaf_preimages, low_nullifier_membership_witness, sorted_new_nullifiers, sorted_new_nullifiers_indexes)
        } 

        fn build_inputs(mut self) -> BaseRollupInputs {
            let mut kernel_data = self.kernel_data.map(|builder: PreviousKernelDataBuilder|{
                builder.finish()
            });

            let start_note_hash_tree = NonEmptyMerkleTree::new(self.pre_existing_notes, [0; NOTE_HASH_TREE_HEIGHT], [0; NOTE_HASH_TREE_HEIGHT - NOTE_HASH_SUBTREE_HEIGHT], [0; NOTE_HASH_SUBTREE_HEIGHT]);
            let start_note_hash_tree_snapshot = AppendOnlyTreeSnapshot {
                root: start_note_hash_tree.get_root(),
                next_available_leaf_index: start_note_hash_tree.get_next_available_index() as u32,
            };
            let new_commitments_subtree_sibling_path = BaseRollupInputsBuilder::extract_subtree_sibling_path(start_note_hash_tree.get_sibling_path(self.pre_existing_notes.len()), [0; NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH]);

            let mut start_nullifier_tree = NonEmptyMerkleTree::new(
                self.pre_existing_nullifiers.map(|preimage: NullifierLeafPreimage| preimage.hash()), 
                [0; NULLIFIER_TREE_HEIGHT], 
                [0; NULLIFIER_TREE_HEIGHT - NULLIFIER_SUBTREE_HEIGHT], 
                [0; NULLIFIER_SUBTREE_HEIGHT]
            );

            let start_nullifier_tree_snapshot = AppendOnlyTreeSnapshot {
                root: start_nullifier_tree.get_root(),
                next_available_leaf_index: start_nullifier_tree.get_next_available_index() as u32,
            };

            let start_contract_tree = NonEmptyMerkleTree::new(self.pre_existing_contracts, [0; CONTRACT_TREE_HEIGHT], [0; CONTRACT_TREE_HEIGHT - 1], [0; 1]);
            let start_contract_tree_snapshot = AppendOnlyTreeSnapshot {
                root: start_contract_tree.get_root(),
                next_available_leaf_index: start_contract_tree.get_next_available_index() as u32,
            };
            let new_contracts_subtree_sibling_path = BaseRollupInputsBuilder::extract_subtree_sibling_path(start_contract_tree.get_sibling_path(self.pre_existing_contracts.len()), [0; CONTRACT_SUBTREE_SIBLING_PATH_LENGTH]);

            let mut start_public_data_tree = NonEmptyMerkleTree::new(self.pre_existing_public_data, [0; PUBLIC_DATA_TREE_HEIGHT], [0; PUBLIC_DATA_TREE_HEIGHT - 5], [0; 5]);
            let start_public_data_tree_root = start_public_data_tree.get_root();

            let start_archive = NonEmptyMerkleTree::new(self.pre_existing_blocks, [0; ARCHIVE_HEIGHT], [0; ARCHIVE_HEIGHT - 1], [0; 1]);
            let archive_snapshot = AppendOnlyTreeSnapshot {
                root: start_archive.get_root(),
                next_available_leaf_index: start_archive.get_next_available_index() as u32,
            };

            self.constants.archive_snapshot = archive_snapshot;

            let mut new_public_data_reads_sibling_paths: [[Field; PUBLIC_DATA_TREE_HEIGHT]; MAX_PUBLIC_DATA_READS_PER_BASE_ROLLUP] = dep::std::unsafe::zeroed();

            for i in 0..self.public_data_reads.max_len() {
                if (i as u64) < (self.public_data_reads.len() as u64) {
                    let index = self.public_data_reads.get_unchecked(i);
                    let value = self.pre_existing_public_data[index];
                    kernel_data[0].public_inputs.end.public_data_reads[i] = PublicDataRead {
                        leaf_index: index as Field,
                        value: value,
                    };
                    new_public_data_reads_sibling_paths[i] = start_public_data_tree.get_sibling_path(index as Field);
                }
            }

            let mut new_public_data_update_requests_sibling_paths: [[Field; PUBLIC_DATA_TREE_HEIGHT]; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_BASE_ROLLUP] = dep::std::unsafe::zeroed();

            for i in 0..self.public_data_writes.max_len() {
                if (i as u64) < (self.public_data_writes.len() as u64) {
                    let write = self.public_data_writes.get_unchecked(i);
                    let index = write.0;
                    let new_value = write.1;
                    let old_value = self.pre_existing_public_data[index];
                    kernel_data[0].public_inputs.end.public_data_update_requests[i] = PublicDataUpdateRequest {
                        leaf_index : index as Field,
                        old_value,
                        new_value,
                    };
                    new_public_data_update_requests_sibling_paths[i] = start_public_data_tree.get_sibling_path(index as Field);
                    start_public_data_tree.update_leaf(index, new_value);
                }
            }

            let (
                low_nullifier_leaf_preimages, 
                low_nullifier_membership_witness,
                sorted_new_nullifiers,
                sorted_new_nullifiers_indexes
            ) = self.update_nullifier_tree_with_new_leaves(&mut start_nullifier_tree, &mut kernel_data, start_nullifier_tree_snapshot);
            
            let new_nullifiers_subtree_sibling_path = BaseRollupInputsBuilder::extract_subtree_sibling_path(start_nullifier_tree.get_sibling_path(self.pre_existing_nullifiers.len()), [0; NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH]);

            BaseRollupInputs {
                kernel_data: kernel_data,
                start_note_hash_tree_snapshot,
                start_nullifier_tree_snapshot,
                start_contract_tree_snapshot,
                start_public_data_tree_root,
                archive_snapshot,

                sorted_new_nullifiers,
                sorted_new_nullifiers_indexes,

                low_nullifier_leaf_preimages,
                low_nullifier_membership_witness,

                new_commitments_subtree_sibling_path, 
                new_nullifiers_subtree_sibling_path, 
                new_contracts_subtree_sibling_path, 
                new_public_data_update_requests_sibling_paths,  
                new_public_data_reads_sibling_paths,  

                archive_root_membership_witnesses: [
                    ArchiveRootMembershipWitness {
                        leaf_index: 0,
                        sibling_path: start_archive.get_sibling_path(0)
                    },
                    ArchiveRootMembershipWitness {
                        leaf_index: 1,
                        sibling_path: start_archive.get_sibling_path(1)
                    },
                ],

                constants: self.constants,
            }
        }

        fn execute(self) -> BaseOrMergeRollupPublicInputs {
            self.build_inputs().base_rollup_circuit()
        }

        fn succeeds(self) {
            let _ = self.execute();
        }

        fn fails(self) {
            let _ = self.execute();
        }
    }

    #[test]
    unconstrained fn no_new_contract_leaves() {
        let outputs = BaseRollupInputsBuilder::new().execute();
        let expected_start_contract_tree_snapshot = AppendOnlyTreeSnapshot { root: test_compute_empty_root([0; CONTRACT_TREE_HEIGHT]), next_available_leaf_index: 2 };
        let expected_end_contract_tree_snapshot = AppendOnlyTreeSnapshot { root: test_compute_empty_root([0; CONTRACT_TREE_HEIGHT]), next_available_leaf_index: 4 };
        assert(outputs.start_contract_tree_snapshot.eq(expected_start_contract_tree_snapshot));
        assert(outputs.end_contract_tree_snapshot.eq(expected_end_contract_tree_snapshot));
    }

    #[test]
    unconstrained fn contract_leaf_inserted() {
        let new_contract = NewContractData {
            contract_address: Address::from_field(1),
            portal_contract_address: EthAddress::from_field(2),
            function_tree_root: 3
        };

        let mut builder = BaseRollupInputsBuilder::new();

        let mut new_contracts = builder.kernel_data[0].end.new_contracts;
        new_contracts.push(new_contract);
        builder.kernel_data[0].end.new_contracts = new_contracts;

        let mut expected_contracts_tree = NonEmptyMerkleTree::new(
            [0; 4],
            [0; CONTRACT_TREE_HEIGHT],
            [0; CONTRACT_TREE_HEIGHT - 2],
            [0; 2]
        );

        let outputs = builder.execute();

        let expected_start_contract_tree_snapshot = AppendOnlyTreeSnapshot { root: expected_contracts_tree.get_root(), next_available_leaf_index: 2 };
        assert(outputs.start_contract_tree_snapshot.eq(expected_start_contract_tree_snapshot));

        expected_contracts_tree.update_leaf(2, new_contract.hash());
        let expected_end_contract_tree_snapshot = AppendOnlyTreeSnapshot { root: expected_contracts_tree.get_root(), next_available_leaf_index: 4 };
        assert(outputs.end_contract_tree_snapshot.eq(expected_end_contract_tree_snapshot));
    }

    #[test]
    unconstrained fn contract_leaf_inserted_in_non_empty_snapshot_tree() {
        let new_contract = NewContractData {
            contract_address: Address::from_field(1),
            portal_contract_address: EthAddress::from_field(2),
            function_tree_root: 3
        };

        let mut builder = BaseRollupInputsBuilder::new();

        builder.pre_existing_contracts = [1,2];

        let mut new_contracts = builder.kernel_data[0].end.new_contracts;
        new_contracts.push(new_contract);
        builder.kernel_data[0].end.new_contracts = new_contracts;

        let mut expected_contracts_tree = NonEmptyMerkleTree::new(
            [1, 2, 0, 0],
            [0; CONTRACT_TREE_HEIGHT],
            [0; CONTRACT_TREE_HEIGHT - 2],
            [0; 2]
        );

        let outputs = builder.execute();

        let expected_start_contract_tree_snapshot = AppendOnlyTreeSnapshot { root: expected_contracts_tree.get_root(), next_available_leaf_index: 2 };
        assert(outputs.start_contract_tree_snapshot.eq(expected_start_contract_tree_snapshot));

        expected_contracts_tree.update_leaf(2, new_contract.hash());
        let expected_end_contract_tree_snapshot = AppendOnlyTreeSnapshot { root: expected_contracts_tree.get_root(), next_available_leaf_index: 4 };
        assert(outputs.end_contract_tree_snapshot.eq(expected_end_contract_tree_snapshot));
    }

    #[test]
    unconstrained fn new_commitments_tree() {
        let mut builder = BaseRollupInputsBuilder::new();

        let new_commitments = [27, 28, 29, 30, 31, 32];
        let mut new_commitments_vec = builder.kernel_data[0].end.new_commitments;

        for i in 0..new_commitments.len() {
            new_commitments_vec.push(new_commitments[i]);
        }

        builder.kernel_data[0].end.new_commitments = new_commitments_vec;

        let mut expected_commitments_tree = NonEmptyMerkleTree::new(
            [0; NOTE_HASH_SUBTREE_WIDTH * 2],
            [0; NOTE_HASH_TREE_HEIGHT],
            [0; NOTE_HASH_TREE_HEIGHT - NOTE_HASH_SUBTREE_HEIGHT - 1],
            [0; NOTE_HASH_SUBTREE_HEIGHT + 1]
        );

        let outputs = builder.execute();

        let expected_start_note_hash_tree_snapshot = AppendOnlyTreeSnapshot { root: expected_commitments_tree.get_root(), next_available_leaf_index: NOTE_HASH_SUBTREE_WIDTH };
        assert(outputs.start_note_hash_tree_snapshot.eq(expected_start_note_hash_tree_snapshot));

        for i in 0..new_commitments.len() {
            expected_commitments_tree.update_leaf(
                (i as u64) + (NOTE_HASH_SUBTREE_WIDTH as u64),
                new_commitments[i]
            );
        }

        let expected_end_note_hash_tree_snapshot = AppendOnlyTreeSnapshot { root: expected_commitments_tree.get_root(), next_available_leaf_index: NOTE_HASH_SUBTREE_WIDTH * 2 };
        assert(outputs.end_note_hash_tree_snapshot.eq(expected_end_note_hash_tree_snapshot));
    }

    #[test]
    unconstrained fn new_nullifier_tree_empty() {
        /**
        * DESCRIPTION
        */

        // This test checks for insertions of all 0 values
        // In this special case we will not need to provide sibling paths to check insertion of the nullifier values
        // This is because 0 values are not actually inserted into the tree, rather the inserted subtree is left
        // empty to begin with.

        let mut builder = BaseRollupInputsBuilder::new();

        builder.pre_existing_nullifiers[0] = NullifierLeafPreimage {
            leaf_value : 0,
            next_value : 7,
            next_index : 1,
        };
        builder.pre_existing_nullifiers[1] = NullifierLeafPreimage {
            leaf_value : 7,
            next_value : 0,
            next_index : 0,
        };

        builder.succeeds();
    }

    #[test]
    unconstrained fn nullifier_insertion_test() {
        let mut builder = BaseRollupInputsBuilder::new();

        builder.pre_existing_nullifiers[0] = NullifierLeafPreimage {
            leaf_value : 0,
            next_value : 7,
            next_index : 1,
        };
        builder.pre_existing_nullifiers[1] = NullifierLeafPreimage {
            leaf_value : 7,
            next_value : 0,
            next_index : 0,
        };

        builder.new_nullifiers.push(NullifierInsertion {
            existing_index: 0,
            value: 1,
        });

        let mut tree_nullifiers = [NullifierLeafPreimage::default(); MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP * 2];
        tree_nullifiers[0] = NullifierLeafPreimage {
            leaf_value : 0,
            next_value : 1,
            next_index : MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP as u32,
        };
        tree_nullifiers[1] = builder.pre_existing_nullifiers[1];
        tree_nullifiers[MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP] = NullifierLeafPreimage {
            leaf_value : 1,
            next_value : 7,
            next_index : 1,
        };

        let mut end_nullifier_tree = NonEmptyMerkleTree::new(
            tree_nullifiers.map(|preimage: NullifierLeafPreimage| preimage.hash()), 
            [0; NULLIFIER_TREE_HEIGHT], 
            [0; NULLIFIER_TREE_HEIGHT - NULLIFIER_SUBTREE_HEIGHT - 1], 
            [0; NULLIFIER_SUBTREE_HEIGHT + 1]
        );

        let output = builder.execute();

        assert(output.end_nullifier_tree_snapshot.eq(AppendOnlyTreeSnapshot {
            root: end_nullifier_tree.get_root(),
            next_available_leaf_index: 2 * MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP as u32,
        }));
    }

    #[test]
    unconstrained fn new_nullifier_tree_all_larger() {
        let mut builder = BaseRollupInputsBuilder::new();

        builder.pre_existing_nullifiers[0] = NullifierLeafPreimage {
            leaf_value : 0,
            next_value : 7,
            next_index : 1,
        };
        builder.pre_existing_nullifiers[1] = NullifierLeafPreimage {
            leaf_value : 7,
            next_value : 0,
            next_index : 0,
        };

        builder.new_nullifiers.push(NullifierInsertion {
            existing_index: 1,
            value: 8,
        });
        for i in 1..builder.new_nullifiers.max_len() {
            builder.new_nullifiers.push(NullifierInsertion {
                existing_index: 1,
                value: (8 + i) as Field,
            });
        }

        let output = builder.execute();

        let mut tree_nullifiers = [NullifierLeafPreimage::default(); MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP * 2];
        tree_nullifiers[0] = builder.pre_existing_nullifiers[0];

        tree_nullifiers[1] = NullifierLeafPreimage {
            leaf_value : 7,
            next_value : 8,
            next_index : MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP as u32,
        };

        let last_index = builder.new_nullifiers.max_len() - 1;
        for i in 0..last_index {
            tree_nullifiers[MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP + i] = NullifierLeafPreimage {
                leaf_value : (8 + i) as Field,
                next_value : (8 + i + 1) as Field,
                next_index : (MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP + i) as u32 + 1,
            };
        }

        tree_nullifiers[MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP+last_index] = NullifierLeafPreimage {
            leaf_value : (8 + last_index) as Field,
            next_value : 0,
            next_index : 0,
        };

        let mut end_nullifier_tree = NonEmptyMerkleTree::new(
            tree_nullifiers.map(|preimage: NullifierLeafPreimage| preimage.hash()), 
            [0; NULLIFIER_TREE_HEIGHT], 
            [0; NULLIFIER_TREE_HEIGHT - NULLIFIER_SUBTREE_HEIGHT - 1], 
            [0; NULLIFIER_SUBTREE_HEIGHT + 1]
        );

        assert(output.end_nullifier_tree_snapshot.eq(AppendOnlyTreeSnapshot {
            root: end_nullifier_tree.get_root(),
            next_available_leaf_index: 2 * MAX_NEW_NULLIFIERS_PER_BASE_ROLLUP as u32,
        }));
    }

    #[test(should_fail_with = "Invalid low leaf")]
    unconstrained fn new_nullifier_tree_double_spend() {
        let mut builder = BaseRollupInputsBuilder::new();

        builder.pre_existing_nullifiers[0] = NullifierLeafPreimage {
            leaf_value : 0,
            next_value : 7,
            next_index : 1,
        };
        builder.pre_existing_nullifiers[1] = NullifierLeafPreimage {
            leaf_value : 7,
            next_value : 0,
            next_index : 0,
        };

        builder.new_nullifiers.push(NullifierInsertion {
            existing_index: 1,
            value: 8,
        });
        builder.new_nullifiers.push(NullifierInsertion {
            existing_index: 1,
            value: 8,
        });

        builder.fails();
    }

    #[test(should_fail_with = "Invalid low leaf")]
    unconstrained fn new_nullifier_tree_double_spend_same_batch() {
        let mut builder = BaseRollupInputsBuilder::new();

        builder.pre_existing_nullifiers[0] = NullifierLeafPreimage {
            leaf_value : 0,
            next_value : 7,
            next_index : 1,
        };
        builder.pre_existing_nullifiers[1] = NullifierLeafPreimage {
            leaf_value : 7,
            next_value : 0,
            next_index : 0,
        };

        builder.new_nullifiers.push(NullifierInsertion {
            existing_index: 1,
            value: 8,
        });
        builder.new_nullifiers.push(NullifierInsertion {
            existing_index: 1,
            value: 8,
        });

        builder.fails();
    }

    #[test]
    unconstrained fn empty_block_calldata_hash() {
        let outputs = BaseRollupInputsBuilder::new().execute();

        let hash_input_flattened = [0; CALL_DATA_HASH_FULL_FIELDS * 32 + CALL_DATA_HASH_LOG_FIELDS * 16];
        let sha_digest = dep::std::hash::sha256(hash_input_flattened);
        let expected_calldata_hash = U256::from_bytes32(sha_digest).to_u128_limbs();
        for i in 0..NUM_FIELDS_PER_SHA256 {
            assert_eq(outputs.calldata_hash[i], expected_calldata_hash[i]);
        }
    }

    #[test(should_fail_with = "membership check failed")]
    unconstrained fn compute_membership_archive_negative() {
        let mut inputs = BaseRollupInputsBuilder::new().build_inputs();

        inputs.archive_root_membership_witnesses[0].sibling_path[0] = 27;

        let _output = inputs.base_rollup_circuit();
    }

    #[test]
    unconstrained fn constants_dont_change() {
        let inputs = BaseRollupInputsBuilder::new().build_inputs();
        let outputs = inputs.base_rollup_circuit();

        assert(inputs.constants.eq(outputs.constants));
    }

    #[test(should_fail_with = "kernel chain_id does not match the rollup chain_id")]
    unconstrained fn constants_dont_match_kernels_chain_id() {
        let mut builder = BaseRollupInputsBuilder::new();
        builder.constants.global_variables.chain_id = 3;
        builder.fails();
    }

    #[test(should_fail_with = "kernel version does not match the rollup version")]
    unconstrained fn constants_dont_match_kernels_version() {
        let mut builder = BaseRollupInputsBuilder::new();
        builder.constants.global_variables.version = 3;
        builder.fails();
    }

    #[test]
    unconstrained fn subtree_height_is_0() {
        let outputs = BaseRollupInputsBuilder::new().execute();

        assert_eq(outputs.rollup_subtree_height, 0);
    }

    #[test]
    unconstrained fn single_public_state_read() {
        let mut builder = BaseRollupInputsBuilder::new();

        builder.pre_existing_public_data[0] = 27;
        builder.public_data_reads.push(0);

        builder.succeeds();
    }

    #[test]
    unconstrained fn single_public_state_write() {
        let mut builder = BaseRollupInputsBuilder::new();

        builder.pre_existing_public_data[0] = 27;
        builder.public_data_writes.push((0, 28));

        let outputs = builder.execute();

        let mut expected_public_data_tree = NonEmptyMerkleTree::new(
            [28, 0],
            [0; PUBLIC_DATA_TREE_HEIGHT],
            [0; PUBLIC_DATA_TREE_HEIGHT - 1],
            [0; 1]
        );

        assert_eq(outputs.end_public_data_tree_root, expected_public_data_tree.get_root());
    }

    #[test]
    unconstrained fn multiple_public_state_read_writes() {
        let mut builder = BaseRollupInputsBuilder::new();

        builder.pre_existing_public_data[0] = 27;
        builder.pre_existing_public_data[1] = 28;
        builder.pre_existing_public_data[2] = 29;
        builder.pre_existing_public_data[3] = 30;

        builder.public_data_reads.push(0);
        builder.public_data_writes.push((0, 60));
        builder.public_data_writes.push((2, 61));
        builder.public_data_reads.push(3);

        let outputs = builder.execute();

        let mut expected_public_data_tree = NonEmptyMerkleTree::new(
            [60, 28, 61, 30],
            [0; PUBLIC_DATA_TREE_HEIGHT],
            [0; PUBLIC_DATA_TREE_HEIGHT - 2],
            [0; 2]
        );

        assert_eq(outputs.end_public_data_tree_root, expected_public_data_tree.get_root());
    }
}