use std::embedded_curve_ops::EmbeddedCurvePoint;
use std::embedded_curve_ops::EmbeddedCurveScalar;

fn main(is_active: bool) {
    let a = EmbeddedCurvePoint {
        x: 0x1d8eb4378a3bde41e0b6a9a8dcbd21b7ff9c51bdd6ca13ce989abbbf90df3666,
        y: 0x06075b63354f2504f9cddba0b94ed0cef35fc88615e69ec1f853b51eb79a24a0,
        is_infinite: false,
    };

    if is_active {
        let bad = EmbeddedCurvePoint { x: 0, y: 5, is_infinite: false };
        let d = bad.double();
        let e = std::embedded_curve_ops::multi_scalar_mul(
            [a, bad],
            [EmbeddedCurveScalar { lo: 1, hi: 0 }, EmbeddedCurveScalar { lo: 1, hi: 0 }],
        );
        assert(e.x != d.x);
    }
}
