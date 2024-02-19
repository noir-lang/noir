use crate::{
    abis::{
    call_request::CallRequest, new_contract_data::NewContractData,
    nullifier_key_validation_request::NullifierKeyValidationRequestContext,
    public_data_read::PublicDataRead, public_data_update_request::PublicDataUpdateRequest,
    side_effect::{SideEffect, SideEffectLinkedToNoteHash}
}
};
use crate::constants::{
    MAX_READ_REQUESTS_PER_TX, MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX, MAX_NEW_COMMITMENTS_PER_TX,
    MAX_NEW_NULLIFIERS_PER_TX, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, MAX_NEW_L2_TO_L1_MSGS_PER_TX, MAX_NEW_CONTRACTS_PER_TX,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, MAX_PUBLIC_DATA_READS_PER_TX, NUM_FIELDS_PER_SHA256,
    MAX_NON_REVERTIBLE_COMMITMENTS_PER_TX, MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX,
    MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX, MAX_REVERTIBLE_COMMITMENTS_PER_TX,
    MAX_REVERTIBLE_NULLIFIERS_PER_TX, MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX,
    MAX_REVERTIBLE_PUBLIC_DATA_READS_PER_TX, MAX_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
    MAX_NON_REVERTIBLE_PUBLIC_DATA_READS_PER_TX, MAX_NON_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX
};

use dep::std::unsafe;
use crate::traits::is_empty;

use crate::utils::arrays::{array_cp, array_concat, array_to_bounded_vec};

struct CombinedAccumulatedData {
    read_requests: [SideEffect; MAX_READ_REQUESTS_PER_TX],
    nullifier_key_validation_requests: [NullifierKeyValidationRequestContext; MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX],

    new_commitments: [SideEffect; MAX_NEW_COMMITMENTS_PER_TX],
    new_nullifiers: [SideEffectLinkedToNoteHash; MAX_NEW_NULLIFIERS_PER_TX],

    private_call_stack: [CallRequest; MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX],
    public_call_stack: [CallRequest; MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX],
    new_l2_to_l1_msgs: [Field; MAX_NEW_L2_TO_L1_MSGS_PER_TX],

    encrypted_logs_hash: [Field; NUM_FIELDS_PER_SHA256],
    unencrypted_logs_hash: [Field; NUM_FIELDS_PER_SHA256],

    // Here so that the gas cost of this request can be measured by circuits, without actually needing to feed in the
    // variable-length data.
    encrypted_log_preimages_length: Field,
    unencrypted_log_preimages_length: Field,

    new_contracts: [NewContractData; MAX_NEW_CONTRACTS_PER_TX],

    public_data_update_requests: [PublicDataUpdateRequest; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],

    public_data_reads: [PublicDataRead; MAX_PUBLIC_DATA_READS_PER_TX],
}

impl CombinedAccumulatedData {
    pub fn needs_app_logic(self) -> bool {
        // if we have any enqueued revertible public calls, we need to run the public app logic circuit.
        !self.public_call_stack[0].is_empty()
    }

    pub fn recombine(
        non_revertible: PublicAccumulatedNonRevertibleData,
        revertible: PublicAccumulatedRevertibleData
    ) -> CombinedAccumulatedData {
        CombinedAccumulatedData {
            read_requests: revertible.read_requests,
            nullifier_key_validation_requests: revertible.nullifier_key_validation_requests,
            new_commitments: array_concat(non_revertible.new_commitments, revertible.new_commitments),
            new_nullifiers: array_concat(non_revertible.new_nullifiers, revertible.new_nullifiers),
            private_call_stack: revertible.private_call_stack,
            public_call_stack: array_concat(
                non_revertible.public_call_stack,
                revertible.public_call_stack
            ),
            new_l2_to_l1_msgs: revertible.new_l2_to_l1_msgs,
            encrypted_logs_hash: revertible.encrypted_logs_hash,
            unencrypted_logs_hash: revertible.unencrypted_logs_hash,
            encrypted_log_preimages_length: revertible.encrypted_log_preimages_length,
            unencrypted_log_preimages_length: revertible.unencrypted_log_preimages_length,
            new_contracts: revertible.new_contracts,
            public_data_update_requests: array_concat(
                non_revertible.public_data_update_requests,
                revertible.public_data_update_requests
            ),
            public_data_reads: array_concat(
                non_revertible.public_data_reads,
                revertible.public_data_reads
            )
        }
    }
}

