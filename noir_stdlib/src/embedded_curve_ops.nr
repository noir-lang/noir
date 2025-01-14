use crate::cmp::Eq;
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
    let point_array = multi_scalar_mul_array_return(points, scalars);
    EmbeddedCurvePoint { x: point_array[0], y: point_array[1], is_infinite: point_array[2] as bool }
}

#[foreign(multi_scalar_mul)]
pub(crate) fn multi_scalar_mul_array_return<let N: u32>(
    points: [EmbeddedCurvePoint; N],
    scalars: [EmbeddedCurveScalar; N],
) -> [Field; 3] {}

// docs:start:fixed_base_scalar_mul
pub fn fixed_base_scalar_mul(scalar: EmbeddedCurveScalar) -> EmbeddedCurvePoint
// docs:end:fixed_base_scalar_mul
{
    let g1 = EmbeddedCurvePoint {
        x: 1,
        y: 17631683881184975370165255887551781615748388533673675138860,
        is_infinite: false,
    };
    multi_scalar_mul([g1], [scalar])
}

/// This function only assumes that the points are on the curve
/// It handles corner cases around the infinity point causing some overhead compared to embedded_curve_add_not_nul and embedded_curve_add_unsafe
// This is a hack because returning an `EmbeddedCurvePoint` from a foreign function in brillig returns a [BrilligVariable::SingleAddr; 2] rather than BrilligVariable::BrilligArray
// as is defined in the brillig bytecode format. This is a workaround which allows us to fix this without modifying the serialization format.
// docs:start:embedded_curve_add
pub fn embedded_curve_add(
    point1: EmbeddedCurvePoint,
    point2: EmbeddedCurvePoint,
) -> EmbeddedCurvePoint {
    // docs:end:embedded_curve_add
    let x_coordinates_match = point1.x == point2.x;
    let y_coordinates_match = point1.y == point2.y;
    let double_predicate = (x_coordinates_match & y_coordinates_match);
    let infinity_predicate = (x_coordinates_match & !y_coordinates_match);
    let point1_1 = EmbeddedCurvePoint {
        x: point1.x + (x_coordinates_match as Field),
        y: point1.y,
        is_infinite: x_coordinates_match,
    };
    // point1_1 is guaranteed to have a different abscissa than point2
    let mut result = embedded_curve_add_unsafe(point1_1, point2);
    result.is_infinite = x_coordinates_match;

    // dbl if x_match, y_match
    let double = embedded_curve_add_unsafe(point1, point1);
    result = if double_predicate { double } else { result };

    // infinity if x_match, !y_match
    if point1.is_infinite {
        result = point2;
    }
    if point2.is_infinite {
        result = point1;
    }
    let mut result_is_infinity = infinity_predicate & (!point1.is_infinite & !point2.is_infinite);
    result.is_infinite = result_is_infinity | (point1.is_infinite & point2.is_infinite);
    result
}

#[foreign(embedded_curve_add)]
fn embedded_curve_add_array_return(
    _point1: EmbeddedCurvePoint,
    _point2: EmbeddedCurvePoint,
) -> [Field; 3] {}

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
    embedded_curve_add_unsafe(point1, point2)
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
    let point_array = embedded_curve_add_array_return(point1, point2);
    let x = point_array[0];
    let y = point_array[1];

    EmbeddedCurvePoint { x, y, is_infinite: false }
}
