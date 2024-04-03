use crate::{
    abis::{
    constant_rollup_data::ConstantRollupData,
    base_or_merge_rollup_public_inputs::{BaseOrMergeRollupPublicInputs, BASE_ROLLUP_TYPE}
},
    base::state_diff_hints::StateDiffHints,
    components::{compute_tx_effects_hash, compute_kernel_out_hash}
};
use dep::types::{
    hash::sha256_to_field,
    abis::{
    append_only_tree_snapshot::AppendOnlyTreeSnapshot,
    membership_witness::{ArchiveRootMembershipWitness, NullifierMembershipWitness, PublicDataMembershipWitness},
    nullifier_leaf_preimage::NullifierLeafPreimage, public_data_update_request::PublicDataUpdateRequest,
    public_data_read::PublicDataRead, kernel_data::KernelData,
    side_effect::{SideEffect, SideEffectLinkedToNoteHash}, accumulated_data::CombinedAccumulatedData
},
    constants::{
    NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH, NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH,
    PUBLIC_DATA_TREE_HEIGHT, NOTE_HASH_SUBTREE_HEIGHT, MAX_NEW_NOTE_HASHES_PER_TX,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, MAX_PUBLIC_DATA_READS_PER_TX, MAX_NEW_NULLIFIERS_PER_TX,
    NUM_ENCRYPTED_LOGS_HASHES_PER_TX, MAX_NEW_L2_TO_L1_MSGS_PER_TX, NUM_UNENCRYPTED_LOGS_HASHES_PER_TX,
    NULLIFIER_SUBTREE_HEIGHT, NULLIFIER_TREE_HEIGHT, PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH,
    PUBLIC_DATA_SUBTREE_HEIGHT
},
    merkle_tree::{
    append_only_tree, assert_check_membership, calculate_empty_tree_root, calculate_subtree_root,
    indexed_tree, MembershipWitness
},
    mocked::{AggregationObject, Proof}, partial_state_reference::PartialStateReference,
    public_data_tree_leaf::PublicDataTreeLeaf,
    public_data_tree_leaf_preimage::PublicDataTreeLeafPreimage,
    utils::{field::{full_field_less_than, full_field_greater_than}, uint256::U256}
};

struct BaseRollupInputs {
    kernel_data: KernelData,
    start: PartialStateReference,

    state_diff_hints: StateDiffHints,

