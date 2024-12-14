use crate::cmp::Eq;
use crate::ops::{Add, Div, Mul, Sub};

global bn254_fq: [u8] = &[
    0x47, 0xFD, 0x7C, 0xD8, 0x16, 0x8C, 0x20, 0x3C, 0x8d, 0xca, 0x71, 0x68, 0x91, 0x6a, 0x81, 0x97,
    0x5d, 0x58, 0x81, 0x81, 0xb6, 0x45, 0x50, 0xb8, 0x29, 0xa0, 0x31, 0xe1, 0x72, 0x4e, 0x64, 0x30,
];
global bn254_fr: [u8] = &[
    1, 0, 0, 240, 147, 245, 225, 67, 145, 112, 185, 121, 72, 232, 51, 40, 93, 88, 129, 129, 182, 69,
    80, 184, 41, 160, 49, 225, 114, 78, 100, 48,
];
global secpk1_fr: [u8] = &[
    0x41, 0x41, 0x36, 0xD0, 0x8C, 0x5E, 0xD2, 0xBF, 0x3B, 0xA0, 0x48, 0xAF, 0xE6, 0xDC, 0xAE, 0xBA,
    0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
];
global secpk1_fq: [u8] = &[
    0x2F, 0xFC, 0xFF, 0xFF, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
];
global secpr1_fq: [u8] = &[
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF,
];
global secpr1_fr: [u8] = &[
    81, 37, 99, 252, 194, 202, 185, 243, 132, 158, 23, 167, 173, 250, 230, 188, 255, 255, 255, 255,
    255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255,
];
// docs:start:big_int_definition
pub struct BigInt {
    pointer: u32,
    modulus: u32,
}
// docs:end:big_int_definition

impl BigInt {
    #[builtin(bigint_add)]
    fn bigint_add(self, other: BigInt) -> BigInt {}
    #[builtin(bigint_sub)]
    fn bigint_sub(self, other: BigInt) -> BigInt {}
    #[builtin(bigint_mul)]
    fn bigint_mul(self, other: BigInt) -> BigInt {}
    #[builtin(bigint_div)]
    fn bigint_div(self, other: BigInt) -> BigInt {}
    #[builtin(bigint_from_le_bytes)]
    fn from_le_bytes(bytes: [u8], modulus: [u8]) -> BigInt {}
    #[builtin(bigint_to_le_bytes)]
    fn to_le_bytes(self) -> [u8; 32] {}

    fn check_32_bytes(self: Self, other: BigInt) -> bool {
        let bytes = self.to_le_bytes();
        let o_bytes = other.to_le_bytes();
        let mut result = true;
        for i in 0..32 {
            result = result & (bytes[i] == o_bytes[i]);
        }
        result
    }
}

pub trait BigField {
    fn from_le_bytes(bytes: [u8]) -> Self;
    fn from_le_bytes_32(bytes: [u8; 32]) -> Self;
    fn to_le_bytes(self) -> [u8];
}

pub struct Secpk1Fq {
    array: [u8; 32],
}

impl BigField for Secpk1Fq {
    fn from_le_bytes(bytes: [u8]) -> Secpk1Fq {
        assert(bytes.len() <= 32);
        let mut array = [0; 32];
        for i in 0..bytes.len() {
            array[i] = bytes[i];
        }
        Secpk1Fq { array }
    }

    fn from_le_bytes_32(bytes: [u8; 32]) -> Secpk1Fq {
        Secpk1Fq { array: bytes }
    }

    fn to_le_bytes(self) -> [u8] {
        self.array
    }
}

