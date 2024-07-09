use std::{cmp::Eq, embedded_curve_ops::fixed_base_scalar_mul};
use crate::{point::Point, traits::Empty};

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

impl Empty for GrumpkinPrivateKey {
    fn empty() -> Self {
        Self { high: 0, low: 0 }
    }
}

impl GrumpkinPrivateKey {
    pub fn new(high: Field, low: Field) -> Self {
        GrumpkinPrivateKey { high, low }
    }

    pub fn zero() -> Self {
        Self { high: 0, low: 0 }
    }

    pub fn is_zero(self) -> bool {
        (self.high == 0) & (self.low == 0)
    }

    pub fn serialize(self) -> [Field; GRUMPKIN_PRIVATE_KEY_SERIALIZED_LEN] {
        [self.high, self.low]
    }

    pub fn derive_public_key(self) -> Point {
        let public_key = fixed_base_scalar_mul(self.low, self.high);
        Point { x: public_key[0], y: public_key[1], is_infinite: public_key[2] as bool }
    }
}
