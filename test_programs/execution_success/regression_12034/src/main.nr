// Regression test for #12034: MSM opcode must not mix constant/witness
// for scalar halves (lo, hi). When one half is constant and the other
// is a witness, the backend's to_grumpkin_scalar rejects the input.
//
// The scalar fits in the lower 128 bits, so lo is a witness and hi is
// constant zero, exercising the mixed constant/witness path.
pub fn main(x: Field) {
    let scalar = std::embedded_curve_ops::EmbeddedCurveScalar { lo: x, hi: 0 };
    let point = std::embedded_curve_ops::fixed_base_scalar_mul(scalar);
    assert(point.x != 0);
}
