// NOTE: this file is forked from https://github.com/shuklaayush/noir-bigint/blob/d3682670f4323f7f273043c281f365a7bfb992d4/crates/biguint/src/utils.nr

use crate::BITS_PER_LIMB as BITS;
use dep::std;

// Compute a + b + carry, returning the result and the new carry over.
// TODO: Does carry need to be a u56?
pub fn adc(a: u56, b: u56, carry: u56) -> (u56, u56) {
    let ret = a as Field + b as Field + carry as Field;
    (ret as u56, (ret as u112 >> BITS as u112) as u56)
}

// Compute a - (b + borrow), returning the result and the new borrow.
pub fn sbb(a: u56, b: u56, borrow: u56) -> (u56, u56) {
    let ret = std::wrapping_sub(a as u112, b as u112 + (borrow as u112 >> (BITS as u112 - 1)));
    (ret as u56, (ret >> 56) as u56)
}

// Compute a + (b * c) + carry, returning the result and the new carry over.
pub fn mac(a: u56, b: u56, c: u56, carry: u56) -> (u56, u56) {
    let ret = (a as Field) + ((b as Field) * (c as Field)) + (carry as Field);
    (ret as u56, (ret as u112 >> BITS as u112) as u56)
}
