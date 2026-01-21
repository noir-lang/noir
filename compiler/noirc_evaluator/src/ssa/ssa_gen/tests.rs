#![cfg(test)]

use crate::{assert_ssa_snapshot, errors::RuntimeError, ssa::opt::assert_normalized_ssa_equals};

use super::{Ssa, generate_ssa};

use noirc_frontend::test_utils::{
    GetProgramOptions, get_monomorphized, get_monomorphized_with_options,
};

fn get_initial_ssa(src: &str) -> Result<Ssa, RuntimeError> {
    let program = match get_monomorphized(src) {
        Ok(program) => program,
        Err(errors) => {
            panic!(
                "Expected program to have no errors before SSA generation, but found: {errors:?}"
            )
        }
    };

    generate_ssa(program)
}

#[test]
fn assert() {
    let assert_src = "
    fn main(input: u32) {
        assert(input == 5);
    }
    ";
    let assert_ssa = get_initial_ssa(assert_src).unwrap();

    let expected = "
    acir(inline) fn main f0 {
      b0(v0: u32):
        v2 = eq v0, u32 5
        constrain v0 == u32 5
        return
    }
    ";
    assert_normalized_ssa_equals(assert_ssa, expected);
}

#[test]
fn assert_eq() {
    let assert_eq_src = "
    fn main(input: u32) {
        assert_eq(input, 5);
    }
    ";

    let assert_eq_ssa = get_initial_ssa(assert_eq_src).unwrap();

    let expected = "
    acir(inline) fn main f0 {
      b0(v0: u32):
        v2 = eq v0, u32 5
        constrain v0 == u32 5
        return
    }
    ";
    // The SSA from assert_eq should match that from a regular assert checking for equality
    // The expected SSA above should match that in the `assert()` test
    assert_normalized_ssa_equals(assert_eq_ssa, expected);
}

#[test]
fn basic_loop() {
    let src = "
    fn main(sum_to_check: u32) {
        let mut sum = 0;
        for i in 0..4 {
            sum = sum + i;
        }
        assert(sum_to_check == sum);
    }
    ";

    let ssa = get_initial_ssa(src).unwrap();

    let expected = "
    acir(inline) fn main f0 {
      b0(v0: u32):
        v2 = allocate -> &mut u32
        store u32 0 at v2
        jmp b1(u32 0)
      b1(v4: u32):
        v5 = lt v4, u32 4
        jmpif v5 then: b2, else: b3
      b2():
        v6 = load v2 -> u32
        v7 = add v6, v4
        store v7 at v2
        v9 = unchecked_add v4, u32 1
        jmp b1(v9)
      b3():
        v10 = load v2 -> u32
        v11 = eq v0, v10
        constrain v0 == v10
        return
    }
    ";

    assert_normalized_ssa_equals(ssa, expected);
}

#[test]
fn acir_no_access_check_on_array_read() {
    let src = "
    fn main(mut array: [Field; 3], index: u32) -> pub Field {
        array[index]
    }
    ";
    let ssa = get_initial_ssa(src).unwrap();

    let expected = "
    acir(inline) fn main f0 {
      b0(v0: [Field; 3], v1: u32):
        v2 = allocate -> &mut [Field; 3]
        store v0 at v2
        v3 = load v2 -> [Field; 3]
        v4 = array_get v3, index v1 -> Field
        return v4
    }
    ";
    assert_normalized_ssa_equals(ssa, expected);
}

#[test]
fn acir_no_access_check_on_array_assignment() {
    let src = "
    fn main(mut array: [Field; 3], index: u32, x: Field) {
        array[index] = x;
    }
    ";
    let ssa = get_initial_ssa(src).unwrap();

    let expected = "
    acir(inline) fn main f0 {
      b0(v0: [Field; 3], v1: u32, v2: Field):
        v3 = allocate -> &mut [Field; 3]
        store v0 at v3
        v4 = load v3 -> [Field; 3]
        v5 = array_set v4, index v1, value v2
        v7 = unchecked_add v1, u32 1
        store v5 at v3
        return
    }
    ";
    assert_normalized_ssa_equals(ssa, expected);
}

