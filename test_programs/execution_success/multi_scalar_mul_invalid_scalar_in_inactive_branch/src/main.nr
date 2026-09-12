use std::embedded_curve_ops::{EmbeddedCurvePoint, EmbeddedCurveScalar, multi_scalar_mul};

// Companion to `execution_failure/msm_infinity_scalar_out_of_range`, which checks that the
// scalar limbs of a live `multi_scalar_mul` are range-validated. Here two out-of-range
// scalars sit in a branch which is not taken, so the predicate must suppress that validation.
//
// `over_limb` has a `hi` limb of 2^128, which does not fit in a scalar limb at all.
// `over_order` is exactly the embedded curve's scalar modulus, so its limbs are individually
// in range but the scalar they encode is not.
fn main(
    over_limb_lo: Field,
    over_limb_hi: Field,
    over_order_lo: Field,
    over_order_hi: Field,
    active: bool,
) {
    if active {
        let generator = EmbeddedCurvePoint::generator();

        let over_limb = EmbeddedCurveScalar::new(over_limb_lo, over_limb_hi);
        let result = multi_scalar_mul([generator], [over_limb]);
        assert(!result.is_infinite());

        let over_order = EmbeddedCurveScalar::new(over_order_lo, over_order_hi);
        let result = multi_scalar_mul([generator], [over_order]);
        assert(!result.is_infinite());
    }
}
