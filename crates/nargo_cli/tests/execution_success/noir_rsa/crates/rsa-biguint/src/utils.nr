use crate::BITS_PER_LIMB as BITS;

// NOTE: this file is forked from https://github.com/shuklaayush/noir-bigint v0.1.0

// Compute a + b + carry, returning the result and the new carry over.
// TODO: Does carry need to be a u56?
fn adc(a: u56, b: u56, carry: u56) -> (u56, u56) {
    let ret = a as Field + b as Field + carry as Field;
    (ret as u56, (ret as u112 >> BITS as u112) as u56)
}

// Compute a - (b + borrow), returning the result and the new borrow.
fn sbb(a: u56, b: u56, borrow: u56) -> (u56, u56) {
    let ret = a as u112 - (b as u112 + (borrow as u112 >> (BITS as u112 - 1)));
    (ret as u56, (ret >> 56) as u56)
}

// Compute a + (b * c) + carry, returning the result and the new carry over.
fn mac(a: u56, b: u56, c: u56, carry: u56) -> (u56, u56) {
    let ret = (a as Field) + ((b as Field) * (c as Field)) + (carry as Field);
    (ret as u56, (ret as u112 >> BITS as u112) as u56)
}