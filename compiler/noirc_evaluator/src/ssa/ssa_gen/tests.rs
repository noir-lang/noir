#![cfg(test)]

use crate::{errors::RuntimeError, ssa::opt::assert_normalized_ssa_equals};

use super::{Ssa, generate_ssa};

use function_name::named;

use noirc_frontend::function_path;
use noirc_frontend::test_utils::{Expect, get_monomorphized};

fn get_initial_ssa(src: &str, test_path: &str) -> Result<Ssa, RuntimeError> {
    let program = match get_monomorphized(src, Some(test_path), Expect::Success) {
        Ok(program) => program,
        Err(errors) => {
            panic!(
                "Expected program to have no errors before SSA generation, but found: {errors:?}"
            )
        }
    };

    generate_ssa(program)
}

#[named]
#[test]
fn assert() {
    let assert_src = "
    fn main(input: u32) {
        assert(input == 5);
    }
    ";
    let assert_ssa = get_initial_ssa(assert_src, function_path!()).unwrap();

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

#[named]
#[test]
fn assert_eq() {
    let assert_eq_src = "
    fn main(input: u32) {
        assert_eq(input, 5);
    }
    ";

    let assert_eq_ssa = get_initial_ssa(assert_eq_src, function_path!()).unwrap();

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

#[named]
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

    let ssa = get_initial_ssa(src, function_path!()).unwrap();

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
