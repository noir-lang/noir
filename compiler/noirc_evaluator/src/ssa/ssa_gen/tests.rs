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

#[named]
#[test]
fn databus_no_dead_param_on_empty_array() {
    let src = "
    fn main(a: (i8, u32, i8), b: call_data(0) [(i8, i8, bool, bool, str<0>); 2]) -> pub [(bool, str<3>, str<0>, u32); 0] {
        []
    }
    ";

    let ssa = get_initial_ssa(src, function_path!()).unwrap();

    // We expect that there to be no `array_get` attempting to fetch from v3
    // the empty nested array `[u8; 0]`.
    // The databus is only going to be initialized with actual numeric values so keeping
    // an empty array in the databus is pointless.
    // The databus is not mutated after initialization as well. So if we have instructions
    // on the data bus (such as an `array_get` on an empty array) that go unused, it becomes
    // more difficult to eliminate those unused instructions. Thus, we just do not generate them.
    let expected = "
    acir(inline) fn main f0 {
      b0(v0: i8, v1: u32, v2: i8, v3: [(i8, i8, u1, u1, [u8; 0]); 2]):
        v5 = array_get v3, index u32 0 -> i8
        v7 = array_get v3, index u32 1 -> i8
        v9 = array_get v3, index u32 2 -> u1
        v11 = array_get v3, index u32 3 -> u1
        v13 = array_get v3, index u32 4 -> i8
        v15 = array_get v3, index u32 5 -> i8
        v17 = array_get v3, index u32 6 -> u1
        v19 = array_get v3, index u32 7 -> u1
        v20 = make_array [v5, v7, v9, v11, v13, v15, v17, v19] : [Field; 8]
        v21 = make_array [] : [(u1, [u8; 3], [u8; 0], u32); 0]
        return v21
    }
    ";
    assert_normalized_ssa_equals(ssa, expected);
}
