mod hash;
mod array;
mod merkle;
mod schnorr;
mod ecdsa_secp256k1;
mod scalar_mul;

#[builtin(set_pub)]
fn set_as_public(_input : Field) {}

// Returns base^exponent. 
// ^ means to the power of and not xor
// Note that this function may wrap around
#[builtin(pow_const)]
fn pow_const(_base : const Field, _exponent : const Field) -> const Field {}


// fn pow(base : Field, exponent : Field) -> Field {
//     let r: Field = 1 as Field;
//     let b = dep::std::to_bits(exponent, 32 as u32);
//     for i in 1..33 {
//         r = r*r;
//         r = (b[32-i] as Field)*(r*base) + (1-b[32-i] as Field)*r;
//     };
//     r
// }

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


#[foreign(to_bits)]
fn to_bits(_x : Field, _bit_size: u32) -> [256]u1{}

// impl Field {
//     #[builtin(to_bits_field)]
//     fn to_bits(self) -> []bool{}
// }

// impl u32 {
//     #[builtin(to_bits_u32)]
//     fn to_bits(self) -> []bool{}
// }