impl Add for Secpk1Fq {
    fn add(self: Self, other: Secpk1Fq) -> Secpk1Fq {
        let a = BigInt::from_le_bytes(self.array.as_slice(), secpk1_fq);
        let b = BigInt::from_le_bytes(other.array.as_slice(), secpk1_fq);
        Secpk1Fq { array: a.bigint_add(b).to_le_bytes() }
    }
}
impl Sub for Secpk1Fq {
    fn sub(self: Self, other: Secpk1Fq) -> Secpk1Fq {
        let a = BigInt::from_le_bytes(self.array.as_slice(), secpk1_fq);
        let b = BigInt::from_le_bytes(other.array.as_slice(), secpk1_fq);
        Secpk1Fq { array: a.bigint_sub(b).to_le_bytes() }
    }
}
impl Mul for Secpk1Fq {
    fn mul(self: Self, other: Secpk1Fq) -> Secpk1Fq {
        let a = BigInt::from_le_bytes(self.array.as_slice(), secpk1_fq);
        let b = BigInt::from_le_bytes(other.array.as_slice(), secpk1_fq);
        Secpk1Fq { array: a.bigint_mul(b).to_le_bytes() }
    }
}
impl Div for Secpk1Fq {
    fn div(self: Self, other: Secpk1Fq) -> Secpk1Fq {
        let a = BigInt::from_le_bytes(self.array.as_slice(), secpk1_fq);
        let b = BigInt::from_le_bytes(other.array.as_slice(), secpk1_fq);
        Secpk1Fq { array: a.bigint_div(b).to_le_bytes() }
    }
}
impl Eq for Secpk1Fq {
    fn eq(self: Self, other: Secpk1Fq) -> bool {
        self.array == other.array
    }
}

pub struct Secpk1Fr {
    array: [u8; 32],
}

impl BigField for Secpk1Fr {
    fn from_le_bytes(bytes: [u8]) -> Secpk1Fr {
        assert(bytes.len() <= 32);
        let mut array = [0; 32];
        for i in 0..bytes.len() {
            array[i] = bytes[i];
        }
        Secpk1Fr { array }
    }

    fn from_le_bytes_32(bytes: [u8; 32]) -> Secpk1Fr {
        Secpk1Fr { array: bytes }
    }

    fn to_le_bytes(self) -> [u8] {
        self.array
    }
}

impl Add for Secpk1Fr {
    fn add(self: Self, other: Secpk1Fr) -> Secpk1Fr {
        let a = BigInt::from_le_bytes(self.array.as_slice(), secpk1_fr);
        let b = BigInt::from_le_bytes(other.array.as_slice(), secpk1_fr);
        Secpk1Fr { array: a.bigint_add(b).to_le_bytes() }
    }
}
impl Sub for Secpk1Fr {
    fn sub(self: Self, other: Secpk1Fr) -> Secpk1Fr {
        let a = BigInt::from_le_bytes(self.array.as_slice(), secpk1_fr);
        let b = BigInt::from_le_bytes(other.array.as_slice(), secpk1_fr);
        Secpk1Fr { array: a.bigint_sub(b).to_le_bytes() }
    }
}
impl Mul for Secpk1Fr {
    fn mul(self: Self, other: Secpk1Fr) -> Secpk1Fr {
        let a = BigInt::from_le_bytes(self.array.as_slice(), secpk1_fr);
        let b = BigInt::from_le_bytes(other.array.as_slice(), secpk1_fr);
        Secpk1Fr { array: a.bigint_mul(b).to_le_bytes() }
    }
}
impl Div for Secpk1Fr {
    fn div(self: Self, other: Secpk1Fr) -> Secpk1Fr {
        let a = BigInt::from_le_bytes(self.array.as_slice(), secpk1_fr);
        let b = BigInt::from_le_bytes(other.array.as_slice(), secpk1_fr);
        Secpk1Fr { array: a.bigint_div(b).to_le_bytes() }
    }
}
impl Eq for Secpk1Fr {
    fn eq(self: Self, other: Secpk1Fr) -> bool {
        self.array == other.array
    }
}

pub struct Bn254Fr {
    array: [u8; 32],
}

impl BigField for Bn254Fr {
    fn from_le_bytes(bytes: [u8]) -> Bn254Fr {
        assert(bytes.len() <= 32);
        let mut array = [0; 32];
        for i in 0..bytes.len() {
            array[i] = bytes[i];
        }
        Bn254Fr { array }
    }

    fn from_le_bytes_32(bytes: [u8; 32]) -> Bn254Fr {
        Bn254Fr { array: bytes }
    }

    fn to_le_bytes(self) -> [u8] {
        self.array
    }
}

