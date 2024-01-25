use dep::std;

fn main(
    a: Field,
    a_pub_x: pub Field,
    a_pub_y: pub Field,
    b: Field,
    b_pub_x: pub Field,
    b_pub_y: pub Field
) {
    let mut priv_key = a;
    let mut pub_x: Field = a_pub_x;
    let mut pub_y: Field = a_pub_y;
    if a != 1 {
        // Change `a` in Prover.toml to test input `b`
        priv_key = b;
        pub_x = b_pub_x;
        pub_y = b_pub_y;
    }
    let res = std::scalar_mul::fixed_base_embedded_curve(priv_key, 0);
    assert(res[0] == pub_x);
    assert(res[1] == pub_y);
    let pub_point= std::scalar_mul::EmbeddedCurvePoint {
        x: pub_x,
        y: pub_y
    };
    let g1_y = 17631683881184975370165255887551781615748388533673675138860;
    let g1= std::scalar_mul::EmbeddedCurvePoint {
        x: 1,
        y: g1_y
    };

    let res = std::scalar_mul::embedded_curve_double(pub_point);
    let double = std::scalar_mul::embedded_curve_add(g1,g1 );

    assert(double.x == res.x);
}
