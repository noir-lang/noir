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
        self.__assert_max_bit_size(BIT_SIZE);
    }

    #[builtin(apply_range_constraint)]
    fn __assert_max_bit_size(self, bit_size: u32) {}

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
    fn _to_le_bits<let N: u32>(self: Self) -> [u1; N] {}

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
    fn _to_be_bits<let N: u32>(self: Self) -> [u1; N] {}

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
        let bits = self._to_le_bits();

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
        let bits = self._to_be_bits();

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

    // docs:start:to_le_radix
    pub fn to_le_radix<let N: u32>(self: Self, radix: u32) -> [u8; N] {
        // Brillig does not need an immediate radix
        if !crate::runtime::is_unconstrained() {
            static_assert(1 < radix, "radix must be greater than 1");
            static_assert(radix <= 256, "radix must be less than or equal to 256");
            static_assert(radix & (radix - 1) == 0, "radix must be a power of 2");
        }
        self.__to_le_radix(radix)
    }
    // docs:end:to_le_radix

    // docs:start:to_be_radix
    pub fn to_be_radix<let N: u32>(self: Self, radix: u32) -> [u8; N] {
        // Brillig does not need an immediate radix
        if !crate::runtime::is_unconstrained() {
            crate::assert_constant(radix);
        }
        self.__to_be_radix(radix)
    }
    // docs:end:to_be_radix

    // `_radix` must be less than 256
    #[builtin(to_le_radix)]
    fn __to_le_radix<let N: u32>(self, radix: u32) -> [u8; N] {}

    // `_radix` must be less than 256
    #[builtin(to_be_radix)]
    fn __to_be_radix<let N: u32>(self, radix: u32) -> [u8; N] {}

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
        /// Safety: unconstrained context
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
    use crate::{panic::panic, runtime};
    use super::field_less_than;

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

    // TODO: Update this test to account for the Brillig restriction that the radix must be greater than 2
    //#[test]
    //fn test_to_le_radix_brillig_1() {
    //    // this test should only fail in constrained mode
    //    if runtime::is_unconstrained() {
    //        let field = 1;
    //        let out: [u8; 8] = field.to_le_radix(1);
    //        crate::println(out);
    //        let expected = [0; 8];
    //        assert(out == expected, "unexpected result");
    //    }
    //}

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

    // TODO: Update this test to account for the Brillig restriction that the radix must be less than 512
    //#[test]
    //fn test_to_le_radix_brillig_512() {
    //    // this test should only fail in constrained mode
    //    if runtime::is_unconstrained() {
    //        let field = 1;
    //        let out: [u8; 8] = field.to_le_radix(512);
    //        let mut expected = [0; 8];
    //        expected[0] = 1;
    //        assert(out == expected, "unexpected result");
    //    }
    //}

    #[test]
    unconstrained fn test_field_less_than() {
        assert(field_less_than(0, 1));
        assert(field_less_than(0, 0x100));
        assert(field_less_than(0x100, 0 - 1));
        assert(!field_less_than(0 - 1, 0));
    }
}