#[test]
fn brillig_access_check_on_array_read() {
    let src = "
    unconstrained fn main(mut array: [Field; 3], index: u32) -> pub Field {
        array[index]
    }
    ";
    let ssa = get_initial_ssa(src).unwrap();

    let expected = r#"
    brillig(inline) fn main f0 {
      b0(v0: [Field; 3], v1: u32):
        v2 = allocate -> &mut [Field; 3]
        store v0 at v2
        v3 = load v2 -> [Field; 3]
        v5 = lt v1, u32 3
        constrain v5 == u1 1, "Index out of bounds"
        v7 = array_get v3, index v1 -> Field
        return v7
    }
    "#;
    assert_normalized_ssa_equals(ssa, expected);
}

#[test]
fn brillig_access_check_on_array_assignment() {
    let src = "
    unconstrained fn main(mut array: [Field; 3], index: u32, x: Field) {
        array[index] = x;
    }
    ";
    let ssa = get_initial_ssa(src).unwrap();

    let expected = r#"
    brillig(inline) fn main f0 {
      b0(v0: [Field; 3], v1: u32, v2: Field):
        v3 = allocate -> &mut [Field; 3]
        store v0 at v3
        v4 = load v3 -> [Field; 3]
        v6 = lt v1, u32 3
        constrain v6 == u1 1, "Index out of bounds"
        v8 = array_set v4, index v1, value v2
        v10 = unchecked_add v1, u32 1
        store v8 at v3
        return
    }
    "#;
    assert_normalized_ssa_equals(ssa, expected);
}

#[test]
fn pure_builtin_call_args_do_not_get_cloned() {
    let src = "
    #[builtin(as_vector)]
    pub fn as_vector<T, let N: u32>(arr: [T; N]) -> [T] {}

    unconstrained fn main() -> pub u32 {
        let a = [1, 2];
        let x = as_vector(a);
        let y = as_vector(a);
        x[0] + y[1]
    }
    ";

    let program = get_monomorphized_with_options(
        src,
        GetProgramOptions { root_and_stdlib: true, ..Default::default() },
    )
    .unwrap();

    let ssa = generate_ssa(program).unwrap();

    let expected = r#"
    brillig(inline) fn main f0 {
      b0():
        v2 = make_array [u32 1, u32 2] : [u32; 2]
        v3 = make_array [u32 1, u32 2] : [u32]
        v4 = make_array [u32 1, u32 2] : [u32]
        return u32 3
    }
    "#;
    assert_normalized_ssa_equals(ssa, expected);
}

#[test]
fn foreign_call_args_do_not_get_cloned() {
    let src = "
    #[oracle(print)]
    unconstrained fn print_oracle<T>(with_newline: bool, input: T) {}

    unconstrained fn main() {
        let a = [1, 2];
        print_oracle(true, a);
        print_oracle(true, a);
    }
    ";

    let program = get_monomorphized(src).unwrap();

    let ssa = generate_ssa(program).unwrap();

    let expected = r#"
    brillig(inline) fn main f0 {
      b0():
        v2 = make_array [Field 1, Field 2] : [Field; 2]
        v23 = make_array b"{\"kind\":\"array\",\"length\":2,\"type\":{\"kind\":\"field\"}}"
        call print(u1 1, v2, v23, u1 0)
        v27 = make_array b"{\"kind\":\"array\",\"length\":2,\"type\":{\"kind\":\"field\"}}"
        call print(u1 1, v2, v27, u1 0)
        return
    }
    "#;
    assert_normalized_ssa_equals(ssa, expected);
}

