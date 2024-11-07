#![cfg(test)]

use crate::ssa::Ssa;

fn assert_ssa_roundtrip(src: &str) {
    let ssa = Ssa::from_str(src).unwrap();
    assert_eq!(ssa.to_string().trim(), src.trim());
}

#[test]
fn test_empty_acir_function() {
    let src = "
acir(inline) fn main f0 {
  b0():
    return
}
";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_empty_brillig_function() {
    let src = "
brillig(inline) fn main f0 {
  b0():
    return
}
";
    assert_ssa_roundtrip(src);
}
