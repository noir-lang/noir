use crate::hash::poseidon;
use crate::ec::tecurve::affine::Point as TEPoint;
use crate::ec::tecurve::affine::Curve as AffineCurve;

fn eddsa_poseidon_verify(
    pub_key_x: Field,
    pub_key_y: Field,
    signature_s: Field,
    signature_r8_x: Field,
    signature_r8_y: Field,
    message: Field,
) -> bool {
    // Verifies by testing:
    // S * G8 = R8 + H(R8, A, m) * A8

    // TODO: lift bjj into shared const/config module
    // Define Baby Jubjub (ERC-2494) parameters in affine representation
    let bjj = AffineCurve::new(
        168700,
        168696,
        TEPoint::new(
            995203441582195749578291179787384436505546430278305826713579947235728471134,
            5472060717959818805561601436314318772137091100104008585924551046643952123905
        ),
    );

    let pub_key = TEPoint::new(pub_key_x, pub_key_y);
    constrain bjj.contains(pub_key);

    let signature_r8 = TEPoint::new(signature_r8_x, signature_r8_y);
    constrain bjj.contains(signature_r8);


    // Ensure S<Subgroup Order
    // TODO: document and possibily lift constant
    constrain signature_s <= 2736030358979909402780800718157159386076813972158567259200215660948447373040;

    // Calculate the h = H(R,A, msg)
    let hash = poseidon::bn254::sponge([
        signature_r8_x,
        signature_r8_y,
        pub_key_x,
        pub_key_y,
        message,
    ]);

    // Calculate second part of the right side:  right2 = h*8*A

    // Multiply by 8 by doubling 3 times. This also ensures that the result is in the subgroup.
    let pub_key_mul_2 = bjj.add(pub_key, pub_key);
    let pub_key_mul_4 = bjj.add(pub_key_mul_2, pub_key_mul_2);
    let pub_key_mul_8 = bjj.add(pub_key_mul_4, pub_key_mul_4);

    // We check that A8 is not zero.
    constrain !pub_key_mul_8.is_zero();

    // Compute the right side: R8 + h * A8
    let right = bjj.add(signature_r8, bjj.mul(hash, pub_key_mul_8));

    // Calculate left side of equation left = S * G8

    // TODO: document and possibily lift constant
    let base8 = TEPoint::new(
        5299619240641551281634865583518297030282874472190772894086521144482721001553,
        16950150798460657717958625567821834550301663161624707787222815936182638968203
    );
    let left = bjj.mul(signature_s, base8);

    left.eq(right)
}
