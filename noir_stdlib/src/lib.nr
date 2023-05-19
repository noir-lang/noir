mod hash;
mod array;
mod merkle;
mod schnorr;
mod ecdsa_secp256k1;
mod scalar_mul;
mod sha256;
mod sha512;
mod field;
mod ec;
mod unsafe;
mod collections;

#[builtin(println)]
fn println<T>(_input : T) {}

// TODO: have the compiler insert the key_hash and remove as param 
#[foreign(verify_proof)]
fn verify_proof(_verification_key : [Field], _proof : [Field], _public_inputs : [Field], _key_hash : Field, _input_aggregation_object : [Field]) -> [Field] {}