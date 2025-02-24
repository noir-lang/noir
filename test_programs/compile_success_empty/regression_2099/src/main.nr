use ec::tecurve::affine::Curve as AffineCurve;
use ec::tecurve::affine::Point as Gaffine;

fn main() {
    // Define Baby Jubjub (ERC-2494) parameters in affine representation
    let bjj_affine = AffineCurve::new(
        168700,
        168696,
        Gaffine::new(
            995203441582195749578291179787384436505546430278305826713579947235728471134,
            5472060717959818805561601436314318772137091100104008585924551046643952123905,
        ),
    );
    // Test addition
    let p1_affine = Gaffine::new(
        17777552123799933955779906779655732241715742912184938656739573121738514868268,
        2626589144620713026669568689430873010625803728049924121243784502389097019475,
    );
    let p2_affine = Gaffine::new(
        16540640123574156134436876038791482806971768689494387082833631921987005038935,
        20819045374670962167435360035096875258406992893633759881276124905556507972311,
    );
    let _p3_affine = bjj_affine.add(p1_affine, p2_affine);
    // Test SWCurve equivalents of the above
    // First the affine representation
    let bjj_swcurve_affine = bjj_affine.into_swcurve();

    let p1_swcurve_affine = bjj_affine.map_into_swcurve(p1_affine);
    let p2_swcurve_affine = bjj_affine.map_into_swcurve(p2_affine);

    let _p3_swcurve_affine_from_add = bjj_swcurve_affine.add(p1_swcurve_affine, p2_swcurve_affine);
    // Check that these points are on the curve
    assert(bjj_swcurve_affine.contains(p1_swcurve_affine));
}
