// Regression test for #12034: MSM opcode must not mix constant/witness
// for scalar halves (lo, hi). When one half is constant and the other
// is a witness, the backend's to_grumpkin_scalar rejects the input.
pub fn main(inp: bool) {
    if inp {
        // Use from_field to prevent constant folding the scalar construction;
        // if both lo and hi are known constants, the MSM gets simplified away
        // entirely and the bug is not exercised.
        let scalar = std::embedded_curve_ops::EmbeddedCurveScalar::from_field(
            340282366920938463463374607431768211456,
        );
        let point = std::embedded_curve_ops::fixed_base_scalar_mul(scalar);
        assert(point.x != 0);
    }
}
