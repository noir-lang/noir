use crate::{
    abis::{
    accumulated_data::{
    accumulated_non_revertible_data_builder::AccumulatedNonRevertibleDataBuilder,
    accumulated_revertible_data_builder::AccumulatedRevertibleDataBuilder,
    combined_accumulated_data::CombinedAccumulatedData,
    private_accumulated_revertible_data::PrivateAccumulatedRevertibleData,
    private_accumulated_non_revertible_data::PrivateAccumulatedNonRevertibleData,
    public_accumulated_revertible_data::PublicAccumulatedRevertibleData,
    public_accumulated_non_revertible_data::PublicAccumulatedNonRevertibleData
},
    call_request::CallRequest, new_contract_data::NewContractData,
    nullifier_key_validation_request::NullifierKeyValidationRequestContext,
    public_data_read::PublicDataRead, public_data_update_request::PublicDataUpdateRequest,
    side_effect::{SideEffect, SideEffectLinkedToNoteHash}
}
};
use crate::constants::{
    MAX_READ_REQUESTS_PER_TX, MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX, MAX_NEW_NOTE_HASHES_PER_TX,
    MAX_NEW_NULLIFIERS_PER_TX, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, MAX_NEW_L2_TO_L1_MSGS_PER_TX, MAX_NEW_CONTRACTS_PER_TX,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, MAX_PUBLIC_DATA_READS_PER_TX, NUM_FIELDS_PER_SHA256,
    MAX_NON_REVERTIBLE_NOTE_HASHES_PER_TX, MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX,
    MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX, MAX_REVERTIBLE_NOTE_HASHES_PER_TX,
    MAX_REVERTIBLE_NULLIFIERS_PER_TX, MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX,
    MAX_REVERTIBLE_PUBLIC_DATA_READS_PER_TX, MAX_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
    MAX_NON_REVERTIBLE_PUBLIC_DATA_READS_PER_TX, MAX_NON_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX
};

use dep::std::unsafe;
use crate::traits::is_empty;

use crate::utils::arrays::{array_cp, array_concat, array_to_bounded_vec};

struct CombinedAccumulatedDataBuilder {
    read_requests: BoundedVec<SideEffect,MAX_READ_REQUESTS_PER_TX>,
    nullifier_key_validation_requests: BoundedVec<NullifierKeyValidationRequestContext, MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX>,

    new_note_hashes: BoundedVec<SideEffect, MAX_NEW_NOTE_HASHES_PER_TX>,
    new_nullifiers: BoundedVec<SideEffectLinkedToNoteHash, MAX_NEW_NULLIFIERS_PER_TX>,

    private_call_stack: BoundedVec<CallRequest, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX>,
    public_call_stack: BoundedVec<CallRequest, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX>,
    new_l2_to_l1_msgs: BoundedVec<Field, MAX_NEW_L2_TO_L1_MSGS_PER_TX>,

    encrypted_logs_hash: [Field; NUM_FIELDS_PER_SHA256],
    unencrypted_logs_hash: [Field; NUM_FIELDS_PER_SHA256],

    // Here so that the gas cost of this request can be measured by circuits, without actually needing to feed in the
    // variable-length data.
    encrypted_log_preimages_length: Field,
    unencrypted_log_preimages_length: Field,

    new_contracts: BoundedVec<NewContractData, MAX_NEW_CONTRACTS_PER_TX>,

    public_data_update_requests: BoundedVec<PublicDataUpdateRequest, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,

    public_data_reads: BoundedVec<PublicDataRead, MAX_PUBLIC_DATA_READS_PER_TX>,
}

impl CombinedAccumulatedDataBuilder {
    pub fn recombine(
        non_revertible: PublicAccumulatedNonRevertibleData,
        revertible: PublicAccumulatedRevertibleData
    ) -> CombinedAccumulatedDataBuilder {
        CombinedAccumulatedDataBuilder {
            read_requests: array_to_bounded_vec(revertible.read_requests),
            nullifier_key_validation_requests: array_to_bounded_vec(revertible.nullifier_key_validation_requests),
            new_note_hashes: array_to_bounded_vec(array_concat(non_revertible.new_note_hashes, revertible.new_note_hashes)),
            new_nullifiers: array_to_bounded_vec(array_concat(non_revertible.new_nullifiers, revertible.new_nullifiers)),
            private_call_stack: array_to_bounded_vec(revertible.private_call_stack),
            public_call_stack: array_to_bounded_vec(
                array_concat(
                    non_revertible.public_call_stack,
                    revertible.public_call_stack
                )
            ),
            new_l2_to_l1_msgs: array_to_bounded_vec(revertible.new_l2_to_l1_msgs),
            encrypted_logs_hash: revertible.encrypted_logs_hash,
            unencrypted_logs_hash: revertible.unencrypted_logs_hash,
            encrypted_log_preimages_length: revertible.encrypted_log_preimages_length,
            unencrypted_log_preimages_length: revertible.unencrypted_log_preimages_length,
            new_contracts: array_to_bounded_vec(revertible.new_contracts),
            public_data_update_requests: array_to_bounded_vec(
                array_concat(
                    non_revertible.public_data_update_requests,
                    revertible.public_data_update_requests
                )
            ),
            public_data_reads: array_to_bounded_vec(
                array_concat(
                    non_revertible.public_data_reads,
                    revertible.public_data_reads
                )
            )
        }
    }

