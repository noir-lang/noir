#![cfg(test)]

use crate::{
    ssa::{Ssa, opt::assert_normalized_ssa_equals},
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
    for typ in ["u1", "u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64", "Field"] {
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
fn test_make_composite_slice() {
    let src = "
        acir(inline) predicate_pure fn main f0 {
          b0():
            v2 = make_array [Field 2, Field 3] : [Field; 2]
            v4 = make_array [Field 1, v2] : [(Field, [Field; 2])]
            return v4
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
fn test_does_not_use_byte_array_literal_for_form_feed() {
    // 12 is '\f', which isn't available in string literals (because in Rust it's the same)
    let src = "
        acir(inline) fn main f0 {
          b0():
            v1 = make_array [u8 12] : [u8; 1]
            return v1
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
            jmpif v0 then: b1, else: b2
          b1():
            jmp b2()
          b2():
            return
        }
        ";
    assert_ssa_roundtrip(src);

    let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            jmpif v0 then: b2, else: b1
          b1():
            jmp b2()
          b2():
            return
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_multiple_jmpif() {
    let src = "
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            jmpif v0 then: b1, else: b2
          b1():
            jmp b4()
          b2():
            jmpif v1 then: b3, else: b1
          b3():
            jmp b4()
          b4():
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
fn test_constrain_not_equal() {
    let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            constrain v0 != Field 1
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
            v2 = array_get v0, index u32 0 -> Field
            return
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_array_get_with_index_minus_1() {
    let src: &'static str = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v2 = array_get v0, index u32 3 minus 1 -> Field
            return
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_array_get_with_index_minus_3() {
    let src: &'static str = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v2 = array_get v0, index u32 6 minus 3 -> Field
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
            v3 = array_set v0, index u32 0, value Field 1
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
            v3 = array_set mut v0, index u32 0, value Field 1
            return
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_array_set_with_index_minus_1() {
    let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v3 = array_set v0, index u32 2 minus 1, value Field 1
            return
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_array_set_with_index_minus_3() {
    let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v3 = array_set v0, index u32 4 minus 3, value Field 1
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
    for op in [
        "add",
        "sub",
        "mul",
        "div",
        "eq",
        "mod",
        "lt",
        "and",
        "or",
        "xor",
        "shl",
        "shr",
        "unchecked_add",
        "unchecked_sub",
        "unchecked_mul",
    ] {
        let src = format!(
            "
            acir(inline) fn main f0 {{
              b0(v0: u32, v1: u32):
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
fn test_range_check_with_message() {
    let src = r#"
        acir(inline) fn main f0 {
          b0(v0: Field):
            range_check v0 to 8 bits, "overflow error\n\t"
            return
        }
        "#;
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

#[test]
fn parses_globals() {
    let src = "
        g0 = Field 0
        g1 = u32 1
        g2 = make_array [] : [Field; 0]
        g3 = make_array [g2] : [[Field; 0]; 1]

        acir(inline) fn main f0 {
          b0():
            return g3
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
fn parses_purity() {
    let src = "
        acir(inline) pure fn main f0 {
          b0():
            return
        }
        acir(inline) predicate_pure fn one f1 {
          b0():
            return
        }
        acir(inline) impure fn two f2 {
          b0():
            return
        }
        acir(inline) fn three f3 {
          b0():
            return
        }
    ";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_parses_if_else() {
    let src = "
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u1):
            v4 = if v0 then Field 1 else (if v1) Field 2
            return v4
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_parses_keyword_in_function_name() {
    let src = "
        acir(inline) fn add f0 {
          b0():
            return
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
#[should_panic = "Attempt to modulo fields"]
fn regression_modulo_fields_brillig() {
    use crate::brillig::BrilligOptions;

    let src = "
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: Field, v1: Field):
            v2 = mod v0, v1
            return v2
        }
        ";
    let ssa = Ssa::from_str(src).unwrap();
    ssa.to_brillig(&BrilligOptions::default());
}

#[test]
fn test_parses_nop() {
    let src = "
        acir(inline) fn add f0 {
          b0():
            nop
            return
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
fn test_parses_print() {
    let src = "
        brillig(inline) impure fn main f0 {
          b0():
            call print()
            return
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
fn parses_variable_from_a_syntantically_following_block_but_logically_preceding_block_with_jmp() {
    let src = "
        acir(inline) impure fn main f0 {
          b0():
            jmp b2()
          b1():
            v5 = add v2, v4
            return
          b2():
            v2 = add Field 1, Field 2
            v4 = add v2, Field 3
            jmp b1()
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
fn parses_variable_from_a_syntantically_following_block_but_logically_preceding_block_with_jmpif() {
    let src = "
        acir(inline) impure fn main f0 {
          b0(v0: u1):
            jmpif v0 then: b2, else: b3
          b1():
            v6 = add v3, v5
            return
          b2():
            jmp b3()
          b3():
            v3 = add Field 1, Field 2
            v5 = add v3, Field 3
            jmp b1()
        }
        ";
    assert_ssa_roundtrip(src);
}
