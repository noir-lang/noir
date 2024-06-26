use crate::{
    constants::KEY_VALIDATION_REQUEST_LENGTH, traits::{Empty, Serialize, Deserialize},
    grumpkin_point::GrumpkinPoint
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