struct PrivateAccumulatedRevertibleData {
    new_commitments: [SideEffect; MAX_REVERTIBLE_COMMITMENTS_PER_TX],
    new_nullifiers: [SideEffectLinkedToNoteHash; MAX_REVERTIBLE_NULLIFIERS_PER_TX],

    private_call_stack: [CallRequest; MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX],
    public_call_stack: [CallRequest; MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX],
    new_l2_to_l1_msgs: [Field; MAX_NEW_L2_TO_L1_MSGS_PER_TX],

    encrypted_logs_hash: [Field; NUM_FIELDS_PER_SHA256],
    unencrypted_logs_hash: [Field; NUM_FIELDS_PER_SHA256],

    // Here so that the gas cost of this request can be measured by circuits, without actually needing to feed in the
    // variable-length data.
    encrypted_log_preimages_length: Field,
    unencrypted_log_preimages_length: Field,

    new_contracts: [NewContractData; MAX_NEW_CONTRACTS_PER_TX],
}

impl PrivateAccumulatedRevertibleData {
    pub fn needs_app_logic(self) -> bool {
        // if we have any enqueued revertible public calls, we need to run the public app logic circuit.
        !self.public_call_stack[0].is_empty()
    }
}

struct PublicAccumulatedRevertibleData {
    read_requests: [SideEffect; MAX_READ_REQUESTS_PER_TX],
    nullifier_key_validation_requests: [NullifierKeyValidationRequestContext; MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX],

    new_commitments: [SideEffect; MAX_REVERTIBLE_COMMITMENTS_PER_TX],
    new_nullifiers: [SideEffectLinkedToNoteHash; MAX_REVERTIBLE_NULLIFIERS_PER_TX],

    private_call_stack: [CallRequest; MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX],
    public_call_stack: [CallRequest; MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX],
    new_l2_to_l1_msgs: [Field; MAX_NEW_L2_TO_L1_MSGS_PER_TX],

    encrypted_logs_hash: [Field; NUM_FIELDS_PER_SHA256],
    unencrypted_logs_hash: [Field; NUM_FIELDS_PER_SHA256],

    // Here so that the gas cost of this request can be measured by circuits, without actually needing to feed in the
    // variable-length data.
    encrypted_log_preimages_length: Field,
    unencrypted_log_preimages_length: Field,

    new_contracts: [NewContractData; MAX_NEW_CONTRACTS_PER_TX],

    public_data_update_requests: [PublicDataUpdateRequest; MAX_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
    public_data_reads: [PublicDataRead; MAX_REVERTIBLE_PUBLIC_DATA_READS_PER_TX],
}

impl PublicAccumulatedRevertibleData {
    pub fn needs_app_logic(self) -> bool {
        // if we have any enqueued revertible public calls, we need to run the public app logic circuit.
        !self.public_call_stack[0].is_empty()
    }
}

struct AccumulatedRevertibleDataBuilder {
    read_requests: BoundedVec<SideEffect, MAX_READ_REQUESTS_PER_TX>,
    nullifier_key_validation_requests: BoundedVec<NullifierKeyValidationRequestContext, MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX>,

    new_commitments: BoundedVec<SideEffect, MAX_REVERTIBLE_COMMITMENTS_PER_TX>,
    new_nullifiers: BoundedVec<SideEffectLinkedToNoteHash, MAX_REVERTIBLE_NULLIFIERS_PER_TX>,

    private_call_stack: BoundedVec<CallRequest, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX>,
    public_call_stack: BoundedVec<CallRequest, MAX_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX>,
    new_l2_to_l1_msgs: BoundedVec<Field, MAX_NEW_L2_TO_L1_MSGS_PER_TX>,

    encrypted_logs_hash: [Field; NUM_FIELDS_PER_SHA256],
    unencrypted_logs_hash: [Field; NUM_FIELDS_PER_SHA256],

    // Here so that the gas cost of this request can be measured by circuits, without actually needing to feed in the
    // variable-length data.
    encrypted_log_preimages_length: Field,
    unencrypted_log_preimages_length: Field,

    new_contracts: BoundedVec<NewContractData, MAX_NEW_CONTRACTS_PER_TX>,

