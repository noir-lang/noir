global pow64 = 18446744073709551616;  //2^64;

struct U128 {
    lo: Field,
    hi: Field,
}

impl U128 {
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


    pub fn add(self: Self, b: U128) -> U128{
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