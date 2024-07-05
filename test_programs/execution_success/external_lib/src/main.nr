use edwards::Curve;
use edwards::bjj::BabyJubJubParams;
use std::ec::consts::te::baby_jubjub;
use std::ec::tecurve::affine::Point as TEPoint;

fn main(x: Field, y: pub Field) -> pub Field {
    let bjj = baby_jubjub();
    let bjj_point = bjj.base8;
    let point:  Curve<BabyJubJubParams> = Curve { x: bjj_point.x, y: bjj_point.y };
    (point.x == x) as Field
}
