// Decomposing the zero field into zero limbs must succeed with an empty array
// in every evaluation context, for all four public decomposition entry points.
// See https://github.com/noir-lang/noir-claude/issues/1019.
fn main(x: pub Field) {
    // Constrained, witness input (`x == 0`).
    let _: [u8; 0] = x.to_le_bytes();
    let _: [u8; 0] = x.to_be_bytes();
    let _: [bool; 0] = x.to_le_bits();
    let _: [bool; 0] = x.to_be_bits();

    // Constrained, literal-zero input.
    let _: [u8; 0] = (0 as Field).to_le_bytes();
    let _: [u8; 0] = (0 as Field).to_be_bytes();
    let _: [bool; 0] = (0 as Field).to_le_bits();
    let _: [bool; 0] = (0 as Field).to_be_bits();

    // Comptime.
    let _: [u8; 0] = comptime { (0 as Field).to_le_bytes() };
    let _: [u8; 0] = comptime { (0 as Field).to_be_bytes() };
    let _: [bool; 0] = comptime { (0 as Field).to_le_bits() };
    let _: [bool; 0] = comptime { (0 as Field).to_be_bits() };

    // Unconstrained Brillig.
    // Safety: decomposing zero into zero limbs has no side effects.
    unsafe {
        unconstrained_decompose();
    }
}

unconstrained fn unconstrained_decompose() {
    let _: [u8; 0] = (0 as Field).to_le_bytes();
    let _: [u8; 0] = (0 as Field).to_be_bytes();
    let _: [bool; 0] = (0 as Field).to_le_bits();
    let _: [bool; 0] = (0 as Field).to_be_bits();
}
