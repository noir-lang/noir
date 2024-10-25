use crate::ec::tecurve::affine::Curve as TECurve;
use crate::ec::tecurve::affine::Point as TEPoint;

pub struct BabyJubjub {
    pub curve: TECurve,
    pub base8: TEPoint,
    pub suborder: Field,
}

#[field(bn254)]
// Uncommenting this results in deprecated warnings in the stdlib
// #[deprecated]
pub fn baby_jubjub() -> BabyJubjub {
    BabyJubjub {
        // Baby Jubjub (ERC-2494) parameters in affine representation
        curve: TECurve::new(
            168700,
            168696,
            // G
            TEPoint::new(
                995203441582195749578291179787384436505546430278305826713579947235728471134,
                5472060717959818805561601436314318772137091100104008585924551046643952123905,
            ),
        ),
        // [8]G precalculated
        base8: TEPoint::new(
            5299619240641551281634865583518297030282874472190772894086521144482721001553,
            16950150798460657717958625567821834550301663161624707787222815936182638968203,
        ),
        // The size of the group formed from multiplying the base field by 8.
        suborder: 2736030358979909402780800718157159386076813972158567259200215660948447373041,
    }
}
