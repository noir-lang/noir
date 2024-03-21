use crate::ops::{Add, Sub, Mul, Div};
use crate::cmp::Eq;

// docs:start:curve_order_base
global bn254_fq = [0x47, 0xFD, 0x7C, 0xD8, 0x16, 0x8C, 0x20, 0x3C, 0x8d, 0xca, 0x71, 0x68, 0x91, 0x6a, 0x81, 0x97,
                   0x5d, 0x58, 0x81, 0x81, 0xb6, 0x45, 0x50, 0xb8, 0x29, 0xa0, 0x31, 0xe1, 0x72, 0x4e, 0x64, 0x30];
global bn254_fr = [0x01, 0x00, 0x00, 0x00, 0x3F, 0x59, 0x1F, 0x43, 0x09, 0x97, 0xB9, 0x79, 0x48, 0xE8, 0x33, 0x28, 
                   0x5D, 0x58, 0x81, 0x81, 0xB6, 0x45, 0x50, 0xB8, 0x29, 0xA0, 0x31, 0xE1, 0x72, 0x4E, 0x64, 0x30];
global secpk1_fr = [0x41, 0x41, 0x36, 0xD0, 0x8C, 0x5E, 0xD2, 0xBF, 0x3B, 0xA0, 0x48, 0xAF, 0xE6, 0xDC, 0xAE, 0xBA,
                    0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
global secpk1_fq = [0x2F, 0xFC, 0xFF, 0xFF, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
global secpr1_fq = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF];
global secpr1_fr = [0x51, 0x25, 0x63, 0xFC, 0xC2, 0xCA, 0xB9, 0xF3, 0x84, 0x9E, 0x17, 0xA7, 0xAD, 0xFA, 0xE6, 0xBC,
                    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,0xFF, 0xFF, 0xFF, 0xFF];
// docs:end:curve_order_base

struct BigInt {
    pointer: u32,
    modulus: u32,
}

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
    fn to_le_bytes(self) -> [u8] {}

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

trait BigField {
    fn from_le_bytes(bytes: [u8]) -> Self;
    fn to_le_bytes(self) -> [u8];
}

struct Secpk1Fq {
    inner: BigInt,
}

impl BigField for Secpk1Fq {
    fn from_le_bytes(bytes: [u8]) -> Secpk1Fq {
        Secpk1Fq {
            inner: BigInt::from_le_bytes(bytes, secpk1_fq)
        }
    }
    fn to_le_bytes(self) -> [u8] {
        self.inner.to_le_bytes()
    }
}

impl Add for Secpk1Fq { 
    fn add(self: Self, other: Secpk1Fq) -> Secpk1Fq {
        Secpk1Fq {
            inner: self.inner.bigint_add(other.inner)
        }
    }
}
impl Sub for Secpk1Fq { 
    fn sub(self: Self, other: Secpk1Fq) -> Secpk1Fq {
        Secpk1Fq {
            inner: self.inner.bigint_sub(other.inner)
        }
    }
}
impl Mul for Secpk1Fq { 
    fn mul(self: Self, other: Secpk1Fq) -> Secpk1Fq {
        Secpk1Fq {
            inner: self.inner.bigint_mul(other.inner)
        }

    }
}
impl Div for Secpk1Fq { 
    fn div(self: Self, other: Secpk1Fq) -> Secpk1Fq {
        Secpk1Fq {
            inner: self.inner.bigint_div(other.inner)
        }
    }
}
impl Eq for Secpk1Fq {
    fn eq(self: Self, other: Secpk1Fq) -> bool {
        self.inner.check_32_bytes(other.inner)
    }
}

struct Secpk1Fr {
    inner: BigInt,
}

impl BigField for Secpk1Fr {
    fn from_le_bytes(bytes: [u8]) -> Secpk1Fr {
        Secpk1Fr {
            inner: BigInt::from_le_bytes(bytes, secpk1_fr)
        }
    }
    fn to_le_bytes(self) -> [u8] {
        self.inner.to_le_bytes()
    }
}

impl Add for Secpk1Fr { 
    fn add(self: Self, other: Secpk1Fr) -> Secpk1Fr {
        Secpk1Fr {
            inner: self.inner.bigint_add(other.inner)
        }
    }
}
impl Sub for Secpk1Fr { 
    fn sub(self: Self, other: Secpk1Fr) -> Secpk1Fr {
        Secpk1Fr {
            inner: self.inner.bigint_sub(other.inner)
        }
    }
}
impl Mul for Secpk1Fr { 
    fn mul(self: Self, other: Secpk1Fr) -> Secpk1Fr {
        Secpk1Fr {
            inner: self.inner.bigint_mul(other.inner)
        }

    }
}
impl Div for Secpk1Fr { 
    fn div(self: Self, other: Secpk1Fr) -> Secpk1Fr {
        Secpk1Fr {
            inner: self.inner.bigint_div(other.inner)
        }
    }
}
impl Eq for Secpk1Fr {
    fn eq(self: Self, other: Secpk1Fr) -> bool {
        self.inner.check_32_bytes(other.inner)
    }
}

struct Bn254Fr {
    inner: BigInt,
}

impl BigField for Bn254Fr {
    fn from_le_bytes(bytes: [u8]) -> Bn254Fr {
        Bn254Fr {
            inner: BigInt::from_le_bytes(bytes, bn254_fr)
        }
    }
    fn to_le_bytes(self) -> [u8] {
        self.inner.to_le_bytes()
    }
}

impl Add for Bn254Fr { 
    fn add(self: Self, other: Bn254Fr) -> Bn254Fr {
        Bn254Fr {
            inner: self.inner.bigint_add(other.inner)
        }
    }
}
impl Sub for Bn254Fr { 
    fn sub(self: Self, other: Bn254Fr) -> Bn254Fr {
        Bn254Fr {
            inner: self.inner.bigint_sub(other.inner)
        }
    }
}
impl Mul for Bn254Fr { 
    fn mul(self: Self, other: Bn254Fr) -> Bn254Fr {
        Bn254Fr {
            inner: self.inner.bigint_mul(other.inner)
        }

    }
}
impl Div for Bn254Fr { 
    fn div(self: Self, other: Bn254Fr) -> Bn254Fr {
        Bn254Fr {
            inner: self.inner.bigint_div(other.inner)
        }
    }
}
impl Eq for Bn254Fr {
    fn eq(self: Self, other: Bn254Fr) -> bool {
        self.inner.check_32_bytes(other.inner)
    }
}

struct Bn254Fq {
    inner: BigInt,
}

impl BigField for Bn254Fq {
    fn from_le_bytes(bytes: [u8]) -> Bn254Fq {
        Bn254Fq {
            inner: BigInt::from_le_bytes(bytes, bn254_fq)
        }
    }
    fn to_le_bytes(self) -> [u8] {
        self.inner.to_le_bytes()
    }
}

impl Add for Bn254Fq { 
    fn add(self: Self, other: Bn254Fq) -> Bn254Fq {
        Bn254Fq {
            inner: self.inner.bigint_add(other.inner)
        }
    }
}
impl Sub for Bn254Fq { 
    fn sub(self: Self, other: Bn254Fq) -> Bn254Fq {
        Bn254Fq {
            inner: self.inner.bigint_sub(other.inner)
        }
    }
}
impl Mul for Bn254Fq { 
    fn mul(self: Self, other: Bn254Fq) -> Bn254Fq {
        Bn254Fq {
            inner: self.inner.bigint_mul(other.inner)
        }

    }
}
impl Div for Bn254Fq { 
    fn div(self: Self, other: Bn254Fq) -> Bn254Fq {
        Bn254Fq {
            inner: self.inner.bigint_div(other.inner)
        }
    }
}
impl Eq for Bn254Fq {
    fn eq(self: Self, other: Bn254Fq) -> bool {
        self.inner.check_32_bytes(other.inner)
    }
}

struct Secpr1Fq {
    inner: BigInt,
}

impl BigField for Secpr1Fq {
    fn from_le_bytes(bytes: [u8]) -> Secpr1Fq {
        Secpr1Fq {
            inner: BigInt::from_le_bytes(bytes, secpr1_fq)
        }
    }
    fn to_le_bytes(self) -> [u8] {
        self.inner.to_le_bytes()
    }
}

impl Add for Secpr1Fq { 
    fn add(self: Self, other: Secpr1Fq) -> Secpr1Fq {
        Secpr1Fq {
            inner: self.inner.bigint_add(other.inner)
        }
    }
}
impl Sub for Secpr1Fq { 
    fn sub(self: Self, other: Secpr1Fq) -> Secpr1Fq {
        Secpr1Fq {
            inner: self.inner.bigint_sub(other.inner)
        }
    }
}
impl Mul for Secpr1Fq { 
    fn mul(self: Self, other: Secpr1Fq) -> Secpr1Fq {
        Secpr1Fq {
            inner: self.inner.bigint_mul(other.inner)
        }

    }
}
impl Div for Secpr1Fq { 
    fn div(self: Self, other: Secpr1Fq) -> Secpr1Fq {
        Secpr1Fq {
            inner: self.inner.bigint_div(other.inner)
        }
    }
}
impl Eq for Secpr1Fq {
    fn eq(self: Self, other: Secpr1Fq) -> bool {
        self.inner.check_32_bytes(other.inner)
    }
}

struct Secpr1Fr {
    inner: BigInt,
}

impl BigField for Secpr1Fr {
    fn from_le_bytes(bytes: [u8]) -> Secpr1Fr {
        Secpr1Fr {
            inner: BigInt::from_le_bytes(bytes, secpr1_fr)
        }
    }
    fn to_le_bytes(self) -> [u8] {
        self.inner.to_le_bytes()
    }
}

impl Add for Secpr1Fr { 
    fn add(self: Self, other: Secpr1Fr) -> Secpr1Fr {
        Secpr1Fr {
            inner: self.inner.bigint_add(other.inner)
        }
    }
}
impl Sub for Secpr1Fr { 
    fn sub(self: Self, other: Secpr1Fr) -> Secpr1Fr {
        Secpr1Fr {
            inner: self.inner.bigint_sub(other.inner)
        }
    }
}
impl Mul for Secpr1Fr { 
    fn mul(self: Self, other: Secpr1Fr) -> Secpr1Fr {
        Secpr1Fr {
            inner: self.inner.bigint_mul(other.inner)
        }

    }
}
impl Div for Secpr1Fr { 
    fn div(self: Self, other: Secpr1Fr) -> Secpr1Fr {
        Secpr1Fr {
            inner: self.inner.bigint_div(other.inner)
        }
    }
}
impl Eq for Secpr1Fr {
    fn eq(self: Self, other: Secpr1Fr) -> bool {
        self.inner.check_32_bytes(other.inner)
    }
}
