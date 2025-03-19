// https://github.com/noir-lang/noir/issues/7744

use std::embedded_curve_ops::{EmbeddedCurvePoint, embedded_curve_add_unsafe};

// is_active = false
fn main(is_active: bool) -> pub EmbeddedCurvePoint {
    let bad = EmbeddedCurvePoint { x: 0, y: 5, is_infinite: false };
    if is_active {
        embedded_curve_add_unsafe(bad, bad)
    } else {
        bad
    }
}
