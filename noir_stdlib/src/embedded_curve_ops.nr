use crate::ops::arith::{Add, Sub, Neg};
use crate::cmp::Eq;

/// A point on the embedded elliptic curve
/// By definition, the base field of the embedded curve is the scalar field of the proof system curve, i.e the Noir Field.
/// x and y denotes the Weierstrass coordinates of the point, if is_infinite is false.
struct EmbeddedCurvePoint {
    x: Field,
    y: Field,
    is_infinite: bool
}

impl EmbeddedCurvePoint {
    /// Elliptic curve point doubling operation
    /// returns the doubled point of a point P, i.e P+P
    fn double(self) -> EmbeddedCurvePoint {
        embedded_curve_add(self, self)
    }

    /// Returns the null element of the curve; 'the point at infinity'
    fn point_at_infinity() -> EmbeddedCurvePoint {
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
        EmbeddedCurvePoint {
            x: self.x,
            y: -self.y,
            is_infinite: self.is_infinite
        }
    }
}

impl Eq for EmbeddedCurvePoint {
    /// Checks whether two points are equal
    fn eq(self: Self, b: EmbeddedCurvePoint) -> bool {
        (self.is_infinite & b.is_infinite) | ((self.is_infinite == b.is_infinite) & (self.x == b.x) & (self.y == b.y))
    }
}

/// Scalar for the embedded curve represented as low and high limbs
/// By definition, the scalar field of the embedded curve is base field of the proving system curve.
/// It may not fit into a Field element, so it is represented with two Field elements; its low and high limbs.
struct EmbeddedCurveScalar {
    lo: Field,
    hi: Field,
}

impl EmbeddedCurveScalar {
    pub fn new(lo: Field, hi: Field) -> Self {
        EmbeddedCurveScalar { lo, hi }
    }

    #[field(bn254)]
    fn from_field(scalar: Field) -> EmbeddedCurveScalar {
        let (a,b) = crate::field::bn254::decompose(scalar);
        EmbeddedCurveScalar { lo: a, hi: b }
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
    scalars: [EmbeddedCurveScalar; N]
) -> EmbeddedCurvePoint
// docs:end:multi_scalar_mul
{
    let point_array = multi_scalar_mul_array_return(points, scalars);
    EmbeddedCurvePoint { x: point_array[0], y: point_array[1], is_infinite: point_array[2] as bool }
}

#[foreign(multi_scalar_mul)]
fn multi_scalar_mul_array_return<let N: u32>(points: [EmbeddedCurvePoint; N], scalars: [EmbeddedCurveScalar; N]) -> [Field; 3] {}

// docs:start:fixed_base_scalar_mul
pub fn fixed_base_scalar_mul(scalar: EmbeddedCurveScalar) -> EmbeddedCurvePoint
// docs:end:fixed_base_scalar_mul
{
    let g1 = EmbeddedCurvePoint { x: 1, y: 17631683881184975370165255887551781615748388533673675138860, is_infinite: false };
    multi_scalar_mul([g1], [scalar])
}

// This is a hack as returning an `EmbeddedCurvePoint` from a foreign function in brillig returns a [BrilligVariable::SingleAddr; 2] rather than BrilligVariable::BrilligArray
// as is defined in the brillig bytecode format. This is a workaround which allows us to fix this without modifying the serialization format.
// docs:start:embedded_curve_add
fn embedded_curve_add(
    point1: EmbeddedCurvePoint,
    point2: EmbeddedCurvePoint
) -> EmbeddedCurvePoint
// docs:end:embedded_curve_add
{
    let point_array = embedded_curve_add_array_return(point1, point2);
    let x = point_array[0];
    let y = point_array[1];
    EmbeddedCurvePoint { x, y, is_infinite: point_array[2] == 1 }
}

#[foreign(embedded_curve_add)]
fn embedded_curve_add_array_return(_point1: EmbeddedCurvePoint, _point2: EmbeddedCurvePoint) -> [Field; 3] {}