impl Add for Bn254Fr {
    fn add(self: Self, other: Bn254Fr) -> Bn254Fr {
        let a = BigInt::from_le_bytes(self.array.as_slice(), bn254_fr);
        let b = BigInt::from_le_bytes(other.array.as_slice(), bn254_fr);
        Bn254Fr { array: a.bigint_add(b).to_le_bytes() }
    }
}
impl Sub for Bn254Fr {
    fn sub(self: Self, other: Bn254Fr) -> Bn254Fr {
        let a = BigInt::from_le_bytes(self.array.as_slice(), bn254_fr);
        let b = BigInt::from_le_bytes(other.array.as_slice(), bn254_fr);
        Bn254Fr { array: a.bigint_sub(b).to_le_bytes() }
    }
}
impl Mul for Bn254Fr {
    fn mul(self: Self, other: Bn254Fr) -> Bn254Fr {
        let a = BigInt::from_le_bytes(self.array.as_slice(), bn254_fr);
        let b = BigInt::from_le_bytes(other.array.as_slice(), bn254_fr);
        Bn254Fr { array: a.bigint_mul(b).to_le_bytes() }
    }
}
impl Div for Bn254Fr {
    fn div(self: Self, other: Bn254Fr) -> Bn254Fr {
        let a = BigInt::from_le_bytes(self.array.as_slice(), bn254_fr);
        let b = BigInt::from_le_bytes(other.array.as_slice(), bn254_fr);
        Bn254Fr { array: a.bigint_div(b).to_le_bytes() }
    }
}
impl Eq for Bn254Fr {
    fn eq(self: Self, other: Bn254Fr) -> bool {
        self.array == other.array
    }
}

pub struct Bn254Fq {
    array: [u8; 32],
}

impl BigField for Bn254Fq {
    fn from_le_bytes(bytes: [u8]) -> Bn254Fq {
        assert(bytes.len() <= 32);
        let mut array = [0; 32];
        for i in 0..bytes.len() {
            array[i] = bytes[i];
        }
        Bn254Fq { array }
    }

    fn from_le_bytes_32(bytes: [u8; 32]) -> Bn254Fq {
        Bn254Fq { array: bytes }
    }

    fn to_le_bytes(self) -> [u8] {
        self.array
    }
}

impl Add for Bn254Fq {
    fn add(self: Self, other: Bn254Fq) -> Bn254Fq {
        let a = BigInt::from_le_bytes(self.array.as_slice(), bn254_fq);
        let b = BigInt::from_le_bytes(other.array.as_slice(), bn254_fq);
        Bn254Fq { array: a.bigint_add(b).to_le_bytes() }
    }
}
impl Sub for Bn254Fq {
    fn sub(self: Self, other: Bn254Fq) -> Bn254Fq {
        let a = BigInt::from_le_bytes(self.array.as_slice(), bn254_fq);
        let b = BigInt::from_le_bytes(other.array.as_slice(), bn254_fq);
        Bn254Fq { array: a.bigint_sub(b).to_le_bytes() }
    }
}
impl Mul for Bn254Fq {
    fn mul(self: Self, other: Bn254Fq) -> Bn254Fq {
        let a = BigInt::from_le_bytes(self.array.as_slice(), bn254_fq);
        let b = BigInt::from_le_bytes(other.array.as_slice(), bn254_fq);
        Bn254Fq { array: a.bigint_mul(b).to_le_bytes() }
    }
}
impl Div for Bn254Fq {
    fn div(self: Self, other: Bn254Fq) -> Bn254Fq {
        let a = BigInt::from_le_bytes(self.array.as_slice(), bn254_fq);
        let b = BigInt::from_le_bytes(other.array.as_slice(), bn254_fq);
        Bn254Fq { array: a.bigint_div(b).to_le_bytes() }
    }
}
impl Eq for Bn254Fq {
    fn eq(self: Self, other: Bn254Fq) -> bool {
        self.array == other.array
    }
}

pub struct Secpr1Fq {
    array: [u8; 32],
}

impl BigField for Secpr1Fq {
    fn from_le_bytes(bytes: [u8]) -> Secpr1Fq {
        assert(bytes.len() <= 32);
        let mut array = [0; 32];
        for i in 0..bytes.len() {
            array[i] = bytes[i];
        }
        Secpr1Fq { array }
    }

    fn from_le_bytes_32(bytes: [u8; 32]) -> Secpr1Fq {
        Secpr1Fq { array: bytes }
    }

