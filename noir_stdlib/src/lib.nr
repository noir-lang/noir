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
mod option;
mod string;

// Oracle calls are required to be wrapped in an unconstrained function
// Thus, the only argument to the `println` oracle is expected to always be an ident 
#[oracle(println)]
unconstrained fn println_oracle<T>(_input: T) {}

unconstrained fn println<T>(input: T) {
    println_oracle(input);
}

#[foreign(recursive_aggregation)]
fn verify_proof<N>(_verification_key : [Field], _proof : [Field], _public_inputs : [Field], _key_hash : Field, _input_aggregation_object : [Field; N]) -> [Field; N] {}

// Asserts that the given value is known at compile-time.
// Useful for debugging for-loop bounds.
#[builtin(assert_constant)]
fn assert_constant<T>(_x: T) {}

#[builtin(from_field)]
fn from_field<T>(x : Field) -> T {}

#[builtin(as_field)]
fn as_field<T>(x : T) -> Field {}


fn wrapping_add<T>(x : T, y: T) -> T {
    crate::from_field(crate::as_field(x) + crate::as_field(y))
}


fn wrapping_sub<T>(x : T, y: T) -> T {
    crate::from_field(crate::as_field(x) + 340282366920938463463374607431768211456 - crate::as_field(y))
}

fn wrapping_mul<T>(x : T, y: T) -> T {
    crate::from_field(crate::as_field(x) * crate::as_field(y))
}

fn wrapping_shift_left<T>(x : T, y: T) -> T {
    crate::from_field(crate::as_field(x) * 2.pow_32(crate::as_field(y)))
}