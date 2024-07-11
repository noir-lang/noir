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
    data::{
    public_data_hint::PublicDataHint,
    hash::{compute_public_data_tree_index, compute_public_data_tree_value}
},
    storage::map::derive_storage_slot_in_map, address::AztecAddress,
    abis::{
    append_only_tree_snapshot::AppendOnlyTreeSnapshot, nullifier_leaf_preimage::NullifierLeafPreimage,
    public_data_update_request::PublicDataUpdateRequest, public_data_read::PublicDataRead,
    kernel_data::KernelData
},
    constants::{
    NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH, NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH,
    PUBLIC_DATA_TREE_HEIGHT, NOTE_HASH_SUBTREE_HEIGHT, MAX_NOTE_HASHES_PER_TX,
    MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, MAX_PUBLIC_DATA_READS_PER_TX, MAX_NULLIFIERS_PER_TX,
    MAX_L2_TO_L1_MSGS_PER_TX, NULLIFIER_SUBTREE_HEIGHT, NULLIFIER_TREE_HEIGHT,
    PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH, PUBLIC_DATA_SUBTREE_HEIGHT,
    PROTOCOL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, ARCHIVE_HEIGHT, GAS_TOKEN_ADDRESS,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PUBLIC_KERNEL_TAIL_INDEX, PRIVATE_KERNEL_EMPTY_INDEX,
    PRIVATE_KERNEL_TAIL_INDEX
},
    merkle_tree::{
    append_only_tree, assert_check_membership, calculate_empty_tree_root, calculate_subtree_root,
    indexed_tree, MembershipWitness
},
    partial_state_reference::PartialStateReference, public_data_tree_leaf::PublicDataTreeLeaf,
    public_data_tree_leaf_preimage::PublicDataTreeLeafPreimage, traits::is_empty,
    utils::{field::{full_field_less_than, full_field_greater_than}, uint256::U256}
};

global ALLOWED_PREVIOUS_CIRCUITS = [
    PRIVATE_KERNEL_EMPTY_INDEX,
    PRIVATE_KERNEL_TAIL_INDEX,
    PUBLIC_KERNEL_TAIL_INDEX,
];

struct BaseRollupInputs {
    kernel_data: KernelData,
    start: PartialStateReference,

    state_diff_hints: StateDiffHints,
    fee_payer_gas_token_balance_read_hint: PublicDataHint,

