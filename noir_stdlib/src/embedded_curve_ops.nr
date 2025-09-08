use crate::cmp::Eq;
use crate::hash::Hash;
use crate::ops::arith::{Add, Neg, Sub};

/// A point on the embedded elliptic curve
/// By definition, the base field of the embedded curve is the scalar field of the proof system curve, i.e the Noir Field.
/// x and y denotes the Weierstrass coordinates of the point, if is_infinite is false.
pub struct EmbeddedCurvePoint {
    pub x: Field,
    pub y: Field,
    pub is_infinite: bool,
}

impl EmbeddedCurvePoint {
    /// Elliptic curve point doubling operation
    /// returns the doubled point of a point P, i.e P+P
    pub fn double(self) -> EmbeddedCurvePoint {
        embedded_curve_add(self, self)
    }

    /// Returns the null element of the curve; 'the point at infinity'
    pub fn point_at_infinity() -> EmbeddedCurvePoint {
        EmbeddedCurvePoint { x: 0, y: 0, is_infinite: true }
    }

    /// Returns the curve's generator point.
    pub fn generator() -> EmbeddedCurvePoint {
        // Generator point for the grumpkin curve (y^2 = x^3 - 17)
        EmbeddedCurvePoint {
            x: 1,
            y: 17631683881184975370165255887551781615748388533673675138860, // sqrt(-16)
            is_infinite: false,
        }
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
        EmbeddedCurvePoint { x: self.x, y: -self.y, is_infinite: self.is_infinite }
    }
}

impl Eq for EmbeddedCurvePoint {
    /// Checks whether two points are equal
    fn eq(self: Self, b: EmbeddedCurvePoint) -> bool {
        (self.is_infinite & b.is_infinite)
            | ((self.is_infinite == b.is_infinite) & (self.x == b.x) & (self.y == b.y))
    }
}

