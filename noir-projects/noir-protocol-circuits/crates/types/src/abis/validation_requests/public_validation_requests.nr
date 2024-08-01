use crate::{
    abis::{
    public_data_read::PublicDataRead, read_request::ScopedReadRequest,
    validation_requests::{rollup_validation_requests::RollupValidationRequests}
},
    constants::{
    MAX_NULLIFIER_READ_REQUESTS_PER_TX, MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX,
    MAX_PUBLIC_DATA_READS_PER_TX, PUBLIC_VALIDATION_REQUESTS_LENGTH
},
    traits::{Serialize, Deserialize, Empty}, utils::reader::Reader
};

struct PublicValidationRequests {
    for_rollup: RollupValidationRequests,
    nullifier_read_requests: [ScopedReadRequest; MAX_NULLIFIER_READ_REQUESTS_PER_TX],
    nullifier_non_existent_read_requests: [ScopedReadRequest; MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX],
    public_data_reads: [PublicDataRead; MAX_PUBLIC_DATA_READS_PER_TX],
}

impl Serialize<PUBLIC_VALIDATION_REQUESTS_LENGTH> for PublicValidationRequests {
  fn serialize(self) -> [Field; PUBLIC_VALIDATION_REQUESTS_LENGTH] {
    let mut fields: BoundedVec<Field, PUBLIC_VALIDATION_REQUESTS_LENGTH> = BoundedVec::new();

    fields.extend_from_array(self.for_rollup.serialize());

    for i in 0..MAX_NULLIFIER_READ_REQUESTS_PER_TX {
      fields.extend_from_array(self.nullifier_read_requests[i].serialize());
    }

    for i in 0..MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX {
      fields.extend_from_array(self.nullifier_non_existent_read_requests[i].serialize());
    }

    for i in 0..MAX_PUBLIC_DATA_READS_PER_TX {
      fields.extend_from_array(self.public_data_reads[i].serialize());
    }

    assert_eq(fields.len(), PUBLIC_VALIDATION_REQUESTS_LENGTH);

    fields.storage
  }
}

impl Deserialize<PUBLIC_VALIDATION_REQUESTS_LENGTH> for PublicValidationRequests {
    fn deserialize(serialized: [Field; PUBLIC_VALIDATION_REQUESTS_LENGTH]) -> Self {
        // TODO(#4390): This should accept a reader ^ to avoid copying data.
        let mut reader = Reader::new(serialized);
        let item = Self {
            for_rollup: reader.read_struct(RollupValidationRequests::deserialize),
            nullifier_read_requests: reader.read_struct_array(ScopedReadRequest::deserialize, [ScopedReadRequest::empty(); MAX_NULLIFIER_READ_REQUESTS_PER_TX]),
            nullifier_non_existent_read_requests: reader.read_struct_array(ScopedReadRequest::deserialize, [ScopedReadRequest::empty(); MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX]),
            public_data_reads: reader.read_struct_array(PublicDataRead::deserialize, [PublicDataRead::empty(); MAX_PUBLIC_DATA_READS_PER_TX]),
        };

        reader.finish();
        item
    }
}

impl Empty for PublicValidationRequests {
    fn empty() -> Self {
        PublicValidationRequests {
            for_rollup: RollupValidationRequests::empty(),
            nullifier_read_requests: [ScopedReadRequest::empty(); MAX_NULLIFIER_READ_REQUESTS_PER_TX],
            nullifier_non_existent_read_requests: [ScopedReadRequest::empty(); MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_TX],
            public_data_reads: [PublicDataRead::empty(); MAX_PUBLIC_DATA_READS_PER_TX],
        }
    }
}

impl Eq for PublicValidationRequests {
    fn eq(self, other: Self) -> bool {
        (self.for_rollup.eq(other.for_rollup)) &
        (self.nullifier_read_requests == other.nullifier_read_requests) &
        (self.nullifier_non_existent_read_requests == other.nullifier_non_existent_read_requests) &
        (self.public_data_reads == other.public_data_reads)
    }
}

#[test]
fn serialization_of_empty() {
    let item = PublicValidationRequests::empty();
    let serialized = item.serialize();
    let deserialized = PublicValidationRequests::deserialize(serialized);
    assert(item.eq(deserialized));
}
