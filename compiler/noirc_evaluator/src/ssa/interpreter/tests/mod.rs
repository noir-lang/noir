#![cfg(test)]

use std::sync::Arc;

use acvm::{AcirField, FieldElement};
use insta::assert_snapshot;

use crate::ssa::{
    interpreter::{
        Interpreter, InterpreterOptions,
        value::{ArrayValue, NumericValue},
    },
    ir::{
        function::FunctionId,
        types::{NumericType, Type},
    },
};

use super::{InterpreterError, Ssa, Value};

mod black_box;
mod instructions;
mod intrinsics;

#[track_caller]
fn executes_with_no_errors(src: &str) {
    let ssa = Ssa::from_str(src).unwrap();
    if let Err(error) = ssa.interpret(Vec::new()) {
        panic!("{error}");
    }
}

#[track_caller]
fn expect_values(src: &str) -> Vec<Value> {
    expect_values_with_args(src, Vec::new())
}

#[track_caller]
fn expect_value(src: &str) -> Value {
    expect_value_with_args(src, Vec::new())
}

#[track_caller]
fn expect_error(src: &str) -> InterpreterError {
    let ssa = Ssa::from_str(src).unwrap();
    ssa.interpret(Vec::new()).unwrap_err()
}

#[track_caller]
fn expect_values_with_args(src: &str, args: Vec<Value>) -> Vec<Value> {
    let ssa = Ssa::from_str(src).unwrap();
    ssa.interpret(args).unwrap()
}

#[track_caller]
pub(crate) fn expect_value_with_args(src: &str, args: Vec<Value>) -> Value {
    let mut results = expect_values_with_args(src, args);
    assert_eq!(results.len(), 1);
    results.pop().unwrap()
}

#[track_caller]
fn expect_printed_output(src: &str) -> String {
    let mut output = Vec::new();
    let ssa = Ssa::from_str(src).unwrap();
    let _ = ssa
        .interpret_with_options(Vec::new(), Default::default(), &mut output)
        .expect("interpret not expected to fail");
    String::from_utf8(output).expect("not a UTF-8 string")
}

pub(crate) fn from_constant(constant: FieldElement, typ: NumericType) -> Value {
    Value::from_constant(constant, typ).unwrap()
}

fn from_u32_slice(slice: &[u32], typ: NumericType) -> Value {
    let values = slice.iter().map(|v| from_constant(u128::from(*v).into(), typ)).collect();
    Value::array(values, vec![Type::Numeric(typ)])
}

#[test]
fn value_snapshot_detaches_from_original() {
    // Create a `[[bool; 2]; 2]` of all `false` values.
    let v0 = {
        let a0 = Value::array(vec![Value::bool(false), Value::bool(false)], vec![Type::bool()]);
        let a1 = Value::array(vec![Value::bool(false), Value::bool(false)], vec![Type::bool()]);
        Value::array(vec![a0, a1], vec![Type::Array(Arc::new(vec![Type::bool()]), 2)])
    };
    // Take a clone and a snapshot, to demonstrate the difference.
    let v1 = v0.clone();
    let v2 = v0.snapshot();

    // Access `array[0][0]`
    fn with_0_0<F>(value: &Value, f: F)
    where
        F: FnOnce(&mut bool),
    {
        let Value::ArrayOrSlice(ArrayValue { elements, .. }) = value else {
            unreachable!("values are arrays")
        };
        let elements = elements.borrow_mut();
        let value = &elements[0];
        let Value::ArrayOrSlice(ArrayValue { elements, .. }) = value else {
            unreachable!("inner values are arrays")
        };
        let mut elements = elements.borrow_mut();
        let mut value = &mut elements[0];
        let Value::Numeric(NumericValue::U1(b)) = &mut value else {
            unreachable!("elements are bool");
        };
        f(b);
    }

    // Update the original.
    with_0_0(&v0, |b| {
        *b = true;
    });
    // The clone is also changed.
    with_0_0(&v1, |b| assert!(*b));
    // The snapshot is not changed.
    with_0_0(&v2, |b| assert!(!(*b)));
}

#[test]
fn empty_program() {
    let src = "
        acir(inline) fn main f0 {
          b0():
            return
        }
    ";
    executes_with_no_errors(src);
}

#[test]
fn return_all_numeric_constant_types() {
    let src = "
        acir(inline) fn main f0 {
          b0():
            return Field 0, u1 1, u8 2, u16 3, u32 4, u64 5, u128 6, i8 255, i16 65534, i32 4294967293, i64 18446744073709551612
        }
    ";
    let returns = expect_values(src);
    assert_eq!(returns.len(), 11);

    assert_eq!(returns[0], Value::field(FieldElement::zero()));
    assert_eq!(returns[1], Value::bool(true));
    assert_eq!(returns[2], Value::u8(2));
    assert_eq!(returns[3], Value::u16(3));
    assert_eq!(returns[4], Value::u32(4));
    assert_eq!(returns[5], Value::u64(5));
    assert_eq!(returns[6], Value::u128(6));
    assert_eq!(returns[7], Value::i8(-1));
    assert_eq!(returns[8], Value::i16(-2));
    assert_eq!(returns[9], Value::i32(-3));
    assert_eq!(returns[10], Value::i64(-4));
}

#[test]
fn call_function() {
    let src = "
        acir(inline) fn main f0 {
          b0():
            v1 = call f1(u32 3) -> u32
            return v1
        }

        acir(inline) fn double f1 {
          b0(v1: u32):
            v2 = mul v1, u32 2
            return v2
        }
    ";
    let actual = expect_value(src);
    assert_eq!(Value::u32(6), actual);
}

#[test]
fn run_flattened_function() {
    let src = "
        acir(inline) pure fn main f0 {
          b0(v0: u1, v1: [[u1; 2]; 3]):
            v2 = not v0
            enable_side_effects v0
            v3 = not v0
            enable_side_effects v0
            v5 = array_get v1, index u32 0 -> [u1; 2]
            v6 = not v0
            v7 = unchecked_mul v0, v6
            enable_side_effects v7
            v8 = array_get v1, index u32 1 -> [u1; 2]
            enable_side_effects v0
            v9 = if v0 then v5 else (if v7) v8
            enable_side_effects v6
            v10 = array_get v1, index u32 2 -> [u1; 2]
            enable_side_effects u1 1
            v12 = if v0 then v5 else (if v6) v10
            return v12
        }";

    let v1_elements = vec![
        Value::array(vec![Value::bool(false), Value::bool(false)], vec![Type::unsigned(1)]),
        Value::array(vec![Value::bool(true), Value::bool(true)], vec![Type::unsigned(1)]),
        Value::array(vec![Value::bool(false), Value::bool(true)], vec![Type::unsigned(1)]),
    ];

    let v1_element_types = vec![Type::Array(Arc::new(vec![Type::unsigned(1)]), 2)];
    let v1 = Value::array(v1_elements, v1_element_types);

    let result = expect_value_with_args(src, vec![Value::bool(true), v1.clone()]);
    assert_snapshot!(result.to_string(), @"rc1 [u1 0, u1 0]");

    let result = expect_value_with_args(src, vec![Value::bool(false), v1]);
    assert_snapshot!(result.to_string(), @"rc1 [u1 0, u1 1]");
}

#[test]
fn loads_passed_to_a_call() {
    let src = "
    acir(inline) fn main f0 {
      b0():
        v1 = allocate -> &mut Field
        store Field 0 at v1
        v3 = allocate -> &mut &mut Field
        store v1 at v3
        jmp b1(Field 0)
      b1(v0: Field):
        v4 = eq v0, Field 0
        jmpif v4 then: b3, else: b2
      b2():
        v9 = load v1 -> Field
        v10 = eq v9, Field 2
        constrain v9 == Field 2
        v11 = load v3 -> &mut Field
        call f1(v11)
        v13 = load v3 -> &mut Field
        v14 = load v13 -> Field
        v15 = eq v14, Field 2
        constrain v14 == Field 2
        return v14
      b3():
        v5 = load v3 -> &mut Field
        store Field 2 at v5
        v8 = add v0, Field 1
        jmp b1(v8)
    }
    acir(inline) fn foo f1 {
      b0(v0: &mut Field):
        return
    }
    ";

    let value = expect_value(src);
    assert_eq!(value, from_constant(2_u128.into(), NumericType::NativeField));
}

