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
) -> [Field; 3]
// docs:end:multi_scalar_mul
{}

// docs:start:fixed_base_scalar_mul
pub fn fixed_base_scalar_mul(
    scalar_low: Field,
    scalar_high: Field
) -> [Field; 3]
// docs:end:fixed_base_scalar_mul
{
    let g1 = EmbeddedCurvePoint { x: 1, y: 17631683881184975370165255887551781615748388533673675138860, is_infinite: false };
    let scalar = EmbeddedCurveScalar { lo: scalar_low, hi: scalar_high };
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
