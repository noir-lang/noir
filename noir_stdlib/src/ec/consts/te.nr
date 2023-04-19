use crate::compat;
use crate::ec::tecurve::affine::Point as TEPoint;
use crate::ec::tecurve::affine::Curve as TECurve;

struct BabyJubjub {
    curve: TECurve,
    base8: TEPoint,
    suborder: Field,
}

fn baby_jubjub() -> BabyJubjub {
    constrain compat::is_bn254();

    BabyJubjub {
        // Baby Jubjub (ERC-2494) parameters in affine representation
        curve: TECurve::new(
            168700,
            168696,
            TEPoint::new(
                5299619240641551281634865583518297030282874472190772894086521144482721001553,
                16950150798460657717958625567821834550301663161624707787222815936182638968203,
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