    // TODO: The following 6 values are eventually going to be nuked from here. See discussion:
    // https://aztecprotocol.slack.com/archives/C060PU5R327/p1701965354071269
    sorted_public_data_writes: [PublicDataTreeLeaf; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
    sorted_public_data_writes_indexes: [u64; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
    low_public_data_writes_preimages: [PublicDataTreeLeafPreimage; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
    low_public_data_writes_witnesses: [PublicDataMembershipWitness; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],

    archive_root_membership_witness: ArchiveRootMembershipWitness,

    constants: ConstantRollupData,
}

impl BaseRollupInputs {
    pub fn base_rollup_circuit(self) -> BaseOrMergeRollupPublicInputs {
        // Verify the previous kernel proof
        assert(verify_kernel_proof(self.kernel_data.proof), "kernel proof verification failed");

        // Verify the kernel chain_id and versions
        assert(
            self.kernel_data.public_inputs.constants.tx_context.chain_id
            == self.constants.global_variables.chain_id, "kernel chain_id does not match the rollup chain_id"
        );
        assert(
            self.kernel_data.public_inputs.constants.tx_context.version
            == self.constants.global_variables.version, "kernel version does not match the rollup version"
        );

        let rollup_validation_requests = self.kernel_data.public_inputs.rollup_validation_requests;

        // Verify the max block number
        // TODO #5345: why is block_number a Field and not u32?
        if rollup_validation_requests.max_block_number.is_some() {
            assert(
                self.constants.global_variables.block_number as u32
                <= rollup_validation_requests.max_block_number.unwrap_unchecked(), "kernel max_block_number is smaller than block number"
            );
        }

        let commitments_tree_subroot = self.calculate_commitments_subtree();

        let empty_commitments_subtree_root = calculate_empty_tree_root(NOTE_HASH_SUBTREE_HEIGHT);

        let end_note_hash_tree_snapshot = append_only_tree::insert_subtree_to_snapshot_tree(
            self.start.note_hash_tree,
            self.state_diff_hints.note_hash_subtree_sibling_path,
            empty_commitments_subtree_root,
            commitments_tree_subroot,
            NOTE_HASH_SUBTREE_HEIGHT as u8
        );

        // Insert nullifiers:
        let end_nullifier_tree_snapshot = self.check_nullifier_tree_non_membership_and_insert_to_tree();

        // Validate public public data reads and public data update requests, and update public data tree
        let end_public_data_tree_snapshot = self.validate_and_process_public_state();

        // Calculate the tx effects hash of the transaction
        let tx_effects_hash = compute_tx_effects_hash(
            self.kernel_data.public_inputs.end,
            self.kernel_data.public_inputs.revert_code
        );
        let out_hash = compute_kernel_out_hash(self.kernel_data.public_inputs.end);

        // Perform membership checks that the notes provided exist within the historical trees data
        self.perform_archive_membership_checks();

        let aggregation_object = self.aggregate_proofs();

        BaseOrMergeRollupPublicInputs {
            rollup_type: BASE_ROLLUP_TYPE,
            height_in_block_tree: 0,
            aggregation_object,
            constants: self.constants,
            start: self.start,
            end: PartialStateReference {
                note_hash_tree: end_note_hash_tree_snapshot,
                nullifier_tree: end_nullifier_tree_snapshot,
                public_data_tree: end_public_data_tree_snapshot
            },
            txs_effects_hash: tx_effects_hash,
            out_hash
        }
    }

    // TODO(Kev): This should say calculate_commitments_subtree_root
    // Cpp code says calculate_commitments_subtree, so I'm leaving it as is for now
    fn calculate_commitments_subtree(self) -> Field {
        calculate_subtree_root(self.kernel_data.public_inputs.end.new_note_hashes.map(|c: SideEffect| c.value))
    }

    fn check_nullifier_tree_non_membership_and_insert_to_tree(self) -> AppendOnlyTreeSnapshot {
        indexed_tree::batch_insert(
            self.start.nullifier_tree,
            self.kernel_data.public_inputs.end.new_nullifiers.map(|nullifier: SideEffectLinkedToNoteHash| nullifier.value),
            self.state_diff_hints.sorted_nullifiers,
            self.state_diff_hints.sorted_nullifier_indexes,
            self.state_diff_hints.nullifier_subtree_sibling_path,
            self.state_diff_hints.nullifier_predecessor_preimages,
            self.state_diff_hints.nullifier_predecessor_membership_witnesses.map(
                |witness: NullifierMembershipWitness| {
                MembershipWitness {
                    leaf_index: witness.leaf_index,
                    sibling_path: witness.sibling_path,
                }
            }
            ),
            |low_leaf: NullifierLeafPreimage, nullifier: Field| { // Is valid low leaf
                let is_less_than_nullifier = full_field_less_than(low_leaf.nullifier, nullifier);
                let is_next_greater_than = full_field_less_than(nullifier, low_leaf.next_nullifier);

                (!low_leaf.is_empty()) & is_less_than_nullifier & (
                    is_next_greater_than |
                    ((low_leaf.next_index == 0) & (low_leaf.next_nullifier == 0))
                )
            },
            |low_leaf: NullifierLeafPreimage, nullifier: Field, nullifier_index: u64| { // Update low leaf
                NullifierLeafPreimage{
                    nullifier : low_leaf.nullifier,
                    next_nullifier : nullifier,
                    next_index : nullifier_index,
                }
            },
            |nullifier: Field, low_leaf: NullifierLeafPreimage| { // Build insertion leaf
                NullifierLeafPreimage {
                    nullifier : nullifier,
                    next_nullifier : low_leaf.next_nullifier,
                    next_index : low_leaf.next_index,
                }
            },
            [0; NULLIFIER_SUBTREE_HEIGHT],
            [0; NULLIFIER_TREE_HEIGHT]
        )
    }

    fn create_nullifier_subtree<N>(leaves: [NullifierLeafPreimage; N]) -> Field {
        calculate_subtree_root(leaves.map(|leaf:NullifierLeafPreimage| leaf.hash()))
    }

    fn validate_and_process_public_state(self) -> AppendOnlyTreeSnapshot {
        // TODO(#2521) - data read validation should happen against the current state of the tx and not the start state.
        // Blocks all interesting usecases that read and write to the same public state in the same tx.
        // https://aztecprotocol.slack.com/archives/C02M7VC7TN0/p1695809629015719?thread_ts=1695653252.007339&cid=C02M7VC7TN0

        // Process public data reads and public data update requests for left input
        // validate_public_data_reads(
        //                            self.start_public_data_tree_root,
        //                            self.kernel_data[0].public_inputs.end.public_data_reads,
        //                            0,
        //                            self.new_public_data_reads_sibling_paths);

        let end_public_data_tree_snapshot = insert_public_data_update_requests(
            self.start.public_data_tree,
            self.kernel_data.public_inputs.end.public_data_update_requests.map(
                |request: PublicDataUpdateRequest| {
                PublicDataTreeLeaf {
                    slot: request.leaf_slot,
                    value: request.new_value,
                }
            }
            ),
            self.sorted_public_data_writes,
            self.sorted_public_data_writes_indexes,
            self.low_public_data_writes_preimages,
            self.low_public_data_writes_witnesses,
            self.state_diff_hints.public_data_sibling_path
        );

        end_public_data_tree_snapshot
    }

    // Check that the block header used by each kernel is a member of the blocks tree --> since the block header
    // contains roots of all the trees this is sufficient to verify that the tree roots used by kernels are correct
    fn perform_archive_membership_checks(self) {
        // For each of the block header (their block hashes), we need to do an inclusion proof
        // against the blocks tree root from the beginning of a rollup provided in the rollup constants
        let archive_root = self.constants.last_archive.root;

        // Rebuild the block hash
        let header = self.kernel_data.public_inputs.constants.historical_header;
        let previous_block_hash = header.hash();

        let previous_block_hash_witness = self.archive_root_membership_witness;

        // Now check that the previous block hash is in the blocks tree from the beginning of the rollup
        assert_check_membership(
            previous_block_hash,
            previous_block_hash_witness.leaf_index,
            previous_block_hash_witness.sibling_path,
            archive_root
        );
    }

    // TODO(Kev): This aggregate_proof method is duplicated in a lot of places
    fn aggregate_proofs(self) -> AggregationObject {
        // TODO: for now we simply return the aggregation object from the first proof
        self.kernel_data.public_inputs.aggregation_object
    }
}

fn verify_kernel_proof(_proof: Proof) -> bool {
    // TODO: Just return true as we are mocking out the proof verification
    // and aggregation.
    // TODO(Kev): It may make sense to move all of these methods into a
    // separate module.
    true
}

fn insert_public_data_update_requests(
    prev_snapshot: AppendOnlyTreeSnapshot,
    public_data_writes: [PublicDataTreeLeaf; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
    sorted_public_data_writes: [PublicDataTreeLeaf; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
    sorted_public_data_writes_indexes: [u64; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
    low_public_data_writes_preimages: [PublicDataTreeLeafPreimage; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
    low_public_data_writes_witnesses: [PublicDataMembershipWitness; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
    public_data_writes_subtree_sibling_path: [Field; PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH]
) -> AppendOnlyTreeSnapshot {
    indexed_tree::batch_insert(
        prev_snapshot,
        public_data_writes,
        sorted_public_data_writes,
        sorted_public_data_writes_indexes,
        public_data_writes_subtree_sibling_path,
        low_public_data_writes_preimages,
        low_public_data_writes_witnesses.map(
            |witness: PublicDataMembershipWitness| {
            MembershipWitness {
                leaf_index: witness.leaf_index,
                sibling_path: witness.sibling_path,
            }
        }
        ),
        |low_preimage: PublicDataTreeLeafPreimage, write: PublicDataTreeLeaf| { // Is valid low preimage
            let is_update = low_preimage.slot == write.slot;
            let is_low_empty = low_preimage.is_empty();

            let is_less_than_slot = full_field_less_than(low_preimage.slot, write.slot);
            let is_next_greater_than = full_field_less_than(write.slot, low_preimage.next_slot);
            let is_in_range = is_less_than_slot & (
                is_next_greater_than |
                ((low_preimage.next_index == 0) & (low_preimage.next_slot == 0)));

            (!is_low_empty) & (is_update | is_in_range)
        },
        |low_preimage: PublicDataTreeLeafPreimage, write: PublicDataTreeLeaf, write_index: u64| { // Update low leaf
            let is_update = low_preimage.slot == write.slot;
            if is_update {
                PublicDataTreeLeafPreimage{
                    slot : low_preimage.slot,
                    value: write.value,
                    next_slot : low_preimage.next_slot,
                    next_index : low_preimage.next_index,
                }
            } else {
                PublicDataTreeLeafPreimage{
                    slot : low_preimage.slot,
                    value: low_preimage.value,
                    next_slot : write.slot,
                    next_index : write_index,
                }
            }
        },
        |write: PublicDataTreeLeaf, low_preimage: PublicDataTreeLeafPreimage| { // Build insertion leaf
            let is_update = low_preimage.slot == write.slot;
            if is_update {
                PublicDataTreeLeafPreimage::empty()
            }else {
                PublicDataTreeLeafPreimage {
                    slot: write.slot,
                    value: write.value,
                    next_slot : low_preimage.next_slot,
                    next_index : low_preimage.next_index,
                }
            }
        },
        [0; PUBLIC_DATA_SUBTREE_HEIGHT],
        [0; PUBLIC_DATA_TREE_HEIGHT]
    )
}

fn validate_public_data_reads(
    tree_root: Field,
    public_data_reads: [PublicDataRead; MAX_PUBLIC_DATA_READS_PER_TX],
    public_data_reads_preimages: [PublicDataTreeLeafPreimage; MAX_PUBLIC_DATA_READS_PER_TX],
    public_data_reads_witnesses: [PublicDataMembershipWitness; MAX_PUBLIC_DATA_READS_PER_TX]
) {
    for i in 0..MAX_PUBLIC_DATA_READS_PER_TX {
        let read = public_data_reads[i];
        let low_preimage = public_data_reads_preimages[i];
        let witness = public_data_reads_witnesses[i];

        let is_low_empty = low_preimage.is_empty();
        let is_exact = low_preimage.slot == read.leaf_slot;

        let is_less_than_slot = full_field_less_than(low_preimage.slot, read.leaf_slot);
        let is_next_greater_than = full_field_less_than(read.leaf_slot, low_preimage.next_slot);
        let is_in_range = is_less_than_slot
            & (is_next_greater_than | ((low_preimage.next_index == 0) & (low_preimage.next_slot == 0)));

        if (!read.is_empty()) {
            assert(!is_low_empty, "public data read is not empty but low preimage is empty");
            if is_in_range {
                assert_eq(read.value, 0, "low leaf for public data read is in range but value is not zero");
            } else {
                assert(is_exact, "low leaf for public data read is invalid");
                assert_eq(read.value, low_preimage.value, "low leaf for public data has different value");
            }
            assert_check_membership(
                low_preimage.hash(),
                witness.leaf_index,
                witness.sibling_path,
                tree_root
            );
        }
    }
}

#[test]
fn consistent_not_hash_subtree_width() {
    assert_eq(
        MAX_NEW_NOTE_HASHES_PER_TX as Field, 2.pow_32(NOTE_HASH_SUBTREE_HEIGHT as Field), "note hash subtree width is incorrect"
    );
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
        abis::{
        constant_rollup_data::ConstantRollupData,
        base_or_merge_rollup_public_inputs::BaseOrMergeRollupPublicInputs
    },
        base::{state_diff_hints::StateDiffHints, base_rollup_inputs::BaseRollupInputs},
        components::{TX_EFFECTS_HASH_INPUT_FIELDS, compute_kernel_out_hash}
    };
    use dep::types::{
        abis::{
        append_only_tree_snapshot::AppendOnlyTreeSnapshot,
        membership_witness::{ArchiveRootMembershipWitness, NullifierMembershipWitness, PublicDataMembershipWitness},
        nullifier_leaf_preimage::NullifierLeafPreimage, public_data_read::PublicDataRead,
        public_data_update_request::PublicDataUpdateRequest, kernel_data::KernelData,
        side_effect::SideEffect, accumulated_data::CombinedAccumulatedData
    },
        address::{AztecAddress, EthAddress},
        constants::{
        ARCHIVE_HEIGHT, MAX_PUBLIC_DATA_READS_PER_TX, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
        MAX_NEW_NOTE_HASHES_PER_TX, MAX_NEW_NULLIFIERS_PER_TX, NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH,
        NOTE_HASH_TREE_HEIGHT, NOTE_HASH_SUBTREE_HEIGHT, NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH,
        NULLIFIER_TREE_HEIGHT, NULLIFIER_SUBTREE_HEIGHT, PUBLIC_DATA_TREE_HEIGHT,
        PUBLIC_DATA_SUBTREE_HEIGHT, PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH,
        MAX_NEW_L2_TO_L1_MSGS_PER_TX
    },
        contract_class_id::ContractClassId, partial_state_reference::PartialStateReference,
        public_data_tree_leaf::PublicDataTreeLeaf,
        public_data_tree_leaf_preimage::PublicDataTreeLeafPreimage,
        tests::{
        fixture_builder::FixtureBuilder, merkle_tree_utils::{NonEmptyMerkleTree, compute_zero_hashes},
        sort::sort_high_to_low
    },
        utils::{field::{full_field_less_than, field_from_bytes_32_trunc, field_from_bytes}, uint256::U256}
    };

    struct NullifierInsertion {
        existing_index: u64,
        value: Field,
    }

    global MAX_NEW_NULLIFIERS_PER_TEST = 4;
    global MAX_PUBLIC_DATA_WRITES_PER_TEST = 2;
    global MAX_PUBLIC_DATA_READS_PER_TEST = 2;

    fn update_public_data_tree<EXISTING_LEAVES, SUBTREE_SIBLING_PATH_LENGTH, SUBTREE_HEIGHT>(
        public_data_tree: &mut NonEmptyMerkleTree<EXISTING_LEAVES, PUBLIC_DATA_TREE_HEIGHT, SUBTREE_SIBLING_PATH_LENGTH, SUBTREE_HEIGHT>,
        kernel_data: &mut KernelData,
        snapshot: AppendOnlyTreeSnapshot,
        public_data_writes: BoundedVec<(u64, PublicDataTreeLeaf), 2>,
        mut pre_existing_public_data: [PublicDataTreeLeafPreimage; EXISTING_LEAVES]
    ) -> ([Field; PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH], [PublicDataTreeLeaf; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX], [u64; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX], [PublicDataTreeLeafPreimage; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX], [PublicDataMembershipWitness; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX], [PublicDataTreeLeafPreimage; EXISTING_LEAVES]) {
        let mut subtree_path: [Field; PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH] = dep::std::unsafe::zeroed();
        let mut sorted_public_data_writes: [PublicDataTreeLeaf; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX] = dep::std::unsafe::zeroed();
        let mut sorted_public_data_writes_indexes: [u64; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX] = dep::std::unsafe::zeroed();
        let mut low_public_data_writes_preimages: [PublicDataTreeLeafPreimage; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX] = dep::std::unsafe::zeroed();
        let mut low_public_data_writes_witnesses: [PublicDataMembershipWitness; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX] = dep::std::unsafe::zeroed();
        let mut new_subtree: [PublicDataTreeLeafPreimage; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX] = dep::std::unsafe::zeroed();

        for i in 0..MAX_PUBLIC_DATA_WRITES_PER_TEST {
            if i < (public_data_writes.len()) {
                let leaf = public_data_writes.get_unchecked(i).1;

                kernel_data.public_inputs.end.public_data_update_requests[i] = PublicDataUpdateRequest {
                    leaf_slot : leaf.slot,
                    new_value : leaf.value,
                };
            }
        }
        let mut sorted_write_tuples = sort_high_to_low(
            public_data_writes.storage,
            |(_, leaf_a): (u64, PublicDataTreeLeaf),(_,leaf_b):(u64, PublicDataTreeLeaf)| full_field_less_than(leaf_a.slot, leaf_b.slot)
        );

        for i in 0..MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX {
            if i < MAX_PUBLIC_DATA_WRITES_PER_TEST {
                let (low_leaf_index, leaf): (u64, PublicDataTreeLeaf) = sorted_write_tuples[i].value;

                sorted_public_data_writes[i] = leaf;
                sorted_public_data_writes_indexes[i] = sorted_write_tuples[i].original_index;

                if !leaf.is_empty() {
                    let low_leaf = pre_existing_public_data[low_leaf_index];
                    if low_leaf.slot == leaf.slot {
                        pre_existing_public_data[low_leaf_index].value = leaf.value;
                    } else {
                        new_subtree[sorted_write_tuples[i].original_index] = PublicDataTreeLeafPreimage {
                            slot: leaf.slot,
                            value: leaf.value,
                            next_slot: low_leaf.next_slot,
                            next_index: low_leaf.next_index,
                        };
                        pre_existing_public_data[low_leaf_index] = PublicDataTreeLeafPreimage {
                            slot: low_leaf.slot,
                            value: low_leaf.value,
                            next_slot: leaf.slot,
                            next_index: MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX + i
                        };
                    }
                    low_public_data_writes_preimages[i] = low_leaf;
                    low_public_data_writes_witnesses[i] = PublicDataMembershipWitness {
                        leaf_index: low_leaf_index as Field,
                        sibling_path: public_data_tree.get_sibling_path(low_leaf_index)
                    };

                    public_data_tree.update_leaf(
                        low_leaf_index,
                        pre_existing_public_data[low_leaf_index].hash()
                    );
                }
            } else {
                sorted_public_data_writes[i] = PublicDataTreeLeaf::empty();
                sorted_public_data_writes_indexes[i] = i;
            }
        }

        subtree_path = BaseRollupInputsBuilder::extract_subtree_sibling_path(public_data_tree.get_sibling_path(snapshot.next_available_leaf_index as u64), [0; PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH]);

        (
            subtree_path, sorted_public_data_writes, sorted_public_data_writes_indexes, low_public_data_writes_preimages, low_public_data_writes_witnesses, pre_existing_public_data
        )
    }

    struct BaseRollupInputsBuilder {
        kernel_data: FixtureBuilder,
        pre_existing_notes: [Field; MAX_NEW_NOTE_HASHES_PER_TX],
        pre_existing_nullifiers: [NullifierLeafPreimage; MAX_NEW_NULLIFIERS_PER_TX],
        pre_existing_contracts: [Field; 2],
        pre_existing_public_data: [PublicDataTreeLeafPreimage; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
        pre_existing_blocks: [Field; 2],
        public_data_reads: BoundedVec<u64, MAX_PUBLIC_DATA_READS_PER_TEST>,
        public_data_writes:BoundedVec<(u64, PublicDataTreeLeaf), MAX_PUBLIC_DATA_WRITES_PER_TEST>,
        new_nullifiers: BoundedVec<NullifierInsertion, MAX_NEW_NULLIFIERS_PER_TEST>,
        constants: ConstantRollupData,
    }

    fn test_compute_empty_root<N>(size: [Field; N]) -> Field {
        compute_zero_hashes(size)[N - 1]
    }

    impl BaseRollupInputsBuilder {
        fn new() -> Self {
            let mut inputs: BaseRollupInputsBuilder = dep::std::unsafe::zeroed();
            inputs.kernel_data = FixtureBuilder::new();
            inputs.constants.global_variables.chain_id = 1;
            inputs.constants.global_variables.version = 0;

            inputs.pre_existing_blocks[0] = inputs.kernel_data.historical_header.hash();

            inputs
        }

        fn extract_subtree_sibling_path<FULL_HEIGHT, SIBLING_PATH_LENGTH>(
            path: [Field; FULL_HEIGHT],
            mut sibling_path: [Field; SIBLING_PATH_LENGTH]
        ) -> [Field; SIBLING_PATH_LENGTH] {
            let subtree_height = FULL_HEIGHT - SIBLING_PATH_LENGTH;
            for i in subtree_height..FULL_HEIGHT {
                sibling_path[i - subtree_height] = path[i];
            }
            sibling_path
        }

        fn update_nullifier_tree_with_new_leaves(
            mut self,
            nullifier_tree: &mut NonEmptyMerkleTree<MAX_NEW_NULLIFIERS_PER_TX, NULLIFIER_TREE_HEIGHT, NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH, NULLIFIER_SUBTREE_HEIGHT>,
            kernel_data: &mut KernelData,
            start_nullifier_tree_snapshot: AppendOnlyTreeSnapshot
        ) -> ([NullifierLeafPreimage; MAX_NEW_NULLIFIERS_PER_TX], [NullifierMembershipWitness; MAX_NEW_NULLIFIERS_PER_TX], [Field; MAX_NEW_NULLIFIERS_PER_TX], [u64; MAX_NEW_NULLIFIERS_PER_TX]) {
            let mut nullifier_predecessor_preimages: [NullifierLeafPreimage; MAX_NEW_NULLIFIERS_PER_TX] = dep::std::unsafe::zeroed();
            let mut low_nullifier_membership_witness: [NullifierMembershipWitness; MAX_NEW_NULLIFIERS_PER_TX] = dep::std::unsafe::zeroed();

            let sorted_new_nullifier_tuples = sort_high_to_low(
                self.new_nullifiers.storage.map(|insertion: NullifierInsertion| insertion.value),
                full_field_less_than
            );

            let mut sorted_nullifiers = [0; MAX_NEW_NULLIFIERS_PER_TX];
            let mut sorted_nullifiers_indexes = [0; MAX_NEW_NULLIFIERS_PER_TX];

            for i in 0..MAX_NEW_NULLIFIERS_PER_TX {
                if (i as u32) < (MAX_NEW_NULLIFIERS_PER_TEST as u32) {
                    sorted_nullifiers[i] = sorted_new_nullifier_tuples[i].value;
                    sorted_nullifiers_indexes[i] = sorted_new_nullifier_tuples[i].original_index;
                } else {
                    sorted_nullifiers[i] = 0;
                    sorted_nullifiers_indexes[i] = i;
                }
            }

            let mut pre_existing_nullifiers = self.pre_existing_nullifiers;

            for i in 0..MAX_NEW_NULLIFIERS_PER_TEST {
                if i < self.new_nullifiers.len() {
                    let sorted_tuple = sorted_new_nullifier_tuples[i];
                    let new_nullifier = sorted_tuple.value;
                    let original_index = sorted_tuple.original_index;

                    let low_index = self.new_nullifiers.get_unchecked(original_index).existing_index;

                    kernel_data.public_inputs.end.new_nullifiers[original_index].value = new_nullifier;

                    let mut low_preimage = pre_existing_nullifiers[low_index];
                    nullifier_predecessor_preimages[i] = low_preimage;
                    low_nullifier_membership_witness[i] = NullifierMembershipWitness {
                        leaf_index: low_index as Field,
                        sibling_path: nullifier_tree.get_sibling_path(low_index)
                    };

                    low_preimage.next_nullifier = new_nullifier;
                    low_preimage.next_index = start_nullifier_tree_snapshot.next_available_leaf_index as u64 + original_index;
                    pre_existing_nullifiers[low_index] = low_preimage;

                    nullifier_tree.update_leaf(low_index, low_preimage.hash());
                }
            }

            (
                nullifier_predecessor_preimages, low_nullifier_membership_witness, sorted_nullifiers, sorted_nullifiers_indexes
            )
        }

        fn build_inputs(mut self) -> BaseRollupInputs {
            let mut kernel_data = self.kernel_data.to_kernel_data();

            let start_note_hash_tree = NonEmptyMerkleTree::new(
                self.pre_existing_notes,
                [0; NOTE_HASH_TREE_HEIGHT],
                [0; NOTE_HASH_TREE_HEIGHT - NOTE_HASH_SUBTREE_HEIGHT],
                [0; NOTE_HASH_SUBTREE_HEIGHT]
            );
            let start_note_hash_tree_snapshot = AppendOnlyTreeSnapshot {
                root: start_note_hash_tree.get_root(),
                next_available_leaf_index: start_note_hash_tree.get_next_available_index() as u32
            };
            let note_hash_subtree_sibling_path = BaseRollupInputsBuilder::extract_subtree_sibling_path(
                start_note_hash_tree.get_sibling_path(self.pre_existing_notes.len()),
                [0; NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH]
            );

            let mut start_nullifier_tree = NonEmptyMerkleTree::new(
                self.pre_existing_nullifiers.map(|preimage: NullifierLeafPreimage| preimage.hash()),
                [0; NULLIFIER_TREE_HEIGHT],
                [0; NULLIFIER_TREE_HEIGHT - NULLIFIER_SUBTREE_HEIGHT],
                [0; NULLIFIER_SUBTREE_HEIGHT]
            );

            let start_nullifier_tree_snapshot = AppendOnlyTreeSnapshot {
                root: start_nullifier_tree.get_root(),
                next_available_leaf_index: start_nullifier_tree.get_next_available_index() as u32
            };

            let mut start_public_data_tree = NonEmptyMerkleTree::new(
                self.pre_existing_public_data.map(|preimage: PublicDataTreeLeafPreimage| preimage.hash()),
                [0; PUBLIC_DATA_TREE_HEIGHT],
                [0; PUBLIC_DATA_TREE_HEIGHT - PUBLIC_DATA_SUBTREE_HEIGHT],
                [0; PUBLIC_DATA_SUBTREE_HEIGHT]
            );
            let start_public_data_tree_snapshot = AppendOnlyTreeSnapshot {
                root: start_public_data_tree.get_root(),
                next_available_leaf_index: start_public_data_tree.get_next_available_index() as u32
            };

            let start_archive = NonEmptyMerkleTree::new(
                self.pre_existing_blocks,
                [0; ARCHIVE_HEIGHT],
                [0; ARCHIVE_HEIGHT - 1],
                [0; 1]
            );
            self.constants.last_archive = AppendOnlyTreeSnapshot {
                root: start_archive.get_root(),
                next_available_leaf_index: start_archive.get_next_available_index() as u32,
            };

            let (
                nullifier_predecessor_preimages,
                nullifier_predecessor_membership_witnesses,
                sorted_nullifiers,
                sorted_nullifier_indexes
            ) = self.update_nullifier_tree_with_new_leaves(
                &mut start_nullifier_tree,
                &mut kernel_data,
                start_nullifier_tree_snapshot
            );

            let nullifier_subtree_sibling_path = BaseRollupInputsBuilder::extract_subtree_sibling_path(
                start_nullifier_tree.get_sibling_path(self.pre_existing_nullifiers.len()),
                [0; NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH]
            );

            let (
                public_data_sibling_path,
                sorted_public_data_writes,
                sorted_public_data_writes_indexes,
                low_public_data_writes_preimages,
                low_public_data_writes_witnesses,
                _new_subtree
            ) = update_public_data_tree(
                &mut start_public_data_tree,
                &mut kernel_data,
                start_public_data_tree_snapshot,
                self.public_data_writes,
                self.pre_existing_public_data
            );

            let start = PartialStateReference {
                note_hash_tree: start_note_hash_tree_snapshot,
                nullifier_tree: start_nullifier_tree_snapshot,
                public_data_tree: start_public_data_tree_snapshot
            };

            let state_diff_hints = StateDiffHints {
                nullifier_predecessor_preimages,
                nullifier_predecessor_membership_witnesses,
                sorted_nullifiers,
                sorted_nullifier_indexes,
                note_hash_subtree_sibling_path,
                nullifier_subtree_sibling_path,
                public_data_sibling_path
            };

            BaseRollupInputs {
                kernel_data,
                start,
                state_diff_hints,
                sorted_public_data_writes,
                sorted_public_data_writes_indexes,
                low_public_data_writes_preimages,
                low_public_data_writes_witnesses,
                archive_root_membership_witness: ArchiveRootMembershipWitness { leaf_index: 0, sibling_path: start_archive.get_sibling_path(0) },
                constants: self.constants
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
    unconstrained fn new_note_hashes_tree() {
        let mut builder = BaseRollupInputsBuilder::new();

        let new_note_hashes = [27, 28, 29, 30, 31, 32];
        let mut new_note_hashes_vec = builder.kernel_data.new_note_hashes;
        for i in 0..new_note_hashes.len() {
            new_note_hashes_vec.push(SideEffect { value: new_note_hashes[i], counter: 0 });
        }
        builder.kernel_data.new_note_hashes = new_note_hashes_vec;
        let mut expected_commitments_tree = NonEmptyMerkleTree::new(
            [0; MAX_NEW_NOTE_HASHES_PER_TX * 2],
            [0; NOTE_HASH_TREE_HEIGHT],
            [0; NOTE_HASH_TREE_HEIGHT - NOTE_HASH_SUBTREE_HEIGHT - 1],
            [0; NOTE_HASH_SUBTREE_HEIGHT + 1]
        );

        let outputs = builder.execute();
        let expected_start_note_hash_tree_snapshot = AppendOnlyTreeSnapshot {
            root: expected_commitments_tree.get_root(),
            next_available_leaf_index: MAX_NEW_NOTE_HASHES_PER_TX as u32
        };
        assert(outputs.start.note_hash_tree.eq(expected_start_note_hash_tree_snapshot));

        for i in 0..new_note_hashes.len() {
            expected_commitments_tree.update_leaf(i + MAX_NEW_NOTE_HASHES_PER_TX, new_note_hashes[i]);
        }
        let expected_end_note_hash_tree_snapshot = AppendOnlyTreeSnapshot {
            root: expected_commitments_tree.get_root(),
            next_available_leaf_index: (MAX_NEW_NOTE_HASHES_PER_TX * 2) as u32
        };
        assert(outputs.end.note_hash_tree.eq(expected_end_note_hash_tree_snapshot));
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
            nullifier : 0,
            next_nullifier : 7,
            next_index : 1,
        };
        builder.pre_existing_nullifiers[1] = NullifierLeafPreimage {
            nullifier : 7,
            next_nullifier : 0,
            next_index : 0,
        };

        builder.succeeds();
    }

    #[test]
    unconstrained fn nullifier_insertion_test() {
        let mut builder = BaseRollupInputsBuilder::new();

        builder.pre_existing_nullifiers[0] = NullifierLeafPreimage {
            nullifier : 0,
            next_nullifier : 7,
            next_index : 1,
        };
        builder.pre_existing_nullifiers[1] = NullifierLeafPreimage {
            nullifier : 7,
            next_nullifier : 0,
            next_index : 0,
        };

        builder.new_nullifiers.push(NullifierInsertion { existing_index: 0, value: 1 });
        let mut tree_nullifiers = [NullifierLeafPreimage::empty(); MAX_NEW_NULLIFIERS_PER_TX * 2];
        tree_nullifiers[0] = NullifierLeafPreimage {
            nullifier : 0,
            next_nullifier : 1,
            next_index : MAX_NEW_NULLIFIERS_PER_TX,
        };
        tree_nullifiers[1] = builder.pre_existing_nullifiers[1];
        tree_nullifiers[MAX_NEW_NULLIFIERS_PER_TX] = NullifierLeafPreimage {
            nullifier : 1,
            next_nullifier : 7,
            next_index : 1,
        };

        let mut end_nullifier_tree = NonEmptyMerkleTree::new(
            tree_nullifiers.map(|preimage: NullifierLeafPreimage| preimage.hash()),
            [0; NULLIFIER_TREE_HEIGHT],
            [0; NULLIFIER_TREE_HEIGHT - NULLIFIER_SUBTREE_HEIGHT - 1],
            [0; NULLIFIER_SUBTREE_HEIGHT + 1]
        );

        let output = builder.execute();

        assert(
            output.end.nullifier_tree.eq(
                AppendOnlyTreeSnapshot { root: end_nullifier_tree.get_root(), next_available_leaf_index: 2 * MAX_NEW_NULLIFIERS_PER_TX as u32 }
            )
        );
    }

    #[test]
    unconstrained fn new_nullifier_tree_all_larger() {
        let mut builder = BaseRollupInputsBuilder::new();

        builder.pre_existing_nullifiers[0] = NullifierLeafPreimage {
            nullifier : 0,
            next_nullifier : 7,
            next_index : 1,
        };
        builder.pre_existing_nullifiers[1] = NullifierLeafPreimage {
            nullifier : 7,
            next_nullifier : 0,
            next_index : 0,
        };

        builder.new_nullifiers.push(NullifierInsertion { existing_index: 1, value: 8 });
        for i in 1..builder.new_nullifiers.max_len() {
            builder.new_nullifiers.push(NullifierInsertion { existing_index: 1, value: (8 + i) as Field });
        }

        let output = builder.execute();
        let mut tree_nullifiers = [NullifierLeafPreimage::empty(); MAX_NEW_NULLIFIERS_PER_TX * 2];
        tree_nullifiers[0] = builder.pre_existing_nullifiers[0];

        tree_nullifiers[1] = NullifierLeafPreimage {
            nullifier : 7,
            next_nullifier : 8,
            next_index : MAX_NEW_NULLIFIERS_PER_TX,
        };

        let last_index = builder.new_nullifiers.max_len() - 1;
        for i in 0..last_index {
            tree_nullifiers[MAX_NEW_NULLIFIERS_PER_TX + i] = NullifierLeafPreimage {
                nullifier : (8 + i) as Field,
                next_nullifier : (8 + i + 1) as Field,
                next_index : MAX_NEW_NULLIFIERS_PER_TX + i + 1,
            };
        }
        tree_nullifiers[MAX_NEW_NULLIFIERS_PER_TX+last_index] = NullifierLeafPreimage {
            nullifier : (8 + last_index) as Field,
            next_nullifier : 0,
            next_index : 0,
        };

        let mut end_nullifier_tree = NonEmptyMerkleTree::new(
            tree_nullifiers.map(|preimage: NullifierLeafPreimage| preimage.hash()),
            [0; NULLIFIER_TREE_HEIGHT],
            [0; NULLIFIER_TREE_HEIGHT - NULLIFIER_SUBTREE_HEIGHT - 1],
            [0; NULLIFIER_SUBTREE_HEIGHT + 1]
        );

        assert(
            output.end.nullifier_tree.eq(
                AppendOnlyTreeSnapshot { root: end_nullifier_tree.get_root(), next_available_leaf_index: 2 * MAX_NEW_NULLIFIERS_PER_TX as u32 }
            )
        );
    }

    #[test(should_fail_with = "Invalid low leaf")]
    unconstrained fn new_nullifier_tree_double_spend() {
        let mut builder = BaseRollupInputsBuilder::new();

        builder.pre_existing_nullifiers[0] = NullifierLeafPreimage {
            nullifier : 0,
            next_nullifier : 7,
            next_index : 1,
        };
        builder.pre_existing_nullifiers[1] = NullifierLeafPreimage {
            nullifier : 7,
            next_nullifier : 0,
            next_index : 0,
        };

        builder.new_nullifiers.push(NullifierInsertion { existing_index: 1, value: 8 });
        builder.new_nullifiers.push(NullifierInsertion { existing_index: 1, value: 8 });

        builder.fails();
    }

    #[test(should_fail_with = "Invalid low leaf")]
    unconstrained fn new_nullifier_tree_double_spend_same_batch() {
        let mut builder = BaseRollupInputsBuilder::new();

        builder.pre_existing_nullifiers[0] = NullifierLeafPreimage {
            nullifier : 0,
            next_nullifier : 7,
            next_index : 1,
        };
        builder.pre_existing_nullifiers[1] = NullifierLeafPreimage {
            nullifier : 7,
            next_nullifier : 0,
            next_index : 0,
        };

        builder.new_nullifiers.push(NullifierInsertion { existing_index: 1, value: 8 });
        builder.new_nullifiers.push(NullifierInsertion { existing_index: 1, value: 8 });

        builder.fails();
    }

    #[test]
    unconstrained fn empty_tx_effects_hash() {
        let outputs = BaseRollupInputsBuilder::new().execute();

        let hash_input_flattened = [0; TX_EFFECTS_HASH_INPUT_FIELDS * 32];
        let sha_digest = dep::std::hash::sha256(hash_input_flattened);
        let expected_tx_effects_hash = field_from_bytes_32_trunc(sha_digest);
        assert_eq(outputs.txs_effects_hash, expected_tx_effects_hash);
    }

    #[test]
    unconstrained fn empty_block_out_hash() {
        let outputs = BaseRollupInputsBuilder::new().execute();

        let hash_input_flattened = [0; MAX_NEW_L2_TO_L1_MSGS_PER_TX * 32];
        let sha_digest = dep::std::hash::sha256(hash_input_flattened);
        let expected_out_hash = field_from_bytes_32_trunc(sha_digest);
        assert_eq(outputs.out_hash, expected_out_hash);
    }

    #[test]
    unconstrained fn nonempty_block_out_hash() {
        let mut end: CombinedAccumulatedData = dep::std::unsafe::zeroed();
        end.new_l2_to_l1_msgs[MAX_NEW_L2_TO_L1_MSGS_PER_TX - 1] = 123;

        let out_hash = compute_kernel_out_hash(end);

        let mut hash_input_flattened = [0; MAX_NEW_L2_TO_L1_MSGS_PER_TX * 32];
        hash_input_flattened[MAX_NEW_L2_TO_L1_MSGS_PER_TX * 32 - 1] = 123;
        let sha_digest = dep::std::hash::sha256(hash_input_flattened);
        let expected_out_hash = field_from_bytes_32_trunc(sha_digest);
        assert_eq(out_hash, expected_out_hash);
    }

    #[test(should_fail_with = "membership check failed")]
    unconstrained fn compute_membership_archive_negative() {
        let mut inputs = BaseRollupInputsBuilder::new().build_inputs();
        inputs.archive_root_membership_witness.sibling_path[0] = 27;
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

    #[test(should_fail_with = "kernel max_block_number is smaller than block number")]
    unconstrained fn constants_dont_satisfy_smaller_max_block_number() {
        let mut builder = BaseRollupInputsBuilder::new();
        builder.constants.global_variables.block_number = 42;
        builder.kernel_data.set_max_block_number(5);
        builder.fails();
    }

    #[test]
    unconstrained fn constants_satisfy_equal_max_block_number() {
        let mut builder = BaseRollupInputsBuilder::new();
        builder.constants.global_variables.block_number = 42;
        builder.kernel_data.set_max_block_number(42);
        builder.succeeds();
    }

    #[test]
    unconstrained fn constants_satisfy_larger_max_block_number() {
        let mut builder = BaseRollupInputsBuilder::new();
        builder.constants.global_variables.block_number = 42;
        builder.kernel_data.set_max_block_number(4294967295);
        builder.succeeds();
    }

    #[test]
    unconstrained fn subtree_height_is_0() {
        let outputs = BaseRollupInputsBuilder::new().execute();

        assert_eq(outputs.height_in_block_tree, 0);
    }

    #[test]
    unconstrained fn single_public_state_write() {
        let mut builder = BaseRollupInputsBuilder::new();

        builder.pre_existing_public_data[0] = PublicDataTreeLeafPreimage {
            slot: 27,
            value: 28,
            next_slot: 0,
            next_index: 0,
        };
        builder.public_data_writes.push((0, PublicDataTreeLeaf { slot: 27, value: 29 }));
        let outputs = builder.execute();

        let updated_leaf = PublicDataTreeLeafPreimage { slot: 27, value: 29, next_slot: 0, next_index: 0 };

        let mut expected_public_data_tree = NonEmptyMerkleTree::new(
            [updated_leaf.hash(), 0],
            [0; PUBLIC_DATA_TREE_HEIGHT],
            [0; PUBLIC_DATA_TREE_HEIGHT - 1],
            [0; 1]
        );

        assert_eq(outputs.end.public_data_tree.root, expected_public_data_tree.get_root());
    }

    #[test]
    unconstrained fn multiple_public_state_read_writes() {
        let mut builder = BaseRollupInputsBuilder::new();

        builder.pre_existing_public_data[0] = PublicDataTreeLeafPreimage {
            slot: 20,
            value: 40,
            next_slot: 28,
            next_index: 1,
        };
        builder.pre_existing_public_data[1] = PublicDataTreeLeafPreimage {
            slot: 28,
            value: 41,
            next_slot: 29,
            next_index: 2,
        };
        builder.pre_existing_public_data[2] = PublicDataTreeLeafPreimage {
            slot: 29,
            value: 42,
            next_slot: 30,
            next_index: 3,
        };
        builder.pre_existing_public_data[3] = PublicDataTreeLeafPreimage {
            slot: 30,
            value: 43,
            next_slot: 0,
            next_index: 0,
        };
        builder.public_data_reads.push(0);
        builder.public_data_writes.push((0, PublicDataTreeLeaf { slot: 25, value: 60 }));
        builder.public_data_reads.push(4);
        builder.public_data_writes.push((0, PublicDataTreeLeaf { slot: 20, value: 90 }));

        let outputs = builder.execute();

        let mut public_data_leaves = [0; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX * 2];
        public_data_leaves[0] = PublicDataTreeLeafPreimage {
            slot: 20,
            value: 90,
            next_slot: 25,
            next_index: MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
        }.hash();
        public_data_leaves[1] = PublicDataTreeLeafPreimage {
            slot: 28,
            value: 41,
            next_slot: 29,
            next_index: 2,
        }.hash();
        public_data_leaves[2] = PublicDataTreeLeafPreimage {
            slot: 29,
            value: 42,
            next_slot: 30,
            next_index: 3,
        }.hash();
        public_data_leaves[3] = PublicDataTreeLeafPreimage {
            slot: 30,
            value: 43,
            next_slot: 0,
            next_index: 0,
        }.hash();
        public_data_leaves[MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX] = PublicDataTreeLeafPreimage {
            slot: 25,
            value: 60,
            next_slot: 28,
            next_index: 1,
        }.hash();

        let mut expected_public_data_tree = NonEmptyMerkleTree::new(
            public_data_leaves,
            [0; PUBLIC_DATA_TREE_HEIGHT],
            [0; PUBLIC_DATA_TREE_HEIGHT - PUBLIC_DATA_SUBTREE_HEIGHT - 1],
            [0; PUBLIC_DATA_SUBTREE_HEIGHT + 1]
        );

        assert_eq(outputs.end.public_data_tree.root, expected_public_data_tree.get_root());
    }
}