#[test]
fn without_defunctionalize() {
    let src = "
  acir(inline) fn main f0 {
    b0(v0: u1):
      v1 = allocate -> &mut function
      store f1 at v1
      jmpif v0 then: b1, else: b2
    b1():
      call f2(v1, f1)
      jmp b3()
    b2():
      call f2(v1, f3)
      jmp b3()
    b3():
      v5 = load v1 -> function
      v7 = call f4(v5) -> u1
      constrain v7 == u1 1
      return
  }
  acir(inline) fn f f1 {
    b0(v0: u8):
      v2 = eq v0, u8 0
      return v2
  }
  acir(inline) fn bar f2 {
    b0(v0: &mut function, v1: function):
      store v1 at v0
      return
  }
  acir(inline) fn g f3 {
    b0(v0: u8):
      v2 = eq v0, u8 1
      return v2
  }
  acir(inline) fn foo f4 {
    b0(v0: function):
      v2 = call v0(u8 0) -> u1
      v4 = call v0(u8 1) -> u1
      v5 = eq v2, v4
      v6 = not v5
      return v6
  }
      ";
    let values =
        expect_values_with_args(src, vec![from_constant(0_u128.into(), NumericType::bool())]);

    assert!(values.is_empty());
}

#[test]
fn keep_repeat_loads_with_alias_store() {
    let src = "
    acir(inline) fn main f0 {
      b0(v0: u1):
        jmpif v0 then: b2, else: b1
      b1():
        v6 = allocate -> &mut Field
        store Field 1 at v6
        jmp b3(v6, v6, v6)
      b2():
        v4 = allocate -> &mut Field
        store Field 0 at v4
        jmp b3(v4, v4, v4)
      b3(v1: &mut Field, v2: &mut Field, v3: &mut Field):
        v8 = load v1 -> Field
        store Field 2 at v2
        v10 = load v1 -> Field
        store Field 1 at v3
        v11 = load v1 -> Field
        store Field 3 at v3
        v13 = load v1 -> Field
        constrain v8 == Field 0
        constrain v10 == Field 2
        constrain v11 == Field 1
        constrain v13 == Field 3
        return v8, v11
    }
    ";

    let values = expect_values_with_args(src, vec![Value::bool(true)]);
    assert_eq!(values.len(), 2);

    assert_eq!(values[0], from_constant(FieldElement::zero(), NumericType::NativeField));
    assert_eq!(values[1], from_constant(FieldElement::one(), NumericType::NativeField));
}

#[test]
fn accepts_globals() {
    let src = "
        g0 = Field 1
        g1 = Field 2
        g2 = make_array [Field 1, Field 2] : [Field; 2]

        brillig(inline) predicate_pure fn main f0 {
        b0():
            v0 = make_array [Field 1, Field 2] : [Field; 2]
            constrain v0 == g2
            return
        }
    ";
    executes_with_no_errors(src);
}

#[test]
fn accepts_print() {
    // fn main(x: Field) {
    //     print(x);
    //     println(x);
    // }
    let src = r#"
        brillig(inline) impure fn main f0 {
        b0(v0: Field):
            v12 = make_array b"{\"kind\":\"field\"}"
            call print(u1 0, v0, v12, u1 0)
            inc_rc v12
            call print(u1 1, v0, v12, u1 0)
            return
        }
    "#;
    let values =
        expect_values_with_args(src, vec![from_constant(5u128.into(), NumericType::NativeField)]);
    assert_eq!(values.len(), 0);
}

#[test]
fn calls_with_higher_order_function() {
    let src = r#"
        acir(inline) fn main f0 {
          b0(v0: Field):
            v4 = call f2(f1) -> function
            v5 = call f3(v4) -> function
            v6 = call v5(v0) -> Field
            return v6
        }

        acir(inline) fn square f1 {
          b0(v0: Field):
            v1 = mul v0, v0
            return v1
        }

        acir(inline) fn id1 f2 {
          b0(v0: function):
            return v0
        }

        acir(inline) fn id2 f3 {
          b0(v0: function):
            return v0
        }
    "#;

    // Program simplifies to `mul v0, v0` if inlined
    let input = from_constant(4u128.into(), NumericType::NativeField);
    let output = from_constant(16u128.into(), NumericType::NativeField);
    let result = expect_value_with_args(src, vec![input]);
    assert_eq!(result, output);
}

#[test]
fn is_odd_is_even_recursive_calls() {
    let src = r#"
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u1):
            v3 = call f2(v0) -> u1
            v4 = eq v3, v1
            constrain v3 == v1
            return
        }
        brillig(inline) fn is_even f1 {
          b0(v0: u32):
            v3 = eq v0, u32 0
            jmpif v3 then: b2, else: b1
          b1():
            v5 = call f3(v0) -> u32
            v7 = call f2(v5) -> u1
            jmp b3(v7)
          b2():
            jmp b3(u1 1)
          b3(v1: u1):
            return v1
        }
        brillig(inline) fn is_odd f2 {
          b0(v0: u32):
            v3 = eq v0, u32 0
            jmpif v3 then: b2, else: b1
          b1():
            v5 = call f3(v0) -> u32
            v7 = call f1(v5) -> u1
            jmp b3(v7)
          b2():
            jmp b3(u1 0)
          b3(v1: u1):
            return v1
        }
        brillig(inline) fn decrement f3 {
          b0(v0: u32):
            v2 = sub v0, u32 1
            return v2
        }
    "#;
    let seven = from_constant(7_u128.into(), NumericType::unsigned(32));
    let values = expect_values_with_args(src, vec![seven, Value::bool(true)]);
    assert!(values.is_empty());
}

#[test]
fn store_with_aliases() {
    let src = r#"
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 0 at v0
            v2 = allocate -> &mut &mut Field
            store v0 at v2
            jmp b1(Field 0)
          b1(v3: Field):
            v4 = eq v3, Field 0
            jmpif v4 then: b2, else: b3
          b2():
            v5 = load v2 -> &mut Field
            store Field 2 at v5
            v8 = add v3, Field 1
            jmp b1(v8)
          b3():
            v9 = load v0 -> Field
            v10 = eq v9, Field 2
            constrain v9 == Field 2
            v11 = load v2 -> &mut Field
            v12 = load v11 -> Field
            constrain v12 == Field 2
            return
        }
    "#;
    executes_with_no_errors(src);
}

