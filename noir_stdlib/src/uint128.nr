use crate::ops::{Add, Sub, Mul, Div, Rem, BitOr, BitAnd, BitXor, Shl, Shr};
use crate::cmp::{Eq, Ord, Ordering};

global pow64 : Field = 18446744073709551616; //2^64;

struct U128 {
    lo: Field,
    hi: Field,
}

impl U128 {

    pub fn from_u64s_le(lo: u64, hi: u64) -> U128 {
        // in order to handle multiplication, we need to represent the product of two u64 without overflow
        assert(crate::field::modulus_num_bits() as u32 > 128);
        U128 {
            lo: lo as Field,
            hi: hi as Field,
        }
    }

    pub fn from_u64s_be(hi: u64, lo: u64) -> U128 {
        U128::from_u64s_le(lo,hi)
    }

    pub fn from_le_bytes(bytes: [u8; 16]) -> U128 {
        let mut lo = 0;
        let mut base = 1;
        for i in 0..8 {
            lo += (bytes[i] as Field)*base;
            base *= 256;
        }
        let mut hi = 0;
        base = 1;
        for i in 8..16 {
            hi += (bytes[i] as Field)*base;
            base *= 256;
        }
        U128 {
            lo,
            hi,
        }
    }

    pub fn to_be_bytes(self: Self) -> [u8; 16] {
        let lo = self.lo.to_be_bytes(8);
        let hi = self.hi.to_be_bytes(8);
        let mut bytes = [0;16];
        for i in 0..8 {
            bytes[i] = hi[i];
            bytes[i+8] = lo[i];
        }
        bytes
    }

    pub fn to_le_bytes(self: Self) -> [u8; 16] {
        let lo = self.lo.to_le_bytes(8);
        let hi = self.hi.to_le_bytes(8);
        let mut bytes = [0;16];
        for i in 0..8 {
            bytes[i] = lo[i];
            bytes[i+8] = hi[i];
        }
        bytes
    }

    pub fn from_hex<N>(hex: str<N>) -> U128 {
        let N = N as u32;
        let bytes = hex.as_bytes();
        // string must starts with "0x"
        assert((bytes[0] == 48) & (bytes[1] == 120), "Invalid hexadecimal string");
        assert(N < 35, "Input does not fit into a U128");

        let mut lo = 0;
        let mut hi = 0;
        let mut base = 1; 
        if N <= 18 {
            for i in 0..N-2 {
                lo += U128::decode_ascii(bytes[N-i-1])*base;
                base = base*16;
            }
        } else {
            for i in 0..16 {
                lo += U128::decode_ascii(bytes[N-i-1])*base;
                base = base*16;
            }
            base = 1;
            for i in 17..N-1 {           
                hi += U128::decode_ascii(bytes[N-i])*base;
                base = base*16;
            }
        }
        U128 {
            lo: lo as Field,
            hi: hi as Field,
        }
    }

    fn decode_ascii(ascii: u8) -> Field {
        if ascii < 58 {
            ascii - 48
        } else {
            if ascii < 71 {
                ascii - 55
            } else {
                ascii - 87
            }
            
        } as Field
    }

    unconstrained fn unconstrained_div(self: Self, b: U128) -> (U128, U128) {
        if self < b {
            (U128::from_u64s_le(0, 0), self)
        } else {
            //TODO check if this can overflow?
            let (q,r) = self.unconstrained_div(b * U128::from_u64s_le(2,0));
            let q_mul_2 = q * U128::from_u64s_le(2,0);
            if r < b {
                (q_mul_2, r)
            } else {
                (q_mul_2 + U128::from_u64s_le(1,0), r - b)
            }

        }  
    }

    pub fn from_integer<T>(i: T) -> U128 {
        let f = crate::as_field(i);
        // Reject values which would overflow a u128
        f.assert_max_bit_size(128);
        let lo = f as u64 as Field;
        let hi = (f-lo) / pow64;
        U128 {
            lo,
            hi,
        }
    }

    pub fn to_integer<T>(self) -> T {
        crate::from_field(self.lo+self.hi*pow64)
    }

    fn wrapping_mul(self: Self, b: U128) -> U128 {
        let low = self.lo*b.lo;
        let lo = low as u64 as Field;
        let carry = (low - lo) / pow64;
        let high = if crate::field::modulus_num_bits() as u32 > 196 {
            (self.lo+self.hi)*(b.lo+b.hi) - low + carry
        } else {
            self.lo*b.hi + self.hi*b.lo + carry
        };
        let hi = high as u64 as Field;
        U128 {
            lo,
            hi,
        }
    }
}

