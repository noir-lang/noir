fn main() {
    let pub_x = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let pub_y = 0x0000000000000002cf135e7506a45d632d270d45f1181294833fc48d823f272c;

    let g1_y = 17631683881184975370165255887551781615748388533673675138860;
    let g1 = std::embedded_curve_ops::EmbeddedCurvePoint { x: 1, y: g1_y, is_infinite: false };
    let scalar = std::embedded_curve_ops::EmbeddedCurveScalar { lo: 1, hi: 0 };
    // Test that multi_scalar_mul correctly derives the public key
    let res = std::embedded_curve_ops::multi_scalar_mul([g1], [scalar]);
    assert(res.x == pub_x);
    assert(res.y == pub_y);
}
