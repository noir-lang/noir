use std::embedded_curve_ops::{EmbeddedCurvePoint, EmbeddedCurveScalar, multi_scalar_mul};

// `multi_scalar_mul` is given a predicate by the compiler so that a call sitting in a
// branch which is not taken is not constrained. The point below is not on the embedded
// curve, so the call would fail if it were evaluated as though the branch were live.
fn main(x: Field, y: Field, lo: Field, hi: Field, active: bool) {
    if active {
        let point = EmbeddedCurvePoint { x, y };
        let scalar = EmbeddedCurveScalar::new(lo, hi);
        let result = multi_scalar_mul([point], [scalar]);
        assert(!result.is_infinite());
    }
}
