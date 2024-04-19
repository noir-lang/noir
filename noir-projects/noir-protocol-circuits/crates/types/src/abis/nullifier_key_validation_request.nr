use dep::std::cmp::Eq;
use crate::{
    address::AztecAddress,
    constants::{NULLIFIER_KEY_VALIDATION_REQUEST_CONTEXT_LENGTH, NULLIFIER_KEY_VALIDATION_REQUEST_LENGTH},
    traits::{Empty, Serialize, Deserialize}, grumpkin_point::GrumpkinPoint,
    grumpkin_private_key::GrumpkinPrivateKey
};

struct NullifierKeyValidationRequest {
    master_nullifier_public_key: GrumpkinPoint,
    app_nullifier_secret_key: Field, // not a GrumpkinScalar because it's output of poseidon2
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
    pub fn to_context(self, contract_address: AztecAddress) -> NullifierKeyValidationRequestContext {
        NullifierKeyValidationRequestContext {
            master_nullifier_public_key: self.master_nullifier_public_key,
            app_nullifier_secret_key: self.app_nullifier_secret_key,
            contract_address
        }
    }
}

struct NullifierKeyValidationRequestContext {
    master_nullifier_public_key: GrumpkinPoint,
    app_nullifier_secret_key: Field,
    contract_address: AztecAddress,
}

impl Eq for NullifierKeyValidationRequestContext {
    fn eq(self, request: NullifierKeyValidationRequestContext) -> bool {
        (request.master_nullifier_public_key.eq(self.master_nullifier_public_key))
            & (request.app_nullifier_secret_key.eq(self.app_nullifier_secret_key))
            & (request.contract_address.eq(self.contract_address))
    }
}

impl Empty for NullifierKeyValidationRequestContext {
    fn empty() -> Self {
        NullifierKeyValidationRequestContext {
            master_nullifier_public_key: GrumpkinPoint::zero(),
            app_nullifier_secret_key: 0,
            contract_address: AztecAddress::zero(),
        }
    }
}

impl Serialize<NULLIFIER_KEY_VALIDATION_REQUEST_CONTEXT_LENGTH> for NullifierKeyValidationRequestContext {
    fn serialize(self) -> [Field; NULLIFIER_KEY_VALIDATION_REQUEST_CONTEXT_LENGTH] {
        [
            self.master_nullifier_public_key.x,
            self.master_nullifier_public_key.y,
            self.app_nullifier_secret_key,
            self.contract_address.to_field(),
        ]
    }
}

impl Deserialize<NULLIFIER_KEY_VALIDATION_REQUEST_CONTEXT_LENGTH> for NullifierKeyValidationRequestContext {
    fn deserialize(fields: [Field; NULLIFIER_KEY_VALIDATION_REQUEST_CONTEXT_LENGTH]) -> Self {
        Self {
            master_nullifier_public_key: GrumpkinPoint::new(fields[0], fields[1]),
            app_nullifier_secret_key: fields[2],
            contract_address: AztecAddress::from_field(fields[3]),
        }
    }
}