#[test]
fn literally_just_the_slices_integration_test() {
    let src = r#"
acir(inline) fn main f0 {
  b0(v0: Field, v1: Field):
    v4 = make_array [Field 0, Field 0] : [Field]
    v5 = allocate -> &mut u32
    store u32 2 at v5
    v7 = allocate -> &mut [Field]
    store v4 at v7
    v8 = load v5 -> u32
    v9 = load v7 -> [Field]
    v11 = lt u32 0, v8
    constrain v11 == u1 1, "Index out of bounds"
    v13 = array_get v9, index u32 0 -> Field
    v14 = eq v13, Field 0
    constrain v13 == Field 0
    v15 = load v5 -> u32
    v16 = load v7 -> [Field]
    v17 = lt u32 0, v15
    constrain v17 == u1 1, "Index out of bounds"
    v18 = array_get v16, index u32 0 -> Field
    v20 = eq v18, Field 1
    v21 = not v20
    constrain v20 == u1 0
    v23 = load v5 -> u32
    v24 = load v7 -> [Field]
    v25 = lt u32 0, v23
    constrain v25 == u1 1, "Index out of bounds"
    v26 = array_set v24, index u32 0, value v0
    store v23 at v5
    store v26 at v7
    v27 = load v5 -> u32
    v28 = load v7 -> [Field]
    v29 = lt u32 0, v27
    constrain v29 == u1 1, "Index out of bounds"
    v30 = array_get v28, index u32 0 -> Field
    v31 = eq v30, v0
    constrain v30 == v0
    v32 = load v5 -> u32
    v33 = load v7 -> [Field]
    v35, v36 = call slice_push_back(v32, v33, v1) -> (u32, [Field])
    v37 = lt u32 2, v35
    constrain v37 == u1 1, "Index out of bounds"
    v38 = array_get v36, index u32 2 -> Field
    v40 = eq v38, Field 10
    constrain v38 == Field 10
    v41 = lt u32 2, v35
    constrain v41 == u1 1, "Index out of bounds"
    v42 = array_get v36, index u32 2 -> Field
    v44 = eq v42, Field 8
    v45 = not v44
    constrain v44 == u1 0
    v47 = eq v35, u32 3
    constrain v35 == u32 3
    v48 = make_array [] : [u32]
    v49 = allocate -> &mut u32
    store u32 0 at v49
    v50 = allocate -> &mut [u32]
    store v48 at v50
    jmp b1(u32 0)
  b1(v2: u32):
    v52 = lt v2, u32 5
    jmpif v52 then: b2, else: b3
  b2():
    v167 = load v49 -> u32
    v168 = load v50 -> [u32]
    v169, v170 = call slice_push_back(v167, v168, v2) -> (u32, [u32])
    store v169 at v49
    store v170 at v50
    v171 = unchecked_add v2, u32 1
    jmp b1(v171)
  b3():
    v53 = load v49 -> u32
    v54 = load v50 -> [u32]
    v55 = eq v53, u32 5
    constrain v53 == u32 5
    v56 = load v49 -> u32
    v57 = load v50 -> [u32]
    v60, v61 = call slice_push_front(v56, v57, u32 20) -> (u32, [u32])
    store v60 at v49
    store v61 at v50
    v62 = load v49 -> u32
    v63 = load v50 -> [u32]
    v64 = lt u32 0, v62
    constrain v64 == u1 1, "Index out of bounds"
    v65 = array_get v63, index u32 0 -> u32
    v66 = eq v65, u32 20
    constrain v65 == u32 20
    v67 = load v49 -> u32
    v68 = load v50 -> [u32]
    v70 = eq v67, u32 6
    constrain v67 == u32 6
    v71 = load v49 -> u32
    v72 = load v50 -> [u32]
    v74, v75, v76 = call slice_pop_back(v71, v72) -> (u32, [u32], u32)
    v78 = eq v76, u32 4
    constrain v76 == u32 4
    v79 = eq v74, u32 5
    constrain v74 == u32 5
    v81, v82, v83 = call slice_pop_front(v74, v75) -> (u32, u32, [u32])
    v84 = eq v81, u32 20
    constrain v81 == u32 20
    v85 = eq v82, u32 4
    constrain v82 == u32 4
    v87 = add v82, u32 1
    v88 = lt u32 2, v87
    constrain v88 == u1 1, "Index out of bounds"
    v91, v92 = call slice_insert(v82, v83, u32 2, u32 100) -> (u32, [u32])
    store v91 at v49
    store v92 at v50
    v93 = load v49 -> u32
    v94 = load v50 -> [u32]
    v95 = lt u32 2, v93
    constrain v95 == u1 1, "Index out of bounds"
    v96 = array_get v94, index u32 2 -> u32
    v97 = eq v96, u32 100
    constrain v96 == u32 100
    v98 = load v49 -> u32
    v99 = load v50 -> [u32]
    v100 = lt u32 4, v98
    constrain v100 == u1 1, "Index out of bounds"
    v101 = array_get v99, index u32 4 -> u32
    v102 = eq v101, u32 3
    constrain v101 == u32 3
    v103 = load v49 -> u32
    v104 = load v50 -> [u32]
    v105 = eq v103, u32 5
    constrain v103 == u32 5
    v106 = load v49 -> u32
    v107 = load v50 -> [u32]
    v108 = lt u32 3, v106
    constrain v108 == u1 1, "Index out of bounds"
    v110, v111, v112 = call slice_remove(v106, v107, u32 3) -> (u32, [u32], u32)
    v113 = eq v112, u32 2
    constrain v112 == u32 2
    v114 = lt u32 3, v110
    constrain v114 == u1 1, "Index out of bounds"
    v115 = array_get v111, index u32 3 -> u32
    v116 = eq v115, u32 3
    constrain v115 == u32 3
    v117 = eq v110, u32 4
    constrain v110 == u32 4
    v119 = make_array [Field 1, Field 2] : [Field]
    v123 = make_array [Field 3, Field 4, Field 5] : [Field]
    v125, v126 = call f1(u32 2, v119, u32 3, v123) -> (u32, [Field])
    v127 = eq v125, u32 5
    constrain v125 == u32 5
    v128 = lt u32 0, v125
    constrain v128 == u1 1, "Index out of bounds"
    v129 = array_get v126, index u32 0 -> Field
    v130 = eq v129, Field 1
    constrain v129 == Field 1
    v131 = lt u32 4, v125
    constrain v131 == u1 1, "Index out of bounds"
    v132 = array_get v126, index u32 4 -> Field
    v133 = eq v132, Field 5
    constrain v132 == Field 5
    v134 = make_array [Field 1, Field 2] : [Field]
    v137, v138 = call f2(u32 2, v134, f3) -> (u32, [Field])
    v139 = make_array [Field 2, Field 3] : [Field]
    v141 = call f4(v137, v138, u32 2, v139) -> u1
    constrain v141 == u1 1
    v142 = make_array [Field 1, Field 2, Field 3] : [Field]
    v145 = call f5(u32 3, v142, Field 0, f6) -> Field
    v147 = eq v145, Field 6
    constrain v145 == Field 6
    v148 = make_array [Field 1, Field 2, Field 3] : [Field]
    v151 = call f7(u32 3, v148, f8) -> Field
    v152 = eq v151, Field 6
    constrain v151 == Field 6
    v153 = make_array [u32 2, u32 4, u32 6] : [u32]
    v156 = call f9(u32 3, v153, f10) -> u1
    constrain v156 == u1 1
    v157 = make_array [u32 2, u32 4, u32 6] : [u32]
    v160 = call f11(u32 3, v157, f12) -> u1
    constrain v160 == u1 1
    call f13()
    call f14(v0, v1)
    call f15()
    call f16(v0)
    call f17(v0, v1)
    call f18()
    return
}
acir(inline) fn append f1 {
  b0(v0: u32, v1: [Field], v2: u32, v3: [Field]):
    v5 = allocate -> &mut u32
    store v0 at v5
    v6 = allocate -> &mut [Field]
    store v1 at v6
    jmp b1(u32 0)
  b1(v4: u32):
    v8 = lt v4, v2
    jmpif v8 then: b2, else: b3
  b2():
    v11 = lt v4, v2
    constrain v11 == u1 1, "Index out of bounds"
    v13 = array_get v3, index v4 -> Field
    v14 = load v5 -> u32
    v15 = load v6 -> [Field]
    v17, v18 = call slice_push_back(v14, v15, v13) -> (u32, [Field])
    store v17 at v5
    store v18 at v6
    v20 = unchecked_add v4, u32 1
    jmp b1(v20)
  b3():
    v9 = load v5 -> u32
    v10 = load v6 -> [Field]
    return v9, v10
}
acir(inline) fn map f2 {
  b0(v0: u32, v1: [Field], v2: function):
    v4 = make_array [] : [Field]
    v5 = allocate -> &mut u32
    store u32 0 at v5
    v7 = allocate -> &mut [Field]
    store v4 at v7
    jmp b1(u32 0)
  b1(v3: u32):
    v8 = lt v3, v0
    jmpif v8 then: b2, else: b3
  b2():
    v11 = lt v3, v0
    constrain v11 == u1 1, "Index out of bounds"
    v13 = array_get v1, index v3 -> Field
    v14 = load v5 -> u32
    v15 = load v7 -> [Field]
    v16 = call v2(v13) -> Field
    v18, v19 = call slice_push_back(v14, v15, v16) -> (u32, [Field])
    store v18 at v5
    store v19 at v7
    v21 = unchecked_add v3, u32 1
    jmp b1(v21)
  b3():
    v9 = load v5 -> u32
    v10 = load v7 -> [Field]
    return v9, v10
}
acir(inline) fn lambda f3 {
  b0(v0: Field):
    v2 = add v0, Field 1
    return v2
}
acir(inline) fn eq f4 {
  b0(v0: u32, v1: [Field], v2: u32, v3: [Field]):
    v5 = eq v0, v2
    v6 = allocate -> &mut u1
    store v5 at v6
    jmp b1(u32 0)
  b1(v4: u32):
    v8 = lt v4, v0
    jmpif v8 then: b2, else: b3
  b2():
    v10 = load v6 -> u1
    v11 = lt v4, v0
    constrain v11 == u1 1, "Index out of bounds"
    v13 = array_get v1, index v4 -> Field
    v14 = lt v4, v2
    constrain v14 == u1 1, "Index out of bounds"
    v15 = array_get v3, index v4 -> Field
    v17 = call f31(v13, v15) -> u1
    v18 = unchecked_mul v10, v17
    store v18 at v6
    v20 = unchecked_add v4, u32 1
    jmp b1(v20)
  b3():
    v9 = load v6 -> u1
    return v9
}
acir(inline) fn fold f5 {
  b0(v0: u32, v1: [Field], v2: Field, v3: function):
    v5 = allocate -> &mut Field
    store v2 at v5
    jmp b1(u32 0)
  b1(v4: u32):
    v7 = lt v4, v0
    jmpif v7 then: b2, else: b3
  b2():
    v9 = lt v4, v0
    constrain v9 == u1 1, "Index out of bounds"
    v11 = array_get v1, index v4 -> Field
    v12 = load v5 -> Field
    v13 = call v3(v12, v11) -> Field
    store v13 at v5
    v15 = unchecked_add v4, u32 1
    jmp b1(v15)
  b3():
    v8 = load v5 -> Field
    return v8
}
acir(inline) fn lambda f6 {
  b0(v0: Field, v1: Field):
    v2 = add v0, v1
    return v2
}
acir(inline) fn reduce f7 {
  b0(v0: u32, v1: [Field], v2: function):
    v5 = lt u32 0, v0
    constrain v5 == u1 1, "Index out of bounds"
    v7 = array_get v1, index u32 0 -> Field
    v8 = allocate -> &mut Field
    store v7 at v8
    jmp b1(u32 1)
  b1(v3: u32):
    v10 = lt v3, v0
    jmpif v10 then: b2, else: b3
  b2():
    v12 = load v8 -> Field
    v13 = lt v3, v0
    constrain v13 == u1 1, "Index out of bounds"
    v14 = array_get v1, index v3 -> Field
    v15 = call v2(v12, v14) -> Field
    store v15 at v8
    v16 = unchecked_add v3, u32 1
    jmp b1(v16)
  b3():
    v11 = load v8 -> Field
    return v11
}
acir(inline) fn lambda f8 {
  b0(v0: Field, v1: Field):
    v2 = add v0, v1
    return v2
}
acir(inline) fn all f9 {
  b0(v0: u32, v1: [u32], v2: function):
    v4 = allocate -> &mut u1
    store u1 1 at v4
    jmp b1(u32 0)
  b1(v3: u32):
    v7 = lt v3, v0
    jmpif v7 then: b2, else: b3
  b2():
    v9 = lt v3, v0
    constrain v9 == u1 1, "Index out of bounds"
    v10 = array_get v1, index v3 -> u32
    v11 = load v4 -> u1
    v12 = call v2(v10) -> u1
    v13 = unchecked_mul v11, v12
    store v13 at v4
    v15 = unchecked_add v3, u32 1
    jmp b1(v15)
  b3():
    v8 = load v4 -> u1
    return v8
}
acir(inline) fn lambda f10 {
  b0(v0: u32):
    v2 = lt u32 0, v0
    return v2
}
acir(inline) fn any f11 {
  b0(v0: u32, v1: [u32], v2: function):
    v4 = allocate -> &mut u1
    store u1 0 at v4
    jmp b1(u32 0)
  b1(v3: u32):
    v7 = lt v3, v0
    jmpif v7 then: b2, else: b3
  b2():
    v9 = lt v3, v0
    constrain v9 == u1 1, "Index out of bounds"
    v11 = array_get v1, index v3 -> u32
    v12 = load v4 -> u1
    v13 = call v2(v11) -> u1
    v14 = or v12, v13
    store v14 at v4
    v16 = unchecked_add v3, u32 1
    jmp b1(v16)
  b3():
    v8 = load v4 -> u1
    return v8
}
acir(inline) fn lambda f12 {
  b0(v0: u32):
    v2 = lt u32 5, v0
    return v2
}
acir(inline) fn regression_2083 f13 {
  b0():
    v2 = make_array [Field 1, Field 2] : [(Field, Field)]
    v5 = make_array [Field 1, Field 2, Field 3, Field 4] : [(Field, Field)]
    v8 = make_array [Field 1, Field 2, Field 3, Field 4, Field 5, Field 6] : [(Field, Field)]
    v11 = make_array [Field 10, Field 11, Field 1, Field 2, Field 3, Field 4, Field 5, Field 6] : [(Field, Field)]
    v14 = make_array [Field 12, Field 13, Field 10, Field 11, Field 1, Field 2, Field 3, Field 4, Field 5, Field 6] : [(Field, Field)]
    v17 = make_array [Field 12, Field 13, Field 55, Field 56, Field 10, Field 11, Field 1, Field 2, Field 3, Field 4, Field 5, Field 6] : [(Field, Field)]
    v18 = make_array [Field 12, Field 13, Field 55, Field 56, Field 1, Field 2, Field 3, Field 4, Field 5, Field 6] : [(Field, Field)]
    v19 = make_array [Field 55, Field 56, Field 1, Field 2, Field 3, Field 4, Field 5, Field 6] : [(Field, Field)]
    return
}
acir(inline) fn regression_merge_slices f14 {
  b0(v0: Field, v1: Field):
    call f22(v0, v1)
    call f23(v0)
    return
}
acir(inline) fn regression_2370 f15 {
  b0():
    v0 = make_array [] : [Field]
    v1 = allocate -> &mut u32
    store u32 0 at v1
    v3 = allocate -> &mut [Field]
    store v0 at v3
    v7 = make_array [Field 1, Field 2, Field 3] : [Field]
    store u32 3 at v1
    store v7 at v3
    return
}
acir(inline) fn regression_4418 f16 {
  b0(v0: Field):
    v2 = call f20(v0) -> [u8; 32]
    v3 = allocate -> &mut [u8; 32]
    store v2 at v3
    v5 = eq v0, Field 0
    v6 = not v5
    jmpif v6 then: b1, else: b2
  b1():
    v7 = load v3 -> [u8; 32]
    v10 = array_set v7, index u32 0, value u8 10
    store v10 at v3
    jmp b2()
  b2():
    return
}
acir(inline) fn regression_slice_call_result f17 {
  b0(v0: Field, v1: Field):
    v3, v4 = call f19(v0, v1) -> (u32, [Field])
    v5 = allocate -> &mut u32
    store v3 at v5
    v6 = allocate -> &mut [Field]
    store v4 at v6
    v8 = eq v0, Field 0
    v9 = not v8
    jmpif v9 then: b1, else: b2
  b1():
    v16 = load v5 -> u32
    v17 = load v6 -> [Field]
    v18, v19 = call slice_push_back(v16, v17, Field 5) -> (u32, [Field])
    store v18 at v5
    store v19 at v6
    v20 = load v5 -> u32
    v21 = load v6 -> [Field]
    v23, v24 = call slice_push_back(v20, v21, Field 10) -> (u32, [Field])
    store v23 at v5
    store v24 at v6
    jmp b3()
  b2():
    v10 = load v5 -> u32
    v11 = load v6 -> [Field]
    v14, v15 = call slice_push_back(v10, v11, Field 5) -> (u32, [Field])
    store v14 at v5
    store v15 at v6
    jmp b3()
  b3():
    v25 = load v5 -> u32
    v26 = load v6 -> [Field]
    v28 = eq v25, u32 5
    constrain v25 == u32 5
    v29 = load v5 -> u32
    v30 = load v6 -> [Field]
    v32 = lt u32 0, v29
    constrain v32 == u1 1, "Index out of bounds"
    v34 = array_get v30, index u32 0 -> Field
    v35 = eq v34, Field 0
    constrain v34 == Field 0
    v36 = load v5 -> u32
    v37 = load v6 -> [Field]
    v39 = lt u32 1, v36
    constrain v39 == u1 1, "Index out of bounds"
    v40 = array_get v37, index u32 1 -> Field
    v41 = eq v40, Field 0
    constrain v40 == Field 0
    v42 = load v5 -> u32
    v43 = load v6 -> [Field]
    v45 = lt u32 2, v42
    constrain v45 == u1 1, "Index out of bounds"
    v46 = array_get v43, index u32 2 -> Field
    v47 = eq v46, Field 10
    constrain v46 == Field 10
    v48 = load v5 -> u32
    v49 = load v6 -> [Field]
    v51 = lt u32 3, v48
    constrain v51 == u1 1, "Index out of bounds"
    v52 = array_get v49, index u32 3 -> Field
    v53 = eq v52, Field 5
    constrain v52 == Field 5
    v54 = load v5 -> u32
    v55 = load v6 -> [Field]
    v57 = lt u32 4, v54
    constrain v57 == u1 1, "Index out of bounds"
    v58 = array_get v55, index u32 4 -> Field
    v59 = eq v58, Field 10
    constrain v58 == Field 10
    return
}
acir(inline) fn regression_4506 f18 {
  b0():
    v3 = make_array [Field 1, Field 2, Field 3] : [Field]
    v6 = call f4(u32 3, v3, u32 3, v3) -> u1
    constrain v6 == u1 1
    return
}
acir(inline) fn merge_slices_return f19 {
  b0(v0: Field, v1: Field):
    v7 = make_array [Field 0, Field 0] : [Field]
    v8 = eq v0, v1
    v9 = not v8
    jmpif v9 then: b1, else: b2
  b1():
    v12 = eq v0, Field 20
    v13 = not v12
    jmpif v13 then: b3, else: b4
  b2():
    jmp b6(u32 2, v7)
  b3():
    v14 = make_array [Field 0, Field 0, v1] : [Field]
    v15 = make_array [Field 0, Field 0, v1] : [Field]
    v16 = make_array [Field 0, Field 0, v1] : [Field]
    jmp b5(u32 3, v16)
  b4():
    jmp b5(u32 2, v7)
  b5(v2: u32, v3: [Field]):
    jmp b6(v2, v3)
  b6(v4: u32, v5: [Field]):
    return v4, v5
}
acir(inline) fn to_be_bytes f20 {
  b0(v0: Field):
    v31 = make_array [u8 1, u8 0, u8 0, u8 240, u8 147, u8 245, u8 225, u8 67, u8 145, u8 112, u8 185, u8 121, u8 72, u8 232, u8 51, u8 40, u8 93, u8 88, u8 129, u8 129, u8 182, u8 69, u8 80, u8 184, u8 41, u8 160, u8 49, u8 225, u8 114, u8 78, u8 100, u8 48] : [u8]
    v47 = make_array b"N must be less than or equal to modulus_le_bytes().len()"
    v50 = call f21(v0, u32 256) -> [u8; 32]
    v51 = make_array [u8 48, u8 100, u8 78, u8 114, u8 225, u8 49, u8 160, u8 41, u8 184, u8 80, u8 69, u8 182, u8 129, u8 129, u8 88, u8 93, u8 40, u8 51, u8 232, u8 72, u8 121, u8 185, u8 112, u8 145, u8 67, u8 225, u8 245, u8 147, u8 240, u8 0, u8 0, u8 1] : [u8]
    v52 = allocate -> &mut u1
    store u1 0 at v52
    jmp b1(u32 0)
  b1(v1: u32):
    v56 = lt v1, u32 32
    jmpif v56 then: b2, else: b3
  b2():
    v59 = load v52 -> u1
    v60 = not v59
    jmpif v60 then: b4, else: b5
  b3():
    v57 = load v52 -> u1
    constrain v57 == u1 1
    return v50
  b4():
    v61 = lt v1, u32 32
    constrain v61 == u1 1, "Index out of bounds"
    v62 = array_get v50, index v1 -> u8
    v63 = lt v1, u32 32
    constrain v63 == u1 1, "Index out of bounds"
    v64 = array_get v51, index v1 -> u8
    v65 = eq v62, v64
    v66 = not v65
    jmpif v66 then: b6, else: b7
  b5():
    v73 = unchecked_add v1, u32 1
    jmp b1(v73)
  b6():
    v67 = lt v1, u32 32
    constrain v67 == u1 1, "Index out of bounds"
    v68 = array_get v50, index v1 -> u8
    v69 = lt v1, u32 32
    constrain v69 == u1 1, "Index out of bounds"
    v70 = array_get v51, index v1 -> u8
    v71 = lt v68, v70
    constrain v71 == u1 1
    store u1 1 at v52
    jmp b7()
  b7():
    jmp b5()
}
acir(inline) fn to_be_radix f21 {
  b0(v0: Field, v1: u32):
    call assert_constant(v1)
    v4 = call to_be_radix(v0, v1) -> [u8; 32]
    return v4
}
acir(inline) fn merge_slices_if f22 {
  b0(v0: Field, v1: Field):
    v3, v4 = call f19(v0, v1) -> (u32, [Field])
    v6 = eq v3, u32 3
    constrain v3 == u32 3
    v8 = lt u32 2, v3
    constrain v8 == u1 1, "Index out of bounds"
    v10 = array_get v4, index u32 2 -> Field
    v12 = eq v10, Field 10
    constrain v10 == Field 10
    v14, v15 = call f24(v0, v1) -> (u32, [Field])
    v17 = eq v14, u32 4
    constrain v14 == u32 4
    v18 = lt u32 3, v14
    constrain v18 == u1 1, "Index out of bounds"
    v19 = array_get v15, index u32 3 -> Field
    v21 = eq v19, Field 5
    constrain v19 == Field 5
    v23, v24 = call f25(v0, v1) -> (u32, [Field])
    v26 = eq v23, u32 7
    constrain v23 == u32 7
    v28 = lt u32 6, v23
    constrain v28 == u1 1, "Index out of bounds"
    v29 = array_get v24, index u32 6 -> Field
    v31 = eq v29, Field 4
    constrain v29 == Field 4
    v33, v34 = call f26(v0, v1) -> (u32, [Field])
    v35 = eq v33, u32 6
    constrain v33 == u32 6
    v36 = lt u32 3, v33
    constrain v36 == u1 1, "Index out of bounds"
    v37 = array_get v34, index u32 3 -> Field
    v38 = eq v37, Field 5
    constrain v37 == Field 5
    v39 = lt u32 4, v33
    constrain v39 == u1 1, "Index out of bounds"
    v40 = array_get v34, index u32 4 -> Field
    v42 = eq v40, Field 15
    constrain v40 == Field 15
    v44 = lt u32 5, v33
    constrain v44 == u1 1, "Index out of bounds"
    v45 = array_get v34, index u32 5 -> Field
    v47 = eq v45, Field 30
    constrain v45 == Field 30
    v49, v50 = call f27(v0, v1) -> (u32, [Field])
    v52 = eq v49, u32 8
    constrain v49 == u32 8
    v53 = lt u32 3, v49
    constrain v53 == u1 1, "Index out of bounds"
    v54 = array_get v50, index u32 3 -> Field
    v55 = eq v54, Field 5
    constrain v54 == Field 5
    v56 = lt u32 4, v49
    constrain v56 == u1 1, "Index out of bounds"
    v57 = array_get v50, index u32 4 -> Field
    v58 = eq v57, Field 30
    constrain v57 == Field 30
    v59 = lt u32 5, v49
    constrain v59 == u1 1, "Index out of bounds"
    v60 = array_get v50, index u32 5 -> Field
    v61 = eq v60, Field 15
    constrain v60 == Field 15
    v62 = lt u32 6, v49
    constrain v62 == u1 1, "Index out of bounds"
    v63 = array_get v50, index u32 6 -> Field
    v65 = eq v63, Field 50
    constrain v63 == Field 50
    v66 = lt u32 7, v49
    constrain v66 == u1 1, "Index out of bounds"
    v67 = array_get v50, index u32 7 -> Field
    v69 = eq v67, Field 60
    constrain v67 == Field 60
    call f28(v0, v1)
    v72, v73 = call f29(v0, v1) -> (u32, [Field])
    v74 = eq v72, u32 7
    constrain v72 == u32 7
    v76 = lt u32 1, v72
    constrain v76 == u1 1, "Index out of bounds"
    v77 = array_get v73, index u32 1 -> Field
    v78 = eq v77, Field 50
    constrain v77 == Field 50
    v79 = lt u32 2, v72
    constrain v79 == u1 1, "Index out of bounds"
    v80 = array_get v73, index u32 2 -> Field
    v82 = eq v80, Field 0
    constrain v80 == Field 0
    v83 = lt u32 5, v72
    constrain v83 == u1 1, "Index out of bounds"
    v84 = array_get v73, index u32 5 -> Field
    v85 = eq v84, Field 30
    constrain v84 == Field 30
    v86 = lt u32 6, v72
    constrain v86 == u1 1, "Index out of bounds"
    v87 = array_get v73, index u32 6 -> Field
    v89 = eq v87, Field 100
    constrain v87 == Field 100
    v91, v92 = call f30(v0, v1) -> (u32, [Field])
    v93 = eq v91, u32 5
    constrain v91 == u32 5
    return
}
acir(inline) fn merge_slices_else f23 {
  b0(v0: Field):
    v3, v4 = call f19(v0, Field 5) -> (u32, [Field])
    v6 = lt u32 0, v3
    constrain v6 == u1 1, "Index out of bounds"
    v8 = array_get v4, index u32 0 -> Field
    v10 = eq v8, Field 0
    constrain v8 == Field 0
    v12 = lt u32 1, v3
    constrain v12 == u1 1, "Index out of bounds"
    v13 = array_get v4, index u32 1 -> Field
    v14 = eq v13, Field 0
    constrain v13 == Field 0
    v16 = eq v3, u32 2
    constrain v3 == u32 2
    v18, v19 = call f24(v0, Field 5) -> (u32, [Field])
    v20 = lt u32 2, v18
    constrain v20 == u1 1, "Index out of bounds"
    v21 = array_get v19, index u32 2 -> Field
    v22 = eq v21, Field 5
    constrain v21 == Field 5
    v24 = eq v18, u32 3
    constrain v18 == u32 3
    v26, v27 = call f25(v0, Field 5) -> (u32, [Field])
    v28 = lt u32 2, v26
    constrain v28 == u1 1, "Index out of bounds"
    v29 = array_get v27, index u32 2 -> Field
    v30 = eq v29, Field 5
    constrain v29 == Field 5
    v31 = eq v26, u32 3
    constrain v26 == u32 3
    return
}
acir(inline) fn merge_slices_mutate f24 {
  b0(v0: Field, v1: Field):
    v3 = make_array [Field 0, Field 0] : [Field]
    v4 = allocate -> &mut u32
    store u32 2 at v4
    v6 = allocate -> &mut [Field]
    store v3 at v6
    v7 = eq v0, v1
    v8 = not v7
    jmpif v8 then: b1, else: b2
  b1():
    v14 = load v4 -> u32
    v15 = load v6 -> [Field]
    v16, v17 = call slice_push_back(v14, v15, v1) -> (u32, [Field])
    store v16 at v4
    store v17 at v6
    v18 = load v4 -> u32
    v19 = load v6 -> [Field]
    v20, v21 = call slice_push_back(v18, v19, v0) -> (u32, [Field])
    store v20 at v4
    store v21 at v6
    jmp b3()
  b2():
    v9 = load v4 -> u32
    v10 = load v6 -> [Field]
    v12, v13 = call slice_push_back(v9, v10, v0) -> (u32, [Field])
    store v12 at v4
    store v13 at v6
    jmp b3()
  b3():
    v22 = load v4 -> u32
    v23 = load v6 -> [Field]
    return v22, v23
}
acir(inline) fn merge_slices_mutate_in_loop f25 {
  b0(v0: Field, v1: Field):
    v4 = make_array [Field 0, Field 0] : [Field]
    v5 = allocate -> &mut u32
    store u32 2 at v5
    v7 = allocate -> &mut [Field]
    store v4 at v7
    v8 = eq v0, v1
    v9 = not v8
    jmpif v9 then: b1, else: b2
  b1():
    jmp b3(u32 0)
  b2():
    v10 = load v5 -> u32
    v11 = load v7 -> [Field]
    v13, v14 = call slice_push_back(v10, v11, v0) -> (u32, [Field])
    store v13 at v5
    store v14 at v7
    jmp b6()
  b3(v2: u32):
    v17 = lt v2, u32 5
    jmpif v17 then: b4, else: b5
  b4():
    v20 = load v5 -> u32
    v21 = load v7 -> [Field]
    v22 = cast v2 as Field
    v23, v24 = call slice_push_back(v20, v21, v22) -> (u32, [Field])
    store v23 at v5
    store v24 at v7
    v26 = unchecked_add v2, u32 1
    jmp b3(v26)
  b5():
    jmp b6()
  b6():
    v18 = load v5 -> u32
    v19 = load v7 -> [Field]
    return v18, v19
}
acir(inline) fn merge_slices_mutate_two_ifs f26 {
  b0(v0: Field, v1: Field):
    v3 = make_array [Field 0, Field 0] : [Field]
    v4 = allocate -> &mut u32
    store u32 2 at v4
    v6 = allocate -> &mut [Field]
    store v3 at v6
    v7 = eq v0, v1
    v8 = not v7
    jmpif v8 then: b1, else: b2
  b1():
    v14 = load v4 -> u32
    v15 = load v6 -> [Field]
    v16, v17 = call slice_push_back(v14, v15, v1) -> (u32, [Field])
    store v16 at v4
    store v17 at v6
    v18 = load v4 -> u32
    v19 = load v6 -> [Field]
    v20, v21 = call slice_push_back(v18, v19, v0) -> (u32, [Field])
    store v20 at v4
    store v21 at v6
    jmp b3()
  b2():
    v9 = load v4 -> u32
    v10 = load v6 -> [Field]
    v12, v13 = call slice_push_back(v9, v10, v0) -> (u32, [Field])
    store v12 at v4
    store v13 at v6
    jmp b3()
  b3():
    v23 = eq v0, Field 20
    jmpif v23 then: b4, else: b5
  b4():
    v24 = load v4 -> u32
    v25 = load v6 -> [Field]
    v26, v27 = call slice_push_back(v24, v25, Field 20) -> (u32, [Field])
    store v26 at v4
    store v27 at v6
    jmp b5()
  b5():
    v28 = load v4 -> u32
    v29 = load v6 -> [Field]
    v31, v32 = call slice_push_back(v28, v29, Field 15) -> (u32, [Field])
    store v31 at v4
    store v32 at v6
    v33 = load v4 -> u32
    v34 = load v6 -> [Field]
    v36, v37 = call slice_push_back(v33, v34, Field 30) -> (u32, [Field])
    store v36 at v4
    store v37 at v6
    v38 = load v4 -> u32
    v39 = load v6 -> [Field]
    return v38, v39
}
acir(inline) fn merge_slices_mutate_between_ifs f27 {
  b0(v0: Field, v1: Field):
    v3 = make_array [Field 0, Field 0] : [Field]
    v4 = allocate -> &mut u32
    store u32 2 at v4
    v6 = allocate -> &mut [Field]
    store v3 at v6
    v7 = eq v0, v1
    v8 = not v7
    jmpif v8 then: b1, else: b2
  b1():
    v14 = load v4 -> u32
    v15 = load v6 -> [Field]
    v16, v17 = call slice_push_back(v14, v15, v1) -> (u32, [Field])
    store v16 at v4
    store v17 at v6
    v18 = load v4 -> u32
    v19 = load v6 -> [Field]
    v20, v21 = call slice_push_back(v18, v19, v0) -> (u32, [Field])
    store v20 at v4
    store v21 at v6
    jmp b3()
  b2():
    v9 = load v4 -> u32
    v10 = load v6 -> [Field]
    v12, v13 = call slice_push_back(v9, v10, v0) -> (u32, [Field])
    store v12 at v4
    store v13 at v6
    jmp b3()
  b3():
    v22 = load v4 -> u32
    v23 = load v6 -> [Field]
    v25, v26 = call slice_push_back(v22, v23, Field 30) -> (u32, [Field])
    store v25 at v4
    store v26 at v6
    v28 = eq v0, Field 20
    jmpif v28 then: b4, else: b5
  b4():
    v29 = load v4 -> u32
    v30 = load v6 -> [Field]
    v31, v32 = call slice_push_back(v29, v30, Field 20) -> (u32, [Field])
    store v31 at v4
    store v32 at v6
    jmp b5()
  b5():
    v33 = load v4 -> u32
    v34 = load v6 -> [Field]
    v36, v37 = call slice_push_back(v33, v34, Field 15) -> (u32, [Field])
    store v36 at v4
    store v37 at v6
    v38 = eq v0, Field 20
    v39 = not v38
    jmpif v39 then: b6, else: b7
  b6():
    v40 = load v4 -> u32
    v41 = load v6 -> [Field]
    v43, v44 = call slice_push_back(v40, v41, Field 50) -> (u32, [Field])
    store v43 at v4
    store v44 at v6
    jmp b7()
  b7():
    v45 = load v4 -> u32
    v46 = load v6 -> [Field]
    v48, v49 = call slice_push_back(v45, v46, Field 60) -> (u32, [Field])
    store v48 at v4
    store v49 at v6
    v50 = load v4 -> u32
    v51 = load v6 -> [Field]
    return v50, v51
}
acir(inline) fn merge_slices_push_then_pop f28 {
  b0(v0: Field, v1: Field):
    v3 = make_array [Field 0, Field 0] : [Field]
    v4 = allocate -> &mut u32
    store u32 2 at v4
    v6 = allocate -> &mut [Field]
    store v3 at v6
    v7 = eq v0, v1
    v8 = not v7
    jmpif v8 then: b1, else: b2
  b1():
    v14 = load v4 -> u32
    v15 = load v6 -> [Field]
    v16, v17 = call slice_push_back(v14, v15, v1) -> (u32, [Field])
    store v16 at v4
    store v17 at v6
    v18 = load v4 -> u32
    v19 = load v6 -> [Field]
    v20, v21 = call slice_push_back(v18, v19, v0) -> (u32, [Field])
    store v20 at v4
    store v21 at v6
    jmp b3()
  b2():
    v9 = load v4 -> u32
    v10 = load v6 -> [Field]
    v12, v13 = call slice_push_back(v9, v10, v0) -> (u32, [Field])
    store v12 at v4
    store v13 at v6
    jmp b3()
  b3():
    v22 = load v4 -> u32
    v23 = load v6 -> [Field]
    v25, v26 = call slice_push_back(v22, v23, Field 30) -> (u32, [Field])
    store v25 at v4
    store v26 at v6
    v28 = eq v0, Field 20
    jmpif v28 then: b4, else: b5
  b4():
    v29 = load v4 -> u32
    v30 = load v6 -> [Field]
    v31, v32 = call slice_push_back(v29, v30, Field 20) -> (u32, [Field])
    store v31 at v4
    store v32 at v6
    jmp b5()
  b5():
    v33 = load v4 -> u32
    v34 = load v6 -> [Field]
    v36, v37, v38 = call slice_pop_back(v33, v34) -> (u32, [Field], Field)
    v40 = eq v36, u32 4
    constrain v36 == u32 4
    v41 = eq v38, Field 30
    constrain v38 == Field 30
    v42, v43, v44 = call slice_pop_back(v36, v37) -> (u32, [Field], Field)
    v46 = eq v42, u32 3
    constrain v42 == u32 3
    v47 = eq v44, v0
    constrain v44 == v0
    return
}
acir(inline) fn merge_slices_push_then_insert f29 {
  b0(v0: Field, v1: Field):
    v3 = make_array [Field 0, Field 0] : [Field]
    v4 = allocate -> &mut u32
    store u32 2 at v4
    v6 = allocate -> &mut [Field]
    store v3 at v6
    v7 = eq v0, v1
    v8 = not v7
    jmpif v8 then: b1, else: b2
  b1():
    v14 = load v4 -> u32
    v15 = load v6 -> [Field]
    v16, v17 = call slice_push_back(v14, v15, v1) -> (u32, [Field])
    store v16 at v4
    store v17 at v6
    v18 = load v4 -> u32
    v19 = load v6 -> [Field]
    v20, v21 = call slice_push_back(v18, v19, v0) -> (u32, [Field])
    store v20 at v4
    store v21 at v6
    jmp b3()
  b2():
    v9 = load v4 -> u32
    v10 = load v6 -> [Field]
    v12, v13 = call slice_push_back(v9, v10, v0) -> (u32, [Field])
    store v12 at v4
    store v13 at v6
    jmp b3()
  b3():
    v22 = load v4 -> u32
    v23 = load v6 -> [Field]
    v25, v26 = call slice_push_back(v22, v23, Field 30) -> (u32, [Field])
    store v25 at v4
    store v26 at v6
    v28 = eq v0, Field 20
    jmpif v28 then: b4, else: b5
  b4():
    v29 = load v4 -> u32
    v30 = load v6 -> [Field]
    v31, v32 = call slice_push_back(v29, v30, Field 20) -> (u32, [Field])
    store v31 at v4
    store v32 at v6
    v33 = load v4 -> u32
    v34 = load v6 -> [Field]
    v36, v37 = call slice_push_back(v33, v34, Field 15) -> (u32, [Field])
    store v36 at v4
    store v37 at v6
    jmp b5()
  b5():
    v38 = load v4 -> u32
    v39 = load v6 -> [Field]
    v41 = add v38, u32 1
    v42 = lt u32 1, v41
    constrain v42 == u1 1, "Index out of bounds"
    v46, v47 = call slice_insert(v38, v39, u32 1, Field 50) -> (u32, [Field])
    store v46 at v4
    store v47 at v6
    v48 = load v4 -> u32
    v49 = load v6 -> [Field]
    v50 = add v48, u32 1
    v52 = lt u32 6, v50
    constrain v52 == u1 1, "Index out of bounds"
    v54, v55 = call slice_insert(v48, v49, u32 6, Field 100) -> (u32, [Field])
    store v54 at v4
    store v55 at v6
    v56 = load v4 -> u32
    v57 = load v6 -> [Field]
    return v56, v57
}
acir(inline) fn merge_slices_remove_between_ifs f30 {
  b0(v0: Field, v1: Field):
    v3 = make_array [Field 0, Field 0] : [Field]
    v4 = allocate -> &mut u32
    store u32 2 at v4
    v6 = allocate -> &mut [Field]
    store v3 at v6
    v7 = eq v0, v1
    v8 = not v7
    jmpif v8 then: b1, else: b2
  b1():
    v14 = load v4 -> u32
    v15 = load v6 -> [Field]
    v16, v17 = call slice_push_back(v14, v15, v1) -> (u32, [Field])
    store v16 at v4
    store v17 at v6
    v18 = load v4 -> u32
    v19 = load v6 -> [Field]
    v20, v21 = call slice_push_back(v18, v19, v0) -> (u32, [Field])
    store v20 at v4
    store v21 at v6
    jmp b3()
  b2():
    v9 = load v4 -> u32
    v10 = load v6 -> [Field]
    v12, v13 = call slice_push_back(v9, v10, v0) -> (u32, [Field])
    store v12 at v4
    store v13 at v6
    jmp b3()
  b3():
    v22 = load v4 -> u32
    v23 = load v6 -> [Field]
    v24 = lt u32 2, v22
    constrain v24 == u1 1, "Index out of bounds"
    v27, v28, v29 = call slice_remove(v22, v23, u32 2) -> (u32, [Field], Field)
    v30 = allocate -> &mut u32
    store v27 at v30
    v31 = allocate -> &mut [Field]
    store v28 at v31
    v32 = eq v29, v1
    constrain v29 == v1
    v34 = eq v0, Field 20
    jmpif v34 then: b4, else: b5
  b4():
    v35 = load v30 -> u32
    v36 = load v31 -> [Field]
    v37, v38 = call slice_push_back(v35, v36, Field 20) -> (u32, [Field])
    store v37 at v30
    store v38 at v31
    jmp b5()
  b5():
    v39 = load v30 -> u32
    v40 = load v31 -> [Field]
    v42, v43 = call slice_push_back(v39, v40, Field 15) -> (u32, [Field])
    store v42 at v30
    store v43 at v31
    v44 = eq v0, Field 20
    v45 = not v44
    jmpif v45 then: b6, else: b7
  b6():
    v46 = load v30 -> u32
    v47 = load v31 -> [Field]
    v49, v50 = call slice_push_back(v46, v47, Field 50) -> (u32, [Field])
    store v49 at v30
    store v50 at v31
    jmp b7()
  b7():
    v51 = load v30 -> u32
    v52 = load v31 -> [Field]
    return v51, v52
}
acir(inline) fn eq f31 {
  b0(v0: Field, v1: Field):
    v2 = eq v0, v1
    return v2
}
    "#;

    let values = expect_values_with_args(
        src,
        vec![
            from_constant(5_u128.into(), NumericType::NativeField),
            from_constant(10_u128.into(), NumericType::NativeField),
        ],
    );
    assert!(values.is_empty());
}

