use crate::ops::arith::{Add, Sub, Neg};

// TODO(https://github.com/noir-lang/noir/issues/4931)
struct EmbeddedCurvePoint {
    x: Field,
    y: Field,
}

impl EmbeddedCurvePoint {
    fn double(self) -> EmbeddedCurvePoint {
        embedded_curve_add(self, self)
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
            y: -self.y
        }
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
    points: [Field; N], // points represented as x and y coordinates [x1, y1, x2, y2, ...]
    scalars: [Field; N] // scalars represented as low and high limbs [low1, high1, low2, high2, ...]
) -> [Field; 2]
// docs:end:multi_scalar_mul
{}

// docs:start:fixed_base_scalar_mul
pub fn fixed_base_scalar_mul(
    scalar_low: Field,
    scalar_high: Field
) -> [Field; 2]
// docs:end:fixed_base_scalar_mul
{
    let g1_x = 1;
    let g1_y = 17631683881184975370165255887551781615748388533673675138860;
    multi_scalar_mul([g1_x, g1_y], [scalar_low, scalar_high])
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
    EmbeddedCurvePoint { x, y }
}

#[foreign(embedded_curve_add)]
fn embedded_curve_add_array_return(_point1: EmbeddedCurvePoint, _point2: EmbeddedCurvePoint) -> [Field; 2] {}
