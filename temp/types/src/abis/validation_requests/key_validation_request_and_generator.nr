use crate::{
    abis::validation_requests::{
        key_validation_request::KeyValidationRequest,
        scoped_key_validation_request_and_generator::ScopedKeyValidationRequestAndGenerator,
    },
    address::AztecAddress,
    constants::KEY_VALIDATION_REQUEST_AND_GENERATOR_LENGTH,
    traits::{Deserialize, Empty, Serialize},
    utils::{arrays::array_concat, reader::Reader},
};

pub struct KeyValidationRequestAndGenerator {
    pub request: KeyValidationRequest,
    pub sk_app_generator: Field,
}

impl Eq for KeyValidationRequestAndGenerator {
    fn eq(self, other: KeyValidationRequestAndGenerator) -> bool {
        (self.request == other.request) & (self.sk_app_generator == other.sk_app_generator)
    }
}

impl Empty for KeyValidationRequestAndGenerator {
    fn empty() -> Self {
        KeyValidationRequestAndGenerator {
            request: KeyValidationRequest::empty(),
            sk_app_generator: 0,
        }
    }
}

impl Serialize<KEY_VALIDATION_REQUEST_AND_GENERATOR_LENGTH> for KeyValidationRequestAndGenerator {
    fn serialize(self) -> [Field; KEY_VALIDATION_REQUEST_AND_GENERATOR_LENGTH] {
        array_concat(self.request.serialize(), [self.sk_app_generator])
    }
}

impl Deserialize<KEY_VALIDATION_REQUEST_AND_GENERATOR_LENGTH> for KeyValidationRequestAndGenerator {
    fn deserialize(fields: [Field; KEY_VALIDATION_REQUEST_AND_GENERATOR_LENGTH]) -> Self {
        let mut reader = Reader::new(fields);
        let res = Self {
            request: reader.read_struct(KeyValidationRequest::deserialize),
            sk_app_generator: reader.read(),
        };
        reader.finish();
        res
    }
}

impl KeyValidationRequestAndGenerator {
    pub fn scope(self, contract_address: AztecAddress) -> ScopedKeyValidationRequestAndGenerator {
        ScopedKeyValidationRequestAndGenerator { request: self, contract_address }
    }
}

#[test]
fn serialization_of_empty() {
    let item = KeyValidationRequestAndGenerator::empty();
    let serialized = item.serialize();
    let deserialized = KeyValidationRequestAndGenerator::deserialize(serialized);
    assert(item.eq(deserialized));
}
