use crate::{
    abis::{
    max_block_number::MaxBlockNumber, public_data_read::PublicDataRead, read_request::ScopedReadRequest,
    validation_requests::{
    validation_requests::ValidationRequests, rollup_validation_requests::RollupValidationRequests,
    scoped_key_validation_request_and_generator::ScopedKeyValidationRequestAndGenerator
}
},
    constants::{
    MAX_NOTE_HASH_READ_REQUESTS_PER_TX, MAX_NULLIFIER_READ_REQUESTS_PER_TX,
    MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX, MAX_KEY_VALIDATION_REQUESTS_PER_TX,
    MAX_PUBLIC_DATA_READS_PER_TX
},
    traits::Empty
};

struct ValidationRequestsBuilder {
    max_block_number: MaxBlockNumber,
    note_hash_read_requests: BoundedVec<ScopedReadRequest, MAX_NOTE_HASH_READ_REQUESTS_PER_TX>,
    nullifier_read_requests: BoundedVec<ScopedReadRequest, MAX_NULLIFIER_READ_REQUESTS_PER_TX>,
    nullifier_non_existent_read_requests: BoundedVec<ScopedReadRequest, MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX>,
    scoped_key_validation_requests_and_generators: BoundedVec<ScopedKeyValidationRequestAndGenerator, MAX_KEY_VALIDATION_REQUESTS_PER_TX>,
    public_data_reads: BoundedVec<PublicDataRead, MAX_PUBLIC_DATA_READS_PER_TX>,
}

impl ValidationRequestsBuilder {
    pub fn finish(self) -> ValidationRequests {
        ValidationRequests {
            for_rollup: self.to_rollup(),
            note_hash_read_requests: self.note_hash_read_requests.storage,
            nullifier_read_requests: self.nullifier_read_requests.storage,
            nullifier_non_existent_read_requests: self.nullifier_non_existent_read_requests.storage,
            scoped_key_validation_requests_and_generators: self.scoped_key_validation_requests_and_generators.storage,
            public_data_reads: self.public_data_reads.storage
        }
    }

    pub fn to_rollup(self) -> RollupValidationRequests {
        RollupValidationRequests { max_block_number: self.max_block_number }
    }
}

impl Empty for ValidationRequestsBuilder {
    fn empty() -> Self {
        ValidationRequestsBuilder {
            max_block_number: MaxBlockNumber::empty(),
            note_hash_read_requests: BoundedVec::new(),
            nullifier_read_requests: BoundedVec::new(),
            nullifier_non_existent_read_requests: BoundedVec::new(),
            scoped_key_validation_requests_and_generators: BoundedVec::new(),
            public_data_reads: BoundedVec::new(),
        }
    }
}
