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

/// Asserts the validity of the provided proof and public inputs against the provided verification key and hash.
///
/// The ACVM cannot determine whether the provided proof is valid during execution as this requires knowledge of
/// the backend against which the program is being proven. However if an invalid proof if submitted, the program may
/// fail to prove or the backend may generate a proof which will subsequently fail to verify.
///
/// # Important Note
///
/// If you are not developing your own backend such as [Barretenberg](https://github.com/AztecProtocol/barretenberg)
/// you probably shouldn't need to interact with this function directly. It's easier and safer to use a verification
/// library which is published by the developers of the backend which will document or enforce any safety requirements.
///
/// If you use this directly, you're liable to introduce underconstrainedness bugs and *your circuit will be insecure*.
///
/// # Arguments
/// - verification_key: The verification key of the circuit to be verified.
/// - proof: The proof to be verified.
/// - public_inputs: The public inputs associated with `proof`
/// - key_hash: The hash of `verification_key` of the form expected by the backend.
/// - proof_type: An identifier for the proving scheme used to generate the proof to be verified. This allows
///               for a single backend to support verifying multiple proving schemes.
///
/// # Constraining `key_hash`
///
/// The Noir compiler does not by itself constrain that `key_hash` is a valid hash of `verification_key`.
/// This is because different backends may differ in how they hash their verification keys.
/// It is then the responsibility of either the noir developer (by explicitly hashing the verification key
/// in the correct manner) or by the proving system itself internally asserting the correctness of `key_hash`.
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

// Asserts that the given value is both true and known at compile-time.
// The message can be a string, a format string, or any value, as long as it is known at compile-time
#[builtin(static_assert)]
pub fn static_assert<T>(predicate: bool, message: T) {}

#[deprecated("wrapping operations should be done with the Wrapping traits. E.g: x.wrapping_add(y)")]
pub fn wrapping_add<T>(x: T, y: T) -> T
where
    T: AsPrimitive<Field>,
    Field: AsPrimitive<T>,
{
    AsPrimitive::as_(x.as_() + y.as_())
}
#[deprecated("wrapping operations should be done with the Wrapping traits. E.g: x.wrapping_sub(y)")]
pub fn wrapping_sub<T>(x: T, y: T) -> T
where
    T: AsPrimitive<Field>,
    Field: AsPrimitive<T>,
{
    //340282366920938463463374607431768211456 is 2^128, it is used to avoid underflow
    AsPrimitive::as_(x.as_() + 340282366920938463463374607431768211456 - y.as_())
}
#[deprecated("wrapping operations should be done with the Wrapping traits. E.g: x.wrapping_mul(y)")]
pub fn wrapping_mul<T>(x: T, y: T) -> T
where
    T: AsPrimitive<Field>,
    Field: AsPrimitive<T>,
{
    AsPrimitive::as_(x.as_() * y.as_())
}

#[builtin(as_witness)]
pub fn as_witness(x: Field) {}

mod tests {
    use super::ops::arith::WrappingMul;

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
        assert_eq(zero, zero.wrapping_mul(one));

        // 0*1==0
        assert_eq(zero, one.wrapping_mul(zero));

        // 1*1==1
        assert_eq(one, one.wrapping_mul(one));

        // 0 * ( 1 << 64 ) ==  0
        assert_eq(zero, zero.wrapping_mul(two_pow_64));

        // ( 1 << 64 ) * 0 == 0
        assert_eq(zero, two_pow_64.wrapping_mul(zero));

        // 1 * ( 1 << 64 ) == 1 << 64
        assert_eq(two_pow_64, two_pow_64.wrapping_mul(one));

        // ( 1 << 64 ) * 1 == 1 << 64
        assert_eq(two_pow_64, one.wrapping_mul(two_pow_64));

        // ( 1 << 64 ) * ( 1 << 64 ) == 1 << 64
        assert_eq(zero, two_pow_64.wrapping_mul(two_pow_64));
        // -1 * -1 == 1
        assert_eq(one, u128_max.wrapping_mul(u128_max));
    }
}
