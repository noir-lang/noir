use crate::{
    abis::{
        read_request::ScopedReadRequest,
        validation_requests::{
            rollup_validation_requests::RollupValidationRequests,
            scoped_key_validation_request_and_generator::ScopedKeyValidationRequestAndGenerator,
        },
    },
    constants::{
        MAX_KEY_VALIDATION_REQUESTS_PER_TX, MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
        MAX_NULLIFIER_READ_REQUESTS_PER_TX, PRIVATE_VALIDATION_REQUESTS_LENGTH,
    },
    traits::{Deserialize, Empty, Serialize},
    utils::reader::Reader,
};

pub struct PrivateValidationRequests {
    pub for_rollup: RollupValidationRequests,
    pub note_hash_read_requests: [ScopedReadRequest; MAX_NOTE_HASH_READ_REQUESTS_PER_TX],
    pub nullifier_read_requests: [ScopedReadRequest; MAX_NULLIFIER_READ_REQUESTS_PER_TX],
    pub scoped_key_validation_requests_and_generators: [ScopedKeyValidationRequestAndGenerator; MAX_KEY_VALIDATION_REQUESTS_PER_TX],
    pub split_counter: Option<u32>,
}

impl Serialize<PRIVATE_VALIDATION_REQUESTS_LENGTH> for PrivateValidationRequests {
    fn serialize(self) -> [Field; PRIVATE_VALIDATION_REQUESTS_LENGTH] {
        let mut fields: BoundedVec<Field, PRIVATE_VALIDATION_REQUESTS_LENGTH> = BoundedVec::new();

        fields.extend_from_array(self.for_rollup.serialize());

        for i in 0..MAX_NOTE_HASH_READ_REQUESTS_PER_TX {
            fields.extend_from_array(self.note_hash_read_requests[i].serialize());
        }

        for i in 0..MAX_NULLIFIER_READ_REQUESTS_PER_TX {
            fields.extend_from_array(self.nullifier_read_requests[i].serialize());
        }

        for i in 0..MAX_KEY_VALIDATION_REQUESTS_PER_TX {
            fields.extend_from_array(self.scoped_key_validation_requests_and_generators[i]
                .serialize());
        }

        fields.push(self.split_counter.is_some() as Field);
        fields.push(self.split_counter.unwrap_unchecked() as Field);

        assert_eq(fields.len(), PRIVATE_VALIDATION_REQUESTS_LENGTH);

        fields.storage()
    }
}

impl Deserialize<PRIVATE_VALIDATION_REQUESTS_LENGTH> for PrivateValidationRequests {
    fn deserialize(serialized: [Field; PRIVATE_VALIDATION_REQUESTS_LENGTH]) -> Self {
        let mut reader = Reader::new(serialized);
        let item = Self {
            for_rollup: reader.read_struct(RollupValidationRequests::deserialize),
            note_hash_read_requests: reader.read_struct_array(
                ScopedReadRequest::deserialize,
                [ScopedReadRequest::empty(); MAX_NOTE_HASH_READ_REQUESTS_PER_TX],
            ),
            nullifier_read_requests: reader.read_struct_array(
                ScopedReadRequest::deserialize,
                [ScopedReadRequest::empty(); MAX_NULLIFIER_READ_REQUESTS_PER_TX],
            ),
            scoped_key_validation_requests_and_generators: reader.read_struct_array(
                ScopedKeyValidationRequestAndGenerator::deserialize,
                [
                    ScopedKeyValidationRequestAndGenerator::empty();
                         MAX_KEY_VALIDATION_REQUESTS_PER_TX
                    ],
                ),
            split_counter: if reader.read_bool() {
                Option::some(reader.read_u32())
            } else {
                // We need to read the final reader value even if we don't use it in order to flush the reader.
                let _ = reader.read_u32();
                Option::none()
            },
        };

        reader.finish();
        item
    }
}

impl Empty for PrivateValidationRequests {
    fn empty() -> Self {
        PrivateValidationRequests {
            for_rollup: RollupValidationRequests::empty(),
            note_hash_read_requests: [
                ScopedReadRequest::empty(); MAX_NOTE_HASH_READ_REQUESTS_PER_TX
            ],
            nullifier_read_requests: [
                ScopedReadRequest::empty(); MAX_NULLIFIER_READ_REQUESTS_PER_TX
            ],
            scoped_key_validation_requests_and_generators: [
                ScopedKeyValidationRequestAndGenerator::empty(); MAX_KEY_VALIDATION_REQUESTS_PER_TX
            ],
            split_counter: Option::none(),
        }
    }
}

impl Eq for PrivateValidationRequests {
    fn eq(self, other: Self) -> bool {
        (self.for_rollup.eq(other.for_rollup))
            & (self.note_hash_read_requests == other.note_hash_read_requests)
            & (self.nullifier_read_requests == other.nullifier_read_requests)
            & (
                self.scoped_key_validation_requests_and_generators
                    == other.scoped_key_validation_requests_and_generators
            )
            & (self.split_counter == other.split_counter)
    }
}

#[test]
fn serialization_of_empty() {
    let item = PrivateValidationRequests::empty();
    let serialized = item.serialize();
    let deserialized = PrivateValidationRequests::deserialize(serialized);
    assert(item.eq(deserialized));
}
