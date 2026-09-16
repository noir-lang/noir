// A checked u128 multiplication whose true product exceeds the field modulus, while the reduced
// (mod-p) product still fits in 128 bits. The `check_u128_mul_overflow` SSA pass exists to reject
// exactly this case, and the SSA interpreter must reject it too (noir-lang/noir-claude#1623):
// both `nargo execute` and `nargo interpret` must report a multiplication overflow.
fn main(a: u128, b: u128) -> pub u128 {
    a * b
}
