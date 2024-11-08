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

#[test]
fn test_return_integer() {
    for typ in ["u1", "u8", "u16", "u32", "u64", "i1", "i8", "i16", "i32", "i64", "Field"] {
        let src = format!(
            "
acir(inline) fn main f0 {{
  b0():
    return {typ} 1
}}
"
        );
        assert_ssa_roundtrip(&src);
    }
}

#[test]
fn test_return_array() {
    let src = "
acir(inline) fn main f0 {
  b0():
    return [Field 1] of Field
}
";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_return_empty_array() {
    let src = "
acir(inline) fn main f0 {
  b0():
    return [] of Field
}
";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_return_composite_array() {
    let src = "
acir(inline) fn main f0 {
  b0():
    return [Field 1, Field 2] of (Field, Field)
}
";
    assert_ssa_roundtrip(src);
}
