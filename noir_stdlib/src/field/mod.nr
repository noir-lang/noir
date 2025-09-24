pub mod bn254;
use crate::{runtime::is_unconstrained, static_assert};
use bn254::lt as bn254_lt;

impl Field {
    /// Asserts that `self` can be represented in `bit_size` bits.
    ///
    /// # Failures
    /// Causes a constraint failure for `Field` values exceeding `2^{bit_size}`.
    // docs:start:assert_max_bit_size
    pub fn assert_max_bit_size<let BIT_SIZE: u32>(self) {
        // docs:end:assert_max_bit_size
        static_assert(
            BIT_SIZE < modulus_num_bits() as u32,
            "BIT_SIZE must be less than modulus_num_bits",
        );
        __assert_max_bit_size(self, BIT_SIZE);
    }

    /// Decomposes `self` into its little endian bit decomposition as a `[u1; N]` array.
    /// This slice will be zero padded should not all bits be necessary to represent `self`.
    ///
    /// # Failures
    /// Causes a constraint failure for `Field` values exceeding `2^N` as the resulting slice will not
    /// be able to represent the original `Field`.
    ///
    /// # Safety
    /// The bit decomposition returned is canonical and is guaranteed to not overflow the modulus.
    // docs:start:to_le_bits
    pub fn to_le_bits<let N: u32>(self: Self) -> [u1; N] {
        // docs:end:to_le_bits
        let bits = __to_le_bits(self);

        if !is_unconstrained() {
            // Ensure that the byte decomposition does not overflow the modulus
            let p = modulus_le_bits();
            assert(bits.len() <= p.len());
            let mut ok = bits.len() != p.len();
            for i in 0..N {
                if !ok {
                    if (bits[N - 1 - i] != p[N - 1 - i]) {
                        assert(p[N - 1 - i] == 1);
                        ok = true;
                    }
                }
            }
            assert(ok);
        }
        bits
    }

    /// Decomposes `self` into its big endian bit decomposition as a `[u1; N]` array.
    /// This array will be zero padded should not all bits be necessary to represent `self`.
    ///
    /// # Failures
    /// Causes a constraint failure for `Field` values exceeding `2^N` as the resulting slice will not
    /// be able to represent the original `Field`.
    ///
    /// # Safety
    /// The bit decomposition returned is canonical and is guaranteed to not overflow the modulus.
    // docs:start:to_be_bits
    pub fn to_be_bits<let N: u32>(self: Self) -> [u1; N] {
        // docs:end:to_be_bits
        let bits = __to_be_bits(self);

        if !is_unconstrained() {
            // Ensure that the decomposition does not overflow the modulus
            let p = modulus_be_bits();
            assert(bits.len() <= p.len());
            let mut ok = bits.len() != p.len();
            for i in 0..N {
                if !ok {
                    if (bits[i] != p[i]) {
                        assert(p[i] == 1);
                        ok = true;
                    }
                }
            }
            assert(ok);
        }
        bits
    }

    /// Decomposes `self` into its little endian byte decomposition as a `[u8;N]` array
    /// This array will be zero padded should not all bytes be necessary to represent `self`.
    ///
    /// # Failures
    ///  The length N of the array must be big enough to contain all the bytes of the 'self',
    ///  and no more than the number of bytes required to represent the field modulus
    ///
    /// # Safety
    /// The result is ensured to be the canonical decomposition of the field element
    // docs:start:to_le_bytes
    pub fn to_le_bytes<let N: u32>(self: Self) -> [u8; N] {
        // docs:end:to_le_bytes
        static_assert(
            N <= modulus_le_bytes().len(),
            "N must be less than or equal to modulus_le_bytes().len()",
        );
        // Compute the byte decomposition
        let bytes = self.to_le_radix(256);

        if !is_unconstrained() {
            // Ensure that the byte decomposition does not overflow the modulus
            let p = modulus_le_bytes();
            assert(bytes.len() <= p.len());
            let mut ok = bytes.len() != p.len();
            for i in 0..N {
                if !ok {
                    if (bytes[N - 1 - i] != p[N - 1 - i]) {
                        assert(bytes[N - 1 - i] < p[N - 1 - i]);
                        ok = true;
                    }
                }
            }
            assert(ok);
        }
        bytes
    }

