use crate::{
    abis::{
        side_effect::Scoped,
        validation_requests::key_validation_request_and_generator::KeyValidationRequestAndGenerator,
    },
    address::AztecAddress,
    constants::SCOPED_KEY_VALIDATION_REQUEST_AND_GENERATOR_LENGTH,
    traits::{Deserialize, Empty, Serialize, ToField},
    utils::{arrays::array_concat, reader::Reader},
};

pub struct ScopedKeyValidationRequestAndGenerator {
    pub request: KeyValidationRequestAndGenerator,
    pub contract_address: AztecAddress,
}

impl Scoped<KeyValidationRequestAndGenerator> for ScopedKeyValidationRequestAndGenerator {
    fn inner(self) -> KeyValidationRequestAndGenerator {
        self.request
    }
    fn contract_address(self) -> AztecAddress {
        self.contract_address
    }
}

impl Eq for ScopedKeyValidationRequestAndGenerator {
    fn eq(self, other: ScopedKeyValidationRequestAndGenerator) -> bool {
        (self.request.eq(other.request)) & (self.contract_address.eq(other.contract_address))
    }
}

impl Empty for ScopedKeyValidationRequestAndGenerator {
    fn empty() -> Self {
        ScopedKeyValidationRequestAndGenerator {
            request: KeyValidationRequestAndGenerator::empty(),
            contract_address: AztecAddress::zero(),
        }
    }
}

impl Serialize<SCOPED_KEY_VALIDATION_REQUEST_AND_GENERATOR_LENGTH> for ScopedKeyValidationRequestAndGenerator {
    fn serialize(self) -> [Field; SCOPED_KEY_VALIDATION_REQUEST_AND_GENERATOR_LENGTH] {
        array_concat(self.request.serialize(), [self.contract_address.to_field()])
    }
}

impl Deserialize<SCOPED_KEY_VALIDATION_REQUEST_AND_GENERATOR_LENGTH> for ScopedKeyValidationRequestAndGenerator {
    fn deserialize(fields: [Field; SCOPED_KEY_VALIDATION_REQUEST_AND_GENERATOR_LENGTH]) -> Self {
        let mut reader = Reader::new(fields);
        let res = Self {
            request: reader.read_struct(KeyValidationRequestAndGenerator::deserialize),
            contract_address: reader.read_struct(AztecAddress::deserialize),
        };
        reader.finish();
        res
    }
}

#[test]
fn serialization_of_empty() {
    let item = ScopedKeyValidationRequestAndGenerator::empty();
    let serialized = item.serialize();
    let deserialized = ScopedKeyValidationRequestAndGenerator::deserialize(serialized);
    assert(item.eq(deserialized));
}
