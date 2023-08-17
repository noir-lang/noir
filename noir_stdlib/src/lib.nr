mod hash;
mod array;
mod slice;
mod merkle;
mod schnorr;
mod ecdsa_secp256k1;
mod ecdsa_secp256r1;
mod eddsa;
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
