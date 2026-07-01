use crate::cmp::Eq;
use crate::hash::Hash;
use crate::ops::arith::{Add, Neg, Sub};

/// A point on the embedded elliptic curve
/// By definition, the base field of the embedded curve is the scalar field of the proof system curve, i.e the Noir Field.
/// x and y denotes the Weierstrass coordinates of the point.
pub struct EmbeddedCurvePoint {
    pub x: Field,
    pub y: Field,
}

impl EmbeddedCurvePoint {
    /// Create a new point using the provided (x, y) pair
    pub fn new(x: Field, y: Field) -> Self {
        EmbeddedCurvePoint { x, y }
    }

    /// Elliptic curve point doubling operation
    /// returns the doubled point of a point P, i.e P+P
    pub fn double(self) -> EmbeddedCurvePoint {
        embedded_curve_add(self, self)
    }

    /// Returns the null element of the curve; 'the point at infinity'
    pub fn point_at_infinity() -> EmbeddedCurvePoint {
        EmbeddedCurvePoint { x: 0, y: 0 }
    }

    /// Returns the curve's generator point.
    pub fn generator() -> EmbeddedCurvePoint {
        // Generator point for the grumpkin curve (y^2 = x^3 - 17)
        EmbeddedCurvePoint {
            x: 1,
            y: 17631683881184975370165255887551781615748388533673675138860, // sqrt(-16)
        }
    }

    /// True if this point is the point at infinity
    pub fn is_infinite(self) -> bool {
        (self.x == 0) & (self.y == 0)
    }
}

impl Add for EmbeddedCurvePoint {
    /// Adds two points P+Q, using the curve addition formula, and also handles point at infinity
    fn add(self, other: EmbeddedCurvePoint) -> EmbeddedCurvePoint {
        embedded_curve_add(self, other)
    }
}

impl Sub for EmbeddedCurvePoint {
    /// Points subtraction operation, using addition and negation
    fn sub(self, other: EmbeddedCurvePoint) -> EmbeddedCurvePoint {
        self + other.neg()
    }
}

impl Neg for EmbeddedCurvePoint {
    /// Negates a point P, i.e returns -P, by negating the y coordinate.
    /// If the point is at infinity, then the result is also at infinity.
    fn neg(self) -> EmbeddedCurvePoint {
        EmbeddedCurvePoint { x: self.x, y: -self.y }
    }
}

impl Eq for EmbeddedCurvePoint {
    /// Checks whether two points are equal
    fn eq(self: Self, b: EmbeddedCurvePoint) -> bool {
        (self.x == b.x) & (self.y == b.y)
    }
}

impl Hash for EmbeddedCurvePoint {
    fn hash<H>(self, state: &mut H)
    where
        H: crate::hash::Hasher,
    {
        self.x.hash(state);
        self.y.hash(state);
    }
}

/// Scalar for the embedded curve represented as low and high limbs
/// By definition, the scalar field of the embedded curve is base field of the proving system curve.
/// It may not fit into a Field element, so it is represented with two Field elements; its low and high limbs.
pub struct EmbeddedCurveScalar {
    pub lo: Field,
    pub hi: Field,
}

impl EmbeddedCurveScalar {
    /// Create a new scalar using the provided (lo, hi) pair
    pub fn new(lo: Field, hi: Field) -> Self {
        EmbeddedCurveScalar { lo, hi }
    }

    /// Create a scalar from the given bn254 field value
    #[field(bn254)]
    pub fn from_field(scalar: Field) -> EmbeddedCurveScalar {
        let (a, b) = crate::field::bn254::decompose(scalar);
        EmbeddedCurveScalar { lo: a, hi: b }
    }
}

impl Eq for EmbeddedCurveScalar {
    fn eq(self, other: Self) -> bool {
        (other.hi == self.hi) & (other.lo == self.lo)
    }
}

impl Hash for EmbeddedCurveScalar {
    fn hash<H>(self, state: &mut H)
    where
        H: crate::hash::Hasher,
    {
        self.hi.hash(state);
        self.lo.hash(state);
    }
}

