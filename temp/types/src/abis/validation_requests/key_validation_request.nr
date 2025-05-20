use crate::{point::Point, traits::{Deserialize, Empty, Serialize}};
pub use crate::constants::KEY_VALIDATION_REQUEST_LENGTH;

pub struct KeyValidationRequest {
    pub pk_m: Point,
    pub sk_app: Field, // not a grumpkin scalar because it's output of poseidon2
}

impl Eq for KeyValidationRequest {
    fn eq(self, request: KeyValidationRequest) -> bool {
        (request.pk_m.eq(self.pk_m)) & (request.sk_app.eq(self.sk_app))
    }
}

impl Empty for KeyValidationRequest {
    fn empty() -> Self {
        KeyValidationRequest { pk_m: Point::empty(), sk_app: 0 }
    }
}

impl Serialize<KEY_VALIDATION_REQUEST_LENGTH> for KeyValidationRequest {
    fn serialize(self) -> [Field; KEY_VALIDATION_REQUEST_LENGTH] {
        [self.pk_m.x, self.pk_m.y, self.pk_m.is_infinite as Field, self.sk_app]
    }
}

impl Deserialize<KEY_VALIDATION_REQUEST_LENGTH> for KeyValidationRequest {
    fn deserialize(fields: [Field; KEY_VALIDATION_REQUEST_LENGTH]) -> Self {
        Self {
            pk_m: Point { x: fields[0], y: fields[1], is_infinite: fields[2] as bool },
            sk_app: fields[3],
        }
    }
}
