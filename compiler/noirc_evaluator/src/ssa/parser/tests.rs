#![cfg(test)]

use crate::{
    ssa::{opt::assert_normalized_ssa_equals, Ssa},
    trim_leading_whitespace_from_lines,
};

fn assert_ssa_roundtrip(src: &str) {
    let ssa = Ssa::from_str(src).unwrap();
    let ssa = ssa.to_string();
    let ssa = trim_leading_whitespace_from_lines(&ssa);
    let src = trim_leading_whitespace_from_lines(src);
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
fn test_make_array() {
    let src = "
        acir(inline) fn main f0 {
          b0():
            v1 = make_array [Field 1] : [Field; 1]
            return v1
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_make_empty_array() {
    let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = make_array [] : [Field; 0]
            return v0
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_make_composite_array() {
    let src = "
        acir(inline) fn main f0 {
          b0():
            v2 = make_array [Field 1, Field 2] : [(Field, Field); 1]
            return v2
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_make_byte_array_with_string_literal() {
    let src = "
        acir(inline) fn main f0 {
          b0():
            v9 = make_array b\"Hello world!\"
            return v9
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_make_byte_slice_with_string_literal() {
    let src = "
        acir(inline) fn main f0 {
          b0():
            v9 = make_array &b\"Hello world!\"
            return v9
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
          b1(v0: Field):
            return v0
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_jmpif() {
    let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            jmpif v0 then: b2, else: b1
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
            v3 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
            v5 = make_array [Field 4] : [Field; 1]
            return v3, v5
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_call_no_return_value() {
    let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            call f1(v0)
            return
        }
        acir(inline) fn foo f1 {
          b0(v0: Field):
            return
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_call_intrinsic() {
    let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            call assert_constant(v0)
            return
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
fn test_constrain_with_static_message() {
    let src = r#"
        acir(inline) fn main f0 {
          b0(v0: Field):
            constrain v0 == Field 1, "Oh no!"
            return
        }
        "#;
    assert_ssa_roundtrip(src);
}

#[test]
fn test_constrain_with_dynamic_message() {
    let src = r#"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v7 = make_array b"{x} {y}"
            constrain v0 == Field 1, data v7, u32 2, v0, v1
            return
        }
        "#;
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
fn test_mutable_array_set() {
    let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v3 = array_set mut v0, index Field 0, value Field 1
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

#[test]
fn test_allocate() {
    let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut [Field; 3]
            return
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_load() {
    let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v1 = load v0 -> Field
            return
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_store() {
    let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            store Field 1 at v0
            return
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_inc_rc() {
    let src = "
        brillig(inline) fn main f0 {
          b0(v0: [Field; 3]):
            inc_rc v0
            return
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_dec_rc() {
    let src = "
        brillig(inline) fn main f0 {
          b0(v0: [Field; 3]):
            dec_rc v0
            return
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_mutable_reference_type() {
    let src = "
        acir(inline) fn main f0 {
          b0(v0: &mut Field):
            return
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_parses_with_comments() {
    let src = "
        // This is a comment
        acir(inline) fn main f0 {
          b0(v0: &mut Field): // This is a block
            return // Returns nothing
        }
        ";

    let expected = "
        acir(inline) fn main f0 {
          b0(v0: &mut Field):
            return
        }
        ";

    let ssa = Ssa::from_str(src).unwrap();
    assert_normalized_ssa_equals(ssa, expected);
}

#[test]
fn test_slice() {
    let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v2, v3 = call as_slice(v0) -> (u32, [Field])
            return
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_negative() {
    let src = "
        acir(inline) fn main f0 {
          b0():
            return Field -1
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_function_type() {
    let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut function
            return
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_does_not_simplify() {
    let src = "
        acir(inline) fn main f0 {
          b0():
            v2 = add Field 1, Field 2
            return v2
        }
        ";
    assert_ssa_roundtrip(src);
}
