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

// TODO(#1850): Remove this builtin to use the foreign call based println
#[builtin(println)]
fn println<T>(_input : T) {}

#[oracle(println)]
unconstrained fn println_oracle<T, A>(_input: T) {}

unconstrained fn println<T>(input: T) {
    println_oracle(input);
}

#[foreign(recursive_aggregation)]
fn verify_proof(_verification_key : [Field], _proof : [Field], _public_inputs : [Field], _key_hash : Field, _input_aggregation_object : [Field]) -> [Field] {}
