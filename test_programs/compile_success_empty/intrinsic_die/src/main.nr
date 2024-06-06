use dep::std;
// This test checks that we perform dead-instruction-elimination on intrinsic functions.
fn main(x: Field) {
    let hash = std::hash::pedersen_commitment([x]);
    let g1_x = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let g1_y = 0x0000000000000002cf135e7506a45d632d270d45f1181294833fc48d823f272c;
    let g1 = std::embedded_curve_ops::EmbeddedCurvePoint { x: g1_x, y: g1_y, is_infinite: false };
    let scalar = std::embedded_curve_ops::EmbeddedCurveScalar { lo: x, hi: 0 };
    let _p1 = std::embedded_curve_ops::multi_scalar_mul([g1], [scalar]);
}