/// Computes a multi scalar multiplication over the embedded curve.
/// For bn254, We have Grumpkin.
///
/// The embedded curve being used is decided by the
/// underlying proof system.
///
/// IMPORTANT: Prefer `multi_scalar_mul()` over repeated `embedded_curve_add()`
/// for adding multiple points. This is significantly more efficient.
/// For adding exactly 2 points, use `embedded_curve_add()` directly.
// docs:start:multi_scalar_mul
pub fn multi_scalar_mul<let N: u32>(
    points: [EmbeddedCurvePoint; N],
    scalars: [EmbeddedCurveScalar; N],
) -> EmbeddedCurvePoint
// docs:end:multi_scalar_mul
{
    multi_scalar_mul_array_return(points, scalars, true)[0]
}

// docs:start:fixed_base_scalar_mul
pub fn fixed_base_scalar_mul(scalar: EmbeddedCurveScalar) -> EmbeddedCurvePoint
// docs:end:fixed_base_scalar_mul
{
    multi_scalar_mul([EmbeddedCurvePoint::generator()], [scalar])
}

#[foreign(multi_scalar_mul)]
pub(crate) fn multi_scalar_mul_array_return<let N: u32>(
    points: [EmbeddedCurvePoint; N],
    scalars: [EmbeddedCurveScalar; N],
    predicate: bool,
) -> [EmbeddedCurvePoint; 1] {}

/// Elliptic curve addition
/// IMPORTANT: this function is expected to perform a full addition in order to handle all corner cases:
/// - points on the curve
/// - point doubling
/// - point at infinity
/// As a result, you may not get optimal performance, depending on the assumptions of your inputs.
// docs:start:embedded_curve_add
pub fn embedded_curve_add(
    point1: EmbeddedCurvePoint,
    point2: EmbeddedCurvePoint,
) -> EmbeddedCurvePoint {
    // docs:end:embedded_curve_add
    embedded_curve_add_array_return(point1, point2, true)[0]
}

#[foreign(embedded_curve_add)]
fn embedded_curve_add_array_return(
    _point1: EmbeddedCurvePoint,
    _point2: EmbeddedCurvePoint,
    _predicate: bool,
) -> [EmbeddedCurvePoint; 1] {}

mod tests {
    // TODO: Allow imports from "super"
    use crate::default::Default;
    use crate::embedded_curve_ops::{
        embedded_curve_add, EmbeddedCurvePoint, EmbeddedCurveScalar, fixed_base_scalar_mul,
        multi_scalar_mul,
    };
    use crate::field::bn254::TWO_POW_128;
    use crate::hash::Hash;
    use crate::hash::Hasher;
    use crate::hash::poseidon2::Poseidon2Hasher;

    fn hash_point(p: EmbeddedCurvePoint) -> Field {
        let mut hasher: Poseidon2Hasher = Default::default();
        p.hash(&mut hasher);
        hasher.finish()
    }

