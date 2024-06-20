use crate::ops::arith::{Add, Sub, Neg};
use crate::cmp::Eq;

// TODO(https://github.com/noir-lang/noir/issues/4931)
struct EmbeddedCurvePoint {
    x: Field,
    y: Field,
    is_infinite: bool
}

impl EmbeddedCurvePoint {
    fn double(self) -> EmbeddedCurvePoint {
        embedded_curve_add(self, self)
    }

    fn point_at_infinity() -> EmbeddedCurvePoint {
        EmbeddedCurvePoint { x: 0, y: 0, is_infinite: true }
    }
}

impl Add for EmbeddedCurvePoint {
    fn add(self, other: EmbeddedCurvePoint) -> EmbeddedCurvePoint { 
        embedded_curve_add(self, other)
    }
}

impl Sub for EmbeddedCurvePoint {
    fn sub(self, other: EmbeddedCurvePoint) -> EmbeddedCurvePoint { 
        self + other.neg()
    }
}

impl Neg for EmbeddedCurvePoint {
    fn neg(self) -> EmbeddedCurvePoint { 
        EmbeddedCurvePoint {
            x: self.x,
            y: -self.y,
            is_infinite: self.is_infinite
        }
    }
}

impl Eq for EmbeddedCurvePoint {
    fn eq(self: Self, b: EmbeddedCurvePoint) -> bool {
        (self.is_infinite & b.is_infinite) | ((self.is_infinite == b.is_infinite) & (self.x == b.x) & (self.y == b.y))
    }
}

// Scalar represented as low and high limbs
struct EmbeddedCurveScalar {
    lo: Field,
    hi: Field,
}

impl EmbeddedCurveScalar {
    #[field(bn254)]
    fn from_field(scalar: Field) -> EmbeddedCurveScalar {
        let (a,b) = crate::field::bn254::decompose(scalar);
        EmbeddedCurveScalar { lo: a, hi: b }
    }
}

// Computes a multi scalar multiplication over the embedded curve.
// For bn254, We have Grumpkin and Baby JubJub.
// For bls12-381, we have JubJub and Bandersnatch.
//
// The embedded curve being used is decided by the 
// underlying proof system.
#[foreign(multi_scalar_mul)]
// docs:start:multi_scalar_mul
pub fn multi_scalar_mul<N>(
    points: [EmbeddedCurvePoint; N],
    scalars: [EmbeddedCurveScalar; N]
) -> EmbeddedCurvePoint
// docs:end:multi_scalar_mul
{}

// docs:start:fixed_base_scalar_mul
pub fn fixed_base_scalar_mul(
    scalar_low: Field,
    scalar_high: Field
) -> EmbeddedCurvePoint
// docs:end:fixed_base_scalar_mul
{
    let g1 = EmbeddedCurvePoint { x: 1, y: 17631683881184975370165255887551781615748388533673675138860, is_infinite: false };
    let scalar = EmbeddedCurveScalar { lo: scalar_low, hi: scalar_high };
    multi_scalar_mul([g1], [scalar])
}

#[foreign(embedded_curve_add)]
// docs:start:embedded_curve_add
fn embedded_curve_add(
    _point1: EmbeddedCurvePoint,
    _point2: EmbeddedCurvePoint
) -> EmbeddedCurvePoint
// docs:end:embedded_curve_add
{}
