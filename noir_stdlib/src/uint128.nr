use crate::cmp::{Eq, Ord, Ordering};
use crate::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Not, Rem, Shl, Shr, Sub};
use super::convert::AsPrimitive;

global pow64: Field = 18446744073709551616; //2^64;
global pow63: Field = 9223372036854775808; // 2^63;
pub struct U128 {
    pub(crate) lo: Field,
    pub(crate) hi: Field,
}

impl U128 {

    pub fn from_u64s_le(lo: u64, hi: u64) -> U128 {
        // in order to handle multiplication, we need to represent the product of two u64 without overflow
        assert(crate::field::modulus_num_bits() as u32 > 128);
        U128 { lo: lo as Field, hi: hi as Field }
    }

    pub fn from_u64s_be(hi: u64, lo: u64) -> U128 {
        U128::from_u64s_le(lo, hi)
    }

    pub fn zero() -> U128 {
        U128 { lo: 0, hi: 0 }
    }

    pub fn one() -> U128 {
        U128 { lo: 1, hi: 0 }
    }
    pub fn from_le_bytes(bytes: [u8; 16]) -> U128 {
        let mut lo = 0;
        let mut base = 1;
        for i in 0..8 {
            lo += (bytes[i] as Field) * base;
            base *= 256;
        }
        let mut hi = 0;
        base = 1;
        for i in 8..16 {
            hi += (bytes[i] as Field) * base;
            base *= 256;
        }
        U128 { lo, hi }
    }

    pub fn to_be_bytes(self: Self) -> [u8; 16] {
        let lo: [u8; 8] = self.lo.to_be_bytes();
        let hi: [u8; 8] = self.hi.to_be_bytes();
        let mut bytes = [0; 16];
        for i in 0..8 {
            bytes[i] = hi[i];
            bytes[i + 8] = lo[i];
        }
        bytes
    }

    pub fn to_le_bytes(self: Self) -> [u8; 16] {
        let lo: [u8; 8] = self.lo.to_le_bytes();
        let hi: [u8; 8] = self.hi.to_le_bytes();
        let mut bytes = [0; 16];
        for i in 0..8 {
            bytes[i] = lo[i];
            bytes[i + 8] = hi[i];
        }
        bytes
    }

    pub fn from_hex<let N: u32>(hex: str<N>) -> U128 {
        let N = N as u32;
        let bytes = hex.as_bytes();
        // string must starts with "0x"
        assert((bytes[0] == 48) & (bytes[1] == 120), "Invalid hexadecimal string");
        assert(N < 35, "Input does not fit into a U128");

        let mut lo = 0;
        let mut hi = 0;
        let mut base = 1;
        if N <= 18 {
            for i in 0..N - 2 {
                lo += U128::decode_ascii(bytes[N - i - 1]) * base;
                base = base * 16;
            }
        } else {
            for i in 0..16 {
                lo += U128::decode_ascii(bytes[N - i - 1]) * base;
                base = base * 16;
            }
            base = 1;
            for i in 17..N - 1 {
                hi += U128::decode_ascii(bytes[N - i]) * base;
                base = base * 16;
            }
        }
        U128 { lo: lo as Field, hi: hi as Field }
    }

    unconstrained fn uconstrained_check_is_upper_ascii(ascii: u8) -> bool {
        ((ascii >= 65) & (ascii <= 90)) // Between 'A' and 'Z'
    }

    pub(crate) fn decode_ascii(ascii: u8) -> Field {
        (
            if ascii < 58 {
                ascii - 48
            } else {
                let ascii = ascii
                    + 32
                        * (
                            unsafe {
                                /*@safety : optionally adds 32 and then check (below) the result is in 'a..f' range
                                    This enforces that the input is in either 'A..F' or 'a..f' range
                                */
                                U128::uconstrained_check_is_upper_ascii(ascii) as u8
                            }
                        );
                assert(ascii >= 97); // enforce >= 'a'
                assert(ascii <= 102); // enforce <= 'f'
                ascii - 87
            }
        ) as Field
    }

    // TODO: Replace with a faster version.
    // A circuit that uses this function can be slow to compute
    // (we're doing up to 127 calls to compute the quotient)
    unconstrained fn unconstrained_div(self: Self, b: U128) -> (U128, U128) {
        if b == U128::zero() {
            // Return 0,0 to avoid eternal loop
            (U128::zero(), U128::zero())
        } else if self < b {
            (U128::zero(), self)
        } else if self == b {
            (U128::one(), U128::zero())
        } else {
            let (q, r) = if b.hi as u64 >= pow63 as u64 {
                // The result of multiplication by 2 would overflow
                (U128::zero(), self)
            } else {
                self.unconstrained_div(b * U128::from_u64s_le(2, 0))
            };
            let q_mul_2 = q * U128::from_u64s_le(2, 0);
            if r < b {
                (q_mul_2, r)
            } else {
                (q_mul_2 + U128::one(), r - b)
            }
        }
    }

