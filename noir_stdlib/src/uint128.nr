use crate::ops::{Add, Sub, Mul, Div, Eq};

global pow64 : Field = 18446744073709551616; //2^64;

struct U128 {
    lo: Field,
    hi: Field,
}

impl U128 {

    pub fn new(lo: u64, hi: u64) -> U128 {
        // in order to handle multiplication, we need to represent the product of two u64 without overflow
        assert(crate::field::modulus_num_bits() as u32 > 128);
        U128 {
            lo: lo as Field,
            hi: hi as Field,
        }
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
        assert(N < 35, "Input does not fit into a u128");

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
   
    pub fn less_than(self: Self, b: U128) -> bool {
        if self.hi == b.hi {
            self.lo as u64 < b.lo as u64
        } else {
            self.hi as u64 < b.hi as u64
        }
    }

    unconstrained fn unconstrained_div(self: Self, b: U128) -> (U128, U128) {
        if self.less_than(b) {
            (U128::new(0, 0), self)
        } else {
            //TODO check if this can overflow?
            let (q,r) = self.unconstrained_div(b.mul(U128::new(2,0)));
            if r.less_than(b) {
                (q.mul(U128::new(2,0)), r)
            } else {
                (q.mul(U128::new(2,0)).add(U128::new(1,0)), r.sub(b))
            }

        }  
    }

    pub fn from_integer<T>(i: T) -> U128 {
        let f = crate::as_field(i);
        let lo = f as u64 as Field;
        let hi = (f-lo ) / pow64;
        U128 {
            lo,
            hi,
        }
    }

    pub fn to_integer<T>(self) -> T {
        crate::from_field(self.lo+self.hi*pow64)
    }
}

impl Add for U128 {
    pub fn add(self: Self, b: U128) -> U128 {
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
    pub fn sub(self: Self, b: U128) -> U128 {
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
    pub fn mul(self: Self, b: U128) -> U128 {
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
    pub fn div(self: Self, b: U128) -> U128 {
        let (q,r) = self.unconstrained_div(b);
        let a = b.mul(q).add(r);
        assert(self.lo == a.lo);
        assert(self.hi == a.hi);
        assert(r.less_than(b));
        q
    }
}

impl Eq for U128 {
    pub fn eq(self: Self, b: U128) -> bool {
        (self.lo == b.lo) & (self.hi == b.hi)
    }
}