    // TODO: The following 6 values are eventually going to be nuked from here. See discussion:
    // https://aztecprotocol.slack.com/archives/C060PU5R327/p1701965354071269
    sorted_public_data_writes: [PublicDataTreeLeaf; MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
    sorted_public_data_writes_indexes: [u32; MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
    low_public_data_writes_preimages: [PublicDataTreeLeafPreimage; MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
    low_public_data_writes_witnesses: [MembershipWitness<PUBLIC_DATA_TREE_HEIGHT>; MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],

    archive_root_membership_witness: MembershipWitness<ARCHIVE_HEIGHT>,

    constants: ConstantRollupData,
}

impl BaseRollupInputs {
    pub fn base_rollup_circuit(self) -> BaseOrMergeRollupPublicInputs {
        if !dep::std::runtime::is_unconstrained() {
            // Recursively verify the tube proof if straight from private, or the public kernel tail
            self.kernel_data.verify();
            // TODO(#7410) we need the tube vk to reinstate this
            // self.kernel_data.validate_in_vk_tree(ALLOWED_PREVIOUS_CIRCUITS);
        }
        // Verify the kernel chain_id and versions
        assert(
            self.kernel_data.public_inputs.constants.tx_context.chain_id
            == self.constants.global_variables.chain_id, "kernel chain_id does not match the rollup chain_id"
        );
        assert(
            self.kernel_data.public_inputs.constants.tx_context.version
            == self.constants.global_variables.version, "kernel version does not match the rollup version"
        );
        assert(
            self.kernel_data.public_inputs.constants.vk_tree_root == self.constants.vk_tree_root, "kernel vk_tree_root does not match the rollup vk_tree_root"
        );

        // Verify the kernel global variables if set, note these can be empty if this is a request coming directly from the private kernel tail.
        // TODO(@spalladino) How can we check that this is a request coming from the private kernel tail?
        assert(
            self.kernel_data.public_inputs.constants.global_variables.is_empty()
            | (self.kernel_data.public_inputs.constants.global_variables
                == self.constants.global_variables), "kernel global variables do not match the rollup global variables"
        );

        self.validate_kernel_start_state();

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

        // Inject protocol update requests for deducting tx_fee from fee_payer's balance
        let gas_fees = self.constants.global_variables.gas_fees;
        let transaction_fee = self.kernel_data.public_inputs.compute_transaction_fee(gas_fees);
        let all_public_data_update_requests = self.calculate_all_public_data_update_requests(transaction_fee);

        // Validate public data update requests and update public data tree
        let end_public_data_tree_snapshot = self.validate_and_process_public_state(all_public_data_update_requests);

        // Calculate the tx effects hash of the transaction
        let tx_effects_hash = compute_tx_effects_hash(
            self.kernel_data.public_inputs.end,
            self.kernel_data.public_inputs.revert_code,
            transaction_fee,
            all_public_data_update_requests
        );
        let out_hash = compute_kernel_out_hash(self.kernel_data.public_inputs.end);

        // Perform membership checks that the notes provided exist within the historical trees data
        self.perform_archive_membership_checks();

        BaseOrMergeRollupPublicInputs {
            rollup_type: BASE_ROLLUP_TYPE,
            num_txs: 1,
            constants: self.constants,
            start: self.start,
            end: PartialStateReference {
                note_hash_tree: end_note_hash_tree_snapshot,
                nullifier_tree: end_nullifier_tree_snapshot,
                public_data_tree: end_public_data_tree_snapshot
            },
            txs_effects_hash: tx_effects_hash,
            out_hash,
            accumulated_fees: transaction_fee
        }
    }

    // TODO(Kev): This should say calculate_commitments_subtree_root
    // Cpp code says calculate_commitments_subtree, so I'm leaving it as is for now
    fn calculate_commitments_subtree(self) -> Field {
        calculate_subtree_root(self.kernel_data.public_inputs.end.note_hashes)
    }

    fn check_nullifier_tree_non_membership_and_insert_to_tree(self) -> AppendOnlyTreeSnapshot {
        indexed_tree::batch_insert(
            self.start.nullifier_tree,
            self.kernel_data.public_inputs.end.nullifiers,
            self.state_diff_hints.sorted_nullifiers,
            self.state_diff_hints.sorted_nullifier_indexes,
            self.state_diff_hints.nullifier_subtree_sibling_path,
            self.state_diff_hints.nullifier_predecessor_preimages,
            self.state_diff_hints.nullifier_predecessor_membership_witnesses.map(
                |witness: MembershipWitness<NULLIFIER_TREE_HEIGHT>| {
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
            |low_leaf: NullifierLeafPreimage, nullifier: Field, nullifier_index: u32| { // Update low leaf
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

    fn validate_kernel_start_state(self) {
        let kernel_state = self.kernel_data.public_inputs.start_state;
        if !is_empty(kernel_state) {
            assert(
                kernel_state.note_hash_tree.eq(self.start.note_hash_tree), "Mismatch start state for note hash tree"
            );
            assert(
                kernel_state.nullifier_tree.eq(self.start.nullifier_tree), "Mismatch start state for nullifier tree"
            );
            assert(
                kernel_state.public_data_tree.eq(self.start.public_data_tree), "Mismatch start state for public data tree"
            );
        }
    }

    fn validate_and_process_public_state(
        self,
        all_update_requests: [PublicDataUpdateRequest; MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX]
    ) -> AppendOnlyTreeSnapshot {
        let end_public_data_tree_snapshot = insert_public_data_update_requests(
            self.start.public_data_tree,
            all_update_requests.map(PublicDataTreeLeaf::from),
            self.sorted_public_data_writes,
            self.sorted_public_data_writes_indexes,
            self.low_public_data_writes_preimages,
            self.low_public_data_writes_witnesses,
            self.state_diff_hints.public_data_sibling_path
        );

        end_public_data_tree_snapshot
    }

    // Returns an array with all public data update requests for this tx. This includes all update requests
    // generated by app circuits, plus the protocol update requests injected by this circuit. The only protocol
    // update request we have at the time of this writing is deducting the tx_fee from the fee_payer balance.
    fn calculate_all_public_data_update_requests(
        self,
        tx_fee: Field
    ) -> [PublicDataUpdateRequest; MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX] {
        let mut all_update_requests: [PublicDataUpdateRequest; MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX] = [PublicDataUpdateRequest::empty(); MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX];
        for i in 0..MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX {
            all_update_requests[i] = self.kernel_data.public_inputs.end.public_data_update_requests[i];
        }

        let (payment_update_request, payment_update_index) = self.build_or_patch_payment_update_request(tx_fee);
        all_update_requests[payment_update_index] = payment_update_request;

        all_update_requests
    }

    // Deducts the tx_fee from the GasToken balance of the fee_payer. If there is already a PublicDataUpdateRequest
    // in this tx for their balance (because they issued a 'claim' to increase their balance by bridging from L1),
    // update it by subtracting the tx_fee. Otherwise, build a new PublicDataUpdateRequest to subtract the tx_fee
    // from the balance of the fee_payer, using the fee_payer_gas_token_balance_read_hint to read the current balance.
    // Returns the data update request that subtracts the tx_fee from the fee_payer's balance, and the index where it
    // should be inserted in the public data update requests array.
    fn build_or_patch_payment_update_request(self, tx_fee: Field) -> (PublicDataUpdateRequest, u32) {
        let fee_payer = self.kernel_data.public_inputs.fee_payer;

        // TODO(@spalladino) Eventually remove the is_zero condition as we should always charge fees to every tx
        if !fee_payer.is_zero() {
            let read_hint = self.fee_payer_gas_token_balance_read_hint;
            let leaf_slot = compute_fee_payer_gas_token_balance_leaf_slot(fee_payer);

            if read_hint.leaf_slot == 0 {
                // Is there a balance update already in this tx? If so, update it and return its index.
                let existing_update_index = self.find_fee_payer_gas_token_update_index(leaf_slot);
                let existing_update = self.kernel_data.public_inputs.end.public_data_update_requests[existing_update_index];
                assert(
                    existing_update.leaf_slot == leaf_slot, "Wrong leaf slot for gas token balance update request"
                );
                assert(
                    !existing_update.new_value.lt(tx_fee), "Not enough balance for fee payer after claim to pay for transaction"
                );

                let new_value = compute_public_data_tree_value(existing_update.new_value - tx_fee);
                let protocol_update_request = PublicDataUpdateRequest { leaf_slot, new_value, counter: 0 };
                (protocol_update_request, existing_update_index as u32)
            } else {
                // Otherwise, build a new one to be inserted into the protocol update requests at the end of the array.
                read_hint.validate(self.start.public_data_tree.root);

                let balance = read_hint.value;
                assert(read_hint.leaf_slot == leaf_slot, "Wrong leaf slot for gas token balance read hint");
                assert(!balance.lt(tx_fee), "Not enough balance for fee payer to pay for transaction");

                let new_value = compute_public_data_tree_value(balance - tx_fee);
                let protocol_update_request = PublicDataUpdateRequest { leaf_slot, new_value, counter: 0 };
                (protocol_update_request, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX)
            }
        } else {
            // Nothing to do, just place an empty update request at the end of the array
            (PublicDataUpdateRequest::empty(), MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX)
        }
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

    unconstrained fn find_fee_payer_gas_token_update_index(self, leaf_slot: Field) -> u32 {
        let mut update_index = MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX + 1;
        for i in 0..MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX {
            if self.kernel_data.public_inputs.end.public_data_update_requests[i].leaf_slot == leaf_slot {
                update_index = i;
            }
        }
        assert(
            update_index < MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, "Could not find fee payer gas token update request"
        );
        update_index
    }
}

fn insert_public_data_update_requests(
    prev_snapshot: AppendOnlyTreeSnapshot,
    public_data_writes: [PublicDataTreeLeaf; MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
    sorted_public_data_writes: [PublicDataTreeLeaf; MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
    sorted_public_data_writes_indexes: [u32; MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
    low_public_data_writes_preimages: [PublicDataTreeLeafPreimage; MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
    low_public_data_writes_witnesses: [MembershipWitness<PUBLIC_DATA_TREE_HEIGHT>; MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
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
            |witness: MembershipWitness<PUBLIC_DATA_TREE_HEIGHT>| {
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
        |low_preimage: PublicDataTreeLeafPreimage, write: PublicDataTreeLeaf, write_index: u32| { // Update low leaf
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

fn compute_fee_payer_gas_token_balance_leaf_slot(fee_payer: AztecAddress) -> Field {
    let balances_slot_in_gas_token_contract = 1;
    let gas_token = AztecAddress::from_field(GAS_TOKEN_ADDRESS);
    let fee_payer_balance_slot_in_gas_token_contract = derive_storage_slot_in_map(balances_slot_in_gas_token_contract, fee_payer);
    compute_public_data_tree_index(gas_token, fee_payer_balance_slot_in_gas_token_contract)
}

#[test]
fn consistent_not_hash_subtree_width() {
    assert_eq(
        MAX_NOTE_HASHES_PER_TX as Field, 2.pow_32(NOTE_HASH_SUBTREE_HEIGHT as Field), "note hash subtree width is incorrect"
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
        base::{
        state_diff_hints::StateDiffHints,
        base_rollup_inputs::{BaseRollupInputs, compute_fee_payer_gas_token_balance_leaf_slot, ALLOWED_PREVIOUS_CIRCUITS}
    },
        components::{TX_EFFECTS_HASH_INPUT_FIELDS, compute_kernel_out_hash}
    };
    use dep::types::{
        abis::{
        append_only_tree_snapshot::AppendOnlyTreeSnapshot,
        nullifier_leaf_preimage::NullifierLeafPreimage, public_data_read::PublicDataRead,
        public_data_update_request::PublicDataUpdateRequest, kernel_data::KernelData,
        accumulated_data::CombinedAccumulatedData, gas::Gas
    },
        merkle_tree::MembershipWitness, data::public_data_hint::PublicDataHint,
        address::{AztecAddress, EthAddress},
        constants::{
        ARCHIVE_HEIGHT, MAX_PUBLIC_DATA_READS_PER_TX, MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
        MAX_NOTE_HASHES_PER_TX, MAX_NULLIFIERS_PER_TX, NOTE_HASH_SUBTREE_SIBLING_PATH_LENGTH,
        NOTE_HASH_TREE_HEIGHT, NOTE_HASH_SUBTREE_HEIGHT, NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH,
        NULLIFIER_TREE_HEIGHT, NULLIFIER_SUBTREE_HEIGHT, PUBLIC_DATA_TREE_HEIGHT,
        PUBLIC_DATA_SUBTREE_HEIGHT, PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH, MAX_L2_TO_L1_MSGS_PER_TX,
        PROTOCOL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
        BASE_ROLLUP_INDEX, PRIVATE_KERNEL_EMPTY_INDEX, PRIVATE_KERNEL_TAIL_INDEX,
        PUBLIC_KERNEL_TAIL_INDEX
    },
        contract_class_id::ContractClassId, partial_state_reference::PartialStateReference,
        public_data_tree_leaf::PublicDataTreeLeaf,
        public_data_tree_leaf_preimage::PublicDataTreeLeafPreimage,
        tests::{
        fixtures, fixture_builder::FixtureBuilder,
        merkle_tree_utils::{NonEmptyMerkleTree, compute_zero_hashes}, sort::sort_high_to_low
    },
        utils::{field::{full_field_less_than, field_from_bytes_32_trunc, field_from_bytes}, uint256::U256},
        traits::Empty
    };

    struct NullifierInsertion {
        existing_index: u32,
        value: Field,
    }

    global MAX_nullifiers_PER_TEST = 4;
    global MAX_PUBLIC_DATA_READS_PER_TEST = 2;

    fn update_public_data_tree<EXISTING_LEAVES, SUBTREE_SIBLING_PATH_LENGTH, SUBTREE_HEIGHT>(
        public_data_tree: &mut NonEmptyMerkleTree<EXISTING_LEAVES, PUBLIC_DATA_TREE_HEIGHT, SUBTREE_SIBLING_PATH_LENGTH, SUBTREE_HEIGHT>,
        kernel_data: &mut KernelData,
        snapshot: AppendOnlyTreeSnapshot,
        user_public_data_writes: BoundedVec<(u32, PublicDataTreeLeaf), MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
        protocol_public_data_writes: BoundedVec<(u32, PublicDataTreeLeaf), PROTOCOL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
        mut final_public_data_writes: BoundedVec<(u32, PublicDataTreeLeaf), MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
        mut pre_existing_public_data: [PublicDataTreeLeafPreimage; EXISTING_LEAVES]
    ) -> ([Field; PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH], [PublicDataTreeLeaf; MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX], [u32; MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX], [PublicDataTreeLeafPreimage; MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX], [MembershipWitness<PUBLIC_DATA_TREE_HEIGHT>; MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX], [PublicDataTreeLeafPreimage; EXISTING_LEAVES]) {
        let mut subtree_path = [0; PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH];
        let mut sorted_public_data_writes = [PublicDataTreeLeaf::empty(); MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX];
        let mut sorted_public_data_writes_indexes = [0 as u32; MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX];
        let mut low_public_data_writes_preimages = [PublicDataTreeLeafPreimage::empty(); MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX];
        let mut low_public_data_writes_witnesses = [MembershipWitness::empty(); MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX];
        let mut new_subtree = [PublicDataTreeLeafPreimage::empty(); MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX];

        for i in 0..MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX {
            if i < (user_public_data_writes.len()) {
                let leaf = user_public_data_writes.get_unchecked(i).1;

                kernel_data.public_inputs.end.public_data_update_requests[i] = PublicDataUpdateRequest {
                    leaf_slot : leaf.slot,
                    new_value : leaf.value,
                    counter : 0
                };
            }
        }

        // Default final_public_data_writes to user_public_data_writes + protocol_public_data_writes if not set
        if final_public_data_writes.len() == 0 {
            final_public_data_writes.extend_from_array(user_public_data_writes.storage);
            final_public_data_writes.extend_from_array(protocol_public_data_writes.storage);
        }

        let mut sorted_write_tuples = sort_high_to_low(
            final_public_data_writes.storage,
            |(_, leaf_a): (u32, PublicDataTreeLeaf),(_,leaf_b):(u32, PublicDataTreeLeaf)| full_field_less_than(leaf_a.slot, leaf_b.slot)
        );

        for i in 0..MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX {
            if i < final_public_data_writes.len() {
                let (low_leaf_index, leaf): (u32, PublicDataTreeLeaf) = sorted_write_tuples[i].value;

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
                            next_index: MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX + i
                        };
                    }
                    low_public_data_writes_preimages[i] = low_leaf;
                    low_public_data_writes_witnesses[i] = MembershipWitness {
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

        subtree_path = BaseRollupInputsBuilder::extract_subtree_sibling_path(public_data_tree.get_sibling_path(snapshot.next_available_leaf_index as u32), [0; PUBLIC_DATA_SUBTREE_SIBLING_PATH_LENGTH]);

        (
            subtree_path, sorted_public_data_writes, sorted_public_data_writes_indexes, low_public_data_writes_preimages, low_public_data_writes_witnesses, pre_existing_public_data
        )
    }

    struct BaseRollupInputsBuilder {
        kernel_data: FixtureBuilder,
        pre_existing_notes: [Field; MAX_NOTE_HASHES_PER_TX],
        pre_existing_nullifiers: [NullifierLeafPreimage; MAX_NULLIFIERS_PER_TX],
        pre_existing_contracts: [Field; 2],
        pre_existing_public_data: [PublicDataTreeLeafPreimage; MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
        pre_existing_blocks: [Field; 2],
        public_data_reads: BoundedVec<u32, MAX_PUBLIC_DATA_READS_PER_TEST>,
        // Public data writes as returned from the last public kernel (ie all data writes generated by app code)
        public_data_writes: BoundedVec<(u32, PublicDataTreeLeaf), MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
        // New public data writes to be created by the protocol (eg a data update request for updating fee payer balance)
        protocol_public_data_writes: BoundedVec<(u32, PublicDataTreeLeaf), PROTOCOL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
        // Public data writes after processing by the base rollup circuit (defaults to public_data_writes ++ protocol_public_data_writes if empty)
        final_public_data_writes: BoundedVec<(u32, PublicDataTreeLeaf), MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
        nullifiers: BoundedVec<NullifierInsertion, MAX_nullifiers_PER_TEST>,
        constants: ConstantRollupData,
        // Index of the item in the pre_existing_public_data array that contains the fee payer's gas token balance.
        // Used for building the public data hint read for the payment update request. If set to none, no hint is built.
        fee_payer_gas_token_balance_pre_existing_public_data_index: Option<u32>
    }

    fn test_compute_empty_root<N>(size: [Field; N]) -> Field {
        compute_zero_hashes(size)[N - 1]
    }

    impl BaseRollupInputsBuilder {
        fn new() -> Self {
            let mut inputs = BaseRollupInputsBuilder::empty();
            inputs.kernel_data = FixtureBuilder::new().in_vk_tree(PRIVATE_KERNEL_TAIL_INDEX);
            inputs.constants.global_variables.chain_id = fixtures::CHAIN_ID;
            inputs.constants.global_variables.version = fixtures::VERSION;
            inputs.constants.vk_tree_root = inputs.kernel_data.vk_tree_root;

            inputs.pre_existing_blocks[0] = inputs.kernel_data.historical_header.hash();

            inputs
        }

        fn build_fee_payer_gas_token_balance_read_hint(
            self,
            start_public_data_tree: NonEmptyMerkleTree<MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PUBLIC_DATA_TREE_HEIGHT, PUBLIC_DATA_TREE_HEIGHT - PUBLIC_DATA_SUBTREE_HEIGHT, PUBLIC_DATA_SUBTREE_HEIGHT>
        ) -> PublicDataHint {
            self.fee_payer_gas_token_balance_pre_existing_public_data_index.map_or(
                PublicDataHint::empty(),
                |leaf_index_u32: u32| {
                    let leaf_index = leaf_index_u32 as Field;
                    let leaf_preimage = self.pre_existing_public_data[leaf_index];
                    let membership_witness = MembershipWitness { leaf_index, sibling_path: start_public_data_tree.get_sibling_path(leaf_index_u32) };
                    PublicDataHint {
                        leaf_slot: leaf_preimage.slot,
                        value: leaf_preimage.value,
                        override_counter: 0,
                        membership_witness,
                        leaf_preimage
                    }
                }
            )
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
            nullifier_tree: &mut NonEmptyMerkleTree<MAX_NULLIFIERS_PER_TX, NULLIFIER_TREE_HEIGHT, NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH, NULLIFIER_SUBTREE_HEIGHT>,
            kernel_data: &mut KernelData,
            start_nullifier_tree_snapshot: AppendOnlyTreeSnapshot
        ) -> ([NullifierLeafPreimage; MAX_NULLIFIERS_PER_TX], [MembershipWitness<NULLIFIER_TREE_HEIGHT>; MAX_NULLIFIERS_PER_TX], [Field; MAX_NULLIFIERS_PER_TX], [u32; MAX_NULLIFIERS_PER_TX]) {
            let mut nullifier_predecessor_preimages = [NullifierLeafPreimage::empty(); MAX_NULLIFIERS_PER_TX];
            let mut low_nullifier_membership_witness = [MembershipWitness::empty(); MAX_NULLIFIERS_PER_TX];

            let sorted_new_nullifier_tuples = sort_high_to_low(
                self.nullifiers.storage.map(|insertion: NullifierInsertion| insertion.value),
                full_field_less_than
            );

            let mut sorted_nullifiers = [0; MAX_NULLIFIERS_PER_TX];
            let mut sorted_nullifiers_indexes = [0; MAX_NULLIFIERS_PER_TX];

            for i in 0..MAX_NULLIFIERS_PER_TX {
                if (i as u32) < (MAX_nullifiers_PER_TEST as u32) {
                    sorted_nullifiers[i] = sorted_new_nullifier_tuples[i].value;
                    sorted_nullifiers_indexes[i] = sorted_new_nullifier_tuples[i].original_index;
                } else {
                    sorted_nullifiers[i] = 0;
                    sorted_nullifiers_indexes[i] = i;
                }
            }

            let mut pre_existing_nullifiers = self.pre_existing_nullifiers;

            for i in 0..MAX_nullifiers_PER_TEST {
                if i < self.nullifiers.len() {
                    let sorted_tuple = sorted_new_nullifier_tuples[i];
                    let new_nullifier = sorted_tuple.value;
                    let original_index = sorted_tuple.original_index;

                    let low_index = self.nullifiers.get_unchecked(original_index).existing_index;

                    kernel_data.public_inputs.end.nullifiers[original_index] = new_nullifier;

                    let mut low_preimage = pre_existing_nullifiers[low_index];
                    nullifier_predecessor_preimages[i] = low_preimage;
                    low_nullifier_membership_witness[i] = MembershipWitness {
                        leaf_index: low_index as Field,
                        sibling_path: nullifier_tree.get_sibling_path(low_index)
                    };

                    low_preimage.next_nullifier = new_nullifier;
                    low_preimage.next_index = start_nullifier_tree_snapshot.next_available_leaf_index as u32 + original_index;
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

            let fee_payer_gas_token_balance_read_hint = self.build_fee_payer_gas_token_balance_read_hint(start_public_data_tree);

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
                self.protocol_public_data_writes,
                self.final_public_data_writes,
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
                archive_root_membership_witness: MembershipWitness { leaf_index: 0, sibling_path: start_archive.get_sibling_path(0) },
                constants: self.constants,
                fee_payer_gas_token_balance_read_hint
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

    impl Empty for BaseRollupInputsBuilder {
        fn empty() -> Self {
            BaseRollupInputsBuilder {
                kernel_data: FixtureBuilder::new(),
                pre_existing_notes: [0; MAX_NOTE_HASHES_PER_TX],
                pre_existing_nullifiers: [NullifierLeafPreimage::empty(); MAX_NULLIFIERS_PER_TX],
                pre_existing_contracts: [0; 2],
                pre_existing_public_data: [PublicDataTreeLeafPreimage::empty(); MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
                pre_existing_blocks: [0; 2],
                public_data_reads: BoundedVec::new(),
                public_data_writes: BoundedVec::new(),
                protocol_public_data_writes: BoundedVec::new(),
                final_public_data_writes: BoundedVec::new(),
                nullifiers: BoundedVec::new(),
                constants: ConstantRollupData::empty(),
                fee_payer_gas_token_balance_pre_existing_public_data_index: Option::none()
            }
        }
    }

    #[test]
    unconstrained fn note_hashes_tree() {
        let mut builder = BaseRollupInputsBuilder::new();

        let note_hashes = [27, 28, 29, 30, 31, 32];
        for i in 0..note_hashes.len() {
            builder.kernel_data.add_new_note_hash(note_hashes[i]);
        }
        let mut expected_commitments_tree = NonEmptyMerkleTree::new(
            [0; MAX_NOTE_HASHES_PER_TX * 2],
            [0; NOTE_HASH_TREE_HEIGHT],
            [0; NOTE_HASH_TREE_HEIGHT - NOTE_HASH_SUBTREE_HEIGHT - 1],
            [0; NOTE_HASH_SUBTREE_HEIGHT + 1]
        );

        let outputs = builder.execute();
        let expected_start_note_hash_tree_snapshot = AppendOnlyTreeSnapshot { root: expected_commitments_tree.get_root(), next_available_leaf_index: MAX_NOTE_HASHES_PER_TX as u32 };
        assert(outputs.start.note_hash_tree.eq(expected_start_note_hash_tree_snapshot));

        for i in 0..note_hashes.len() {
            expected_commitments_tree.update_leaf(i + MAX_NOTE_HASHES_PER_TX, note_hashes[i]);
        }
        let expected_end_note_hash_tree_snapshot = AppendOnlyTreeSnapshot {
            root: expected_commitments_tree.get_root(),
            next_available_leaf_index: (MAX_NOTE_HASHES_PER_TX * 2) as u32
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

        builder.nullifiers.push(NullifierInsertion { existing_index: 0, value: 1 });
        let mut tree_nullifiers = [NullifierLeafPreimage::empty(); MAX_NULLIFIERS_PER_TX * 2];
        tree_nullifiers[0] = NullifierLeafPreimage {
            nullifier : 0,
            next_nullifier : 1,
            next_index : MAX_NULLIFIERS_PER_TX,
        };
        tree_nullifiers[1] = builder.pre_existing_nullifiers[1];
        tree_nullifiers[MAX_NULLIFIERS_PER_TX] = NullifierLeafPreimage {
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
                AppendOnlyTreeSnapshot { root: end_nullifier_tree.get_root(), next_available_leaf_index: 2 * MAX_NULLIFIERS_PER_TX as u32 }
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

        builder.nullifiers.push(NullifierInsertion { existing_index: 1, value: 8 });
        for i in 1..builder.nullifiers.max_len() {
            builder.nullifiers.push(NullifierInsertion { existing_index: 1, value: (8 + i) as Field });
        }

        let output = builder.execute();
        let mut tree_nullifiers = [NullifierLeafPreimage::empty(); MAX_NULLIFIERS_PER_TX * 2];
        tree_nullifiers[0] = builder.pre_existing_nullifiers[0];

        tree_nullifiers[1] = NullifierLeafPreimage {
            nullifier : 7,
            next_nullifier : 8,
            next_index : MAX_NULLIFIERS_PER_TX,
        };

        let last_index = builder.nullifiers.max_len() - 1;
        for i in 0..last_index {
            tree_nullifiers[MAX_NULLIFIERS_PER_TX + i] = NullifierLeafPreimage {
                nullifier : (8 + i) as Field,
                next_nullifier : (8 + i + 1) as Field,
                next_index : MAX_NULLIFIERS_PER_TX + i + 1,
            };
        }
        tree_nullifiers[MAX_NULLIFIERS_PER_TX+last_index] = NullifierLeafPreimage {
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
                AppendOnlyTreeSnapshot { root: end_nullifier_tree.get_root(), next_available_leaf_index: 2 * MAX_NULLIFIERS_PER_TX as u32 }
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

        builder.nullifiers.push(NullifierInsertion { existing_index: 1, value: 8 });
        builder.nullifiers.push(NullifierInsertion { existing_index: 1, value: 8 });

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

        builder.nullifiers.push(NullifierInsertion { existing_index: 1, value: 8 });
        builder.nullifiers.push(NullifierInsertion { existing_index: 1, value: 8 });

        builder.fails();
    }

    #[test]
    unconstrained fn empty_tx_effects_hash() {
        let outputs = BaseRollupInputsBuilder::new().execute();

        let hash_input_flattened = [0; TX_EFFECTS_HASH_INPUT_FIELDS * 32];
        let sha_digest = std::hash::sha256(hash_input_flattened);
        let expected_tx_effects_hash = field_from_bytes_32_trunc(sha_digest);
        assert_eq(outputs.txs_effects_hash, expected_tx_effects_hash);
    }

    #[test]
    unconstrained fn empty_block_out_hash() {
        let outputs = BaseRollupInputsBuilder::new().execute();
        // For now setting an empty out hash to be H(0,0) to keep consistent
        // with prev work.
        let hash_input_flattened = [0; 64];
        let sha_digest = std::hash::sha256(hash_input_flattened);
        let expected_out_hash = field_from_bytes_32_trunc(sha_digest);
        assert_eq(outputs.out_hash, expected_out_hash);
    }

    #[test]
    unconstrained fn nonempty_block_out_hash() {
        let mut end = CombinedAccumulatedData::empty();
        for i in 0..MAX_L2_TO_L1_MSGS_PER_TX {
            end.l2_to_l1_msgs[i] = 10 + i as Field;
        }

        let out_hash = compute_kernel_out_hash(end);
        // Since we fill the tree completely, we know to expect a full tree as below
        let expected_tree = dep::types::merkle_tree::variable_merkle_tree::tests::generate_full_sha_tree(end.l2_to_l1_msgs);
        assert_eq(out_hash, expected_tree.get_root());
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
        builder.constants.global_variables.version += 1;
        builder.fails();
    }

    #[test(should_fail_with = "kernel global variables do not match the rollup global variables")]
    unconstrained fn constants_global_variables_dont_match_kernels() {
        let mut builder = BaseRollupInputsBuilder::new();
        builder.kernel_data.global_variables.block_number = 6;
        builder.constants.global_variables.block_number = 7;
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
    unconstrained fn num_txs_is_1() {
        let outputs = BaseRollupInputsBuilder::new().execute();

        assert_eq(outputs.num_txs, 1);
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

        let mut public_data_leaves = [0; MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX * 2];
        public_data_leaves[0] = PublicDataTreeLeafPreimage {
            slot: 20,
            value: 90,
            next_slot: 25,
            next_index: MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
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
        public_data_leaves[MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX] = PublicDataTreeLeafPreimage {
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

    #[test]
    unconstrained fn updates_fee_payer_balance_with_new_data_write() {
        let fee_payer = AztecAddress::from_field(0x1234);
        let balance_slot = compute_fee_payer_gas_token_balance_leaf_slot(fee_payer);
        let initial_balance = 300_000;
        let tx_fee = 100_000;
        let expected_balance = 200_000;

        let mut builder = BaseRollupInputsBuilder::new();

        // Set fee payer
        builder.kernel_data.fee_payer = fee_payer;

        // Set pre-existing balance
        builder.pre_existing_public_data[0] = PublicDataTreeLeafPreimage {
            slot: balance_slot,
            value: initial_balance,
            next_slot: 0,
            next_index: 0,
        };
        builder.fee_payer_gas_token_balance_pre_existing_public_data_index = Option::some(0);

        // Set values for computing exact tx_fee
        builder.kernel_data.tx_context.gas_settings.inclusion_fee = tx_fee;

        // Set expected protocol data update
        builder.protocol_public_data_writes.push((0, PublicDataTreeLeaf { slot: balance_slot, value: expected_balance }));

        let outputs = builder.execute();

        // The new public data tree should have updated the balance of the fee payer
        let updated_leaf = PublicDataTreeLeafPreimage { slot: balance_slot, value: expected_balance, next_slot: 0, next_index: 0 };
        let mut expected_public_data_tree = NonEmptyMerkleTree::new(
            [updated_leaf.hash(), 0],
            [0; PUBLIC_DATA_TREE_HEIGHT],
            [0; PUBLIC_DATA_TREE_HEIGHT - 1],
            [0; 1]
        );

        assert_eq(outputs.end.public_data_tree.root, expected_public_data_tree.get_root());
    }

    #[test]
    unconstrained fn updates_fee_payer_balance_in_existing_data_write() {
        let fee_payer = AztecAddress::from_field(0x1234);
        let balance_slot = compute_fee_payer_gas_token_balance_leaf_slot(fee_payer);
        let initial_balance = 100_000;
        let after_claim_balance = 300_000;
        let tx_fee = 100_000;
        let expected_balance = 200_000;

        let mut builder = BaseRollupInputsBuilder::new();

        // Set fee payer
        builder.kernel_data.fee_payer = fee_payer;

        // Set pre-existing balance, but set no hint for it since we'll update a user update request
        builder.pre_existing_public_data[0] = PublicDataTreeLeafPreimage {
            slot: balance_slot,
            value: initial_balance,
            next_slot: 0,
            next_index: 0,
        };

        // Set values for computing exact tx_fee
        builder.kernel_data.tx_context.gas_settings.inclusion_fee = tx_fee;

        // Create an existing data update that corresponds to a claim
        builder.public_data_writes.push((0, PublicDataTreeLeaf { slot: balance_slot, value: after_claim_balance }));

        // Set expected data updates after base rollup runs
        // Note that we tweak the final_public_data_writes directly, since we need to overwrite the output of the user public_data_writes
        builder.final_public_data_writes.push((0, PublicDataTreeLeaf { slot: balance_slot, value: expected_balance }));

        let outputs = builder.execute();

        // The new public data tree should have updated the balance of the fee payer
        let updated_leaf = PublicDataTreeLeafPreimage { slot: balance_slot, value: expected_balance, next_slot: 0, next_index: 0 };
        let mut expected_public_data_tree = NonEmptyMerkleTree::new(
            [updated_leaf.hash(), 0],
            [0; PUBLIC_DATA_TREE_HEIGHT],
            [0; PUBLIC_DATA_TREE_HEIGHT - 1],
            [0; 1]
        );

        assert_eq(outputs.end.public_data_tree.root, expected_public_data_tree.get_root());
    }

    #[test(should_fail_with="Not enough balance for fee payer to pay for transaction")]
    unconstrained fn fails_to_update_fee_payer_balance_if_not_enough_funds() {
        let fee_payer = AztecAddress::from_field(0x1234);
        let balance_slot = compute_fee_payer_gas_token_balance_leaf_slot(fee_payer);
        // Set low initial balance so it fails!
        let initial_balance = 10_000;
        let tx_fee = 100_000;

        let mut builder = BaseRollupInputsBuilder::new();

        // Set fee payer
        builder.kernel_data.fee_payer = fee_payer;

        // Set pre-existing balance
        builder.pre_existing_public_data[0] = PublicDataTreeLeafPreimage {
            slot: balance_slot,
            value: initial_balance,
            next_slot: 0,
            next_index: 0,
        };
        builder.fee_payer_gas_token_balance_pre_existing_public_data_index = Option::some(0);

        // Set values for computing exact tx_fee
        builder.kernel_data.tx_context.gas_settings.inclusion_fee = tx_fee;

        // Set expected protocol data update
        builder.protocol_public_data_writes.push((0, PublicDataTreeLeaf { slot: balance_slot, value: -90_000 }));

        builder.fails();
    }

    #[test(should_fail_with="Wrong leaf slot for gas token balance read hint")]
    unconstrained fn fails_to_update_fee_payer_balance_if_wrong_read_hint() {
        let fee_payer = AztecAddress::from_field(0x1234);
        let balance_slot = compute_fee_payer_gas_token_balance_leaf_slot(fee_payer);
        let initial_balance = 300_000;
        let expected_balance = 200_000;
        let tx_fee = 100_000;

        let mut builder = BaseRollupInputsBuilder::new();

        // Set fee payer
        builder.kernel_data.fee_payer = fee_payer;

        // Set pre-existing balance in index 0
        builder.pre_existing_public_data[0] = PublicDataTreeLeafPreimage {
            slot: balance_slot,
            value: initial_balance,
            next_slot: 0,
            next_index: 0,
        };
        builder.pre_existing_public_data[1] = PublicDataTreeLeafPreimage {
            slot: 1,
            value: initial_balance,
            next_slot: balance_slot,
            next_index: 0,
        };

        // But point the read hint to the wrong one!
        builder.fee_payer_gas_token_balance_pre_existing_public_data_index = Option::some(1);

        // Set values for computing exact tx_fee
        builder.kernel_data.tx_context.gas_settings.inclusion_fee = tx_fee;

        // Set expected protocol data update
        builder.protocol_public_data_writes.push((0, PublicDataTreeLeaf { slot: balance_slot, value: expected_balance }));

        builder.fails();
    }

    #[test]
    fn valid_previous_kernel_empty() {
        let mut builder = BaseRollupInputsBuilder::new();
        builder.kernel_data = builder.kernel_data.in_vk_tree(PRIVATE_KERNEL_EMPTY_INDEX);

        let _res = builder.execute();
    }

    #[test]
    fn valid_previous_kernel_tail() {
        let mut builder = BaseRollupInputsBuilder::new();
        builder.kernel_data = builder.kernel_data.in_vk_tree(PRIVATE_KERNEL_TAIL_INDEX);

        let _res = builder.execute();
    }

    #[test]
    fn valid_previous_kernel_public_tail() {
        let mut builder = BaseRollupInputsBuilder::new();
        builder.kernel_data = builder.kernel_data.in_vk_tree(PUBLIC_KERNEL_TAIL_INDEX);

        let _res = builder.execute();
    }
    // TODO(#7410) we need the tube vk to reinstate this
    // #[test(should_fail_with="Invalid vk index")]
    // fn invalid_previous_kernel() {
    //     let mut builder = BaseRollupInputsBuilder::new();
    //     builder.kernel_data = builder.kernel_data.in_vk_tree(BASE_ROLLUP_INDEX);
    //     let _res = builder.execute();
    // }
}
