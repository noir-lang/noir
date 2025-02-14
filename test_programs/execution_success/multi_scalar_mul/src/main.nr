// This test provides a basic implementation of a MSM in Noir, that allows us to check
// performance improvements and regressions.
use std::embedded_curve_ops::embedded_curve_add_unsafe;
use std::embedded_curve_ops::EmbeddedCurvePoint;

// `main` must be marked unconstrained as the function uses `break` internally
unconstrained fn main(
    points: [EmbeddedCurvePoint; 5],
    scalars: [Field; 5],
) -> pub EmbeddedCurvePoint {
    double_then_add_msm(points, scalars)
}

unconstrained fn double_then_add_msm<let N: u32>(
    points: [EmbeddedCurvePoint; N],
    scalars: [Field; N],
) -> EmbeddedCurvePoint {
    // EmbeddedCurveScalar are two 128-bit numbers
    let mut acc = EmbeddedCurvePoint::point_at_infinity();
    for i in 0..N {
        // These should probably be EmbeddedCurveScalars
        // let full_scalar: Field = scalars[i].hi * 2.pow_32(128) + scalars[i].lo;
        let full_scalar = scalars[i];
        // If the scalar is zero we won't add anything to acc
        if full_scalar == 0 {
            continue;
        }
        let full_scalar_bits: [u1; 254] = full_scalar.to_be_bits();
        let mut index_of_msb = 0;
        // Iterates in BE
        for j in 0..254 {
            if full_scalar_bits[j] == 1 {
                index_of_msb = j;
                break;
            }
        }

        let mut temp = points[i];
        let mut res = points[i];
        // traversing from second MSB to LSB
        for j in (index_of_msb + 1)..(254) {
            // Double
            res = embedded_curve_add_unsafe(res, res);
            // Add
            if full_scalar_bits[j] == 1 {
                res = embedded_curve_add_unsafe(res, temp);
            }
        }

        acc = embedded_curve_add_unsafe(acc, res);
    }
    acc
}
