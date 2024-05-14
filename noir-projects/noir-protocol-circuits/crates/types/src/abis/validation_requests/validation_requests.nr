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
    MAX_PUBLIC_DATA_READS_PER_TX, VALIDATION_REQUESTS_LENGTH
},
    traits::{Serialize, Deserialize, Empty}, utils::reader::Reader
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

impl Serialize<VALIDATION_REQUESTS_LENGTH> for ValidationRequests {
  fn serialize(self) -> [Field; VALIDATION_REQUESTS_LENGTH] {
    let mut fields: BoundedVec<Field, VALIDATION_REQUESTS_LENGTH> = BoundedVec::new();

    fields.extend_from_array(self.for_rollup.serialize());

    for i in 0..MAX_NOTE_HASH_READ_REQUESTS_PER_TX {
      fields.extend_from_array(self.note_hash_read_requests[i].serialize());
    }

    for i in 0..MAX_NULLIFIER_READ_REQUESTS_PER_TX {
      fields.extend_from_array(self.nullifier_read_requests[i].serialize());
    }

    for i in 0..MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX {
      fields.extend_from_array(self.nullifier_non_existent_read_requests[i].serialize());
    }

    for i in 0..MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX {
      fields.extend_from_array(self.nullifier_key_validation_requests[i].serialize());
    }

    for i in 0..MAX_PUBLIC_DATA_READS_PER_TX {
      fields.extend_from_array(self.public_data_reads[i].serialize());
    }

    assert_eq(fields.len(), VALIDATION_REQUESTS_LENGTH);

    fields.storage
  }
}

impl Deserialize<VALIDATION_REQUESTS_LENGTH> for ValidationRequests {
    fn deserialize(serialized: [Field; VALIDATION_REQUESTS_LENGTH]) -> Self {
        // TODO(#4390): This should accept a reader ^ to avoid copying data.
        let mut reader = Reader::new(serialized);
        let item = Self {
            for_rollup: reader.read_struct(RollupValidationRequests::deserialize),
            note_hash_read_requests: reader.read_struct_array(ScopedReadRequest::deserialize, [ScopedReadRequest::empty(); MAX_NOTE_HASH_READ_REQUESTS_PER_TX]),
            nullifier_read_requests: reader.read_struct_array(ScopedReadRequest::deserialize, [ScopedReadRequest::empty(); MAX_NULLIFIER_READ_REQUESTS_PER_TX]),
            nullifier_non_existent_read_requests: reader.read_struct_array(ScopedReadRequest::deserialize, [ScopedReadRequest::empty(); MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX]),
            nullifier_key_validation_requests: reader.read_struct_array(ScopedNullifierKeyValidationRequest::deserialize, [ScopedNullifierKeyValidationRequest::empty(); MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX]),
            public_data_reads: reader.read_struct_array(PublicDataRead::deserialize, [PublicDataRead::empty(); MAX_PUBLIC_DATA_READS_PER_TX]),
        };

        reader.finish();
        item
    }
}

impl Empty for ValidationRequests {
    fn empty() -> Self {
        
        ValidationRequests {
            for_rollup: RollupValidationRequests::empty(),
            note_hash_read_requests: [ScopedReadRequest::empty(); MAX_NOTE_HASH_READ_REQUESTS_PER_TX],
            nullifier_read_requests: [ScopedReadRequest::empty(); MAX_NULLIFIER_READ_REQUESTS_PER_TX],
            nullifier_non_existent_read_requests: [ScopedReadRequest::empty(); MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX],
            nullifier_key_validation_requests: [ScopedNullifierKeyValidationRequest::empty(); MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX],
            public_data_reads: [PublicDataRead::empty(); MAX_PUBLIC_DATA_READS_PER_TX],
        }
    }
}

impl Eq for ValidationRequests {
    fn eq(self, other: Self) -> bool {
        (self.for_rollup.eq(other.for_rollup)) &
        (self.note_hash_read_requests == other.note_hash_read_requests) &
        (self.nullifier_read_requests == other.nullifier_read_requests) &
        (self.nullifier_non_existent_read_requests == other.nullifier_non_existent_read_requests) &
        (self.nullifier_key_validation_requests == other.nullifier_key_validation_requests) &
        (self.public_data_reads == other.public_data_reads)
    }
}

#[test]
fn serialization_of_empty() {
    let item = ValidationRequests::empty();
    let serialized = item.serialize();
    let deserialized = ValidationRequests::deserialize(serialized);
    assert(item.eq(deserialized));
}