#[test]
fn signed_integer_conversions() {
    // fn main() -> pub i16 {
    //   foo() as i16
    // }
    // fn foo() -> i8 {
    //   -65
    // }
    let src = r#"
        acir(inline) fn main f0 {
          b0():
            v1 = call f1() -> i8
            v2 = cast v1 as u8
            v4 = lt v2, u8 128
            v5 = not v4
            v6 = cast v5 as u16
            v8 = unchecked_mul u16 65280, v6
            v9 = cast v1 as u16
            v10 = unchecked_add v8, v9
            v11 = cast v10 as i16
            return v11
        }
        acir(inline) fn foo f1 {
          b0():
            return i8 191
        }
    "#;
    executes_with_no_errors(src);
}

#[test]
fn signed_integer_casting() {
    //  fn main() -> pub i8 {
    //      let a: i8 = 28;
    //      let b = (1, -a, 0);
    //      let mut c = (a + (((b.1 as i64) << (b.2 as i64)) as i8));
    //      c = -c;
    //      c
    //  }

    let src = r#"
      acir(inline) fn main f0 {
        b0():
          v1 = cast u8 228 as i8
          v2 = cast u8 228 as u8
          v4 = lt v2, u8 128
          v5 = not v4
          v6 = cast v5 as u64
          v8 = unchecked_mul u64 18446744073709551360, v6
          v9 = cast u8 228 as u64
          v10 = unchecked_add v8, v9
          v11 = cast v10 as i64
          v13 = shl v11, i64 0
          v14 = truncate v13 to 64 bits, max_bit_size: 65
          v15 = truncate v14 to 8 bits, max_bit_size: 64
          v16 = cast v15 as i8
          v18 = add i8 28, v16
          v19 = truncate v18 to 8 bits, max_bit_size: 9
          v20 = cast v19 as u8
          v21 = cast v15 as u8
          v22 = lt v21, u8 128
          v23 = lt v20, u8 128
          v24 = unchecked_mul v23, v22
          constrain v24 == v22, "attempt to add with overflow"
          v25 = cast v19 as i8
          v26 = allocate -> &mut i8
          store v25 at v26
          v27 = load v26 -> i8
          v29 = sub i8 0, v27
          v30 = truncate v29 to 8 bits, max_bit_size: 9
          v31 = cast v30 as u8
          v32 = cast v27 as u8
          v33 = lt v32, u8 128
          v34 = not v33
          v35 = lt v31, u8 128
          v36 = unchecked_mul v35, v34
          constrain v36 == v34, "attempt to subtract with overflow"
          v37 = cast v30 as i8
          store v37 at v26
          v38 = load v26 -> i8
          return v38
      }
      "#;
    let value = expect_value(src);
    assert_eq!(value, Value::i8(0));
}