impl Add for U128 {
    fn add(self: Self, b: U128) -> U128 {
        let low = self.lo + b.lo;
        let lo = low as u64 as Field;
        let carry = (low - lo) / pow64;  
        let high = self.hi + b.hi + carry;
        let hi = high as u64 as Field;
        assert(hi == high, "attempt to add with overflow");
        U128 {
            lo,
            hi,
        }
    }
}

impl Sub for U128 {
    fn sub(self: Self, b: U128) -> U128 {
        let low = pow64 + self.lo - b.lo;
        let lo = low as u64 as Field;
        let borrow = (low == lo) as Field;
        let high = self.hi - b.hi - borrow;
        let hi = high as u64 as Field;
        assert(hi == high, "attempt to subtract with overflow");
        U128 {
            lo,
            hi,
        }
    }
}

impl Mul for U128 {
    fn mul(self: Self, b: U128) -> U128 {
        assert(self.hi*b.hi == 0, "attempt to multiply with overflow");
        let low = self.lo*b.lo;
        let lo = low as u64 as Field;
        let carry = (low - lo) / pow64;
        let high = if crate::field::modulus_num_bits() as u32 > 196 {
            (self.lo+self.hi)*(b.lo+b.hi) - low + carry
        } else {
            self.lo*b.hi + self.hi*b.lo + carry
        };
        let hi = high as u64 as Field;
        assert(hi == high, "attempt to multiply with overflow");
        U128 {
            lo,
            hi,
        }
    }
}

impl Div for U128 {
    fn div(self: Self, b: U128) -> U128 {
        let (q,r) = self.unconstrained_div(b);
        let a = b * q + r;
        assert_eq(self, a);
        assert(r < b);
        q
    }
}

impl Rem for U128 {
    fn rem(self: Self, b: U128) -> U128 {
        let (q,r) = self.unconstrained_div(b);
        let a = b * q + r;
        assert_eq(self, a);
        assert(r < b);
        r
    }
}

impl Eq for U128 {
    fn eq(self: Self, b: U128) -> bool {
        (self.lo == b.lo) & (self.hi == b.hi)
    }
}

impl Ord for U128 {
    fn cmp(self, other: Self) -> Ordering {
        let hi_ordering = (self.hi as u64).cmp((other.hi as u64));
        let lo_ordering = (self.lo as u64).cmp((other.lo as u64));
        
        if hi_ordering == Ordering::equal() {
            lo_ordering
        } else {
            hi_ordering
        }
    }
}

impl BitOr for U128 { 
    fn bitor(self, other: U128) -> U128 {
        U128 {
            lo: ((self.lo as u64) | (other.lo as u64)) as Field,
            hi: ((self.hi as u64) | (other.hi as u64))as Field
        }
    }
}

impl BitAnd for U128 {
    fn bitand(self, other: U128) -> U128 { 
        U128 {
            lo: ((self.lo as u64) & (other.lo as u64)) as Field,
            hi: ((self.hi as u64) & (other.hi as u64)) as Field
        }
    }
}

impl BitXor for U128 {
    fn bitxor(self, other: U128) -> U128 { 
        U128 {
            lo: ((self.lo as u64) ^ (other.lo as u64)) as Field,
            hi: ((self.hi as u64) ^ (other.hi as u64)) as Field
        }
    }
}

impl Shl for U128 { 
    fn shl(self, other: U128) -> U128 { 
        assert(other < U128::from_u64s_le(128,0), "attempt to shift left with overflow");
        let exp_bits = other.lo.to_be_bits(7);

        let mut r: Field = 2;
        let mut y: Field = 1;
        for i in 1..8 {
            y = (exp_bits[7-i] as Field) * (r * y) + (1 - exp_bits[7-i] as Field) * y;
            r *= r;
        }
        self.wrapping_mul(U128::from_integer(y))
    } 
}

impl Shr for U128 { 
    fn shr(self, other: U128) -> U128 { 
        assert(other < U128::from_u64s_le(128,0), "attempt to shift right with overflow");
        let exp_bits = other.lo.to_be_bits(7);

        let mut r: Field = 2;
        let mut y: Field = 1;
        for i in 1..8 {
            y = (exp_bits[7-i] as Field) * (r * y) + (1 - exp_bits[7-i] as Field) * y;
            r *= r;
        }
        self / U128::from_integer(y)
    } 
}
