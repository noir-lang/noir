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

// Precondition: x is a boolean
// returns 1 if x is 0
// returns 0 if x is 1
// XXX: This function should ideally take in a u1 and return a u1
// Then we would not need to state the precondition as it would be 
// encoded in the type
// Check issue#105 for the reason it does not.
fn bool_not(x : Field) -> Field {
    1 - x
}

// Precondition: x and y are booleans
// Check issue#105 for the reason the input and output
// is not a u1/bool
fn bool_or(x : Field, y : Field) -> Field {
    x + y - (x * y)
}
