use dep::std;

fn main(point_x: pub Field, point_y: pub Field, scalar_low: pub Field, scalar_high: pub Field) {
    // We multiply the point by 3 and check it matches result out of embedded_curve_add func
    let res = std::scalar_mul::variable_base_embedded_curve(point_x, point_y, scalar_low, scalar_high);

    let point = std::scalar_mul::EmbeddedCurvePoint { x: point_x, y: point_y };

    let double = point.double();
    let triple = point + double;

    assert(triple.x == res[0]);
    assert(triple.y == res[1]);

    // We test that brillig gives us the same result
    let brillig_res = get_brillig_result(point_x, point_y, scalar_low, scalar_high);
    assert(res[0] == brillig_res[0]);
    assert(res[1] == brillig_res[1]);

    // Multiplying the point by 1 should return the same point
    let res = std::scalar_mul::variable_base_embedded_curve(point_x, point_y, 1, 0);
    assert(point_x == res[0]);
    assert(point_y == res[1]);
}

unconstrained fn get_brillig_result(
    point_x: Field,
    point_y: Field,
    scalar_low: Field,
    scalar_high: Field
) -> [Field; 2] {
    std::scalar_mul::variable_base_embedded_curve(point_x, point_y, scalar_low, scalar_high)
}
