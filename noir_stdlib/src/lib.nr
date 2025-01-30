pub mod hash;
pub mod aes128;
pub mod array;
pub mod slice;
pub mod merkle;
pub mod ecdsa_secp256k1;
pub mod ecdsa_secp256r1;
pub mod embedded_curve_ops;
pub mod sha256;
pub mod sha512;
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
pub mod uint128;
pub mod bigint;
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
    /// Safety: a print statement cannot be constrained
    unsafe {
        print_unconstrained(true, input);
    }
}

pub fn print<T>(input: T) {
    /// Safety: a print statement cannot be constrained
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
    #[test(should_fail_with = "custom message")]
    fn test_static_assert_custom_message() {
        super::static_assert(1 == 2, "custom message");
    }
}
