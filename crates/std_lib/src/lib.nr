mod hash;
mod array;
mod merkle;
mod schnorr;
mod ecdsa_secp256k1;
mod scalar_mul;

#[builtin(set_pub)]
fn set_as_public(_input : Field) {}

// Returns 1 if x == y, and 0 otherwise
#[builtin(predicate_equal)]
fn predicate_eq(_x : Field, _y : Field) -> Field{}

// Returns base^exponent. 
// ^ means to the power of and not xor
// Note that this function may wrap around
#[builtin(pow_const)]
fn pow_const(_base : const Field, _exponent : const Field) -> const Field {}

