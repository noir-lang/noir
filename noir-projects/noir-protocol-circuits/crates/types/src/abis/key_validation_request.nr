use dep::std::cmp::Eq;
use crate::{
    address::AztecAddress,
    constants::{SCOPED_KEY_VALIDATION_REQUEST_LENGTH, KEY_VALIDATION_REQUEST_LENGTH},
    traits::{Empty, Serialize, Deserialize}, grumpkin_point::GrumpkinPoint,
    utils::{arrays::array_concat, reader::Reader}
};

struct KeyValidationRequest {
    pk_m: GrumpkinPoint,
    sk_app: Field, // not a grumpkin scalar because it's output of poseidon2
}

impl Eq for KeyValidationRequest {
    fn eq(self, request: KeyValidationRequest) -> bool {
        (request.pk_m.eq(self.pk_m))
            & (request.sk_app.eq(self.sk_app))
    }
}

impl Empty for KeyValidationRequest {
    fn empty() -> Self {
        KeyValidationRequest {
            pk_m: GrumpkinPoint::zero(),
            sk_app: 0,
        }
    }
}

impl Serialize<KEY_VALIDATION_REQUEST_LENGTH> for KeyValidationRequest {
    fn serialize(self) -> [Field; KEY_VALIDATION_REQUEST_LENGTH] {
        [
            self.pk_m.x,
            self.pk_m.y,
            self.sk_app,
        ]
    }
}

impl Deserialize<KEY_VALIDATION_REQUEST_LENGTH> for KeyValidationRequest {
    fn deserialize(fields: [Field; KEY_VALIDATION_REQUEST_LENGTH]) -> Self {
        Self {
            pk_m: GrumpkinPoint::new(fields[0], fields[1]),
            sk_app: fields[2],
        }
    }
}

impl KeyValidationRequest {
    pub fn scope(self, contract_address: AztecAddress) -> ScopedKeyValidationRequest {
        ScopedKeyValidationRequest { request: self, contract_address }
    }
}

struct ScopedKeyValidationRequest {
    request: KeyValidationRequest,
    contract_address: AztecAddress,
}

impl Eq for ScopedKeyValidationRequest {
    fn eq(self, other: ScopedKeyValidationRequest) -> bool {
        (self.request.eq(other.request))
            & (self.contract_address.eq(other.contract_address))
    }
}

impl Empty for ScopedKeyValidationRequest {
    fn empty() -> Self {
        ScopedKeyValidationRequest {
            request: KeyValidationRequest::empty(),
            contract_address: AztecAddress::zero(),
        }
    }
}

impl Serialize<SCOPED_KEY_VALIDATION_REQUEST_LENGTH> for ScopedKeyValidationRequest {
    fn serialize(self) -> [Field; SCOPED_KEY_VALIDATION_REQUEST_LENGTH] {
        array_concat(self.request.serialize(), [self.contract_address.to_field()])
    }
}

impl Deserialize<SCOPED_KEY_VALIDATION_REQUEST_LENGTH> for ScopedKeyValidationRequest {
    fn deserialize(fields: [Field; SCOPED_KEY_VALIDATION_REQUEST_LENGTH]) -> Self {
        let mut reader = Reader::new(fields);
        let res = Self {
            request: reader.read_struct(KeyValidationRequest::deserialize),
            contract_address: reader.read_struct(AztecAddress::deserialize),
        };
        reader.finish();
        res
    }
}

#[test]
fn serialization_of_empty() {
    let item = ScopedKeyValidationRequest::empty();
    let serialized = item.serialize();
    let deserialized = ScopedKeyValidationRequest::deserialize(serialized);
    assert(item.eq(deserialized));
}

