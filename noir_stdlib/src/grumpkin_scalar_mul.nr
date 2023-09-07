use crate::grumpkin_scalar::GrumpkinScalar;
use crate::scalar_mul::fixed_base_embedded_curve;

fn grumpkin_fixed_base(scalar: GrumpkinScalar) -> [Field; 2] {
    fixed_base_embedded_curve(scalar.low)
}