#[test]
fn for_loop_exclusive() {
    let assert_src = "
    fn main() -> pub u32 {
        let mut sum = 0;
        for i in 0..5 {
          sum += i;
        }
        sum
    }
    ";
    let ssa = get_initial_ssa(assert_src).unwrap();

    // This is a regular for loop, nothing special here
    assert_ssa_snapshot!(ssa, @r"
    acir(inline) fn main f0 {
      b0():
        v1 = allocate -> &mut u32
        store u32 0 at v1
        jmp b1(u32 0)
      b1(v0: u32):
        v4 = lt v0, u32 5
        jmpif v4 then: b2, else: b3
      b2():
        v6 = load v1 -> u32
        v7 = add v6, v0
        store v7 at v1
        v9 = unchecked_add v0, u32 1
        jmp b1(v9)
      b3():
        v5 = load v1 -> u32
        return v5
    }
    ");
}

#[test]
fn for_loop_inclusive_max_value_without_break() {
    let assert_src = "
    fn main() -> pub u8 {
        let mut sum = 0;
        for i in 0..=255_u8 {
          sum += i;
        }
        sum
    }
    ";
    let ssa = get_initial_ssa(assert_src).unwrap();

    // - b1 is the loop header
    // - b2 is the loop body
    // - b3 is the loop exit, but it performs a check to determine whether the final iteration
    //   should be executed. In this case we check if no break was hit. It's multiplied by
    //   one because that "one" is (start < end) which is true in this case.
    // - b4 is the final iteration where `index == end`
    assert_ssa_snapshot!(ssa, @r"
    acir(inline) fn main f0 {
      b0():
        v1 = allocate -> &mut u8
        store u8 0 at v1
        v3 = allocate -> &mut u1
        store u1 1 at v3
        jmp b1(u8 0)
      b1(v0: u8):
        v6 = lt v0, u8 255
        jmpif v6 then: b2, else: b3
      b2():
        v12 = load v1 -> u8
        v13 = add v12, v0
        store v13 at v1
        v15 = unchecked_add v0, u8 1
        jmp b1(v15)
      b3():
        v7 = load v3 -> u1
        v8 = unchecked_mul v7, u1 1
        jmpif v8 then: b4, else: b5
      b4():
        v9 = load v1 -> u8
        v10 = add v9, u8 255
        store v10 at v1
        jmp b5()
      b5():
        v11 = load v1 -> u8
        return v11
    }
    ");
}

#[test]
fn for_loop_inclusive_end_is_known_and_not_a_maximum() {
    let assert_src = "
    fn main() -> pub u8 {
        let mut sum = 0;
        for i in 0..=254_u8 {
          sum += i;
        }
        sum
    }
    ";
    let ssa = get_initial_ssa(assert_src).unwrap();

    // We end up generating an exclusive for loop up to 255
    assert_ssa_snapshot!(ssa, @r"
    acir(inline) fn main f0 {
      b0():
        v1 = allocate -> &mut u8
        store u8 0 at v1
        jmp b1(u8 0)
      b1(v0: u8):
        v4 = lt v0, u8 255
        jmpif v4 then: b2, else: b3
      b2():
        v6 = load v1 -> u8
        v7 = add v6, v0
        store v7 at v1
        v9 = unchecked_add v0, u8 1
        jmp b1(v9)
      b3():
        v5 = load v1 -> u8
        return v5
    }
    ");
}

#[test]
fn for_loop_inclusive_max_value_with_break() {
    let assert_src = "
    unconstrained fn main(cond: bool) -> pub u8 {
        let mut sum = 0;
        for i in 0..=255_u8 {
          if cond {
              break;
          }
          sum += i;
        }
        sum
    }
    ";
    let ssa = get_initial_ssa(assert_src).unwrap();

    // - b1 is the loop header
    // - b2, b4  and b5 are the loop body
    // - b4 has the logic that happens when a break is hit. In this case we store 0 at v3
    //   to signal this.
    // - b3 is the loop exit, but it performs a check to determine whether the final iteration
    //   should be executed. In this case we check if no break was hit. It's multiplied by
    //   one because that "one" is (start < end) which is true in this case.
    // - b6 is the final loop iteration where `index == end`. Note that the code for
    //   `if cond { break; }` now has the break take us to b8, which jumps to b7, which
    //   exits main (that is, the break skips the final iteration).
    assert_ssa_snapshot!(ssa, @r"
    brillig(inline) fn main f0 {
      b0(v0: u1):
        v2 = allocate -> &mut u8
        store u8 0 at v2
        v4 = allocate -> &mut u1
        store u1 1 at v4
        jmp b1(u8 0)
      b1(v1: u8):
        v7 = lt v1, u8 255
        jmpif v7 then: b2, else: b3
      b2():
        jmpif v0 then: b4, else: b5
      b3():
        v13 = load v4 -> u1
        v14 = unchecked_mul v13, u1 1
        jmpif v14 then: b6, else: b7
      b4():
        store u1 0 at v4
        jmp b3()
      b5():
        v8 = load v2 -> u8
        v9 = add v8, v1
        store v9 at v2
        v11 = unchecked_add v1, u8 1
        jmp b1(v11)
      b6():
        jmpif v0 then: b8, else: b9
      b7():
        v17 = load v2 -> u8
        return v17
      b8():
        jmp b7()
      b9():
        v15 = load v2 -> u8
        v16 = add v15, u8 255
        store v16 at v2
        jmp b7()
    }
    ");
}

#[test]
fn for_loop_inclusive_unknown_range_with_break() {
    let assert_src = "
    unconstrained fn main(start: u8, end: u8) -> pub u8 {
        let mut sum = 0;
        for i in start..=end {
          if i == 10 { 
              break; 
          }
          sum += i;
        }
        sum
    }
    ";
    let ssa = get_initial_ssa(assert_src).unwrap();

    // Here we can see in b3 that we do `lt v0, v1`, which is the condition that checks
    // `start < end` to determine whether the final iteration should be executed
    // (in addition to checking if a break was hit or not).
    assert_ssa_snapshot!(ssa, @r"
    brillig(inline) fn main f0 {
      b0(v0: u8, v1: u8):
        v3 = allocate -> &mut u8
        store u8 0 at v3
        v5 = allocate -> &mut u1
        store u1 1 at v5
        jmp b1(v0)
      b1(v2: u8):
        v7 = lt v2, v1
        jmpif v7 then: b2, else: b3
      b2():
        v9 = eq v2, u8 10
        jmpif v9 then: b4, else: b5
      b3():
        v15 = load v5 -> u1
        v16 = lt v1, v0
        v17 = not v16
        v18 = unchecked_mul v15, v17
        jmpif v18 then: b6, else: b7
      b4():
        store u1 0 at v5
        jmp b3()
      b5():
        v10 = load v3 -> u8
        v11 = add v10, v2
        store v11 at v3
        v13 = unchecked_add v2, u8 1
        jmp b1(v13)
      b6():
        v19 = eq v1, u8 10
        jmpif v19 then: b8, else: b9
      b7():
        v22 = load v3 -> u8
        return v22
      b8():
        jmp b7()
      b9():
        v20 = load v3 -> u8
        v21 = add v20, v1
        store v21 at v3
        jmp b7()
    }
    ");
}

#[test]
fn for_loop_inclusive_with_continue() {
    let assert_src = "
    unconstrained fn main() {
        for _ in 0..=255_u8 {
            continue;
        }
    }
    ";
    let ssa = get_initial_ssa(assert_src).unwrap();

    // Here we can see that the `continue` in the final iteration jumps
    // to the end of the loop (from b4 to b5).
    assert_ssa_snapshot!(ssa, @r"
    brillig(inline) fn main f0 {
      b0():
        v1 = allocate -> &mut u1
        store u1 1 at v1
        jmp b1(u8 0)
      b1(v0: u8):
        v5 = lt v0, u8 255
        jmpif v5 then: b2, else: b3
      b2():
        v9 = unchecked_add v0, u8 1
        jmp b1(v9)
      b3():
        v6 = load v1 -> u1
        v7 = unchecked_mul v6, u1 1
        jmpif v7 then: b4, else: b5
      b4():
        jmp b5()
      b5():
        return
    }
    ");
}

#[test]
fn for_loop_inclusive_max_value_to_max_value() {
    let assert_src = "
    fn main() -> pub u8 {
        let mut sum = 0;
        for i in 255_u8..=255_u8 {
          sum += i;
        }
        sum
    }
    ";
    let ssa = get_initial_ssa(assert_src).unwrap();

    // Check that the final iteration is included
    assert_ssa_snapshot!(ssa, @r"
    acir(inline) fn main f0 {
      b0():
        v1 = allocate -> &mut u8
        store u8 0 at v1
        v3 = allocate -> &mut u1
        store u1 1 at v3
        jmp b1(u8 255)
      b1(v0: u8):
        v6 = lt v0, u8 255
        jmpif v6 then: b2, else: b3
      b2():
        v12 = load v1 -> u8
        v13 = add v12, v0
        store v13 at v1
        v15 = unchecked_add v0, u8 1
        jmp b1(v15)
      b3():
        v7 = load v3 -> u1
        v8 = unchecked_mul v7, u1 1
        jmpif v8 then: b4, else: b5
      b4():
        v9 = load v1 -> u8
        v10 = add v9, u8 255
        store v10 at v1
        jmp b5()
      b5():
        v11 = load v1 -> u8
        return v11
    }
    ");
}