    /// Decomposes `self` into its big endian byte decomposition as a `[u8;N]` array of length required to represent the field modulus
    /// This array will be zero padded should not all bytes be necessary to represent `self`.
    ///
    /// # Failures
    ///  The length N of the array must be big enough to contain all the bytes of the 'self',
    ///  and no more than the number of bytes required to represent the field modulus
    ///
    /// # Safety
    /// The result is ensured to be the canonical decomposition of the field element
    // docs:start:to_be_bytes
    pub fn to_be_bytes<let N: u32>(self: Self) -> [u8; N] {
        // docs:end:to_be_bytes
        static_assert(
            N <= modulus_le_bytes().len(),
            "N must be less than or equal to modulus_le_bytes().len()",
        );
        // Compute the byte decomposition
        let bytes = self.to_be_radix(256);

        if !is_unconstrained() {
            // Ensure that the byte decomposition does not overflow the modulus
            let p = modulus_be_bytes();
            assert(bytes.len() <= p.len());
            let mut ok = bytes.len() != p.len();
            for i in 0..N {
                if !ok {
                    if (bytes[i] != p[i]) {
                        assert(bytes[i] < p[i]);
                        ok = true;
                    }
                }
            }
            assert(ok);
        }
        bytes
    }

    fn to_le_radix<let N: u32>(self: Self, radix: u32) -> [u8; N] {
        // Brillig does not need an immediate radix
        if !crate::runtime::is_unconstrained() {
            static_assert(1 < radix, "radix must be greater than 1");
            static_assert(radix <= 256, "radix must be less than or equal to 256");
            static_assert(radix & (radix - 1) == 0, "radix must be a power of 2");
        }
        __to_le_radix(self, radix)
    }

    fn to_be_radix<let N: u32>(self: Self, radix: u32) -> [u8; N] {
        // Brillig does not need an immediate radix
        if !crate::runtime::is_unconstrained() {
            static_assert(1 < radix, "radix must be greater than 1");
            static_assert(radix <= 256, "radix must be less than or equal to 256");
            static_assert(radix & (radix - 1) == 0, "radix must be a power of 2");
        }
        __to_be_radix(self, radix)
    }

    // Returns self to the power of the given exponent value.
    // Caution: we assume the exponent fits into 32 bits
    // using a bigger bit size impacts negatively the performance and should be done only if the exponent does not fit in 32 bits
    pub fn pow_32(self, exponent: Field) -> Field {
        let mut r: Field = 1;
        let b: [u1; 32] = exponent.to_le_bits();

        for i in 1..33 {
            r *= r;
            r = (b[32 - i] as Field) * (r * self) + (1 - b[32 - i] as Field) * r;
        }
        r
    }

    // Parity of (prime) Field element, i.e. sgn0(x mod p) = 0 if x `elem` {0, ..., p-1} is even, otherwise sgn0(x mod p) = 1.
    pub fn sgn0(self) -> u1 {
        self as u1
    }

    pub fn lt(self, another: Field) -> bool {
        if crate::compat::is_bn254() {
            bn254_lt(self, another)
        } else {
            lt_fallback(self, another)
        }
    }

    /// Convert a little endian byte array to a field element.
    /// If the provided byte array overflows the field modulus then the Field will silently wrap around.
    pub fn from_le_bytes<let N: u32>(bytes: [u8; N]) -> Field {
        static_assert(
            N <= modulus_le_bytes().len(),
            "N must be less than or equal to modulus_le_bytes().len()",
        );
        let mut v = 1;
        let mut result = 0;

        for i in 0..N {
            result += (bytes[i] as Field) * v;
            v = v * 256;
        }
        result
    }

    /// Convert a big endian byte array to a field element.
    /// If the provided byte array overflows the field modulus then the Field will silently wrap around.
    pub fn from_be_bytes<let N: u32>(bytes: [u8; N]) -> Field {
        let mut v = 1;
        let mut result = 0;

        for i in 0..N {
            result += (bytes[N - 1 - i] as Field) * v;
            v = v * 256;
        }
        result
    }
}

#[builtin(apply_range_constraint)]
fn __assert_max_bit_size(value: Field, bit_size: u32) {}

// `_radix` must be less than 256
#[builtin(to_le_radix)]
fn __to_le_radix<let N: u32>(value: Field, radix: u32) -> [u8; N] {}

