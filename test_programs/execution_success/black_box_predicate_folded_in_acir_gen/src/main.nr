// `x + FIVE == x` is not simplified in SSA but folds to a constant `false` during ACIR gen.
// Every blackbox which takes a predicate is called under that guard, so each call is
// statically disabled and must not reach the backend with a constant zero predicate.
use std::embedded_curve_ops::{
    embedded_curve_add, EmbeddedCurvePoint, EmbeddedCurveScalar, fixed_base_scalar_mul,
    multi_scalar_mul,
};

global FIVE: Field = 5;

fn main(x: Field) -> pub Field {
    let mut r = x;
    if x + FIVE == x {
        r += fixed_base_scalar_mul(EmbeddedCurveScalar::from_field(x)).x;
        r += multi_scalar_mul(
            [EmbeddedCurvePoint::generator()],
            [EmbeddedCurveScalar::from_field(x)],
        )
            .x;
        r += std::hash::pedersen_commitment([x]).x;
        r += embedded_curve_add(
            EmbeddedCurvePoint::generator(),
            EmbeddedCurvePoint { x, y: x },
        )
            .x;

        let bytes: [u8; 32] = x.to_be_bytes();
        let signature = [0; 64];
        if std::ecdsa_secp256k1::verify_signature(bytes, bytes, signature, bytes) {
            r += 1;
        }
        if std::ecdsa_secp256r1::verify_signature(bytes, bytes, signature, bytes) {
            r += 1;
        }

        // Recursive aggregation has no Brillig implementation.
        if !std::runtime::is_unconstrained() {
            std::verify_proof_with_type([x], [x], [x], x, 1);
        }
    }
    r
}
