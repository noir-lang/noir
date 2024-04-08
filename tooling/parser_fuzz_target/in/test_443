// Elliptic curve implementation
// Overview
// ========
// The following three elliptic curve representations are admissible:
mod tecurve; // Twisted Edwards curves
mod swcurve; // Elliptic curves in Short Weierstraß form
mod montcurve; // Montgomery curves
mod consts; // Commonly used curve presets
//
// Note that Twisted Edwards and Montgomery curves are (birationally) equivalent, so that
// they may be freely converted between one another, whereas Short Weierstraß curves are
// more general. Diagramatically:
//
// tecurve == montcurve ⊂ swcurve
//
// Each module is further divided into two submodules, 'affine' and 'curvegroup', depending
// on the preferred coordinate representation. Affine coordinates are none other than the usual
// two-dimensional Cartesian coordinates used in the definitions of these curves, whereas
// 'CurveGroup' coordinates (terminology borrowed from Arkworks, whose conventions we try
// to follow) are special coordinate systems with respect to which the group operations may be
// implemented more efficiently, usually by means of an appropriate choice of projective coordinates.
//
// In each of these submodules, there is a Point struct and a Curve struct, the former
// representing a point in the coordinate system and the latter a curve configuration.
//
// Points
// ======
// Points may be instantiated using the associated function `new`, which takes coordinates
// as its arguments. For instance,
//
// `let p = swcurve::Point::new(1,1);`
//
// The additive identity may be constructed by a call to the associated function `zero` of no
// arguments:
//
// `let zero = swcurve::Point::zero();`
//
// Points may be tested for equality by calling the method `eq`:
//
// `let pred = p.eq(zero);`
//
// There is also the method `is_zero` to explicitly check whether a point is the additive identity:
//
// `constrain pred == p.is_zero();`
//
// Points may be negated by calling the `negate` method and converted to CurveGroup (or affine)
// coordinates by calling the `into_group` (resp. `into_affine`) method on them. Finally,
// Points may be freely mapped between their respective Twisted Edwards and Montgomery
// representations by calling the `into_montcurve` or `into_tecurve` methods. For mappings
// between Twisted Edwards/Montgomery curves and Short Weierstraß curves, see the Curve section
// below, as the underlying mappings are those of curves rather than ambient spaces.
// As a rule, Points in affine (or CurveGroup) coordinates are mapped to Points in affine
// (resp. CurveGroup) coordinates.
//
// Curves
// ======
// A curve configuration (Curve) is completely determined by the Field coefficients of its defining
// equation (a and b in the case of swcurve, a and d in the case of tecurve, and j and k in
// the case of montcurve) together with a generator (`gen`) in the corresponding coordinate system.
// For example, the Baby Jubjub curve configuration as defined in ERC-2494 may be instantiated as a Twisted
// Edwards curve in affine coordinates as follows:
//
// `let bjj_affine = tecurve::Curve::new(168700, 168696, tecurve::Point::new(995203441582195749578291179787384436505546430278305826713579947235728471134,5472060717959818805561601436314318772137091100104008585924551046643952123905));`
//
// The `contains` method may be used to check whether a Point lies on a given curve:
//
// `constrain bjj_affine.contains(tecurve::Point::zero());`
//
// The elliptic curve group's addition operation is exposed as the `add` method, e.g.
//
// `let p = bjj_affine.add(bjj_affine.gen, bjj_affine.gen);`
//
// subtraction as the `subtract` method, e.g.
//
// `constrain tecurve::Point::zero().eq(bjj_affine.subtract(bjj_affine.gen, bjj_affine.gen));`
//
// scalar multiplication as the `mul` method, where the scalar is assumed to be a Field* element, e.g.
//
// `constrain tecurve::Point::zero().eq(bjj_affine.mul(2, tecurve::Point::zero());`
//
// There is a scalar multiplication method (`bit_mul`) provided where the scalar input is expected to be
// an array of bits (little-endian convention), as well as a multi-scalar multiplication method** (`msm`)
// which takes an array of Field elements and an array of elliptic curve points as arguments, both assumed
// to be of the same length.
//
// Curve configurations may be converted between different coordinate representations by calling the `into_group`
// and `into_affine` methods on them, e.g.
//
// `let bjj_curvegroup = bjj_affine.into_group();`
//
// Curve configurations may also be converted between different curve representations by calling the `into_swcurve`,
// `into_montcurve` and `into_tecurve` methods subject to the relation between the curve representations mentioned
// above. Note that it is possible to map Points from a Twisted Edwards/Montgomery curve to the corresponding
// Short Weierstraß representation and back, and the methods to do so are exposed as `map_into_swcurve` and
// `map_from_swcurve`, which each take one argument, the point to be mapped.
//
// Curve maps
// ==========
// There are a few different ways of mapping Field elements to elliptic curves. Here we provide the simplified
// Shallue-van de Woestijne-Ulas and Elligator 2 methods, the former being applicable to all curve types
// provided above subject to the constraint that the coefficients of the corresponding Short Weierstraß curve satisfies
// a*b != 0 and the latter being applicable to Montgomery and Twisted Edwards curves subject to the constraint that
// the coefficients of the corresponding Montgomery curve satisfy j*k != 0 and (j^2 - 4)/k^2 is non-square.
//
// The simplified Shallue-van de Woestijne-Ulas method is exposed as the method `swu_map` on the Curve configuration and
// depends on two parameters, a Field element z != -1 for which g(x) - z is irreducible over Field and g(b/(z*a)) is
// square, where g(x) = x^3 + a*x + b is the right-hand side of the defining equation of the corresponding Short
// Weierstraß curve, and a Field element u to be mapped onto the curve. For example, in the case of bjj_affine above,
// it may be determined using the scripts provided at <https://github.com/cfrg/draft-irtf-cfrg-hash-to-curve> that z = 5.
//
// The Elligator 2 method is exposed as the method `elligator2_map` on the Curve configurations of Montgomery and
// Twisted Edwards curves. Like the simplified SWU method above, it depends on a certain non-square element of Field,
// but this element need not satisfy any further conditions, so it is included as the (Field-dependent) constant
//`ZETA` below. Thus, the `elligator2_map` method depends only on one parameter, the Field element to be mapped onto
// the curve.
//
// For details on all of the above in the context of hashing to elliptic curves, see <https://datatracker.ietf.org/doc/id/draft-irtf-cfrg-hash-to-curve-06.html>.
// 
//
// *TODO: Replace Field with Bigint.
// **TODO: Support arrays of structs to make this work.
// Field-dependent constant ZETA = a non-square element of Field
// Required for Elligator 2 map
// TODO: Replace with built-in constant.
global ZETA = 5;
// Field-dependent constants for Tonelli-Shanks algorithm (see sqrt function below)
// TODO: Possibly make this built-in.
global C1 = 28;
global C3 = 40770029410420498293352137776570907027550720424234931066070132305055;
global C5 = 19103219067921713944291392827692070036145651957329286315305642004821462161904;
// Higher-order version of scalar multiplication
// TODO: Make this work so that the submodules' bit_mul may be defined in terms of it.
//fn bit_mul<T,N>(add: fn(T,T) -> T, e: T, bits: [u1; N], p: T) -> T {
//    let mut out = e;
//    let n = bits.len();
// 
//    for i in 0..n {
//        out = add(
//            add(out, out),
//            if(bits[n - i - 1] == 0) {e} else {p});
//    }
// 
//    out
//}
// TODO: Make this built-in.
pub fn safe_inverse(x: Field) -> Field {
    if x == 0 { 0 } else { 1 / x }
}
// Boolean indicating whether Field element is a square, i.e. whether there exists a y in Field s.t. x = y*y.
pub fn is_square(x: Field) -> bool {
    let v = pow(x, 0 - 1 / 2);

    v * (v - 1) == 0
}
// Power function of two Field arguments of arbitrary size.
// Adapted from std::field::pow_32.
pub fn pow(x: Field, y: Field) -> Field {
    // As in tests with minor modifications
    let N_BITS = crate::field::modulus_num_bits();

    let mut r = 1 as Field;
    let b = y.to_le_bits(N_BITS as u32);

    for i in 0..N_BITS {
        r *= r;
        r *= (b[N_BITS - 1 - i] as Field)*x + (1-b[N_BITS - 1 - i] as Field);
    }

    r
}
// Tonelli-Shanks algorithm for computing the square root of a Field element.
// Requires C1 = max{c: 2^c divides (p-1)}, where p is the order of Field
// as well as C3 = (C2 - 1)/2, where C2 = (p-1)/(2^c1),
// and C5 = ZETA^C2, where ZETA is a non-square element of Field.
// These are pre-computed above as globals.
pub fn sqrt(x: Field) -> Field {
    let mut z = pow(x, C3);
    let mut t = z * z * x;
    z *= x;
    let mut b = t;
    let mut c = C5;

    for i in 0..(C1 - 1) {
        for _j in 1..(C1 - i - 1) {
            b *= b;
        }

        z *= if b == 1 { 1 } else { c };

        c *= c;

        t *= if b == 1 { 1 } else { c };

        b = t;
    }

    z
}
