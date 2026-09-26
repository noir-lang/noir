use std::embedded_curve_ops::{embedded_curve_add, EmbeddedCurvePoint};

// `embedded_curve_add` is given a predicate by the compiler so that a call sitting in a
// branch which is not taken is not constrained. Neither point below is on the embedded
// curve, so the call would fail if it were evaluated as though the branch were live.
fn main(x1: Field, y1: Field, x2: Field, y2: Field, active: bool) {
    if active {
        let point1 = EmbeddedCurvePoint { x: x1, y: y1 };
        let point2 = EmbeddedCurvePoint { x: x2, y: y2 };
        let result = embedded_curve_add(point1, point2);
        assert(!result.is_infinite());
    }
}
