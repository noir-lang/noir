// Computes a fixed base scalar multiplication over the embedded curve.
// For bn254, We have Grumpkin and Baby JubJub.
// For bls12-381, we have JubJub and Bandersnatch.
//
// The embedded curve being used is decided by the 
// underlying proof system.
#[foreign(fixed_base_scalar_mul)]
pub fn fixed_base_embedded_curve(_low : Field, _high : Field) -> [Field; 2] {}
