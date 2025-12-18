use crate::ssa::interpreter::{
    errors::InterpreterError,
    tests::{expect_error, expect_printed_output, expect_value},
    value::{NumericValue, Value},
};

#[test]
fn to_le_bits() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = add Field 0, Field 0
            v1 = call to_le_bits(v0) -> [u1; 2]
            v2 = array_get v1, index u32 0 -> u1
            v3 = not v2
            return v3
        }
    ",
    );
    assert_eq!(value, Value::bool(true));
}

#[test]
fn to_le_radix() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
          b0():
            v0 = add Field 0, Field 0
            v1 = call to_le_radix(v0, u32 2) -> [u8; 32]
            v2 = array_get v1, index u32 0 -> u8
            v3 = not v2
            return v3
        }
    ",
    );
    assert_eq!(value, Value::u8(255));
}

#[test]
fn as_witness() {
    let value = expect_value(
        "
        acir(inline) fn main f0 {
        b0():
            v0 = add Field 0, Field 1
            call as_witness(v0)
            return v0
        }
    ",
    );
    assert_eq!(value, Value::Numeric(NumericValue::Field(1_u128.into())));
}

#[test]
fn print() {
    let src = r#"
        acir(inline) fn main f0 {
          b0():
            call f1(u8 123)
            v4 = make_array [Field 1, Field 2] : [Field; 2]
            call f2(i8 0, u1 1, v4)
            v16 = make_array b"hello {a} == {a}"
            call f3(v16, Field 2, i8 0, u1 1, v4, i8 0, u1 1, v4)
            return
        }
        acir(inline) fn println f1 {
          b0(v0: u8):
            call f6(u1 1, v0)
            return
        }
        acir(inline) fn println f2 {
          b0(v0: i8, v1: u1, v2: [Field; 2]):
            call f5(u1 1, v0, v1, v2)
            return
        }
        acir(inline) fn println f3 {
          b0(v0: [u8; 16], v1: Field, v2: i8, v3: u1, v4: [Field; 2], v5: i8, v6: u1, v7: [Field; 2]):
            call f4(u1 1, v0, v1, v2, v3, v4, v5, v6, v7)
            return
        }
        brillig(inline) fn print_unconstrained f4 {
          b0(v0: u1, v1: [u8; 16], v2: Field, v3: i8, v4: u1, v5: [Field; 2], v6: i8, v7: u1, v8: [Field; 2]):
            v37 = make_array b"{\"kind\":\"tuple\",\"types\":[{\"kind\":\"signedinteger\",\"width\":8},{\"kind\":\"boolean\"},{\"kind\":\"array\",\"length\":2,\"type\":{\"kind\":\"field\"}}]}"
            v38 = make_array b"{\"kind\":\"tuple\",\"types\":[{\"kind\":\"signedinteger\",\"width\":8},{\"kind\":\"boolean\"},{\"kind\":\"array\",\"length\":2,\"type\":{\"kind\":\"field\"}}]}"
            call print(v0, v1, v2, v3, v4, v5, v6, v7, v8, v37, v38, u1 1)
            return
        }
        brillig(inline) fn print_unconstrained f5 {
          b0(v0: u1, v1: i8, v2: u1, v3: [Field; 2]):
            v32 = make_array b"{\"kind\":\"tuple\",\"types\":[{\"kind\":\"signedinteger\",\"width\":8},{\"kind\":\"boolean\"},{\"kind\":\"array\",\"length\":2,\"type\":{\"kind\":\"field\"}}]}"
            call print(v0, v1, v2, v3, v32, u1 0)
            return
        }
        brillig(inline) fn print_unconstrained f6 {
          b0(v0: u1, v1: u8):
            v20 = make_array b"{\"kind\":\"unsignedinteger\",\"width\":8}"
            call print(v0, v1, v20, u1 0)
            return
        }
    "#;

    let printed_output = expect_printed_output(src);

    insta::assert_snapshot!(printed_output, @"
    123
    (0, true, [0x01, 0x02])
    hello (0, true, [0x01, 0x02]) == (0, true, [0x01, 0x02])
    ");
}

#[test]
fn print_lambda() {
    // fn main() {
    //     let y = 10;
    //     let z = 20;
    //     let foo = |x| x + y + z;
    //     println(foo);
    // }
    let src = r#"
    acir(inline) fn main f0 {
      b0():
        call f2(Field 10, Field 20, f1)
        return
    }
    acir(inline) fn lambda f1 {
      b0(v0: Field, v1: Field, v2: Field):
        v3 = allocate -> &mut Field
        store v0 at v3
        v4 = allocate -> &mut Field
        store v1 at v4
        v5 = load v3 -> Field
        v6 = load v4 -> Field
        v7 = add v2, v5
        v8 = load v3 -> Field
        v9 = load v4 -> Field
        v10 = add v7, v9
        return v10
    }
    acir(inline) fn println f2 {
      b0(v0: Field, v1: Field, v2: function):
        call f3(u1 1, v0, v1, v2)
        return
    }
    brillig(inline) fn print_unconstrained f3 {
      b0(v0: u1, v1: Field, v2: Field, v3: function):
        v31 = make_array b"{\"kind\":\"function\",\"arguments\":[{\"kind\":\"field\"}],\"return_type\":{\"kind\":\"field\"},\"env\":{\"kind\":\"tuple\",\"types\":[{\"kind\":\"field\"},{\"kind\":\"field\"}]},\"unconstrained\":false}"
        call print(v0, v1, v2, v3, v31, u1 0)
        return
    }
    "#;

    let printed_output = expect_printed_output(src);

    insta::assert_snapshot!(printed_output, @"<<fn(Field) -> Field>>");
}

#[test]
fn vector_pop_from_empty() {
    // Initial SSA of the following program:
    // fn main() -> pub Field {
    //     let s: [Field] = &[0];
    //     let (s, _) = s.pop_back();
    //     let (_, f) = s.pop_back();
    //     f
    // }
    let err = expect_error(
        "
    acir(inline) fn main f0 {
      b0():
        v1 = make_array [Field 0] : [Field]           	// src/main.nr:2:24
        v4 = unchecked_sub u32 0, u32 1
        v6, v7, v8 = call vector_pop_back(u32 0, v1) -> (u32, [Field], Field)	// src/main.nr:4:18
        return v8
    }
    ",
    );
    assert!(matches!(err, InterpreterError::PoppedFromEmptyVector { .. }));
}
