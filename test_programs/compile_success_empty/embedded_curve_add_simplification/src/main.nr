use std::embedded_curve_ops::EmbeddedCurvePoint;

fn main() {
    let zero = EmbeddedCurvePoint::point_at_infinity();
    let g1 = EmbeddedCurvePoint {
        x: 1,
        y: 17631683881184975370165255887551781615748388533673675138860,
        is_infinite: false,
    };

    assert(g1 + zero == g1);
    assert(g1 - g1 == zero);
    assert(g1 - zero == g1);
    assert(zero + zero == zero);
}