    pub fn finish(self) -> CombinedAccumulatedData {
        CombinedAccumulatedData {
            read_requests: self.read_requests.storage,
            nullifier_key_validation_requests: self.nullifier_key_validation_requests.storage,
            new_note_hashes: self.new_note_hashes.storage,
            new_nullifiers: self.new_nullifiers.storage,
            private_call_stack: self.private_call_stack.storage,
            public_call_stack: self.public_call_stack.storage,
            new_l2_to_l1_msgs: self.new_l2_to_l1_msgs.storage,
            encrypted_logs_hash: self.encrypted_logs_hash,
            unencrypted_logs_hash: self.unencrypted_logs_hash,
            encrypted_log_preimages_length: self.encrypted_log_preimages_length,
            unencrypted_log_preimages_length: self.unencrypted_log_preimages_length,
            new_contracts: self.new_contracts.storage,
            public_data_update_requests: self.public_data_update_requests.storage,
            public_data_reads: self.public_data_reads.storage
        }
    }

    pub fn to_private_accumulated_revertible_data(self) -> PrivateAccumulatedRevertibleData {
        PrivateAccumulatedRevertibleData {
            new_note_hashes: array_cp(self.new_note_hashes.storage),
            new_nullifiers: array_cp(self.new_nullifiers.storage),
            private_call_stack: self.private_call_stack.storage,
            public_call_stack: array_cp(self.public_call_stack.storage),
            new_l2_to_l1_msgs: self.new_l2_to_l1_msgs.storage,
            encrypted_logs_hash: self.encrypted_logs_hash,
            unencrypted_logs_hash: self.unencrypted_logs_hash,
            encrypted_log_preimages_length: self.encrypted_log_preimages_length,
            unencrypted_log_preimages_length: self.unencrypted_log_preimages_length,
            new_contracts: self.new_contracts.storage
        }
    }

    pub fn to_public_accumulated_revertible_data(self) -> PublicAccumulatedRevertibleData {
        PublicAccumulatedRevertibleData {
            read_requests: self.read_requests.storage,
            nullifier_key_validation_requests: self.nullifier_key_validation_requests.storage,
            new_note_hashes: array_cp(self.new_note_hashes.storage),
            new_nullifiers: array_cp(self.new_nullifiers.storage),
            private_call_stack: self.private_call_stack.storage,
            public_call_stack: array_cp(self.public_call_stack.storage),
            new_l2_to_l1_msgs: self.new_l2_to_l1_msgs.storage,
            encrypted_logs_hash: self.encrypted_logs_hash,
            unencrypted_logs_hash: self.unencrypted_logs_hash,
            encrypted_log_preimages_length: self.encrypted_log_preimages_length,
            unencrypted_log_preimages_length: self.unencrypted_log_preimages_length,
            new_contracts: self.new_contracts.storage,
            public_data_update_requests: array_cp(self.public_data_update_requests.storage),
            public_data_reads: array_cp(self.public_data_reads.storage)
        }
    }

    pub fn split(
        self,
        min_revertible_side_effect_counter: u32
    ) -> (PrivateAccumulatedNonRevertibleData, PrivateAccumulatedRevertibleData) {
        let mut non_revertible_builder: AccumulatedNonRevertibleDataBuilder = unsafe::zeroed();
        let mut revertible_builder: AccumulatedRevertibleDataBuilder = unsafe::zeroed();

        for i in 0..MAX_NEW_NOTE_HASHES_PER_TX {
            let commitment = self.new_note_hashes.storage[i];
            // TODO(fees) we shouldn't need to check is_empty here,
            // but we do because new_note_hashes is bounded to MAX_REVERTIBLE_NOTE_HASHES_PER_TX
            if !is_empty(commitment) {
                if commitment.counter < min_revertible_side_effect_counter {
                    non_revertible_builder.new_note_hashes.push(commitment);
                } else {
                    revertible_builder.new_note_hashes.push(commitment);
                }
            }
        }
        for i in 0..MAX_NEW_NULLIFIERS_PER_TX {
            let nullifier = self.new_nullifiers.storage[i];
            if !is_empty(nullifier) {
                if nullifier.counter < min_revertible_side_effect_counter {
                    non_revertible_builder.new_nullifiers.push(nullifier);
                } else {
                    revertible_builder.new_nullifiers.push(nullifier);
                }
            }
        }

        for i in 0..MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX {
            let call_stack_item = self.public_call_stack.storage[i];
            if !is_empty(call_stack_item) {
                if call_stack_item.start_side_effect_counter < min_revertible_side_effect_counter {
                    non_revertible_builder.public_call_stack.push(call_stack_item);
                } else {
                    revertible_builder.public_call_stack.push(call_stack_item);
                }
            }
        }

        revertible_builder.private_call_stack = self.private_call_stack;
        revertible_builder.new_l2_to_l1_msgs =  self.new_l2_to_l1_msgs;
        revertible_builder.encrypted_logs_hash = self.encrypted_logs_hash;
        revertible_builder.unencrypted_logs_hash = self.unencrypted_logs_hash;
        revertible_builder.encrypted_log_preimages_length =  self.encrypted_log_preimages_length;
        revertible_builder.unencrypted_log_preimages_length= self.unencrypted_log_preimages_length;
        revertible_builder.new_contracts = self.new_contracts;

        (non_revertible_builder.to_private(), revertible_builder.to_private())
    }
}
