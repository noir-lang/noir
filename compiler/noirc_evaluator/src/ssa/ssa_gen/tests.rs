#![cfg(test)]

use crate::{errors::RuntimeError, ssa::opt::assert_normalized_ssa_equals};

use super::{Ssa, generate_ssa};

use noirc_frontend::{
    hir::{def_collector::dc_crate::CompilationError, resolution::errors::ResolverError},
    test_utils::{get_monomorphized, get_monomorphized_with_error_filter},
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
fn pure_builtin_args_do_not_get_cloned() {
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

    let program = get_monomorphized_with_error_filter(src, |err| {
        matches!(
            err,
            CompilationError::ResolverError(ResolverError::LowLevelFunctionOutsideOfStdlib { .. })
        )
    })
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
