struct EmbeddedCurvePoint {
    x: Field,
    y: Field,
}

// Computes a fixed base scalar multiplication over the embedded curve.
// For bn254, We have Grumpkin and Baby JubJub.
// For bls12-381, we have JubJub and Bandersnatch.
//
// The embedded curve being used is decided by the 
// underlying proof system.
#[foreign(fixed_base_scalar_mul)]
// docs:start:fixed_base_embedded_curve
pub fn fixed_base_embedded_curve(_low: Field, _high: Field) -> [Field; 2] {}
// docs:end:fixed_base_embedded_curve

#[foreign(embedded_curve_add)]
pub fn embedded_curve_add(_point1: EmbeddedCurvePoint, _point2: EmbeddedCurvePoint) -> EmbeddedCurvePoint {}

pub fn embedded_curve_double(point: EmbeddedCurvePoint) -> EmbeddedCurvePoint {
    embedded_curve_add(point, point)
}
