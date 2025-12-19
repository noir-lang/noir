#![cfg(test)]

use crate::{
    ssa::{Ssa, opt::assert_normalized_ssa_equals},
    trim_leading_whitespace_from_lines,
};

fn assert_ssa_roundtrip(src: &str) {
    let ssa = Ssa::from_str(src).unwrap();
    let ssa = ssa.print_without_locations().to_string();
    let ssa = trim_leading_whitespace_from_lines(&ssa);
    let src = trim_leading_whitespace_from_lines(src);
    if ssa != src {
        println!("Expected:\n~~~\n{src}\n~~~\nGot:\n~~~\n{ssa}\n~~~");
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
fn test_return_negative_integer() {
    let src = "
        acir(inline) fn main f0 {
          b0():
            return i8 -53
        }
        ";
    assert_ssa_roundtrip(src);
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
fn test_make_composite_vector() {
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
fn test_make_empty_composite_array() {
    let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = make_array [] : [(); 1]
            return v0
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
fn test_make_byte_vector_with_string_literal() {
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
          b0(v0: u1):
            jmpif v0 then: b1, else: b2
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
          b0(v0: u1, v1: u1):
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
fn test_unreachable() {
    let src = "
        acir(inline) fn main f0 {
          b0():
            unreachable
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
fn test_recursive_call_to_main_function() {
    let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            call f0(v0)
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
            v1 = truncate v0 to 32 bits, max_bit_size: 254
            v2 = cast v1 as i32
            return v2
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
        brillig(inline) fn main f0 {
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
        brillig(inline) fn main f0 {
          b0(v0: [Field]):
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
        brillig(inline) fn main f0 {
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
        brillig(inline) fn main f0 {
          b0(v0: [Field]):
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

    for op in ["shl", "shr"] {
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
          b0(v0: &mut Field):
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
fn test_vector() {
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
fn test_parses_oracle() {
    let src = "
        brillig(inline) impure fn main f0 {
          b0():
            call oracle_call()
            return
        }
        ";
    assert_ssa_roundtrip(src);
}

#[test]
fn parses_variable_from_a_syntactically_following_block_but_logically_preceding_block_with_jmp() {
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
fn parses_variable_from_a_syntactically_following_block_but_logically_preceding_block_with_jmpif() {
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

#[test]
fn function_pointer_in_global_array() {
    let src = "
    g2 = make_array [f1, f2] : [function; 2]
    acir(inline) fn main f0 {
      b0(v3: u32, v4: Field):
        v6 = call f1() -> Field
        v8 = call f2() -> Field
        v10 = lt v3, u32 2
        constrain v10 == u1 1
        v12 = array_get g2, index v3 -> function
        v13 = call v12() -> Field
        v14 = eq v13, v4
        constrain v13 == v4
        return
    }
    acir(inline) fn f1 f1 {
      b0():
        return Field 1
    }
    acir(inline) fn f2 f2 {
      b0():
        return Field 2
    }
    ";
    let _ = Ssa::from_str_no_validation(src).unwrap();
}

#[test]
#[should_panic(expected = "Unknown global")]
fn unknown_function_global_function_pointer() {
    let src = "
    g2 = make_array [f1, f2] : [function; 2]
    acir(inline) fn main f0 {
      b0(v3: u32, v4: Field):
        v6 = call f1() -> Field
        v8 = call f2() -> Field
        v10 = lt v3, u32 2
        constrain v10 == u1 1
        v12 = array_get g2, index v3 -> function
        v13 = call v12() -> Field
        v14 = eq v13, v4
        constrain v13 == v4
        return
    }
    ";
    let _ = Ssa::from_str_no_validation(src).unwrap();
}

#[test]
#[should_panic(expected = "Illegal use of offset")]
fn illegal_offset_in_acir_function() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: [Field; 3]):
        v3 = array_set v0, index u32 2 minus 1, value Field 1
        return
    }
    ";
    let _ = Ssa::from_str_no_validation(src).unwrap();
}

#[test]
fn call_data_and_return_data() {
    let src = "
    acir(inline) predicate_pure fn main f0 {
      call_data(0): array: v18, indices: [v2: 1]
      call_data(1): array: v22, indices: [v3: 1, v4: 3]
      call_data(2): array: v22, indices: []
      return_data: v22
      b0(v0: u32, v1: u32, v2: [u32; 4], v3: [Field; 4], v4: [Field; 4]):
        v5 = cast v1 as Field
        v7 = array_get v2, index u32 0 -> u32
        v8 = cast v7 as Field
        v10 = array_get v2, index u32 1 -> u32
        v11 = cast v10 as Field
        v13 = array_get v2, index u32 2 -> u32
        v14 = cast v13 as Field
        v16 = array_get v2, index u32 3 -> u32
        v17 = cast v16 as Field
        v18 = make_array [v5, v8, v11, v14, v17] : [Field; 5]
        v19 = array_get v2, index v0 -> u32
        v20 = add v19, u32 1
        v21 = cast v20 as Field
        v22 = make_array [v21] : [Field; 1]
        return v22
    }
    ";
    assert_ssa_roundtrip(src);
}

#[test]
fn call_data_without_return_data() {
    let src = "
    acir(inline) predicate_pure fn main f0 {
      call_data(0): array: v16, indices: [v2: 1]
      b0(v0: u32, v1: u32, v2: [u32; 4]):
        v3 = cast v1 as Field
        v5 = array_get v2, index u32 0 -> u32
        v6 = cast v5 as Field
        v8 = array_get v2, index u32 1 -> u32
        v9 = cast v8 as Field
        v11 = array_get v2, index u32 2 -> u32
        v12 = cast v11 as Field
        v14 = array_get v2, index u32 3 -> u32
        v15 = cast v14 as Field
        v16 = make_array [v3, v6, v9, v12, v15] : [Field; 5]
        v17 = array_get v2, index v0 -> u32
        v18 = add v17, u32 1
        v19 = cast v18 as Field
        v20 = make_array [v19] : [Field; 1]
        return v20
    }
    ";
    assert_ssa_roundtrip(src);
}

#[test]
fn return_data_without_call_data() {
    let src = "
    acir(inline) predicate_pure fn main f0 {
      return_data: v20
      b0(v0: u32, v1: u32, v2: [u32; 4]):
        v3 = cast v1 as Field
        v5 = array_get v2, index u32 0 -> u32
        v6 = cast v5 as Field
        v8 = array_get v2, index u32 1 -> u32
        v9 = cast v8 as Field
        v11 = array_get v2, index u32 2 -> u32
        v12 = cast v11 as Field
        v14 = array_get v2, index u32 3 -> u32
        v15 = cast v14 as Field
        v16 = make_array [v3, v6, v9, v12, v15] : [Field; 5]
        v17 = array_get v2, index v0 -> u32
        v18 = add v17, u32 1
        v19 = cast v18 as Field
        v20 = make_array [v19] : [Field; 1]
        return v20
    }
    ";
    assert_ssa_roundtrip(src);
}
