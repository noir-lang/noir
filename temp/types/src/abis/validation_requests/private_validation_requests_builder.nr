use crate::{
    abis::{
        max_block_number::MaxBlockNumber,
        read_request::ScopedReadRequest,
        validation_requests::{
            private_validation_requests::PrivateValidationRequests,
            rollup_validation_requests::RollupValidationRequests,
            scoped_key_validation_request_and_generator::ScopedKeyValidationRequestAndGenerator,
        },
    },
    constants::{
        MAX_KEY_VALIDATION_REQUESTS_PER_TX, MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
        MAX_NULLIFIER_READ_REQUESTS_PER_TX,
    },
    traits::Empty,
};

pub struct PrivateValidationRequestsBuilder {
    pub max_block_number: MaxBlockNumber,
    pub note_hash_read_requests: BoundedVec<ScopedReadRequest, MAX_NOTE_HASH_READ_REQUESTS_PER_TX>,
    pub nullifier_read_requests: BoundedVec<ScopedReadRequest, MAX_NULLIFIER_READ_REQUESTS_PER_TX>,
    pub scoped_key_validation_requests_and_generators: BoundedVec<ScopedKeyValidationRequestAndGenerator, MAX_KEY_VALIDATION_REQUESTS_PER_TX>,
    pub split_counter: Option<u32>,
}

impl PrivateValidationRequestsBuilder {
    pub fn finish(self) -> PrivateValidationRequests {
        PrivateValidationRequests {
            for_rollup: self.for_rollup(),
            note_hash_read_requests: self.note_hash_read_requests.storage(),
            nullifier_read_requests: self.nullifier_read_requests.storage(),
            scoped_key_validation_requests_and_generators: self
                .scoped_key_validation_requests_and_generators
                .storage(),
            split_counter: self.split_counter,
        }
    }

    pub fn for_rollup(self) -> RollupValidationRequests {
        RollupValidationRequests { max_block_number: self.max_block_number }
    }
}

impl Empty for PrivateValidationRequestsBuilder {
    fn empty() -> Self {
        PrivateValidationRequestsBuilder {
            max_block_number: MaxBlockNumber::empty(),
            note_hash_read_requests: BoundedVec::new(),
            nullifier_read_requests: BoundedVec::new(),
            scoped_key_validation_requests_and_generators: BoundedVec::new(),
            split_counter: Option::none(),
        }
    }
}