    pub fn from_integer<T>(i: T) -> U128
    where
        T: AsPrimitive<Field>,
    {
        let f = i.as_();
        // Reject values which would overflow a u128
        f.assert_max_bit_size::<128>();
        let lo = f as u64 as Field;
        let hi = (f - lo) / pow64;
        U128 { lo, hi }
    }

    pub fn to_integer<T>(self) -> T
    where
        Field: AsPrimitive<T>,
    {
        AsPrimitive::as_(self.lo + self.hi * pow64)
    }

    fn wrapping_mul(self: Self, b: U128) -> U128 {
        let low = self.lo * b.lo;
        let lo = low as u64 as Field;
        let carry = (low - lo) / pow64;
        let high = self.lo * b.hi + self.hi * b.lo + carry;
        let hi = high as u64 as Field;
        U128 { lo, hi }
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
        U128 { lo, hi }
    }
}

impl Sub for U128 {
    fn sub(self: Self, b: U128) -> U128 {
        let low = pow64 + self.lo - b.lo;
        let lo = low as u64 as Field;
        let borrow = (low == lo) as Field;
        let high = self.hi - b.hi - borrow;
        let hi = high as u64 as Field;
        assert(hi == high, "attempt to subtract with underflow");
        U128 { lo, hi }
    }
}

impl Mul for U128 {
    fn mul(self: Self, b: U128) -> U128 {
        assert(self.hi * b.hi == 0, "attempt to multiply with overflow");
        let low = self.lo * b.lo;
        let lo = low as u64 as Field;
        let carry = (low - lo) / pow64;
        let high = if crate::field::modulus_num_bits() as u32 > 196 {
            (self.lo + self.hi) * (b.lo + b.hi) - low + carry
        } else {
            self.lo * b.hi + self.hi * b.lo + carry
        };
        let hi = high as u64 as Field;
        assert(hi == high, "attempt to multiply with overflow");
        U128 { lo, hi }
    }
}

impl Div for U128 {
    fn div(self: Self, b: U128) -> U128 {
        unsafe {
            /*@safety : euclidian division is asserted to be correct: assert(a == b * q + r); and assert(r < b);
                Furthermore, U128 addition and multiplication ensures that b * q + r does not overflow
            */
            let (q, r) = self.unconstrained_div(b);
            let a = b * q + r;
            assert_eq(self, a);
            assert(r < b);
            q
        }
    }
}