// `_radix` must be less than 256
#[builtin(to_be_radix)]
fn __to_be_radix<let N: u32>(value: Field, radix: u32) -> [u8; N] {}

/// Decomposes `self` into its little endian bit decomposition as a `[u1; N]` array.
/// This slice will be zero padded should not all bits be necessary to represent `self`.
///
/// # Failures
/// Causes a constraint failure for `Field` values exceeding `2^N` as the resulting slice will not
/// be able to represent the original `Field`.
///
/// # Safety
/// Values of `N` equal to or greater than the number of bits necessary to represent the `Field` modulus
/// (e.g. 254 for the BN254 field) allow for multiple bit decompositions. This is due to how the `Field` will
/// wrap around due to overflow when verifying the decomposition.
#[builtin(to_le_bits)]
fn __to_le_bits<let N: u32>(value: Field) -> [u1; N] {}

/// Decomposes `self` into its big endian bit decomposition as a `[u1; N]` array.
/// This array will be zero padded should not all bits be necessary to represent `self`.
///
/// # Failures
/// Causes a constraint failure for `Field` values exceeding `2^N` as the resulting slice will not
/// be able to represent the original `Field`.
///
/// # Safety
/// Values of `N` equal to or greater than the number of bits necessary to represent the `Field` modulus
/// (e.g. 254 for the BN254 field) allow for multiple bit decompositions. This is due to how the `Field` will
/// wrap around due to overflow when verifying the decomposition.
#[builtin(to_be_bits)]
fn __to_be_bits<let N: u32>(value: Field) -> [u1; N] {}

#[builtin(modulus_num_bits)]
pub comptime fn modulus_num_bits() -> u64 {}

#[builtin(modulus_be_bits)]
pub comptime fn modulus_be_bits() -> [u1] {}

#[builtin(modulus_le_bits)]
pub comptime fn modulus_le_bits() -> [u1] {}

#[builtin(modulus_be_bytes)]
pub comptime fn modulus_be_bytes() -> [u8] {}

#[builtin(modulus_le_bytes)]
pub comptime fn modulus_le_bytes() -> [u8] {}

/// An unconstrained only built in to efficiently compare fields.
#[builtin(field_less_than)]
unconstrained fn __field_less_than(x: Field, y: Field) -> bool {}

pub(crate) unconstrained fn field_less_than(x: Field, y: Field) -> bool {
    __field_less_than(x, y)
}

// Convert a 32 byte array to a field element by modding
pub fn bytes32_to_field(bytes32: [u8; 32]) -> Field {
    // Convert it to a field element
    let mut v = 1;
    let mut high = 0 as Field;
    let mut low = 0 as Field;

    for i in 0..16 {
        high = high + (bytes32[15 - i] as Field) * v;
        low = low + (bytes32[16 + 15 - i] as Field) * v;
        v = v * 256;
    }
    // Abuse that a % p + b % p = (a + b) % p and that low < p
    low + high * v
}

fn lt_fallback(x: Field, y: Field) -> bool {
    if is_unconstrained() {
        // Safety: unconstrained context
        unsafe {
            field_less_than(x, y)
        }
    } else {
        let x_bytes: [u8; 32] = x.to_le_bytes();
        let y_bytes: [u8; 32] = y.to_le_bytes();
        let mut x_is_lt = false;
        let mut done = false;
        for i in 0..32 {
            if (!done) {
                let x_byte = x_bytes[32 - 1 - i] as u8;
                let y_byte = y_bytes[32 - 1 - i] as u8;
                let bytes_match = x_byte == y_byte;
                if !bytes_match {
                    x_is_lt = x_byte < y_byte;
                    done = true;
                }
            }
        }
        x_is_lt
    }
}

mod tests {
    use crate::{panic::panic, runtime, static_assert};
    use super::{
        field_less_than, modulus_be_bits, modulus_be_bytes, modulus_le_bits, modulus_le_bytes,
    };

    #[test]
    // docs:start:to_be_bits_example
    fn test_to_be_bits() {
        let field = 2;
        let bits: [u1; 8] = field.to_be_bits();
        assert_eq(bits, [0, 0, 0, 0, 0, 0, 1, 0]);
    }
    // docs:end:to_be_bits_example

    #[test]
    // docs:start:to_le_bits_example
    fn test_to_le_bits() {
        let field = 2;
        let bits: [u1; 8] = field.to_le_bits();
        assert_eq(bits, [0, 1, 0, 0, 0, 0, 0, 0]);
    }
    // docs:end:to_le_bits_example

