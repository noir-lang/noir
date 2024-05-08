use crate::{
    abis::{
    max_block_number::MaxBlockNumber,
    nullifier_key_validation_request::ScopedNullifierKeyValidationRequest,
    public_data_read::PublicDataRead, read_request::ScopedReadRequest,
    validation_requests::rollup_validation_requests::RollupValidationRequests
},
    constants::{
    MAX_NOTE_HASH_READ_REQUESTS_PER_TX, MAX_NULLIFIER_READ_REQUESTS_PER_TX,
    MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX, MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX,
    MAX_PUBLIC_DATA_READS_PER_TX
}
};

// TODO - Use specific structs for private and public: PrivateValidationRequests vs PublicValidationRequests
struct ValidationRequests {
    for_rollup: RollupValidationRequests,
    note_hash_read_requests: [ScopedReadRequest; MAX_NOTE_HASH_READ_REQUESTS_PER_TX],
    nullifier_read_requests: [ScopedReadRequest; MAX_NULLIFIER_READ_REQUESTS_PER_TX],
    nullifier_non_existent_read_requests: [ScopedReadRequest; MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX],
    nullifier_key_validation_requests: [ScopedNullifierKeyValidationRequest; MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX],
    public_data_reads: [PublicDataRead; MAX_PUBLIC_DATA_READS_PER_TX],
}
