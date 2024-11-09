#![cfg(test)]

use crate::ssa::Ssa;

fn assert_ssa_roundtrip(src: &str) {
    let ssa = Ssa::from_str(src).unwrap();
    let ssa = ssa.to_string();
    let ssa = ssa.trim();
    let src = src.trim();
    if ssa != src {
        println!("Expected:\n~~~\n{}\n~~~\nGot:\n~~~\n{}\n~~~", src, ssa);
        similar_asserts::assert_eq!(ssa, src);
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

#[test]
fn test_block_parameters() {
    let src = "
acir(inline) fn main f0 {
  b0(v0: Field, v1: Field):
    return v0, v1
}
";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_multiple_blocks_and_jmp() {
    let src = "
acir(inline) fn main f0 {
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
    let src = "
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
    let src = "
acir(inline) fn main f0 {
  b0(v0: Field):
    v2 = call f1(v0) -> Field
    return v2
}
acir(inline) fn foo f1 {
  b0(v0: Field):
    return v0
}
";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_call_multiple_return_values() {
    let src = "
acir(inline) fn main f0 {
  b0():
    v1, v2 = call f1() -> ([Field; 3], [Field; 1])
    return v1
}
acir(inline) fn foo f1 {
  b0():
    return [Field 1, Field 2, Field 3] of Field, [Field 4] of Field
}
";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_cast() {
    let src = "
acir(inline) fn main f0 {
  b0(v0: Field):
    v1 = cast v0 as i32
    return v1
}
";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_constrain() {
    let src = "
acir(inline) fn main f0 {
  b0(v0: Field):
    constrain v0 == Field 1
    return
}
";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_enable_side_effects() {
    let src = "
acir(inline) fn main f0 {
  b0(v0: Field):
    enable_side_effects v0
    return
}
";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_array_get() {
    let src = "
acir(inline) fn main f0 {
  b0(v0: [Field; 3]):
    v2 = array_get v0, index Field 0 -> Field
    return
}
";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_array_set() {
    let src = "
acir(inline) fn main f0 {
  b0(v0: [Field; 3]):
    v3 = array_set v0, index Field 0, value Field 1
    return
}
";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_array_get_set_bug() {
    let src = "
acir(inline) fn main f0 {
  b0(v0: [u32; 3]):
    v3 = array_set v0, index u32 1, value u32 2
    v5 = array_get v3, index u32 0 -> u32
    return
}
";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_binary() {
    for op in ["add", "sub", "mul", "div", "eq", "mod", "lt", "and", "or", "xor", "shl", "shr"] {
        let src = format!(
            "
acir(inline) fn main f0 {{
  b0(v0: Field, v1: Field):
    v2 = {op} v0, v1
    return
}}
"
        );
        assert_ssa_roundtrip(&src);
    }
}

#[test]
fn test_truncate() {
    let src = "
acir(inline) fn main f0 {
  b0(v0: Field):
    v1 = truncate v0 to 8 bits, max_bit_size: 16
    return
}
";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_not() {
    let src = "
acir(inline) fn main f0 {
  b0(v0: Field):
    v1 = not v0
    return
}
";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_range_check() {
    let src = "
acir(inline) fn main f0 {
  b0(v0: Field):
    range_check v0 to 8 bits
    return
}
";
    assert_ssa_roundtrip(src);
}