    #[test]
    // docs:start:to_be_bytes_example
    fn test_to_be_bytes() {
        let field = 2;
        let bytes: [u8; 8] = field.to_be_bytes();
        assert_eq(bytes, [0, 0, 0, 0, 0, 0, 0, 2]);
        assert_eq(Field::from_be_bytes::<8>(bytes), field);
    }
    // docs:end:to_be_bytes_example

    #[test]
    // docs:start:to_le_bytes_example
    fn test_to_le_bytes() {
        let field = 2;
        let bytes: [u8; 8] = field.to_le_bytes();
        assert_eq(bytes, [2, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq(Field::from_le_bytes::<8>(bytes), field);
    }
    // docs:end:to_le_bytes_example

    #[test]
    // docs:start:to_be_radix_example
    fn test_to_be_radix() {
        // 259, in base 256, big endian, is [1, 3].
        // i.e. 3 * 256^0 + 1 * 256^1
        let field = 259;

        // The radix (in this example, 256) must be a power of 2.
        // The length of the returned byte array can be specified to be
        // >= the amount of space needed.
        let bytes: [u8; 8] = field.to_be_radix(256);
        assert_eq(bytes, [0, 0, 0, 0, 0, 0, 1, 3]);
        assert_eq(Field::from_be_bytes::<8>(bytes), field);
    }
    // docs:end:to_be_radix_example

    #[test]
    // docs:start:to_le_radix_example
    fn test_to_le_radix() {
        // 259, in base 256, little endian, is [3, 1].
        // i.e. 3 * 256^0 + 1 * 256^1
        let field = 259;

        // The radix (in this example, 256) must be a power of 2.
        // The length of the returned byte array can be specified to be
        // >= the amount of space needed.
        let bytes: [u8; 8] = field.to_le_radix(256);
        assert_eq(bytes, [3, 1, 0, 0, 0, 0, 0, 0]);
        assert_eq(Field::from_le_bytes::<8>(bytes), field);
    }
    // docs:end:to_le_radix_example

    #[test(should_fail_with = "radix must be greater than 1")]
    fn test_to_le_radix_1() {
        // this test should only fail in constrained mode
        if !runtime::is_unconstrained() {
            let field = 2;
            let _: [u8; 8] = field.to_le_radix(1);
        } else {
            panic(f"radix must be greater than 1");
        }
    }

    // Updated test to account for Brillig restriction that radix must be greater than 2
    #[test(should_fail_with = "radix must be greater than 1")]
    fn test_to_le_radix_brillig_1() {
        // this test should only fail in constrained mode
        if !runtime::is_unconstrained() {
            let field = 1;
            let _: [u8; 8] = field.to_le_radix(1);
        } else {
            panic(f"radix must be greater than 1");
        }
    }

    #[test(should_fail_with = "radix must be a power of 2")]
    fn test_to_le_radix_3() {
        // this test should only fail in constrained mode
        if !runtime::is_unconstrained() {
            let field = 2;
            let _: [u8; 8] = field.to_le_radix(3);
        } else {
            panic(f"radix must be a power of 2");
        }
    }

    #[test]
    fn test_to_le_radix_brillig_3() {
        // this test should only fail in constrained mode
        if runtime::is_unconstrained() {
            let field = 1;
            let out: [u8; 8] = field.to_le_radix(3);
            let mut expected = [0; 8];
            expected[0] = 1;
            assert(out == expected, "unexpected result");
        }
    }

    #[test(should_fail_with = "radix must be less than or equal to 256")]
    fn test_to_le_radix_512() {
        // this test should only fail in constrained mode
        if !runtime::is_unconstrained() {
            let field = 2;
            let _: [u8; 8] = field.to_le_radix(512);
        } else {
            panic(f"radix must be less than or equal to 256")
        }
    }

    #[test(should_fail_with = "Field failed to decompose into specified 16 limbs")]
    unconstrained fn not_enough_limbs_brillig() {
        let _: [u8; 16] = 0x100000000000000000000000000000000.to_le_bytes();
    }

    #[test(should_fail_with = "Field failed to decompose into specified 16 limbs")]
    fn not_enough_limbs() {
        let _: [u8; 16] = 0x100000000000000000000000000000000.to_le_bytes();
    }

    #[test]
    unconstrained fn test_field_less_than() {
        assert(field_less_than(0, 1));
        assert(field_less_than(0, 0x100));
        assert(field_less_than(0x100, 0 - 1));
        assert(!field_less_than(0 - 1, 0));
    }

    #[test]
    unconstrained fn test_large_field_values_unconstrained() {
        let large_field = 0xffffffffffffffff;

        let bits: [u1; 64] = large_field.to_le_bits();
        assert_eq(bits[0], 1);

        let bytes: [u8; 8] = large_field.to_le_bytes();
        assert_eq(Field::from_le_bytes::<8>(bytes), large_field);

        let radix_bytes: [u8; 8] = large_field.to_le_radix(256);
        assert_eq(Field::from_le_bytes::<8>(radix_bytes), large_field);
    }

    #[test]
    fn test_large_field_values() {
        let large_val = 0xffffffffffffffff;

        let bits: [u1; 64] = large_val.to_le_bits();
        assert_eq(bits[0], 1);

        let bytes: [u8; 8] = large_val.to_le_bytes();
        assert_eq(Field::from_le_bytes::<8>(bytes), large_val);

        let radix_bytes: [u8; 8] = large_val.to_le_radix(256);
        assert_eq(Field::from_le_bytes::<8>(radix_bytes), large_val);
    }

    #[test]
    fn test_decomposition_edge_cases() {
        let zero_bits: [u1; 8] = 0.to_le_bits();
        assert_eq(zero_bits, [0; 8]);

        let zero_bytes: [u8; 8] = 0.to_le_bytes();
        assert_eq(zero_bytes, [0; 8]);

        let one_bits: [u1; 8] = 1.to_le_bits();
        let expected: [u1; 8] = [1, 0, 0, 0, 0, 0, 0, 0];
        assert_eq(one_bits, expected);

        let pow2_bits: [u1; 8] = 4.to_le_bits();
        let expected: [u1; 8] = [0, 0, 1, 0, 0, 0, 0, 0];
        assert_eq(pow2_bits, expected);
    }

    #[test]
    fn test_pow_32() {
        assert_eq(2.pow_32(3), 8);
        assert_eq(3.pow_32(2), 9);
        assert_eq(5.pow_32(0), 1);
        assert_eq(7.pow_32(1), 7);

        assert_eq(2.pow_32(10), 1024);

        assert_eq(0.pow_32(5), 0);
        assert_eq(0.pow_32(0), 1);

        assert_eq(1.pow_32(100), 1);
    }

    #[test]
    fn test_sgn0() {
        assert_eq(0.sgn0(), 0);
        assert_eq(2.sgn0(), 0);
        assert_eq(4.sgn0(), 0);
        assert_eq(100.sgn0(), 0);

        assert_eq(1.sgn0(), 1);
        assert_eq(3.sgn0(), 1);
        assert_eq(5.sgn0(), 1);
        assert_eq(101.sgn0(), 1);
    }

    #[test(should_fail_with = "Field failed to decompose into specified 8 limbs")]
    fn test_bit_decomposition_overflow() {
        // 8 bits can't represent large field values
        let large_val = 0x1000000000000000;
        let _: [u1; 8] = large_val.to_le_bits();
    }

    #[test(should_fail_with = "Field failed to decompose into specified 4 limbs")]
    fn test_byte_decomposition_overflow() {
        // 4 bytes can't represent large field values
        let large_val = 0x1000000000000000;
        let _: [u8; 4] = large_val.to_le_bytes();
    }

    #[test]
    fn test_to_from_be_bytes_bn254_edge_cases() {
        if crate::compat::is_bn254() {
            // checking that decrementing this byte produces the expected 32 BE bytes for (modulus - 1)
            let mut p_minus_1_bytes: [u8; 32] = modulus_be_bytes().as_array();
            assert(p_minus_1_bytes[32 - 1] > 0);
            p_minus_1_bytes[32 - 1] -= 1;

            let p_minus_1 = Field::from_be_bytes::<32>(p_minus_1_bytes);
            assert_eq(p_minus_1 + 1, 0);

            // checking that converting (modulus - 1) from and then to 32 BE bytes produces the same bytes
            let p_minus_1_converted_bytes: [u8; 32] = p_minus_1.to_be_bytes();
            assert_eq(p_minus_1_converted_bytes, p_minus_1_bytes);

            // checking that incrementing this byte produces 32 BE bytes for (modulus + 1)
            let mut p_plus_1_bytes: [u8; 32] = modulus_be_bytes().as_array();
            assert(p_plus_1_bytes[32 - 1] < 255);
            p_plus_1_bytes[32 - 1] += 1;

            let p_plus_1 = Field::from_be_bytes::<32>(p_plus_1_bytes);
            assert_eq(p_plus_1, 1);

            // checking that converting p_plus_1 to 32 BE bytes produces the same
            // byte set to 1 as p_plus_1_bytes and otherwise zeroes
            let mut p_plus_1_converted_bytes: [u8; 32] = p_plus_1.to_be_bytes();
            assert_eq(p_plus_1_converted_bytes[32 - 1], 1);
            p_plus_1_converted_bytes[32 - 1] = 0;
            assert_eq(p_plus_1_converted_bytes, [0; 32]);

            // checking that Field::from_be_bytes::<32> on the Field modulus produces 0
            assert_eq(modulus_be_bytes().len(), 32);
            let p = Field::from_be_bytes::<32>(modulus_be_bytes().as_array());
            assert_eq(p, 0);

            // checking that converting 0 to 32 BE bytes produces 32 zeroes
            let p_bytes: [u8; 32] = 0.to_be_bytes();
            assert_eq(p_bytes, [0; 32]);
        }
    }

    #[test]
    fn test_to_from_le_bytes_bn254_edge_cases() {
        if crate::compat::is_bn254() {
            // checking that decrementing this byte produces the expected 32 LE bytes for (modulus - 1)
            let mut p_minus_1_bytes: [u8; 32] = modulus_le_bytes().as_array();
            assert(p_minus_1_bytes[0] > 0);
            p_minus_1_bytes[0] -= 1;

            let p_minus_1 = Field::from_le_bytes::<32>(p_minus_1_bytes);
            assert_eq(p_minus_1 + 1, 0);

            // checking that converting (modulus - 1) from and then to 32 BE bytes produces the same bytes
            let p_minus_1_converted_bytes: [u8; 32] = p_minus_1.to_le_bytes();
            assert_eq(p_minus_1_converted_bytes, p_minus_1_bytes);

            // checking that incrementing this byte produces 32 LE bytes for (modulus + 1)
            let mut p_plus_1_bytes: [u8; 32] = modulus_le_bytes().as_array();
            assert(p_plus_1_bytes[0] < 255);
            p_plus_1_bytes[0] += 1;

            let p_plus_1 = Field::from_le_bytes::<32>(p_plus_1_bytes);
            assert_eq(p_plus_1, 1);

            // checking that converting p_plus_1 to 32 LE bytes produces the same
            // byte set to 1 as p_plus_1_bytes and otherwise zeroes
            let mut p_plus_1_converted_bytes: [u8; 32] = p_plus_1.to_le_bytes();
            assert_eq(p_plus_1_converted_bytes[0], 1);
            p_plus_1_converted_bytes[0] = 0;
            assert_eq(p_plus_1_converted_bytes, [0; 32]);

            // checking that Field::from_le_bytes::<32> on the Field modulus produces 0
            assert_eq(modulus_le_bytes().len(), 32);
            let p = Field::from_le_bytes::<32>(modulus_le_bytes().as_array());
            assert_eq(p, 0);

            // checking that converting 0 to 32 LE bytes produces 32 zeroes
            let p_bytes: [u8; 32] = 0.to_le_bytes();
            assert_eq(p_bytes, [0; 32]);
        }
    }

    /// Convert a little endian bit array to a field element.
    /// If the provided bit array overflows the field modulus then the Field will silently wrap around.
    fn from_le_bits<let N: u32>(bits: [u1; N]) -> Field {
        static_assert(
            N <= modulus_le_bits().len(),
            "N must be less than or equal to modulus_le_bits().len()",
        );
        let mut v = 1;
        let mut result = 0;

        for i in 0..N {
            result += (bits[i] as Field) * v;
            v = v * 2;
        }
        result
    }

    /// Convert a big endian bit array to a field element.
    /// If the provided bit array overflows the field modulus then the Field will silently wrap around.
    fn from_be_bits<let N: u32>(bits: [u1; N]) -> Field {
        let mut v = 1;
        let mut result = 0;

        for i in 0..N {
            result += (bits[N - 1 - i] as Field) * v;
            v = v * 2;
        }
        result
    }

    #[test]
    fn test_to_from_be_bits_bn254_edge_cases() {
        if crate::compat::is_bn254() {
            // checking that decrementing this bit produces the expected 254 BE bits for (modulus - 1)
            let mut p_minus_1_bits: [u1; 254] = modulus_be_bits().as_array();
            assert(p_minus_1_bits[254 - 1] > 0);
            p_minus_1_bits[254 - 1] -= 1;

            let p_minus_1 = from_be_bits::<254>(p_minus_1_bits);
            assert_eq(p_minus_1 + 1, 0);

            // checking that converting (modulus - 1) from and then to 254 BE bits produces the same bits
            let p_minus_1_converted_bits: [u1; 254] = p_minus_1.to_be_bits();
            assert_eq(p_minus_1_converted_bits, p_minus_1_bits);

            // checking that incrementing this bit produces 254 BE bits for (modulus + 4)
            let mut p_plus_4_bits: [u1; 254] = modulus_be_bits().as_array();
            assert(p_plus_4_bits[254 - 3] < 1);
            p_plus_4_bits[254 - 3] += 1;

            let p_plus_4 = from_be_bits::<254>(p_plus_4_bits);
            assert_eq(p_plus_4, 4);

            // checking that converting p_plus_4 to 254 BE bits produces the same
            // bit set to 1 as p_plus_4_bits and otherwise zeroes
            let mut p_plus_4_converted_bits: [u1; 254] = p_plus_4.to_be_bits();
            assert_eq(p_plus_4_converted_bits[254 - 3], 1);
            p_plus_4_converted_bits[254 - 3] = 0;
            assert_eq(p_plus_4_converted_bits, [0; 254]);

            // checking that Field::from_be_bits::<254> on the Field modulus produces 0
            assert_eq(modulus_be_bits().len(), 254);
            let p = from_be_bits::<254>(modulus_be_bits().as_array());
            assert_eq(p, 0);

            // checking that converting 0 to 254 BE bytes produces 254 zeroes
            let p_bits: [u1; 254] = 0.to_be_bits();
            assert_eq(p_bits, [0; 254]);
        }
    }

    #[test]
    fn test_to_from_le_bits_bn254_edge_cases() {
        if crate::compat::is_bn254() {
            // checking that decrementing this bit produces the expected 254 LE bits for (modulus - 1)
            let mut p_minus_1_bits: [u1; 254] = modulus_le_bits().as_array();
            assert(p_minus_1_bits[0] > 0);
            p_minus_1_bits[0] -= 1;

            let p_minus_1 = from_le_bits::<254>(p_minus_1_bits);
            assert_eq(p_minus_1 + 1, 0);

            // checking that converting (modulus - 1) from and then to 254 BE bits produces the same bits
            let p_minus_1_converted_bits: [u1; 254] = p_minus_1.to_le_bits();
            assert_eq(p_minus_1_converted_bits, p_minus_1_bits);

            // checking that incrementing this bit produces 254 LE bits for (modulus + 4)
            let mut p_plus_4_bits: [u1; 254] = modulus_le_bits().as_array();
            assert(p_plus_4_bits[2] < 1);
            p_plus_4_bits[2] += 1;

            let p_plus_4 = from_le_bits::<254>(p_plus_4_bits);
            assert_eq(p_plus_4, 4);

            // checking that converting p_plus_4 to 254 LE bits produces the same
            // bit set to 1 as p_plus_4_bits and otherwise zeroes
            let mut p_plus_4_converted_bits: [u1; 254] = p_plus_4.to_le_bits();
            assert_eq(p_plus_4_converted_bits[2], 1);
            p_plus_4_converted_bits[2] = 0;
            assert_eq(p_plus_4_converted_bits, [0; 254]);

            // checking that Field::from_le_bits::<254> on the Field modulus produces 0
            assert_eq(modulus_le_bits().len(), 254);
            let p = from_le_bits::<254>(modulus_le_bits().as_array());
            assert_eq(p, 0);

            // checking that converting 0 to 254 LE bytes produces 254 zeroes
            let p_bits: [u1; 254] = 0.to_le_bits();
            assert_eq(p_bits, [0; 254]);
        }
    }
}
