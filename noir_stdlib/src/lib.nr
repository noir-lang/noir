pub mod hash;
pub mod aes128;
pub mod array;
pub mod slice;
pub mod ecdsa_secp256k1;
pub mod ecdsa_secp256r1;
pub mod embedded_curve_ops;
pub mod field;
pub mod collections;
pub mod compat;
pub mod convert;
pub mod option;
pub mod string;
pub mod test;
pub mod cmp;
pub mod ops;
pub mod default;
pub mod prelude;
pub mod runtime;
pub mod meta;
pub mod append;
pub mod mem;
pub mod panic;
pub mod hint;

use convert::AsPrimitive;

// Oracle calls are required to be wrapped in an unconstrained function
// Thus, the only argument to the `println` oracle is expected to always be an ident
#[oracle(print)]
unconstrained fn print_oracle<T>(with_newline: bool, input: T) {}

unconstrained fn print_unconstrained<T>(with_newline: bool, input: T) {
    print_oracle(with_newline, input);
}

pub fn println<T>(input: T) {
    // Safety: a print statement cannot be constrained
    unsafe {
        print_unconstrained(true, input);
    }
}

pub fn print<T>(input: T) {
    // Safety: a print statement cannot be constrained
    unsafe {
        print_unconstrained(false, input);
    }
}

pub fn verify_proof<let N: u32, let M: u32, let K: u32>(
    verification_key: [Field; N],
    proof: [Field; M],
    public_inputs: [Field; K],
    key_hash: Field,
) {
    verify_proof_internal(verification_key, proof, public_inputs, key_hash, 0);
}

pub fn verify_proof_with_type<let N: u32, let M: u32, let K: u32>(
    verification_key: [Field; N],
    proof: [Field; M],
    public_inputs: [Field; K],
    key_hash: Field,
    proof_type: u32,
) {
    if !crate::runtime::is_unconstrained() {
        crate::assert_constant(proof_type);
    }
    verify_proof_internal(verification_key, proof, public_inputs, key_hash, proof_type);
}

#[foreign(recursive_aggregation)]
fn verify_proof_internal<let N: u32, let M: u32, let K: u32>(
    verification_key: [Field; N],
    proof: [Field; M],
    public_inputs: [Field; K],
    key_hash: Field,
    proof_type: u32,
) {}

// Asserts that the given value is known at compile-time.
// Useful for debugging for-loop bounds.
#[builtin(assert_constant)]
pub fn assert_constant<T>(x: T) {}

// Asserts that the given value is both true and known at compile-time
#[builtin(static_assert)]
pub fn static_assert<let N: u32>(predicate: bool, message: str<N>) {}

pub fn wrapping_add<T>(x: T, y: T) -> T
where
    T: AsPrimitive<Field>,
    Field: AsPrimitive<T>,
{
    AsPrimitive::as_(x.as_() + y.as_())
}

pub fn wrapping_sub<T>(x: T, y: T) -> T
where
    T: AsPrimitive<Field>,
    Field: AsPrimitive<T>,
{
    //340282366920938463463374607431768211456 is 2^128, it is used to avoid underflow
    AsPrimitive::as_(x.as_() + 340282366920938463463374607431768211456 - y.as_())
}

fn wrapping_mul<T>(x: T, y: T) -> T
where
    T: WappingMul,
{
    x.wrapping_mul(y)
}

global two_pow_64: Field = 0x10000000000000000;
/// Splits a 128 bits number into two 64 bits limbs
unconstrained fn split64(x: Field) -> (u64, u64) {
    let lo = x as u64;
    let hi = ((x - lo as Field) / two_pow_64) as u64;
    (lo, hi)
}

/// Split a 128 bits number into two 64 bits limbs
/// It will fail if the number is more than 128 bits
fn split_into_64_bit_limbs(x: Field) -> (u64, u64) {
    // Safety: the limbs are constrained below
    let (x_lo, x_hi) = unsafe { split64(x) };
    assert(x as Field == x_lo as Field + x_hi as Field * two_pow_64);
    (x_lo, x_hi)
}

#[field(bn254)]
pub fn wrapping_mul128(x: Field, y: Field) -> Field {
    let (x_lo, x_hi) = split_into_64_bit_limbs(x);
    let (y_lo, y_hi) = split_into_64_bit_limbs(y);
    // Multiplication using the limbs:(x_lo + 2**64*x_hi)*(y_lo + 2**64*y_hi)=x_lo*y_lo+...
    // and skipping the terms over 2**128
    // Working with u64 limbs ensures that we cannot overflow the field modulus.
    let low = x_lo as Field * y_lo as Field;
    let lo = low as u64 as Field;
    let carry = (low - lo) / two_pow_64;
    let high = x_lo as Field * y_hi as Field + x_hi as Field * y_lo as Field + carry;
    let hi = high as u64 as Field;
    (lo + two_pow_64 * hi)
}

#[builtin(as_witness)]
pub fn as_witness(x: Field) {}

trait WappingMul {
    fn wrapping_mul(self, y: Self) -> Self;
}

impl WappingMul for u64 {
    fn wrapping_mul(self: u64, y: u64) -> u64 {
        //AsPrimitive::as_(self.as_()* y.as_())
        ((self as Field) * (y as Field)) as u64
    }
}
impl WappingMul for u32 {
    fn wrapping_mul(self: u32, y: u32) -> u32 {
        ((self as Field) * (y as Field)) as u32
    }
}
impl WappingMul for u16 {
    fn wrapping_mul(self: u16, y: u16) -> u16 {
        ((self as Field) * (y as Field)) as u16
    }
}
impl WappingMul for u8 {
    fn wrapping_mul(self: u8, y: u8) -> u8 {
        ((self as Field) * (y as Field)) as u8
    }
}
impl WappingMul for bool {
    fn wrapping_mul(self: bool, y: bool) -> bool {
        ((self as Field) * (y as Field)) as bool
    }
}
impl WappingMul for u128 {
    fn wrapping_mul(self: u128, y: u128) -> u128 {
        wrapping_mul128(self as Field, y as Field) as u128
    }
}

mod tests {
    use super::wrapping_mul;

    #[test(should_fail_with = "custom message")]
    fn test_static_assert_custom_message() {
        super::static_assert(1 == 2, "custom message");
    }

    #[test]
    fn test_wrapping_mul() {
        let zero: u128 = 0;
        let one: u128 = 1;
        let two_pow_64: u128 = 0x10000000000000000;
        let u128_max: u128 = 0xffffffffffffffffffffffffffffffff;

        // 1*0==0
        assert_eq(zero, wrapping_mul(zero, one));

        // 0*1==0
        assert_eq(zero, wrapping_mul(one, zero));

        // 1*1==1
        assert_eq(one, wrapping_mul(one, one));

        // 0 * ( 1 << 64 ) ==  0
        assert_eq(zero, wrapping_mul(zero, two_pow_64));

        // ( 1 << 64 ) * 0 == 0
        assert_eq(zero, wrapping_mul(two_pow_64, zero));

        // 1 * ( 1 << 64 ) == 1 << 64
        assert_eq(two_pow_64, wrapping_mul(two_pow_64, one));

        // ( 1 << 64 ) * 1 == 1 << 64
        assert_eq(two_pow_64, wrapping_mul(one, two_pow_64));

        // ( 1 << 64 ) * ( 1 << 64 ) == 1 << 64
        assert_eq(zero, wrapping_mul(two_pow_64, two_pow_64));
        // -1 * -1 == 1
        assert_eq(one, wrapping_mul(u128_max, u128_max));
    }
}
