// Tests may be checked against https://github.com/cfrg/draft-irtf-cfrg-hash-to-curve/tree/main/poc
use std::ec::tecurve::affine::Curve as AffineCurve;
use std::ec::tecurve::affine::Point as Gaffine;
use std::ec::tecurve::curvegroup::Curve;
use std::ec::tecurve::curvegroup::Point as G;

use std::ec::swcurve::affine::Point as SWGaffine;
use std::ec::swcurve::curvegroup::Point as SWG;

use std::ec::montcurve::affine::Point as MGaffine;
use std::ec::montcurve::curvegroup::Point as MG;
use std::compat;

fn main() {
    // This test only makes sense if Field is the right prime field.
    if compat::is_bn254() {
        // Define Baby Jubjub (ERC-2494) parameters in affine representation
        let bjj_affine = AffineCurve::new(
            168700,
            168696,
            Gaffine::new(
                995203441582195749578291179787384436505546430278305826713579947235728471134,
                5472060717959818805561601436314318772137091100104008585924551046643952123905
            )
        );
        // Test addition
        let p1_affine = Gaffine::new(
            17777552123799933955779906779655732241715742912184938656739573121738514868268,
            2626589144620713026669568689430873010625803728049924121243784502389097019475
        );
        let p2_affine = Gaffine::new(
            16540640123574156134436876038791482806971768689494387082833631921987005038935,
            20819045374670962167435360035096875258406992893633759881276124905556507972311
        );

        let p3_affine = bjj_affine.add(p1_affine, p2_affine);
        assert(
            p3_affine.eq(
                Gaffine::new(
                    7916061937171219682591368294088513039687205273691143098332585753343424131937,
                    14035240266687799601661095864649209771790948434046947201833777492504781204499
                )
            )
        );
        // Test scalar multiplication
        let p4_affine = bjj_affine.mul(2, p1_affine);
        assert(
            p4_affine.eq(
                Gaffine::new(
                    6890855772600357754907169075114257697580319025794532037257385534741338397365,
                    4338620300185947561074059802482547481416142213883829469920100239455078257889
                )
            )
        );
        assert(p4_affine.eq(bjj_affine.bit_mul([0, 1], p1_affine)));
        // Test subtraction
        let p5_affine = bjj_affine.subtract(p3_affine, p3_affine);
        assert(p5_affine.eq(Gaffine::zero()));
        // Check that these points are on the curve
        assert(
            bjj_affine.contains(bjj_affine.gen)
            & bjj_affine.contains(p1_affine)
            & bjj_affine.contains(p2_affine)
            & bjj_affine.contains(p3_affine)
            & bjj_affine.contains(p4_affine)
            & bjj_affine.contains(p5_affine)
        );
        // Test CurveGroup equivalents
        let bjj = bjj_affine.into_group(); // Baby Jubjub
        let p1 = p1_affine.into_group();
        let p2 = p2_affine.into_group();
        let p3 = p3_affine.into_group();
        let p4 = p4_affine.into_group();
        let p5 = p5_affine.into_group();
        // Test addition
        assert(p3.eq(bjj.add(p1, p2)));
        // Test scalar multiplication
        assert(p4.eq(bjj.mul(2, p1)));
        assert(p4.eq(bjj.bit_mul([0, 1], p1)));
        // Test subtraction
        assert(G::zero().eq(bjj.subtract(p3, p3)));
        assert(p5.eq(G::zero()));
        // Check that these points are on the curve
        assert(
            bjj.contains(bjj.gen)
            & bjj.contains(p1)
            & bjj.contains(p2)
            & bjj.contains(p3)
            & bjj.contains(p4)
            & bjj.contains(p5)
        );
        // Test SWCurve equivalents of the above
        // First the affine representation
        let bjj_swcurve_affine = bjj_affine.into_swcurve();

        let p1_swcurve_affine = bjj_affine.map_into_swcurve(p1_affine);
        let p2_swcurve_affine = bjj_affine.map_into_swcurve(p2_affine);
        let p3_swcurve_affine = bjj_affine.map_into_swcurve(p3_affine);
        let p4_swcurve_affine = bjj_affine.map_into_swcurve(p4_affine);
        let p5_swcurve_affine = bjj_affine.map_into_swcurve(p5_affine);
        // Addition
        assert(p3_swcurve_affine.eq(bjj_swcurve_affine.add(p1_swcurve_affine, p2_swcurve_affine)));
        // Doubling
        assert(p4_swcurve_affine.eq(bjj_swcurve_affine.mul(2, p1_swcurve_affine)));
        assert(p4_swcurve_affine.eq(bjj_swcurve_affine.bit_mul([0, 1], p1_swcurve_affine)));
        // Subtraction
        assert(SWGaffine::zero().eq(bjj_swcurve_affine.subtract(p3_swcurve_affine, p3_swcurve_affine)));
        assert(p5_swcurve_affine.eq(SWGaffine::zero()));
        // Check that these points are on the curve
        assert(
            bjj_swcurve_affine.contains(bjj_swcurve_affine.gen)
            & bjj_swcurve_affine.contains(p1_swcurve_affine)
            & bjj_swcurve_affine.contains(p2_swcurve_affine)
            & bjj_swcurve_affine.contains(p3_swcurve_affine)
            & bjj_swcurve_affine.contains(p4_swcurve_affine)
            & bjj_swcurve_affine.contains(p5_swcurve_affine)
        );
        // Then the CurveGroup representation
        let bjj_swcurve = bjj.into_swcurve();

        let p1_swcurve = bjj.map_into_swcurve(p1);
        let p2_swcurve = bjj.map_into_swcurve(p2);
        let p3_swcurve = bjj.map_into_swcurve(p3);
        let p4_swcurve = bjj.map_into_swcurve(p4);
        let p5_swcurve = bjj.map_into_swcurve(p5);
        // Addition
        assert(p3_swcurve.eq(bjj_swcurve.add(p1_swcurve, p2_swcurve)));
        // Doubling
        assert(p4_swcurve.eq(bjj_swcurve.mul(2, p1_swcurve)));
        assert(p4_swcurve.eq(bjj_swcurve.bit_mul([0, 1], p1_swcurve)));
        // Subtraction
        assert(SWG::zero().eq(bjj_swcurve.subtract(p3_swcurve, p3_swcurve)));
        assert(p5_swcurve.eq(SWG::zero()));
        // Check that these points are on the curve
        assert(
            bjj_swcurve.contains(bjj_swcurve.gen)
            & bjj_swcurve.contains(p1_swcurve)
            & bjj_swcurve.contains(p2_swcurve)
            & bjj_swcurve.contains(p3_swcurve)
            & bjj_swcurve.contains(p4_swcurve)
            & bjj_swcurve.contains(p5_swcurve)
        );
        // Test MontCurve conversions
        // First the affine representation
        let bjj_montcurve_affine = bjj_affine.into_montcurve();

        let p1_montcurve_affine = p1_affine.into_montcurve();
        let p2_montcurve_affine = p2_affine.into_montcurve();
        let p3_montcurve_affine = p3_affine.into_montcurve();
        let p4_montcurve_affine = p4_affine.into_montcurve();
        let p5_montcurve_affine = p5_affine.into_montcurve();
        // Addition
        assert(p3_montcurve_affine.eq(bjj_montcurve_affine.add(p1_montcurve_affine, p2_montcurve_affine)));
        // Doubling
        assert(p4_montcurve_affine.eq(bjj_montcurve_affine.mul(2, p1_montcurve_affine)));
        assert(p4_montcurve_affine.eq(bjj_montcurve_affine.bit_mul([0, 1], p1_montcurve_affine)));
        // Subtraction
        assert(MGaffine::zero().eq(bjj_montcurve_affine.subtract(p3_montcurve_affine, p3_montcurve_affine)));
        assert(p5_montcurve_affine.eq(MGaffine::zero()));
        // Check that these points are on the curve
        assert(
            bjj_montcurve_affine.contains(bjj_montcurve_affine.gen)
            & bjj_montcurve_affine.contains(p1_montcurve_affine)
            & bjj_montcurve_affine.contains(p2_montcurve_affine)
            & bjj_montcurve_affine.contains(p3_montcurve_affine)
            & bjj_montcurve_affine.contains(p4_montcurve_affine)
            & bjj_montcurve_affine.contains(p5_montcurve_affine)
        );
        // Then the CurveGroup representation
        let bjj_montcurve = bjj.into_montcurve();

        let p1_montcurve = p1_montcurve_affine.into_group();
        let p2_montcurve = p2_montcurve_affine.into_group();
        let p3_montcurve = p3_montcurve_affine.into_group();
        let p4_montcurve = p4_montcurve_affine.into_group();
        let p5_montcurve = p5_montcurve_affine.into_group();
        // Addition
        assert(p3_montcurve.eq(bjj_montcurve.add(p1_montcurve, p2_montcurve)));
        // Doubling
        assert(p4_montcurve.eq(bjj_montcurve.mul(2, p1_montcurve)));
        assert(p4_montcurve.eq(bjj_montcurve.bit_mul([0, 1], p1_montcurve)));
        // Subtraction
        assert(MG::zero().eq(bjj_montcurve.subtract(p3_montcurve, p3_montcurve)));
        assert(p5_montcurve.eq(MG::zero()));
        // Check that these points are on the curve
        assert(
            bjj_montcurve.contains(bjj_montcurve.gen)
            & bjj_montcurve.contains(p1_montcurve)
            & bjj_montcurve.contains(p2_montcurve)
            & bjj_montcurve.contains(p3_montcurve)
            & bjj_montcurve.contains(p4_montcurve)
            & bjj_montcurve.contains(p5_montcurve)
        );
        // Elligator 2 map-to-curve
        let ell2_pt_map = bjj_affine.elligator2_map(27);

        assert(
            ell2_pt_map.eq(
                MGaffine::new(
                    7972459279704486422145701269802978968072470631857513331988813812334797879121,
                    8142420778878030219043334189293412482212146646099536952861607542822144507872
                ).into_tecurve()
            )
        );
        // SWU map-to-curve
        let swu_pt_map = bjj_affine.swu_map(5, 27);

        assert(
            swu_pt_map.eq(
                bjj_affine.map_from_swcurve(
                    SWGaffine::new(
                        2162719247815120009132293839392097468339661471129795280520343931405114293888,
                        5341392251743377373758788728206293080122949448990104760111875914082289313973
                    )
                )
            )
        );
    }
}
