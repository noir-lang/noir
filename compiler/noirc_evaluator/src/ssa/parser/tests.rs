#![cfg(test)]

use crate::ssa::Ssa;

fn assert_ssa_roundtrip(src: &str) {
    let ssa = Ssa::from_str(src).unwrap();
    let ssa = ssa.to_string();
    let ssa = ssa.trim();
    let src = src.trim();
    if ssa != src {
        println!("Expected:\n~~~\n{}\n~~~\nGot:\n~~~\n{}\n~~~", src, ssa);
        assert_eq!(ssa, src);
    }
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
acir(inline) fn main f0 -> {typ} {{
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
acir(inline) fn main f0 -> [Field; 1] {
  b0():
    return [Field 1] of Field
}
";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_return_empty_array() {
    let src = "
acir(inline) fn main f0 -> [Field; 0] {
  b0():
    return [] of Field
}
";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_return_composite_array() {
    let src = "
acir(inline) fn main f0 -> [(Field, Field); 1] {
  b0():
    return [Field 1, Field 2] of (Field, Field)
}
";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_block_parameters() {
    let src = "
acir(inline) fn main f0 -> (Field, Field) {
  b0(v0: Field, v1: Field):
    return v0, v1
}
";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_multiple_blocks_and_jmp() {
    let src: &str = "
acir(inline) fn main f0 -> Field {
  b0():
    jmp b1(Field 1)
  b1(v1: Field):
    return v1
}
";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_jmpif() {
    let src: &str = "
acir(inline) fn main f0 {
  b0(v0: Field):
    jmpif v0 then: b1, else: b2
  b1():
    return
  b2():
    return
}
";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_call() {
    let src: &str = "
acir(inline) fn main f0 -> Field {
  b0(v0: Field):
    v2 = call f1(v0)
    return v2
}
acir(inline) fn foo f1 -> Field {
  b0(v0: Field):
    return v0
}
";
    assert_ssa_roundtrip(src);
}