impl Hash for EmbeddedCurvePoint {
    fn hash<H>(self, state: &mut H)
    where
        H: crate::hash::Hasher,
    {
        if self.is_infinite {
            self.is_infinite.hash(state);
        } else {
            self.x.hash(state);
            self.y.hash(state);
        }
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
    pub fn new(lo: Field, hi: Field) -> Self {
        EmbeddedCurveScalar { lo, hi }
    }

    #[field(bn254)]
    pub fn from_field(scalar: Field) -> EmbeddedCurveScalar {
        let (a, b) = crate::field::bn254::decompose(scalar);
        EmbeddedCurveScalar { lo: a, hi: b }
    }

    //Bytes to scalar: take the first (after the specified offset) 16 bytes of the input as the lo value, and the next 16 bytes as the hi value
    #[field(bn254)]
    pub(crate) fn from_bytes(bytes: [u8; 64], offset: u32) -> EmbeddedCurveScalar {
        let mut v = 1;
        let mut lo = 0 as Field;
        let mut hi = 0 as Field;
        for i in 0..16 {
            lo = lo + (bytes[offset + 31 - i] as Field) * v;
            hi = hi + (bytes[offset + 15 - i] as Field) * v;
            v = v * 256;
        }
        let sig_s = crate::embedded_curve_ops::EmbeddedCurveScalar { lo, hi };
        sig_s
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

// Computes a multi scalar multiplication over the embedded curve.
// For bn254, We have Grumpkin and Baby JubJub.
// For bls12-381, we have JubJub and Bandersnatch.
//
// The embedded curve being used is decided by the
// underlying proof system.
// docs:start:multi_scalar_mul
pub fn multi_scalar_mul<let N: u32>(
    points: [EmbeddedCurvePoint; N],
    scalars: [EmbeddedCurveScalar; N],
) -> EmbeddedCurvePoint
// docs:end:multi_scalar_mul
{
    multi_scalar_mul_array_return(points, scalars)[0]
}

#[foreign(multi_scalar_mul)]
pub(crate) fn multi_scalar_mul_array_return<let N: u32>(
    points: [EmbeddedCurvePoint; N],
    scalars: [EmbeddedCurveScalar; N],
) -> [EmbeddedCurvePoint; 1] {}

// docs:start:fixed_base_scalar_mul
pub fn fixed_base_scalar_mul(scalar: EmbeddedCurveScalar) -> EmbeddedCurvePoint
// docs:end:fixed_base_scalar_mul
{
    multi_scalar_mul([EmbeddedCurvePoint::generator()], [scalar])
}

/// This function only assumes that the points are on the curve
/// It handles corner cases around the infinity point causing some overhead compared to embedded_curve_add_not_nul and embedded_curve_add_unsafe
// docs:start:embedded_curve_add
pub fn embedded_curve_add(
    point1: EmbeddedCurvePoint,
    point2: EmbeddedCurvePoint,
) -> EmbeddedCurvePoint {
    // docs:end:embedded_curve_add
    if crate::runtime::is_unconstrained() {
        // `embedded_curve_add_unsafe` requires the inputs not to be the infinity point, so we check it here.
        // This is because `embedded_curve_add_unsafe` uses the `embedded_curve_add` opcode.
        // For efficiency, the backend does not check the inputs for the infinity point, but it assumes that they are not the infinity point
        // so that it can apply the ec addition formula directly.
        if point1.is_infinite {
            point2
        } else if point2.is_infinite {
            point1
        } else {
            embedded_curve_add_unsafe(point1, point2)
        }
    } else {
        // In a constrained context, we also need to check the inputs are not the infinity point because we also use `embedded_curve_add_unsafe`
        // However we also need to identify the case where the two inputs are the same, because then
        // the addition formula does not work and we need to use the doubling formula instead.
        // In unconstrained context, we can check directly if the input values are the same when solving the opcode, so it is not an issue.

        // x_coordinates_match is true if both abscissae are the same
        let x_coordinates_match = point1.x == point2.x;
        // y_coordinates_match is true if both ordinates are the same
        let y_coordinates_match = point1.y == point2.y;
        // double_predicate is true if both abscissae and ordinates are the same
        let double_predicate = (x_coordinates_match & y_coordinates_match);
        // If the abscissae are the same, but not the ordinates, then one point is the opposite of the other
        let infinity_predicate = (x_coordinates_match & !y_coordinates_match);
        let point1_1 = EmbeddedCurvePoint {
            x: point1.x + (x_coordinates_match as Field),
            y: point1.y,
            is_infinite: false,
        };
        let point2_1 = EmbeddedCurvePoint { x: point2.x, y: point2.y, is_infinite: false };
        // point1_1 is guaranteed to have a different abscissa than point2:
        // - if x_coordinates_match is 0, that means point1.x != point2.x, and point1_1.x = point1.x + 0
        // - if x_coordinates_match is 1, that means point1.x = point2.x, but point1_1.x = point1.x + 1 in this case
        // Because the abscissa is different, the addition formula is guaranteed to succeed, so we can safely use `embedded_curve_add_unsafe`
        // Note that this computation may be garbage: if x_coordinates_match is 1, or if one of the input is the point at infinity.
        let mut result = embedded_curve_add_unsafe(point1_1, point2_1);

        // `embedded_curve_add_unsafe` is doing a doubling if the input is the same variable, because in this case it is guaranteed (at 'compile time') that the input is the same.
        let double = embedded_curve_add_unsafe(point1, point1);
        // `embedded_curve_add_unsafe` would not perform doubling, even if the inputs point1 and point2 are the same, because it cannot know this without adding some logic (and some constraints)
        // However we did this logic when we computed `double_predicate`, so we set the result to 2*point1 if point1 and point2 are the same
        result = if double_predicate { double } else { result };

        // Same logic as above for unconstrained context, we set the proper result when one of the inputs is the infinity point
        if point1.is_infinite {
            result = point2;
        }
        if point2.is_infinite {
            result = point1;
        }

        // Finally, we set the is_infinity flag of the result:
        // Opposite points should sum into the infinity point, however, if one of them is point at infinity, their coordinates are not meaningful
        // so we should not use the fact that the inputs are opposite in this case:
        let mut result_is_infinity =
            infinity_predicate & (!point1.is_infinite & !point2.is_infinite);
        // However, if both of them are at infinity, then the result is also at infinity
        result.is_infinite = result_is_infinity | (point1.is_infinite & point2.is_infinite);
        result
    }
}

#[foreign(embedded_curve_add)]
fn embedded_curve_add_array_return(
    _point1: EmbeddedCurvePoint,
    _point2: EmbeddedCurvePoint,
) -> [EmbeddedCurvePoint; 1] {}

/// This function assumes that:
/// The points are on the curve, and
/// The points don't share an x-coordinate, and
/// Neither point is the infinity point.
/// If it is used with correct input, the function ensures the correct non-zero result is returned.
/// Except for points on the curve, the other assumptions are checked by the function. It will cause assertion failure if they are not respected.
pub fn embedded_curve_add_not_nul(
    point1: EmbeddedCurvePoint,
    point2: EmbeddedCurvePoint,
) -> EmbeddedCurvePoint {
    assert(point1.x != point2.x);
    assert(!point1.is_infinite);
    assert(!point2.is_infinite);
    // Ensure is_infinite is comptime
    let point1_1 = EmbeddedCurvePoint { x: point1.x, y: point1.y, is_infinite: false };
    let point2_1 = EmbeddedCurvePoint { x: point2.x, y: point2.y, is_infinite: false };
    embedded_curve_add_unsafe(point1_1, point2_1)
}

/// Unsafe ec addition
/// If the inputs are the same, it will perform a doubling, but only if point1 and point2 are the same variable.
/// If they have the same value but are different variables, the result will be incorrect because in this case
/// it assumes (but does not check) that the points' x-coordinates are not equal.
/// It also assumes neither point is the infinity point.
pub fn embedded_curve_add_unsafe(
    point1: EmbeddedCurvePoint,
    point2: EmbeddedCurvePoint,
) -> EmbeddedCurvePoint {
    embedded_curve_add_array_return(point1, point2)[0]
}

mod test {
    use crate::embedded_curve_ops::{
        embedded_curve_add, embedded_curve_add_not_nul, EmbeddedCurvePoint, EmbeddedCurveScalar,
        fixed_base_scalar_mul, multi_scalar_mul,
    };
    use crate::ops::arith::Neg;

    // doubling the generator point twice
    global sample_point: EmbeddedCurvePoint = EmbeddedCurvePoint {
        x: 0x01b06105d7dc31e315550bc6bf0e4e5e7148034f7b957a514537d8c21b2db26a,
        y: 0x02c095b9dcadf22a6f64bbcb78a223b6e04014a8c8697eccbcf6b35d44452003,
        is_infinite: false,
    };

    #[test]
    fn test_point_at_infinity() {
        let infinity = EmbeddedCurvePoint::point_at_infinity();

        assert(infinity.is_infinite, "Point at infinity should have is_infinite = true");
        assert_eq(infinity.x, 0, "Point at infinity should have x = 0");
        assert_eq(infinity.y, 0, "Point at infinity should have y = 0");
    }

    #[test]
    fn test_generator_point() {
        let generator = EmbeddedCurvePoint::generator();

        assert(!generator.is_infinite, "Generator should not be at infinity");
        assert_eq(generator.x, 1, "Generator should have correct x coordinate");
        assert_eq(
            generator.y,
            17631683881184975370165255887551781615748388533673675138860,
            "Generator should have correct y coordinate",
        );
    }

    #[test]
    fn test_embedded_curve_point_creation() {
        let point = sample_point;

        assert_eq(
            point.x,
            0x01b06105d7dc31e315550bc6bf0e4e5e7148034f7b957a514537d8c21b2db26a,
            "Point should have correct x coordinate",
        );
        assert_eq(
            point.y,
            0x02c095b9dcadf22a6f64bbcb78a223b6e04014a8c8697eccbcf6b35d44452003,
            "Point should have correct y coordinate",
        );
        assert(!point.is_infinite, "Point should not be at infinity");
    }

    #[test]
    fn test_point_negation() {
        let point = sample_point;

        let neg_point = point.neg();

        assert_eq(
            neg_point.x,
            0x01b06105d7dc31e315550bc6bf0e4e5e7148034f7b957a514537d8c21b2db26a,
            "Negated point should have same x coordinate",
        );
        assert_eq(
            neg_point.y,
            -0x02c095b9dcadf22a6f64bbcb78a223b6e04014a8c8697eccbcf6b35d44452003,
            "Negated point should have negated y coordinate",
        );
        assert(!neg_point.is_infinite, "Negated point should not be at infinity");
    }

    #[test]
    fn test_point_negation_at_infinity() {
        let infinity = EmbeddedCurvePoint::point_at_infinity();
        let neg_infinity = infinity.neg();

        assert(neg_infinity.is_infinite, "Negated infinity should still be at infinity");
        assert_eq(neg_infinity.x, 0, "Negated infinity should have x = 0");
        assert_eq(neg_infinity.y, 0, "Negated infinity should have y = 0");
    }

    #[test]
    fn test_point_doubling() {
        let point = EmbeddedCurvePoint::generator();

        let doubled = point.double();

        assert(!doubled.is_infinite, "Doubled point should not be at infinity");
        // The result should be different from the original point
        assert(
            (doubled.x != point.x) | (doubled.y != point.y),
            "Doubled point should be different from original",
        );
    }

    #[test]
    fn test_point_doubling_at_infinity() {
        let infinity = EmbeddedCurvePoint::point_at_infinity();
        let doubled = infinity.double();

        assert(doubled.is_infinite, "Doubled infinity should still be at infinity");
    }

    #[test]
    fn test_point_equality() {
        let point1 = sample_point;

        let point2 = EmbeddedCurvePoint {
            x: 0x01b06105d7dc31e315550bc6bf0e4e5e7148034f7b957a514537d8c21b2db26a,
            y: 0x02c095b9dcadf22a6f64bbcb78a223b6e04014a8c8697eccbcf6b35d44452003,
            is_infinite: false,
        };

        let point3 = EmbeddedCurvePoint::generator();

        assert_eq(point1, point2, "Identical points should be equal");
        assert(point1 != point3, "Different points should not be equal");
    }

    #[test]
    fn test_point_equality_at_infinity() {
        let infinity1 = EmbeddedCurvePoint::point_at_infinity();
        let infinity2 = EmbeddedCurvePoint::point_at_infinity();

        assert_eq(infinity1, infinity2, "All points at infinity should be equal");
    }

    #[test]
    fn test_point_equality_mixed_infinity() {
        let infinity = EmbeddedCurvePoint::point_at_infinity();

        assert(infinity != sample_point, "Infinity point should not equal regular point");
    }

    #[test]
    fn test_embedded_curve_scalar_creation() {
        let scalar = EmbeddedCurveScalar::new(42, 123);

        assert_eq(scalar.lo, 42, "Scalar should have correct lo value");
        assert_eq(scalar.hi, 123, "Scalar should have correct hi value");
    }

    #[test]
    fn test_embedded_curve_scalar_equality() {
        let scalar1 = EmbeddedCurveScalar::new(42, 123);
        let scalar2 = EmbeddedCurveScalar::new(42, 123);
        let scalar3 = EmbeddedCurveScalar::new(42, 124);

        assert_eq(scalar1, scalar2, "Identical scalars should be equal");
        assert(scalar1 != scalar3, "Different scalars should not be equal");
    }

    #[test]
    fn test_embedded_curve_scalar_from_bytes() {
        let mut bytes = [0; 64];
        bytes[31] = 1; // lo starts at offset 31
        bytes[15] = 2; // hi starts at offset 15

        let scalar = EmbeddedCurveScalar::from_bytes(bytes, 0);

        assert(scalar.lo != 0, "Scalar lo should not be zero");
        assert(scalar.hi != 0, "Scalar hi should not be zero");
    }

    #[test]
    fn test_embedded_curve_add_with_infinity() {
        let infinity = EmbeddedCurvePoint::point_at_infinity();
        let regular_point = sample_point;

        // Adding infinity to a regular point should return the regular point
        let result1 = embedded_curve_add(infinity, regular_point);
        assert_eq(result1, regular_point, "Infinity + point should equal point");

        // Adding a regular point to infinity should return the regular point
        let result2 = embedded_curve_add(regular_point, infinity);
        assert_eq(result2, regular_point, "Point + infinity should equal point");

        // Adding infinity to infinity should return infinity
        let result3 = embedded_curve_add(infinity, infinity);
        assert(result3.is_infinite, "Infinity + infinity should equal infinity");
    }

    #[test]
    fn test_embedded_curve_add_identical_points() {
        let point = EmbeddedCurvePoint::generator();

        // Adding a point to itself should perform doubling
        let result = embedded_curve_add(point, point);

        assert(!result.is_infinite, "Point + point should not result in infinity");
        // The result should be different from the original point
        assert(
            (result.x != point.x) | (result.y != point.y),
            "Point + point should result in different point",
        );
    }

    #[test]
    fn test_embedded_curve_add_opposite_points() {
        let point = sample_point;

        let opposite_point = EmbeddedCurvePoint {
            x: 0x01b06105d7dc31e315550bc6bf0e4e5e7148034f7b957a514537d8c21b2db26a,
            y: -0x02c095b9dcadf22a6f64bbcb78a223b6e04014a8c8697eccbcf6b35d44452003,
            is_infinite: false,
        };

        // Adding opposite points should result in infinity
        let result = embedded_curve_add(point, opposite_point);
        assert(result.is_infinite, "Opposite points should sum to infinity");
    }

    #[test]
    fn test_embedded_curve_add_not_nul_assertions() {
        let point1 = sample_point;

        let point2 = EmbeddedCurvePoint::generator();

        // This should not panic since points have different x coordinates and are not at infinity
        let _ = embedded_curve_add_not_nul(point1, point2);
    }

    #[test(should_fail)]
    fn test_embedded_curve_add_not_nul_same_x_coordinate() {
        let point1 = sample_point;

        let point2 = EmbeddedCurvePoint {
            x: 0x01b06105d7dc31e315550bc6bf0e4e5e7148034f7b957a514537d8c21b2db26a,
            y: -0x02c095b9dcadf22a6f64bbcb78a223b6e04014a8c8697eccbcf6b35d44452003,
            is_infinite: false,
        };

        let _ = embedded_curve_add_not_nul(point1, point2);
    }

    #[test(should_fail)]
    fn test_embedded_curve_add_not_nul_first_point_infinity() {
        let infinity = EmbeddedCurvePoint::point_at_infinity();
        let _ = embedded_curve_add_not_nul(infinity, sample_point);
    }

    #[test(should_fail)]
    fn test_embedded_curve_add_not_nul_second_point_infinity() {
        let infinity = EmbeddedCurvePoint::point_at_infinity();

        let _ = embedded_curve_add_not_nul(sample_point, infinity);
    }

    #[test]
    fn test_fixed_base_scalar_mul() {
        let scalar = EmbeddedCurveScalar::new(1, 0);

        // Multiplying by scalar (1,0) should return the generator
        let result = fixed_base_scalar_mul(scalar);

        assert(!result.is_infinite, "Result should not be at infinity");
        // The result should be the generator point
        assert_eq(result.x, 1, "Result should have generator x coordinate");
        assert_eq(
            result.y,
            17631683881184975370165255887551781615748388533673675138860,
            "Result should have generator y coordinate",
        );
    }

    #[test]
    fn test_fixed_base_scalar_mul_zero() {
        let zero_scalar = EmbeddedCurveScalar::new(0, 0);

        // multiplying by zero should result in infinity
        let result = fixed_base_scalar_mul(zero_scalar);
        assert(result.is_infinite, "Zero scalar multiplication should result in infinity");
    }

    #[test]
    fn test_multi_scalar_mul_single_point() {
        let point = EmbeddedCurvePoint::generator();
        let scalar = EmbeddedCurveScalar::new(1, 0);

        let points = [point];
        let scalars = [scalar];

        let result = multi_scalar_mul(points, scalars);

        let expected = fixed_base_scalar_mul(scalar);
        assert_eq(
            result,
            expected,
            "Single point multi-scalar mul should equal fixed base scalar mul",
        );
    }

    #[test]
    fn test_multi_scalar_mul_two_points() {
        let point1 = EmbeddedCurvePoint::generator();
        let point2 = sample_point;

        let scalar1 = EmbeddedCurveScalar::new(1, 0);
        let scalar2 = EmbeddedCurveScalar::new(2, 0);

        let points = [point1, point2];
        let scalars = [scalar1, scalar2];

        let result = multi_scalar_mul(points, scalars);

        // The result should not be infinity
        assert(!result.is_infinite, "Result should not be at infinity");
        assert(result != point1, "Result should be different from first point");
        assert(result != point2, "Result should be different from second point");
    }

    #[test]
    fn test_curve_addition_properties() {
        // Test that addition is commutative: P + Q = Q + P

        let point2 = EmbeddedCurvePoint::generator();

        let point1 = sample_point;
        let result1 = embedded_curve_add(point1, point2);
        let result2 = embedded_curve_add(point2, point1);

        assert_eq(result1, result2, "Curve addition should be commutative");
    }

    #[test]
    fn test_curve_addition_identity() {
        // Test that P + infinity = P (infinity is the identity element)
        let point = sample_point;

        let infinity = EmbeddedCurvePoint::point_at_infinity();

        let result = embedded_curve_add(point, infinity);
        assert_eq(result, point, "Point + infinity should equal the point");
    }

    #[test]
    fn test_curve_negation_properties() {
        // Test that P + (-P) = infinity
        let point = sample_point;

        let neg_point = point.neg();
        let result = embedded_curve_add(point, neg_point);

        assert(result.is_infinite, "Point + (-Point) should equal infinity");
    }

    #[test]
    fn test_curve_doubling_properties() {
        let point = EmbeddedCurvePoint::generator();

        let doubled = point.double();

        // For most points, doubling should give a different result
        assert(doubled != point, "Doubled point should be different from original");
        assert(!doubled.is_infinite, "Doubled point should not be at infinity");
    }
}