    fn to_le_bytes(self) -> [u8] {
        self.array
    }
}

impl Add for Secpr1Fq {
    fn add(self: Self, other: Secpr1Fq) -> Secpr1Fq {
        let a = BigInt::from_le_bytes(self.array.as_slice(), secpr1_fq);
        let b = BigInt::from_le_bytes(other.array.as_slice(), secpr1_fq);
        Secpr1Fq { array: a.bigint_add(b).to_le_bytes() }
    }
}
impl Sub for Secpr1Fq {
    fn sub(self: Self, other: Secpr1Fq) -> Secpr1Fq {
        let a = BigInt::from_le_bytes(self.array.as_slice(), secpr1_fq);
        let b = BigInt::from_le_bytes(other.array.as_slice(), secpr1_fq);
        Secpr1Fq { array: a.bigint_sub(b).to_le_bytes() }
    }
}
impl Mul for Secpr1Fq {
    fn mul(self: Self, other: Secpr1Fq) -> Secpr1Fq {
        let a = BigInt::from_le_bytes(self.array.as_slice(), secpr1_fq);
        let b = BigInt::from_le_bytes(other.array.as_slice(), secpr1_fq);
        Secpr1Fq { array: a.bigint_mul(b).to_le_bytes() }
    }
}
impl Div for Secpr1Fq {
    fn div(self: Self, other: Secpr1Fq) -> Secpr1Fq {
        let a = BigInt::from_le_bytes(self.array.as_slice(), secpr1_fq);
        let b = BigInt::from_le_bytes(other.array.as_slice(), secpr1_fq);
        Secpr1Fq { array: a.bigint_div(b).to_le_bytes() }
    }
}
impl Eq for Secpr1Fq {
    fn eq(self: Self, other: Secpr1Fq) -> bool {
        self.array == other.array
    }
}

pub struct Secpr1Fr {
    array: [u8; 32],
}

impl BigField for Secpr1Fr {
    fn from_le_bytes(bytes: [u8]) -> Secpr1Fr {
        assert(bytes.len() <= 32);
        let mut array = [0; 32];
        for i in 0..bytes.len() {
            array[i] = bytes[i];
        }
        Secpr1Fr { array }
    }

    fn from_le_bytes_32(bytes: [u8; 32]) -> Secpr1Fr {
        Secpr1Fr { array: bytes }
    }

    fn to_le_bytes(self) -> [u8] {
        self.array
    }
}

impl Add for Secpr1Fr {
    fn add(self: Self, other: Secpr1Fr) -> Secpr1Fr {
        let a = BigInt::from_le_bytes(self.array.as_slice(), secpr1_fr);
        let b = BigInt::from_le_bytes(other.array.as_slice(), secpr1_fr);
        Secpr1Fr { array: a.bigint_add(b).to_le_bytes() }
    }
}
impl Sub for Secpr1Fr {
    fn sub(self: Self, other: Secpr1Fr) -> Secpr1Fr {
        let a = BigInt::from_le_bytes(self.array.as_slice(), secpr1_fr);
        let b = BigInt::from_le_bytes(other.array.as_slice(), secpr1_fr);
        Secpr1Fr { array: a.bigint_sub(b).to_le_bytes() }
    }
}
impl Mul for Secpr1Fr {
    fn mul(self: Self, other: Secpr1Fr) -> Secpr1Fr {
        let a = BigInt::from_le_bytes(self.array.as_slice(), secpr1_fr);
        let b = BigInt::from_le_bytes(other.array.as_slice(), secpr1_fr);
        Secpr1Fr { array: a.bigint_mul(b).to_le_bytes() }
    }
}
impl Div for Secpr1Fr {
    fn div(self: Self, other: Secpr1Fr) -> Secpr1Fr {
        let a = BigInt::from_le_bytes(self.array.as_slice(), secpr1_fr);
        let b = BigInt::from_le_bytes(other.array.as_slice(), secpr1_fr);
        Secpr1Fr { array: a.bigint_div(b).to_le_bytes() }
    }
}
impl Eq for Secpr1Fr {
    fn eq(self: Self, other: Secpr1Fr) -> bool {
        self.array == other.array
    }
}
