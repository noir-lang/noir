// https://github.com/noir-lang/noir/issues/7744

use std::embedded_curve_ops::{embedded_curve_add, EmbeddedCurvePoint};

// is_active = false
fn main(is_active: bool) -> pub EmbeddedCurvePoint {
    let bad = EmbeddedCurvePoint::new(0, 5);
    if is_active {
        embedded_curve_add(bad, bad)
    } else {
        bad
    }
}
