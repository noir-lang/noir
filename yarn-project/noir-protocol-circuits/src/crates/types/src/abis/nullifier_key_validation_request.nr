use dep::std::cmp::Eq;
use crate::{
    address::AztecAddress,
    traits::Empty,
    grumpkin_point::GrumpkinPoint,
    grumpkin_private_key::GrumpkinPrivateKey,
};

global NULLIFIER_KEY_VALIDATION_REQUEST_SERIALIZED_LEN = 4;
global NULLIFIER_KEY_VALIDATION_REQUEST_CONTEXT_SERIALIZED_LEN = 5;

struct NullifierKeyValidationRequest {
    public_key: GrumpkinPoint,
    secret_key: GrumpkinPrivateKey,
}

impl Eq for NullifierKeyValidationRequest {
    fn eq(self, request: NullifierKeyValidationRequest) -> bool {
        (request.public_key.eq(self.public_key))
            & (request.secret_key.eq(self.secret_key))
    }
}

impl Empty for NullifierKeyValidationRequest {
    fn empty() -> Self {
        NullifierKeyValidationRequest {
            public_key: GrumpkinPoint::zero(),
            secret_key: GrumpkinPrivateKey::zero(),
        }
    }
}

impl NullifierKeyValidationRequest {
    pub fn serialize(self) -> [Field; NULLIFIER_KEY_VALIDATION_REQUEST_SERIALIZED_LEN] {
        [
            self.public_key.x,
            self.public_key.y,
            self.secret_key.high,
            self.secret_key.low,
        ]
    }

    pub fn deserialise(fields: [Field; NULLIFIER_KEY_VALIDATION_REQUEST_SERIALIZED_LEN]) -> Self {
        Self {
            public_key: GrumpkinPoint::new(fields[0], fields[1]),
            secret_key: GrumpkinPrivateKey::new(fields[2], fields[3]),
        }
    }

    pub fn to_context(self, contract_address: AztecAddress) -> NullifierKeyValidationRequestContext {
        NullifierKeyValidationRequestContext {
            public_key: self.public_key,
            secret_key: self.secret_key,
            contract_address,
        }
    }
}

struct NullifierKeyValidationRequestContext {
    public_key: GrumpkinPoint,
    secret_key: GrumpkinPrivateKey,
    contract_address: AztecAddress,
}

impl Eq for NullifierKeyValidationRequestContext {
    fn eq(self, request: NullifierKeyValidationRequestContext) -> bool {
        (request.public_key.eq(self.public_key))
            & (request.secret_key.eq(self.secret_key))
            & (request.contract_address.eq(self.contract_address))
    }
}

impl Empty for NullifierKeyValidationRequestContext {
    fn empty() -> Self {
        NullifierKeyValidationRequestContext {
            public_key: GrumpkinPoint::zero(),
            secret_key: GrumpkinPrivateKey::zero(),
            contract_address: AztecAddress::zero(),
        }
    }
}

impl NullifierKeyValidationRequestContext {
    pub fn serialize(self) -> [Field; NULLIFIER_KEY_VALIDATION_REQUEST_CONTEXT_SERIALIZED_LEN] {
        [
            self.public_key.x,
            self.public_key.y,
            self.secret_key.high,
            self.secret_key.low,
            self.contract_address.to_field(),
        ]
    }

    pub fn deserialise(fields: [Field; NULLIFIER_KEY_VALIDATION_REQUEST_CONTEXT_SERIALIZED_LEN]) -> Self {
        Self {
            public_key: GrumpkinPoint::new(fields[0], fields[1]),
            secret_key: GrumpkinPrivateKey::new(fields[2], fields[3]),
            contract_address: AztecAddress::from_field(fields[4]),
        }
    }
}