    fn hash_scalar(s: EmbeddedCurveScalar) -> Field {
        let mut hasher: Poseidon2Hasher = Default::default();
        s.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn point_new_sets_coordinates() {
        let p = EmbeddedCurvePoint::new(3, 5);
        assert_eq(p.x, 3);
        assert_eq(p.y, 5);
    }

    #[test]
    fn point_at_infinity_is_origin() {
        let inf = EmbeddedCurvePoint::point_at_infinity();
        assert_eq(inf.x, 0);
        assert_eq(inf.y, 0);
    }

    #[test]
    fn generator_has_documented_coordinates() {
        let g = EmbeddedCurvePoint::generator();
        assert_eq(g.x, 1);
        assert_eq(g.y, 17631683881184975370165255887551781615748388533673675138860);
    }

    #[test]
    fn is_infinite_only_true_for_origin() {
        assert(EmbeddedCurvePoint::point_at_infinity().is_infinite());
        assert(!EmbeddedCurvePoint::generator().is_infinite());
        assert(!EmbeddedCurvePoint::new(1, 0).is_infinite());
        assert(!EmbeddedCurvePoint::new(0, 1).is_infinite());
    }

    #[test]
    fn points_with_same_coords_are_equal() {
        let p = EmbeddedCurvePoint::new(7, 11);
        let q = EmbeddedCurvePoint::new(7, 11);
        assert_eq(p, q);
    }

    #[test]
    fn points_with_different_coords_are_unequal() {
        let p = EmbeddedCurvePoint::new(7, 11);
        assert(p != EmbeddedCurvePoint::new(7, 12));
        assert(p != EmbeddedCurvePoint::new(8, 11));
        assert(p != EmbeddedCurvePoint::new(8, 12));
    }

    #[test]
    fn neg_negates_y_coordinate() {
        let g = EmbeddedCurvePoint::generator();
        let neg_g = -g;
        assert_eq(neg_g.x, g.x);
        assert_eq(neg_g.y, -g.y);
    }

    #[test]
    fn neg_is_involution() {
        let g = EmbeddedCurvePoint::generator();
        assert_eq(--g, g);
    }

    #[test]
    fn add_with_point_at_infinity_is_identity() {
        let g = EmbeddedCurvePoint::generator();
        let inf = EmbeddedCurvePoint::point_at_infinity();
        assert_eq(g + inf, g);
        assert_eq(inf + g, g);
    }

    #[test]
    fn add_of_two_infinities_is_infinity() {
        let inf = EmbeddedCurvePoint::point_at_infinity();
        assert((inf + inf).is_infinite());
    }

    #[test]
    fn add_with_negation_is_point_at_infinity() {
        let g = EmbeddedCurvePoint::generator();
        assert((g + (-g)).is_infinite());
    }

    #[test]
    fn add_is_commutative() {
        let g = EmbeddedCurvePoint::generator();
        let g2 = g.double();
        assert_eq(g + g2, g2 + g);
    }

    #[test]
    fn add_is_associative() {
        let g = EmbeddedCurvePoint::generator();
        let g2 = g.double();
        let g3 = g + g2;
        assert_eq((g + g2) + g3, g + (g2 + g3));
    }

    #[test]
    fn sub_of_a_point_with_itself_is_infinity() {
        let g = EmbeddedCurvePoint::generator();
        assert((g - g).is_infinite());
    }

    #[test]
    fn sub_with_point_at_infinity_is_identity() {
        let g = EmbeddedCurvePoint::generator();
        let inf = EmbeddedCurvePoint::point_at_infinity();
        assert_eq(g - inf, g);
    }

    #[test]
    fn sub_inverts_add() {
        let g = EmbeddedCurvePoint::generator();
        let g2 = g.double();
        assert_eq((g + g2) - g2, g);
        assert_eq((g + g2) - g, g2);
    }

    #[test]
    fn double_equals_self_plus_self() {
        let g = EmbeddedCurvePoint::generator();
        assert_eq(g.double(), g + g);
    }

    #[test]
    fn double_of_point_at_infinity_is_infinity() {
        assert(EmbeddedCurvePoint::point_at_infinity().double().is_infinite());
    }

    #[test]
    fn embedded_curve_add_matches_add_operator() {
        let g = EmbeddedCurvePoint::generator();
        let g2 = g.double();
        assert_eq(embedded_curve_add(g, g2), g + g2);
    }

    #[test]
    fn point_hash_is_consistent_for_equal_points() {
        let p = EmbeddedCurvePoint::new(3, 5);
        let q = EmbeddedCurvePoint::new(3, 5);
        assert_eq(hash_point(p), hash_point(q));
    }

    #[test]
    fn point_hash_differs_for_distinct_points() {
        let p = EmbeddedCurvePoint::new(3, 5);
        assert(hash_point(p) != hash_point(EmbeddedCurvePoint::new(3, 7)));
        assert(hash_point(p) != hash_point(EmbeddedCurvePoint::new(4, 5)));
        // Distinguishes (x, y) from (y, x) - order of writes matters.
        assert(hash_point(p) != hash_point(EmbeddedCurvePoint::new(5, 3)));
    }

    #[test]
    fn scalar_new_sets_limbs() {
        let s = EmbeddedCurveScalar::new(3, 5);
        assert_eq(s.lo, 3);
        assert_eq(s.hi, 5);
    }

    #[test]
    fn scalars_with_same_limbs_are_equal() {
        let s = EmbeddedCurveScalar::new(7, 11);
        let t = EmbeddedCurveScalar::new(7, 11);
        assert_eq(s, t);
    }

    #[test]
    fn scalars_with_different_limbs_are_unequal() {
        let s = EmbeddedCurveScalar::new(7, 11);
        assert(s != EmbeddedCurveScalar::new(7, 12));
        assert(s != EmbeddedCurveScalar::new(8, 11));
    }

    #[test]
    fn scalar_from_field_decomposes_zero() {
        assert_eq(EmbeddedCurveScalar::from_field(0), EmbeddedCurveScalar::new(0, 0));
    }

    #[test]
    fn scalar_from_field_decomposes_small_value() {
        let s = EmbeddedCurveScalar::from_field(0x1234567890);
        assert_eq(s, EmbeddedCurveScalar::new(0x1234567890, 0));
    }

    #[test]
    fn scalar_from_field_decomposes_two_pow_128() {
        let s = EmbeddedCurveScalar::from_field(TWO_POW_128);
        assert_eq(s, EmbeddedCurveScalar::new(0, 1));
    }

    #[test]
    fn scalar_hash_is_consistent_for_equal_scalars() {
        let s = EmbeddedCurveScalar::new(7, 11);
        let t = EmbeddedCurveScalar::new(7, 11);
        assert_eq(hash_scalar(s), hash_scalar(t));
    }

    #[test]
    fn scalar_hash_differs_for_distinct_scalars() {
        let s = EmbeddedCurveScalar::new(7, 11);
        assert(hash_scalar(s) != hash_scalar(EmbeddedCurveScalar::new(7, 12)));
        assert(hash_scalar(s) != hash_scalar(EmbeddedCurveScalar::new(8, 11)));
        // Distinguishes (lo, hi) from (hi, lo) - limb order matters.
        assert(hash_scalar(s) != hash_scalar(EmbeddedCurveScalar::new(11, 7)));
    }

    #[test]
    fn msm_with_scalar_one_returns_point() {
        let g = EmbeddedCurvePoint::generator();
        let one = EmbeddedCurveScalar::new(1, 0);
        assert_eq(multi_scalar_mul([g], [one]), g);
    }

    #[test]
    fn msm_with_scalar_zero_returns_infinity() {
        let g = EmbeddedCurvePoint::generator();
        let zero = EmbeddedCurveScalar::new(0, 0);
        assert(multi_scalar_mul([g], [zero]).is_infinite());
    }

    #[test]
    fn msm_with_scalar_two_doubles_point() {
        let g = EmbeddedCurvePoint::generator();
        let two = EmbeddedCurveScalar::new(2, 0);
        assert_eq(multi_scalar_mul([g], [two]), g.double());
    }

    #[test]
    fn msm_sums_terms() {
        let g = EmbeddedCurvePoint::generator();
        let one = EmbeddedCurveScalar::new(1, 0);
        assert_eq(multi_scalar_mul([g, g], [one, one]), g.double());
    }

    #[test]
    fn msm_with_opposite_terms_cancels() {
        let g = EmbeddedCurvePoint::generator();
        let one = EmbeddedCurveScalar::new(1, 0);
        assert(multi_scalar_mul([g, -g], [one, one]).is_infinite());
    }

    #[test]
    fn msm_skips_point_at_infinity() {
        let g = EmbeddedCurvePoint::generator();
        let inf = EmbeddedCurvePoint::point_at_infinity();
        let one = EmbeddedCurveScalar::new(1, 0);
        assert_eq(multi_scalar_mul([inf, g], [one, one]), g);
    }

    #[test]
    fn msm_skips_zero_scalar() {
        let g = EmbeddedCurvePoint::generator();
        let one = EmbeddedCurveScalar::new(1, 0);
        let zero = EmbeddedCurveScalar::new(0, 0);
        assert_eq(multi_scalar_mul([g, g], [zero, one]), g);
    }

    #[test]
    fn fixed_base_scalar_mul_with_one_returns_generator() {
        let one = EmbeddedCurveScalar::new(1, 0);
        assert_eq(fixed_base_scalar_mul(one), EmbeddedCurvePoint::generator());
    }

    #[test]
    fn fixed_base_scalar_mul_with_zero_returns_infinity() {
        let zero = EmbeddedCurveScalar::new(0, 0);
        assert(fixed_base_scalar_mul(zero).is_infinite());
    }

    #[test]
    fn fixed_base_scalar_mul_matches_msm_on_generator() {
        let s = EmbeddedCurveScalar::new(2, 0);
        assert_eq(
            fixed_base_scalar_mul(s),
            multi_scalar_mul([EmbeddedCurvePoint::generator()], [s]),
        );
    }
}
