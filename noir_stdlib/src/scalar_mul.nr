use crate::ops::Add;

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

// Computes a fixed base scalar multiplication over the embedded curve.
// For bn254, We have Grumpkin and Baby JubJub.
// For bls12-381, we have JubJub and Bandersnatch.
//
// The embedded curve being used is decided by the 
// underlying proof system.
#[foreign(fixed_base_scalar_mul)]
// docs:start:fixed_base_embedded_curve
pub fn fixed_base_embedded_curve(
    low: Field,
    high: Field
) -> [Field; 2]
// docs:end:fixed_base_embedded_curve
{}

// This is a hack as returning an `EmbeddedCurvePoint` from a foreign function in brillig returns a [BrilligVariable::SingleAddr; 2] rather than BrilligVariable::BrilligArray
// as is defined in the brillig bytecode format. This is a workaround which allows us to fix this without modifying the serialization format.
fn embedded_curve_add(point1: EmbeddedCurvePoint, point2: EmbeddedCurvePoint) -> EmbeddedCurvePoint {
    let point_array = embedded_curve_add_array_return(point1, point2);
    let x = point_array[0];
    let y = point_array[1];
    EmbeddedCurvePoint { x, y }
}

#[foreign(embedded_curve_add)]
fn embedded_curve_add_array_return(_point1: EmbeddedCurvePoint, _point2: EmbeddedCurvePoint) -> [Field; 2] {}
