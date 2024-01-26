use crate::{
    abis::{
        call_request::CallRequest,
        new_contract_data::NewContractData,
        nullifier_key_validation_request::NullifierKeyValidationRequestContext,
        optionally_revealed_data::OptionallyRevealedData,
        public_data_read::PublicDataRead,
        public_data_update_request::PublicDataUpdateRequest,
        side_effect::{SideEffect, SideEffectLinkedToNoteHash},
    },
    mocked::AggregationObject,
    utils::bounded_vec::BoundedVec
};
use crate::constants::{
    MAX_READ_REQUESTS_PER_TX,
    MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX,
    MAX_NEW_COMMITMENTS_PER_TX,
    MAX_NEW_NULLIFIERS_PER_TX,
    MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
    MAX_NEW_L2_TO_L1_MSGS_PER_TX,
    MAX_NEW_CONTRACTS_PER_TX,
    MAX_OPTIONALLY_REVEALED_DATA_LENGTH_PER_TX,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
    MAX_PUBLIC_DATA_READS_PER_TX,
    NUM_FIELDS_PER_SHA256,
};

struct CombinedAccumulatedData {
    aggregation_object: AggregationObject,

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

    optionally_revealed_data: [OptionallyRevealedData; MAX_OPTIONALLY_REVEALED_DATA_LENGTH_PER_TX],

    public_data_update_requests: [PublicDataUpdateRequest; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],

    public_data_reads: [PublicDataRead; MAX_PUBLIC_DATA_READS_PER_TX],
}

struct FinalAccumulatedData {
    aggregation_object: AggregationObject,

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
    optionally_revealed_data: [OptionallyRevealedData; MAX_OPTIONALLY_REVEALED_DATA_LENGTH_PER_TX]
}

struct CombinedAccumulatedDataBuilder {
    aggregation_object: AggregationObject,

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

    optionally_revealed_data: [OptionallyRevealedData; MAX_OPTIONALLY_REVEALED_DATA_LENGTH_PER_TX],

    public_data_update_requests: BoundedVec<PublicDataUpdateRequest, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,

    public_data_reads: BoundedVec<PublicDataRead, MAX_PUBLIC_DATA_READS_PER_TX>,
}

impl CombinedAccumulatedDataBuilder {
    pub fn finish(self) -> CombinedAccumulatedData {
        CombinedAccumulatedData {
            aggregation_object: self.aggregation_object,

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

            optionally_revealed_data: self.optionally_revealed_data,

            public_data_update_requests: self.public_data_update_requests.storage,

            public_data_reads: self.public_data_reads.storage,
        }
    } 

    pub fn to_final(self) -> FinalAccumulatedData {
        assert_eq(self.read_requests.len, 0, "Final accumulated data: read requests not empty");
        assert_eq(self.nullifier_key_validation_requests.len, 0, "Final accumulated data: nullifier key validation requests not empty");
        assert_eq(self.public_data_update_requests.len, 0, "Final accumulated data: public data update requests not empty");
        assert_eq(self.public_data_reads.len, 0, "Final accumulated data: public data reads not empty");

        FinalAccumulatedData {
            aggregation_object: self.aggregation_object,

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
            optionally_revealed_data: self.optionally_revealed_data,
        }
    } 
}
