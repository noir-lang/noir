use dep::std::{
    cmp::Eq,
    grumpkin_scalar::GrumpkinScalar,
    grumpkin_scalar_mul::grumpkin_fixed_base,
};
use crate::grumpkin_point::GrumpkinPoint;

global GRUMPKIN_PRIVATE_KEY_SERIALIZED_LEN: Field = 2;

struct GrumpkinPrivateKey {
    high: Field,
    low: Field,
}

impl Eq for GrumpkinPrivateKey {
    fn eq(self, key: GrumpkinPrivateKey) -> bool {
        (key.high == self.high) & (key.low == self.low)
    }
}

impl GrumpkinPrivateKey {
    pub fn new(high: Field, low: Field) -> Self {
        GrumpkinPrivateKey { high, low }
    }

    pub fn zero() -> Self {
        Self {
            high: 0,
            low: 0,
        }
    }

    pub fn is_zero(self) -> bool {
        (self.high == 0) & (self.low == 0)
    }

    pub fn serialize(self) -> [Field; GRUMPKIN_PRIVATE_KEY_SERIALIZED_LEN] {
        [self.high, self.low]
    }

    pub fn derive_public_key(self) -> GrumpkinPoint {
        let public_key = grumpkin_fixed_base(GrumpkinScalar { high: self.high, low: self.low });
        GrumpkinPoint { x: public_key[0], y: public_key[1] }
    }
}
