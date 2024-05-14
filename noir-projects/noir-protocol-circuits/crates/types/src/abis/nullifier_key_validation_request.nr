use dep::std::cmp::Eq;
use crate::{
    address::AztecAddress,
    constants::{SCOPED_NULLIFIER_KEY_VALIDATION_REQUEST_LENGTH, NULLIFIER_KEY_VALIDATION_REQUEST_LENGTH},
    traits::{Empty, Serialize, Deserialize}, grumpkin_point::GrumpkinPoint,
    utils::{arrays::array_concat, reader::Reader}
};

struct NullifierKeyValidationRequest {
    master_nullifier_public_key: GrumpkinPoint,
    app_nullifier_secret_key: Field, // not a grumpkin scalar because it's output of poseidon2
}

impl Eq for NullifierKeyValidationRequest {
    fn eq(self, request: NullifierKeyValidationRequest) -> bool {
        (request.master_nullifier_public_key.eq(self.master_nullifier_public_key))
            & (request.app_nullifier_secret_key.eq(self.app_nullifier_secret_key))
    }
}

impl Empty for NullifierKeyValidationRequest {
    fn empty() -> Self {
        NullifierKeyValidationRequest {
            master_nullifier_public_key: GrumpkinPoint::zero(),
            app_nullifier_secret_key: 0,
        }
    }
}

impl Serialize<NULLIFIER_KEY_VALIDATION_REQUEST_LENGTH> for NullifierKeyValidationRequest {
    fn serialize(self) -> [Field; NULLIFIER_KEY_VALIDATION_REQUEST_LENGTH] {
        [
            self.master_nullifier_public_key.x,
            self.master_nullifier_public_key.y,
            self.app_nullifier_secret_key,
        ]
    }
}

impl Deserialize<NULLIFIER_KEY_VALIDATION_REQUEST_LENGTH> for NullifierKeyValidationRequest {
    fn deserialize(fields: [Field; NULLIFIER_KEY_VALIDATION_REQUEST_LENGTH]) -> Self {
        Self {
            master_nullifier_public_key: GrumpkinPoint::new(fields[0], fields[1]),
            app_nullifier_secret_key: fields[2],
        }
    }
}

impl NullifierKeyValidationRequest {
    pub fn scope(self, contract_address: AztecAddress) -> ScopedNullifierKeyValidationRequest {
        ScopedNullifierKeyValidationRequest { request: self, contract_address }
    }
}

struct ScopedNullifierKeyValidationRequest {
    request: NullifierKeyValidationRequest,
    contract_address: AztecAddress,
}

impl Eq for ScopedNullifierKeyValidationRequest {
    fn eq(self, other: ScopedNullifierKeyValidationRequest) -> bool {
        (self.request.eq(other.request))
            & (self.contract_address.eq(other.contract_address))
    }
}

impl Empty for ScopedNullifierKeyValidationRequest {
    fn empty() -> Self {
        ScopedNullifierKeyValidationRequest {
            request: NullifierKeyValidationRequest::empty(),
            contract_address: AztecAddress::zero(),
        }
    }
}

impl Serialize<SCOPED_NULLIFIER_KEY_VALIDATION_REQUEST_LENGTH> for ScopedNullifierKeyValidationRequest {
    fn serialize(self) -> [Field; SCOPED_NULLIFIER_KEY_VALIDATION_REQUEST_LENGTH] {
        array_concat(self.request.serialize(), [self.contract_address.to_field()])
    }
}

impl Deserialize<SCOPED_NULLIFIER_KEY_VALIDATION_REQUEST_LENGTH> for ScopedNullifierKeyValidationRequest {
    fn deserialize(fields: [Field; SCOPED_NULLIFIER_KEY_VALIDATION_REQUEST_LENGTH]) -> Self {
        let mut reader = Reader::new(fields);
        let res = Self {
            request: reader.read_struct(NullifierKeyValidationRequest::deserialize),
            contract_address: reader.read_struct(AztecAddress::deserialize),
        };
        reader.finish();
        res
    }
}

#[test]
fn serialization_of_empty() {
    let item = ScopedNullifierKeyValidationRequest::empty();
    let serialized = item.serialize();
    let deserialized = ScopedNullifierKeyValidationRequest::deserialize(serialized);
    assert(item.eq(deserialized));
}

