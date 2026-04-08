// Previously the EmbeddedCurveScalar::from_field call would not be fully constant folded before the CFG
// flattening pass, which would prevent the MSM opcode from being simplified away.
pub fn main(inp: Field) {
    if inp == 1 {
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
