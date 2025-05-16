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
