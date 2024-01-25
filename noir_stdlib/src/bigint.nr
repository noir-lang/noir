
use crate::ops::{Add, Sub, Mul, Div, Rem,};

struct BigInt {
    pointer: u32,
    modulus: u32,
}

impl BigInt {
    #[builtin(bigint_add)]
    pub fn bigint_add(self, other: BigInt) -> BigInt {
    }
    #[builtin(bigint_neg)]
    pub fn bigint_neg(self, other: BigInt) -> BigInt {
    }
    #[builtin(bigint_mul)]
    pub fn bigint_mul(self, other: BigInt) -> BigInt {
    }
    #[builtin(bigint_div)]
    pub fn bigint_div(self, other: BigInt) -> BigInt {
    }
    #[builtin(bigint_from_le_bytes)]
    pub fn from_le_bytes(bytes: [u8], modulus: [u8]) -> BigInt {}
    #[builtin(bigint_to_le_bytes)]
    pub fn to_le_bytes(self) -> [u8] {}
}

impl Add for BigInt { 
    fn add(self: Self, other: BigInt) -> BigInt {
        self.bigint_add(other)
    }
}
impl Sub for BigInt { 
    fn sub(self: Self, other: BigInt) -> BigInt {
        self.bigint_neg(other)
    }
}
impl Mul for BigInt { 
    fn mul(self: Self, other: BigInt) -> BigInt {
        self.bigint_mul(other)
    }
}
impl Div for BigInt { 
    fn div(self: Self, other: BigInt) -> BigInt {
        self.bigint_div(other)
    }
}
impl Rem for BigInt { 
    fn rem(self: Self, other: BigInt) -> BigInt {
        let quotient = self.bigint_div(other);
        self.bigint_neg(quotient.bigint_mul(other))
    }
}

