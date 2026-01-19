use std::embedded_curve_ops::EmbeddedCurvePoint;

fn main() {
    let zero = EmbeddedCurvePoint::point_at_infinity();
    let g1 = EmbeddedCurvePoint::generator();

    assert(g1 + zero == g1);
    assert(g1 - g1 == zero);
    assert(g1 - zero == g1);
    assert(zero + zero == zero);
}
