use crate::{
    abis::{
    max_block_number::MaxBlockNumber, public_data_read::PublicDataRead, read_request::ScopedReadRequest,
    validation_requests::{
    public_validation_requests::PublicValidationRequests,
    rollup_validation_requests::RollupValidationRequests
}
},
    constants::{
    MAX_NULLIFIER_READ_REQUESTS_PER_TX, MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX,
    MAX_PUBLIC_DATA_READS_PER_TX
},
    traits::Empty, utils::arrays::array_to_bounded_vec
};

struct PublicValidationRequestsBuilder {
    max_block_number: MaxBlockNumber,
    nullifier_read_requests: BoundedVec<ScopedReadRequest, MAX_NULLIFIER_READ_REQUESTS_PER_TX>,
    nullifier_non_existent_read_requests: BoundedVec<ScopedReadRequest, MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX>,
    public_data_reads: BoundedVec<PublicDataRead, MAX_PUBLIC_DATA_READS_PER_TX>,
}

impl PublicValidationRequestsBuilder {
    pub fn new(requests: PublicValidationRequests) -> Self {
        PublicValidationRequestsBuilder {
            max_block_number: requests.for_rollup.max_block_number,
            nullifier_read_requests: array_to_bounded_vec(requests.nullifier_read_requests),
            nullifier_non_existent_read_requests: array_to_bounded_vec(requests.nullifier_non_existent_read_requests),
            public_data_reads: array_to_bounded_vec(requests.public_data_reads)
        }
    }

    pub fn finish(self) -> PublicValidationRequests {
        PublicValidationRequests {
            for_rollup: self.for_rollup(),
            nullifier_read_requests: self.nullifier_read_requests.storage,
            nullifier_non_existent_read_requests: self.nullifier_non_existent_read_requests.storage,
            public_data_reads: self.public_data_reads.storage
        }
    }

    pub fn for_rollup(self) -> RollupValidationRequests {
        RollupValidationRequests { max_block_number: self.max_block_number }
    }
}

impl Empty for PublicValidationRequestsBuilder {
    fn empty() -> Self {
        PublicValidationRequestsBuilder {
            max_block_number: MaxBlockNumber::empty(),
            nullifier_read_requests: BoundedVec::new(),
            nullifier_non_existent_read_requests: BoundedVec::new(),
            public_data_reads: BoundedVec::new(),
        }
    }
}
