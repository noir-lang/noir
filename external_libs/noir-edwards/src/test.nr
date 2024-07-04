use dep::std;
use dep::std::ec;
use crate::scalar_field::ScalarField;
use crate::Curve;
use crate::TECurveParameterTrait;
use crate::bjj::BabyJubJubParams;
use std::ec::consts::te::baby_jubjub;
use std::ec::tecurve::affine::Point as TEPoint;

type BabyJubJub = Curve<BabyJubJubParams>;

#[test]
fn test_sub() {
    let bjj = baby_jubjub();
    let bjj_point = bjj.base8;
    let point:  Curve<BabyJubJubParams> = Curve { x: bjj_point.x, y: bjj_point.y };

    let expected = point + (point + (point));
    let result = point.dbl().dbl().sub(point);
    assert(result.x == expected.x);
    assert(result.y == expected.y);
}

#[test]
fn test_mul() {
    let scalar = 13439285682734681346581734618;
    let bjj = baby_jubjub();
    let bjj_point = bjj.base8;

    let expected = bjj.curve.mul(scalar, bjj_point);
    let scalar_f: ScalarField<63> = ScalarField::from(scalar);
    let point:  BabyJubJub = Curve { x: bjj_point.x, y: bjj_point.y };
    let result = point.mul(scalar_f);
    let scalar_converted: Field = ScalarField::into(scalar_f);
    assert(scalar_converted == scalar);
    assert(result.x == expected.x);
    assert(result.y == expected.y);
}

#[test]
fn test_msm() {
    let scalar = 13439285682734681346581734618;
    let bjj = baby_jubjub();
    let bjj_point = bjj.base8;

    let point:  BabyJubJub = Curve { x: bjj_point.x, y: bjj_point.y };
    let mut scalar_values: [Field; 1] = [0; 1];
    let mut points: [BabyJubJub; 1] = [Curve { x: 0, y: 0 }; 1];
    let mut scalars: [ScalarField<63>; 1] = [ScalarField::new(); 1];
    let mut bjj_points: [TEPoint; 1] = [TEPoint::new(0, 0); 1];

    points[0] = point;
    scalar_values[0] = scalar;
    scalars[0] = ScalarField::from(scalar_values[0]);
    bjj_points[0] = TEPoint::new(points[0].x, points[0].y);

    for i in 1..1 {
        points[i] = points[i-1].dbl();
        scalar_values[i] = scalar_values[i-1] + scalar;
    }

    for i in 0..1 {
        scalars[i] = ScalarField::from(scalar_values[i]);
        bjj_points[i] = TEPoint::new(points[i].x, points[i].y);
        let scalar_expected = scalar_values[i];
        let scalar_result: Field = ScalarField::into(scalars[i]);
        assert(scalar_result == scalar_expected);

        let result_point = points[i].mul(scalars[i]);
        let expected_point = bjj.curve.mul(scalar_values[i], bjj_points[i]);
        assert(result_point.x == expected_point.x);
        assert(result_point.y == expected_point.y);
    }
    let expected = bjj.curve.msm(scalar_values, bjj_points);
    let result = Curve::msm(points, scalars);

    assert(result.x == expected.x);
    assert(result.y == expected.y);
}
