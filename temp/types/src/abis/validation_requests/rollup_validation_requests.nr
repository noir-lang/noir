use crate::{
    abis::max_block_number::MaxBlockNumber,
    constants::ROLLUP_VALIDATION_REQUESTS_LENGTH,
    traits::{Deserialize, Empty, Serialize},
    utils::reader::Reader,
};

// These are validation requests that cannot be fulfilled in the current context (private or public), and must be
// instead forwarded to the rollup for it to take care of them.
pub struct RollupValidationRequests {
    pub max_block_number: MaxBlockNumber,
}

impl Empty for RollupValidationRequests {
    fn empty() -> Self {
        RollupValidationRequests { max_block_number: MaxBlockNumber::empty() }
    }
}

impl Serialize<ROLLUP_VALIDATION_REQUESTS_LENGTH> for RollupValidationRequests {
    fn serialize(self) -> [Field; ROLLUP_VALIDATION_REQUESTS_LENGTH] {
        MaxBlockNumber::serialize(self.max_block_number)
    }
}

impl Deserialize<ROLLUP_VALIDATION_REQUESTS_LENGTH> for RollupValidationRequests {
    fn deserialize(fields: [Field; ROLLUP_VALIDATION_REQUESTS_LENGTH]) -> RollupValidationRequests {
        let mut reader = Reader::new(fields);
        let item = Self { max_block_number: reader.read_struct(MaxBlockNumber::deserialize) };
        reader.finish();
        item
    }
}

impl Eq for RollupValidationRequests {
    fn eq(self, other: Self) -> bool {
        self.max_block_number.eq(other.max_block_number)
    }
}

#[test]
fn serialization_of_empty() {
    let item = RollupValidationRequests::empty();
    let serialized = item.serialize();
    let deserialized = RollupValidationRequests::deserialize(serialized);
    assert(item.eq(deserialized));
}