#[test]
fn signed_integer_casting_2() {
    // fn main() -> pub i64 {
    //     (-(func_4() as i64))
    // }
    // fn func_4() -> i8 {
    //     -89
    // }
    let src = r#"
      acir(inline) fn main f0 {
        b0():
          v1 = call f1() -> i8
          v2 = cast v1 as u8
          v4 = lt v2, u8 128
          v5 = not v4
          v6 = cast v5 as u64
          v8 = unchecked_mul u64 18446744073709551360, v6
          v9 = cast v1 as u64
          v10 = unchecked_add v8, v9
          v11 = cast v10 as i64
          v13 = sub i64 0, v11
          v14 = truncate v13 to 64 bits, max_bit_size: 65
          v15 = cast v14 as u64
          v16 = cast v10 as u64
          v18 = lt v16, u64 9223372036854775808
          v19 = not v18
          v20 = lt v15, u64 9223372036854775808
          v21 = unchecked_mul v20, v19
          constrain v21 == v19, "attempt to subtract with overflow"
          v22 = cast v14 as i64
          return v22
      }
      acir(inline_always) fn func_4 f1 {
        b0():
          return i8 167
      }
      "#;
    let value = expect_value(src);
    assert_eq!(value, Value::i64(89));
}

#[test]
fn infinite_loop_with_step_limit() {
    let src = r#"
    acir(inline) predicate_pure fn main f0 {
    b0():
      call f1(u1 0)
      return
    }
    brillig(inline) predicate_pure fn func_2 f1 {
      b0(v0: u1):
        jmp b1()
      b1():
        jmpif v0 then: b2, else: b3
      b2():
        return
      b3():
        jmp b1()
    }
    "#;
    let ssa = Ssa::from_str(src).unwrap();
    let options = InterpreterOptions { step_limit: Some(100), ..Default::default() };
    let result = ssa.interpret_with_options(Vec::new(), options, std::io::empty());
    let Err(InterpreterError::OutOfBudget { .. }) = result else {
        panic!("unexpected result: {result:?}")
    };
}

#[test]
fn call_stack_is_cleared_between_entry_calls() {
    let src = r#"
    acir(inline) fn main f0 {
    b0(v0: u32):
      constrain v0 == u32 0
      return
    }
    "#;
    let ssa = Ssa::from_str(src).unwrap();

    // We are going to reuse the interpreter between calls, like we do in constant folding.
    let mut interpreter = Interpreter::new(&ssa, InterpreterOptions::default(), std::io::empty());
    interpreter.interpret_globals().unwrap();
    assert_eq!(interpreter.call_stack.len(), 1, "starts with the global context");

    let main_id = FunctionId::new(0);
    interpreter.interpret_function(main_id, vec![Value::u32(0)]).expect("0 should succeed");
    assert_eq!(interpreter.call_stack.len(), 1, "reset after successful call");

    interpreter.interpret_function(main_id, vec![Value::u32(1)]).expect_err("1 should fail");
    assert_eq!(interpreter.call_stack.len(), 2, "contains the last entry after failure");

    interpreter.interpret_function(main_id, vec![Value::u32(0)]).expect("0 should succeed");
    assert_eq!(interpreter.call_stack.len(), 1, "should clear the previous leftover");
}
