use crate::{
    abis::{
    max_block_number::MaxBlockNumber,
    nullifier_key_validation_request::NullifierKeyValidationRequestContext,
    public_data_read::PublicDataRead, read_request::ReadRequestContext, side_effect::SideEffect,
    validation_requests::validation_requests::ValidationRequests,
    validation_requests::rollup_validation_requests::RollupValidationRequests
},
    constants::{
    MAX_NOTE_HASH_READ_REQUESTS_PER_TX, MAX_NULLIFIER_READ_REQUESTS_PER_TX,
    MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX, MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX,
    MAX_PUBLIC_DATA_READS_PER_TX
}
};

struct ValidationRequestsBuilder {
    max_block_number: MaxBlockNumber,
    note_hash_read_requests: BoundedVec<SideEffect, MAX_NOTE_HASH_READ_REQUESTS_PER_TX>,
    nullifier_read_requests: BoundedVec<ReadRequestContext, MAX_NULLIFIER_READ_REQUESTS_PER_TX>,
    nullifier_non_existent_read_requests: BoundedVec<ReadRequestContext, MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX>,
    nullifier_key_validation_requests: BoundedVec<NullifierKeyValidationRequestContext, MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX>,
    public_data_reads: BoundedVec<PublicDataRead, MAX_PUBLIC_DATA_READS_PER_TX>,
}

impl ValidationRequestsBuilder {
    pub fn finish(self) -> ValidationRequests {
        ValidationRequests {
            for_rollup: self.to_rollup(),
            note_hash_read_requests: self.note_hash_read_requests.storage,
            nullifier_read_requests: self.nullifier_read_requests.storage,
            nullifier_non_existent_read_requests: self.nullifier_non_existent_read_requests.storage,
            nullifier_key_validation_requests: self.nullifier_key_validation_requests.storage,
            public_data_reads: self.public_data_reads.storage
        }
    }

    pub fn to_rollup(self) -> RollupValidationRequests {
        RollupValidationRequests { max_block_number: self.max_block_number }
    }
}
