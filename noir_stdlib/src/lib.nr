mod hash;
mod array;
mod slice;
mod merkle;
mod schnorr;
mod ecdsa_secp256k1;
mod ecdsa_secp256r1;
mod eddsa;
mod grumpkin_scalar;
mod grumpkin_scalar_mul;
mod scalar_mul;
mod sha256;
mod sha512;
mod field;
mod ec;
mod unsafe;
mod collections;
mod compat;
mod convert;
mod option;
mod string;
mod test;
mod cmp;
mod ops;
mod default;
mod prelude;
mod uint128;
mod bigint;

// Oracle calls are required to be wrapped in an unconstrained function
// Thus, the only argument to the `println` oracle is expected to always be an ident
#[oracle(print)]
unconstrained fn print_oracle<T>(_with_newline: bool, _input: T) {}

unconstrained pub fn print<T>(input: T) {
    print_oracle(false, input);
}

unconstrained pub fn println<T>(input: T) {
    print_oracle(true, input);
}

#[foreign(recursive_aggregation)]
pub fn verify_proof<N>(_verification_key: [Field], _proof: [Field], _public_inputs: [Field], _key_hash: Field) {}

// Asserts that the given value is known at compile-time.
// Useful for debugging for-loop bounds.
#[builtin(assert_constant)]
pub fn assert_constant<T>(_x: T) {}
// from_field and as_field are private since they are not valid for every type.
// `as` should be the default for users to cast between primitive types, and in the future
// traits can be used to work with generic types.
#[builtin(from_field)]
fn from_field<T>(_x: Field) -> T {}

#[builtin(as_field)]
fn as_field<T>(_x: T) -> Field {}

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
