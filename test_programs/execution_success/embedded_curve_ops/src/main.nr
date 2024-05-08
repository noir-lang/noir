use dep::std;

fn main(priv_key: Field, pub_x: pub Field, pub_y: pub Field) {
    let g1_y = 17631683881184975370165255887551781615748388533673675138860;
    let g1 = std::embedded_curve_ops::EmbeddedCurvePoint { x: 1, y: g1_y };

    // Test that multi_scalar_mul correctly derives the public key
    let res = std::embedded_curve_ops::multi_scalar_mul([g1.x, g1.y], [priv_key, 0]);
    assert(res[0] == pub_x);
    assert(res[1] == pub_y);

    // Test that double function calling embedded_curve_add works as expected
    let pub_point = std::embedded_curve_ops::EmbeddedCurvePoint { x: pub_x, y: pub_y };
    let res = pub_point.double();
    let double = g1.add(g1);

    assert(double.x == res.x);

    // Test calling multi_scalar_mul with multiple points and scalars
    let res = std::embedded_curve_ops::multi_scalar_mul([g1.x, g1.y, g1.x, g1.y], [priv_key, 0, priv_key, 0]);

    // The results should be double the g1 point because the scalars are 1 and we pass in g1 twice
    assert(double.x == res[0]);
}