    public_data_update_requests: BoundedVec<PublicDataUpdateRequest, MAX_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
    public_data_reads: BoundedVec<PublicDataRead, MAX_REVERTIBLE_PUBLIC_DATA_READS_PER_TX>,
}

impl AccumulatedRevertibleDataBuilder {
    pub fn to_private(self) -> PrivateAccumulatedRevertibleData {
        PrivateAccumulatedRevertibleData {
            new_commitments: self.new_commitments.storage,
            new_nullifiers: self.new_nullifiers.storage,
            private_call_stack: self.private_call_stack.storage,
            public_call_stack: self.public_call_stack.storage,
            new_l2_to_l1_msgs: self.new_l2_to_l1_msgs.storage,
            encrypted_logs_hash: self.encrypted_logs_hash,
            unencrypted_logs_hash: self.unencrypted_logs_hash,
            encrypted_log_preimages_length: self.encrypted_log_preimages_length,
            unencrypted_log_preimages_length: self.unencrypted_log_preimages_length,
            new_contracts: self.new_contracts.storage
        }
    }

    pub fn to_public(self) -> PublicAccumulatedRevertibleData {
        PublicAccumulatedRevertibleData {
            read_requests: self.read_requests.storage,
            new_commitments: self.new_commitments.storage,
            nullifier_key_validation_requests: self.nullifier_key_validation_requests.storage,
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
}

struct PrivateAccumulatedNonRevertibleData {
    new_commitments: [SideEffect; MAX_NON_REVERTIBLE_COMMITMENTS_PER_TX],
    new_nullifiers: [SideEffectLinkedToNoteHash; MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX],
    public_call_stack: [CallRequest; MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX],
}

impl PrivateAccumulatedNonRevertibleData {
    pub fn needs_setup(self) -> bool {
        // By definition, the final non-revertible enqueued call is for teardown.
        // since this is a stack, the teardown call would be the 0th element.
        // So if we have more than one element, we need setup.
        !self.public_call_stack[1].is_empty()
    }

    pub fn needs_teardown(self) -> bool {
        // By definition, the final non-revertible enqueued call is for teardown.
        // since this is a stack, the teardown call would be the 0th element.
        !self.public_call_stack[0].is_empty()
    }
}

struct PublicAccumulatedNonRevertibleData {
    new_commitments: [SideEffect; MAX_NON_REVERTIBLE_COMMITMENTS_PER_TX],
    new_nullifiers: [SideEffectLinkedToNoteHash; MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX],
    public_call_stack: [CallRequest; MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX],
    public_data_update_requests: [PublicDataUpdateRequest; MAX_NON_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
    public_data_reads: [PublicDataRead; MAX_NON_REVERTIBLE_PUBLIC_DATA_READS_PER_TX],
}

impl PublicAccumulatedNonRevertibleData {
    pub fn needs_setup(self) -> bool {
        // By definition, the final non-revertible enqueued call is for teardown.
        // since this is a stack, the teardown call would be the 0th element.
        // So if we have more than one element, we need setup.
        !self.public_call_stack[1].is_empty()
    }

    pub fn needs_teardown(self) -> bool {
        // By definition, the final non-revertible enqueued call is for teardown.
        // since this is a stack, the teardown call would be the 0th element.
        !self.public_call_stack[0].is_empty()
    }
}

struct AccumulatedNonRevertibleDataBuilder {
    new_commitments: BoundedVec<SideEffect, MAX_NON_REVERTIBLE_COMMITMENTS_PER_TX>,
    new_nullifiers: BoundedVec<SideEffectLinkedToNoteHash, MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX>,
    public_call_stack: BoundedVec<CallRequest, MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX>,
    public_data_update_requests: BoundedVec<PublicDataUpdateRequest, MAX_NON_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
    public_data_reads: BoundedVec<PublicDataRead, MAX_NON_REVERTIBLE_PUBLIC_DATA_READS_PER_TX>,
}

impl AccumulatedNonRevertibleDataBuilder {
    pub fn to_private(self) -> PrivateAccumulatedNonRevertibleData {
        PrivateAccumulatedNonRevertibleData {
            new_commitments: self.new_commitments.storage,
            new_nullifiers: self.new_nullifiers.storage,
            public_call_stack: self.public_call_stack.storage
        }
    }
    pub fn to_public(self) -> PublicAccumulatedNonRevertibleData {
        PublicAccumulatedNonRevertibleData {
            new_commitments: self.new_commitments.storage,
            new_nullifiers: self.new_nullifiers.storage,
            public_call_stack: self.public_call_stack.storage,
            public_data_update_requests: self.public_data_update_requests.storage,
            public_data_reads: self.public_data_reads.storage
        }
    }
}

struct CombinedAccumulatedDataBuilder {
    read_requests: BoundedVec<SideEffect,MAX_READ_REQUESTS_PER_TX>,
    nullifier_key_validation_requests: BoundedVec<NullifierKeyValidationRequestContext, MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX>,

    new_commitments: BoundedVec<SideEffect, MAX_NEW_COMMITMENTS_PER_TX>,
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
            new_commitments: array_to_bounded_vec(array_concat(non_revertible.new_commitments, revertible.new_commitments)),
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
            new_commitments: self.new_commitments.storage,
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
            new_commitments: array_cp(self.new_commitments.storage),
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
            new_commitments: array_cp(self.new_commitments.storage),
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

        for i in 0..MAX_NEW_COMMITMENTS_PER_TX {
            let commitment = self.new_commitments.storage[i];
            // TODO(fees) we shouldn't need to check is_empty here,
            // but we do because new_commitments is bounded to MAX_REVERTIBLE_COMMITMENTS_PER_TX
            if !is_empty(commitment) {
                if commitment.counter < min_revertible_side_effect_counter {
                    non_revertible_builder.new_commitments.push(commitment);
                } else {
                    revertible_builder.new_commitments.push(commitment);
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

mod tests {
    use crate::abis::{
        combined_accumulated_data::CombinedAccumulatedDataBuilder,
        call_request::{CallRequest, CallerContext}, new_contract_data::NewContractData,
        nullifier_key_validation_request::NullifierKeyValidationRequestContext,
        public_data_read::PublicDataRead, public_data_update_request::PublicDataUpdateRequest,
        side_effect::{SideEffect, SideEffectLinkedToNoteHash}
    };
    use crate::address::AztecAddress;
    use crate::utils::arrays::array_eq;
    use dep::std::unsafe;

    #[test]
    unconstrained fn splits_revertible_and_non_revertible() {
        let mut builder: CombinedAccumulatedDataBuilder = unsafe::zeroed();

        let non_revertible_commitments = [
            SideEffect { value: 1, counter: 1 },
            SideEffect { value: 2, counter: 3 }
        ];

        let non_revertible_nullifiers = [
            SideEffectLinkedToNoteHash { value: 10, note_hash: 1, counter: 2 },
            SideEffectLinkedToNoteHash { value: 20, note_hash: 2, counter: 4 }
        ];

        let non_revertible_public_stack = [
            CallRequest {
                hash: 1,
                caller_contract_address: AztecAddress::from_field(1),
                caller_context: CallerContext::empty(),
                start_side_effect_counter: 5,
                end_side_effect_counter: 0
            },
            CallRequest {
                hash: 2,
                caller_contract_address: AztecAddress::from_field(1),
                caller_context: CallerContext::empty(),
                start_side_effect_counter: 6,
                end_side_effect_counter: 0
            }
        ];

        let revertible_commitments = [
            SideEffect { value: 3, counter: 7 },
            SideEffect { value: 4, counter: 10 }
        ];

        let revertible_nullifiers = [
            SideEffectLinkedToNoteHash { value: 30, note_hash: 3, counter: 8 },
            SideEffectLinkedToNoteHash { value: 40, note_hash: 4, counter: 11 }
        ];

        let revertible_public_call_stack = [
            CallRequest {
                hash: 3,
                caller_contract_address: AztecAddress::from_field(3),
                caller_context: CallerContext::empty(),
                start_side_effect_counter: 9,
                end_side_effect_counter: 0
            }
        ];

        builder.new_commitments.extend_from_array(non_revertible_commitments);
        builder.new_commitments.extend_from_array(revertible_commitments);

        builder.new_nullifiers.extend_from_array(non_revertible_nullifiers);
        builder.new_nullifiers.extend_from_array(revertible_nullifiers);

        builder.public_call_stack.extend_from_array(non_revertible_public_stack);
        builder.public_call_stack.extend_from_array(revertible_public_call_stack);

        let (non_revertible, revertible) = builder.split(7);

        assert(array_eq(non_revertible.new_commitments, non_revertible_commitments));
        assert(array_eq(non_revertible.new_nullifiers, non_revertible_nullifiers));
        assert(array_eq(non_revertible.public_call_stack, non_revertible_public_stack));

        assert(array_eq(revertible.new_commitments, revertible_commitments));
        assert(array_eq(revertible.new_nullifiers, revertible_nullifiers));
        assert(array_eq(revertible.public_call_stack, revertible_public_call_stack));
    }
}