impl Rem for U128 {
    fn rem(self: Self, b: U128) -> U128 {
        unsafe {
            //@safety : cf div() above
            let (q, r) = self.unconstrained_div(b);
            let a = b * q + r;
            assert_eq(self, a);
            assert(r < b);

            r
        }
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

impl Not for U128 {
    fn not(self) -> U128 {
        U128 { lo: (!(self.lo as u64)) as Field, hi: (!(self.hi as u64)) as Field }
    }
}

impl BitOr for U128 {
    fn bitor(self, other: U128) -> U128 {
        U128 {
            lo: ((self.lo as u64) | (other.lo as u64)) as Field,
            hi: ((self.hi as u64) | (other.hi as u64)) as Field,
        }
    }
}

impl BitAnd for U128 {
    fn bitand(self, other: U128) -> U128 {
        U128 {
            lo: ((self.lo as u64) & (other.lo as u64)) as Field,
            hi: ((self.hi as u64) & (other.hi as u64)) as Field,
        }
    }
}

impl BitXor for U128 {
    fn bitxor(self, other: U128) -> U128 {
        U128 {
            lo: ((self.lo as u64) ^ (other.lo as u64)) as Field,
            hi: ((self.hi as u64) ^ (other.hi as u64)) as Field,
        }
    }
}

impl Shl for U128 {
    fn shl(self, other: u8) -> U128 {
        assert(other < 128, "attempt to shift left with overflow");
        let exp_bits: [u1; 7] = (other as Field).to_be_bits();

        let mut r: Field = 2;
        let mut y: Field = 1;
        for i in 1..8 {
            let bit = exp_bits[7 - i] as Field;
            y = bit * (r * y) + (1 - bit) * y;
            r *= r;
        }
        self.wrapping_mul(U128::from_integer(y))
    }
}

impl Shr for U128 {
    fn shr(self, other: u8) -> U128 {
        assert(other < 128, "attempt to shift right with overflow");
        let exp_bits: [u1; 7] = (other as Field).to_be_bits();

        let mut r: Field = 2;
        let mut y: Field = 1;
        for i in 1..8 {
            let bit = exp_bits[7 - i] as Field;
            y = bit * (r * y) + (1 - bit) * y;
            r *= r;
        }
        self / U128::from_integer(y)
    }
}

mod tests {
    use crate::uint128::{pow63, pow64, U128};

    #[test]
    fn test_not(lo: u64, hi: u64) {
        let num = U128::from_u64s_le(lo, hi);
        let not_num = num.not();

        assert_eq(not_num.hi, (hi.not() as Field));
        assert_eq(not_num.lo, (lo.not() as Field));

        let not_not_num = not_num.not();
        assert_eq(num, not_not_num);
    }
    #[test]
    fn test_construction() {
        // Check little-endian u64 is inversed with big-endian u64 construction
        let a = U128::from_u64s_le(2, 1);
        let b = U128::from_u64s_be(1, 2);
        assert_eq(a, b);
        // Check byte construction is equivalent
        let c = U128::from_le_bytes([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
        let d = U128::from_u64s_le(0x0706050403020100, 0x0f0e0d0c0b0a0908);
        assert_eq(c, d);
    }
    #[test]
    fn test_byte_decomposition() {
        let a = U128::from_u64s_le(0x0706050403020100, 0x0f0e0d0c0b0a0908);
        // Get big-endian and little-endian byte decompostions
        let le_bytes_a = a.to_le_bytes();
        let be_bytes_a = a.to_be_bytes();

        // Check equivalence
        for i in 0..16 {
            assert_eq(le_bytes_a[i], be_bytes_a[15 - i]);
        }
        // Reconstruct U128 from byte decomposition
        let b = U128::from_le_bytes(le_bytes_a);
        // Check that it's the same element
        assert_eq(a, b);
    }
    #[test]
    fn test_hex_constuction() {
        let a = U128::from_u64s_le(0x1, 0x2);
        let b = U128::from_hex("0x20000000000000001");
        assert_eq(a, b);

        let c = U128::from_hex("0xffffffffffffffffffffffffffffffff");
        let d = U128::from_u64s_le(0xffffffffffffffff, 0xffffffffffffffff);
        assert_eq(c, d);

        let e = U128::from_hex("0x00000000000000000000000000000000");
        let f = U128::from_u64s_le(0, 0);
        assert_eq(e, f);
    }

    // Ascii decode tests

    #[test]
    fn test_ascii_decode_correct_range() {
        // '0'..'9' range
        for i in 0..10 {
            let decoded = U128::decode_ascii(48 + i);
            assert_eq(decoded, i as Field);
        }
        // 'A'..'F' range
        for i in 0..6 {
            let decoded = U128::decode_ascii(65 + i);
            assert_eq(decoded, (i + 10) as Field);
        }
        // 'a'..'f' range
        for i in 0..6 {
            let decoded = U128::decode_ascii(97 + i);
            assert_eq(decoded, (i + 10) as Field);
        }
    }

    #[test(should_fail)]
    fn test_ascii_decode_range_less_than_48_fails_0() {
        crate::println(U128::decode_ascii(0));
    }
    #[test(should_fail)]
    fn test_ascii_decode_range_less_than_48_fails_1() {
        crate::println(U128::decode_ascii(47));
    }

    #[test(should_fail)]
    fn test_ascii_decode_range_58_64_fails_0() {
        let _ = U128::decode_ascii(58);
    }
    #[test(should_fail)]
    fn test_ascii_decode_range_58_64_fails_1() {
        let _ = U128::decode_ascii(64);
    }
    #[test(should_fail)]
    fn test_ascii_decode_range_71_96_fails_0() {
        let _ = U128::decode_ascii(71);
    }
    #[test(should_fail)]
    fn test_ascii_decode_range_71_96_fails_1() {
        let _ = U128::decode_ascii(96);
    }
    #[test(should_fail)]
    fn test_ascii_decode_range_greater_than_102_fails() {
        let _ = U128::decode_ascii(103);
    }

    #[test(should_fail)]
    fn test_ascii_decode_regression() {
        // This code will actually fail because of ascii_decode,
        // but in the past it was possible to create a value > (1<<128)
        let a = U128::from_hex("0x~fffffffffffffffffffffffffffffff");
        let b: Field = a.to_integer();
        let c: [u8; 17] = b.to_le_bytes();
        assert(c[16] != 0);
    }

    #[test]
    fn test_unconstrained_div() {
        // Test the potential overflow case
        let a = U128::from_u64s_le(0x0, 0xffffffffffffffff);
        let b = U128::from_u64s_le(0x0, 0xfffffffffffffffe);
        let c = U128::one();
        let d = U128::from_u64s_le(0x0, 0x1);
        unsafe {
            //@safety: testing context
            let (q, r) = a.unconstrained_div(b);
            assert_eq(q, c);
            assert_eq(r, d);
        }

        let a = U128::from_u64s_le(2, 0);
        let b = U128::one();
        // Check the case where a is a multiple of b
        unsafe {
            //@safety: testing context
            let (c, d) = a.unconstrained_div(b);
            assert_eq((c, d), (a, U128::zero()));
        }

        // Check where b is a multiple of a
        unsafe {
            //@safety: testing context
            let (c, d) = b.unconstrained_div(a);
            assert_eq((c, d), (U128::zero(), b));
        }

        // Dividing by zero returns 0,0
        let a = U128::from_u64s_le(0x1, 0x0);
        let b = U128::zero();
        unsafe {
            //@safety: testing context
            let (c, d) = a.unconstrained_div(b);
            assert_eq((c, d), (U128::zero(), U128::zero()));
        }
        // Dividing 1<<127 by 1<<127 (special case)
        let a = U128::from_u64s_le(0x0, pow63 as u64);
        let b = U128::from_u64s_le(0x0, pow63 as u64);
        unsafe {
            //@safety: testing context
            let (c, d) = a.unconstrained_div(b);
            assert_eq((c, d), (U128::one(), U128::zero()));
        }
    }

    #[test]
    fn integer_conversions() {
        // Maximum
        let start: Field = 0xffffffffffffffffffffffffffffffff;
        let a = U128::from_integer(start);
        let end = a.to_integer();
        assert_eq(start, end);

        // Minimum
        let start: Field = 0x0;
        let a = U128::from_integer(start);
        let end = a.to_integer();
        assert_eq(start, end);

        // Low limb
        let start: Field = 0xffffffffffffffff;
        let a = U128::from_integer(start);
        let end = a.to_integer();
        assert_eq(start, end);

        // High limb
        let start: Field = 0xffffffffffffffff0000000000000000;
        let a = U128::from_integer(start);
        let end = a.to_integer();
        assert_eq(start, end);
    }

    #[test]
    fn integer_conversions_fuzz(lo: u64, hi: u64) {
        let start: Field = (lo as Field) + pow64 * (hi as Field);
        let a = U128::from_integer(start);
        let end = a.to_integer();
        assert_eq(start, end);
    }

    #[test]
    fn test_wrapping_mul() {
        // 1*0==0
        assert_eq(U128::zero(), U128::zero().wrapping_mul(U128::one()));

        // 0*1==0
        assert_eq(U128::zero(), U128::one().wrapping_mul(U128::zero()));

        // 1*1==1
        assert_eq(U128::one(), U128::one().wrapping_mul(U128::one()));

        // 0 * ( 1 << 64 ) ==  0
        assert_eq(U128::zero(), U128::zero().wrapping_mul(U128::from_u64s_le(0, 1)));

        // ( 1 << 64 ) * 0 == 0
        assert_eq(U128::zero(), U128::from_u64s_le(0, 1).wrapping_mul(U128::zero()));

        // 1 * ( 1 << 64 ) == 1 << 64
        assert_eq(U128::from_u64s_le(0, 1), U128::from_u64s_le(0, 1).wrapping_mul(U128::one()));

        // ( 1 << 64 ) * 1 == 1 << 64
        assert_eq(U128::from_u64s_le(0, 1), U128::one().wrapping_mul(U128::from_u64s_le(0, 1)));

        // ( 1 << 64 ) * ( 1 << 64 ) == 1 << 64
        assert_eq(U128::zero(), U128::from_u64s_le(0, 1).wrapping_mul(U128::from_u64s_le(0, 1)));
        // -1 * -1 == 1
        assert_eq(
            U128::one(),
            U128::from_u64s_le(0xffffffffffffffff, 0xffffffffffffffff).wrapping_mul(
                U128::from_u64s_le(0xffffffffffffffff, 0xffffffffffffffff),
            ),
        );
    }
}
