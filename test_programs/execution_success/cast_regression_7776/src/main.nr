// Narrowing casts of a witness-derived value must be preserved rather than
// optimized away. See https://github.com/noir-lang/noir/issues/7776.
pub fn main(a: Field, b: Field, c: bool) -> pub (i8, u8, u64) {
    let d = a - b;
    let e = a - (c as Field);
    (d as i8, d as u8, e as u64)
}
