// This test provides a basic implementation of a MSM in Noir, that allows us to check
// performance improvements and regressions.
use std::embedded_curve_ops::embedded_curve_add_unsafe;
use std::embedded_curve_ops::EmbeddedCurvePoint;

// `main` must be marked unconstrained as the function uses `break` internally
unconstrained fn main(
    points: [EmbeddedCurvePoint; 5],
    scalars: [Field; 5],
) -> pub EmbeddedCurvePoint {
    // EmbeddedCurveScalar are two 128-bit numbers
    let mut acc = EmbeddedCurvePoint::point_at_infinity();
    for i in 0..1 {
        // These should probably be EmbeddedCurveScalars
        // let full_scalar: Field = scalars[i].hi * 2.pow_32(128) + scalars[i].lo;
        let full_scalar = scalars[i];
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
        let mut res = EmbeddedCurvePoint::point_at_infinity();
        // When iterative backwards we want to go to bits.len() - 2
        for j in 0..(254 - index_of_msb) {
            let k = 253 - j;

            // Add
            if full_scalar_bits[k] == 1 {
                res = embedded_curve_add_unsafe(res, temp);
            }
            // Double
            temp = embedded_curve_add_unsafe(temp, temp);
        }

        acc = embedded_curve_add_unsafe(acc, res);
    }
    acc
}

