pub mod hash;
mod array;
pub mod slice;
pub mod merkle;
pub mod schnorr;
pub mod ecdsa_secp256k1;
pub mod ecdsa_secp256r1;
pub mod eddsa;
pub(crate) mod grumpkin_scalar;
mod grumpkin_scalar_mul;
pub mod scalar_mul;
pub mod sha256;
pub mod sha512;
pub mod field;
pub mod ec;
pub mod unsafe;
pub mod collections;
pub mod compat;
pub(crate) mod convert;
pub mod option;
mod string;
pub mod test;
pub mod cmp;
pub mod ops;
pub(crate) mod default;
pub mod prelude;
pub mod uint128;
pub mod bigint;
pub mod internal;

// Oracle calls are required to be wrapped in an unconstrained function
// Thus, the only argument to the `println` oracle is expected to always be an ident
#[oracle(print)]
unconstrained fn print_oracle<T>(with_newline: bool, input: T) {}

unconstrained pub fn print<T>(input: T) {
    print_oracle(false, input);
}

unconstrained pub fn println<T>(input: T) {
    print_oracle(true, input);
}

#[foreign(recursive_aggregation)]
pub fn verify_proof<N>(verification_key: [Field], proof: [Field], public_inputs: [Field], key_hash: Field) {}

// Asserts that the given value is known at compile-time.
// Useful for debugging for-loop bounds.
#[builtin(assert_constant)]
pub fn assert_constant<T>(x: T) {}
// from_field and as_field are private since they are not valid for every type.
// `as` should be the default for users to cast between primitive types, and in the future
// traits can be used to work with generic types.
#[builtin(from_field)]
fn from_field<T>(x: Field) -> T {}

#[builtin(as_field)]
fn as_field<T>(x: T) -> Field {}

pub fn wrapping_add<T>(x: T, y: T) -> T {
    crate::from_field(crate::as_field(x) + crate::as_field(y))
}

pub fn wrapping_sub<T>(x: T, y: T) -> T {
    //340282366920938463463374607431768211456 is 2^128, it is used to avoid underflow
    crate::from_field(crate::as_field(x) + 340282366920938463463374607431768211456 - crate::as_field(y))
}

pub fn wrapping_mul<T>(x: T, y: T) -> T {
    crate::from_field(crate::as_field(x) * crate::as_field(y))
}
