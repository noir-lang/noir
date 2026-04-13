#![cfg(test)]
//! The easiest way to test this pass is a bit indirect. We have to run
//! ownership in its entirety and look at where the clones are inserted.
//! Testing e.g. the last_use pass directly is difficult since it returns
//! sets of IdentIds which can't be matched to the source code easily.

use crate::elaborator::FrontendOptions;
use crate::test_utils::{
    GetProgramOptions, get_monomorphized, get_monomorphized_with_options,
    get_monomorphized_with_stdlib, stdlib_src,
};

#[test]
fn last_use_in_if_branches() {
    let src = "
    unconstrained fn main(d: [Field; 2]) {
        if len(d) == 2 {              // use 1 of d
            if len(d) == 2 {          // use 2 of d
                assert(eq(d, [5, 6]));  // use 3 of d
            }
        } else {
            assert(eq(d, [5, 6]));      // use 4 of d
        }
    }

    fn eq(lhs: [Field; 2], rhs: [Field; 2]) -> bool {
        (lhs[0] == rhs[0]) & (lhs[1] == rhs[1])
    }

    fn len(arr: [Field; 2]) -> u32 {
        2
    }
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(d$l0: [Field; 2]) -> () {
        if (len$f1(d$l0.clone()) == 2) {
            if (len$f1(d$l0.clone()) == 2) {
                assert(eq$f2(d$l0, [5, 6]));
            }
        } else {
            assert(eq$f2(d$l0, [5, 6]));
        }
    }
    unconstrained fn len$f1(arr$l1: [Field; 2]) -> u32 {
        2
    }
    unconstrained fn eq$f2(lhs$l2: [Field; 2], rhs$l3: [Field; 2]) -> bool {
        ((lhs$l2[0] == rhs$l3[0]) & (lhs$l2[1] == rhs$l3[1]))
    }
    ");
}

#[test]
fn does_not_move_into_loop() {
    let src = "
    unconstrained fn main(param: [Field; 2]) {
        let local1 = [0];
        let local2 = [1];
        loop {
            use_var(param);
            use_var(local1);
            use_var(local2);
            break;
        }
        use_var(local2);
    }

    fn use_var<T>(_x: T) {}
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(param$l0: [Field; 2]) -> () {
        let local1$l1 = [0];
        let local2$l2 = [1];
        loop {
            use_var$f1(param$l0.clone());;
            use_var$f2(local1$l1.clone());;
            use_var$f2(local2$l2.clone());;
            break
        };
        use_var$f2(local2$l2);
    }
    unconstrained fn use_var$f1(_x$l3: [Field; 2]) -> () {
    }
    unconstrained fn use_var$f2(_x$l4: [Field; 1]) -> () {
    }
    ");
}

#[test]
fn can_move_within_loop() {
    let src = "
    unconstrained fn main() {
        for _ in 0 .. 10 {
            let x = [1, 2];
            use_var(x);
            use_var(x);
        }
    }

    fn use_var<T>(_x: T) {}
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0() -> () {
        for _$l0 in 0 .. 10 {
            let x$l1 = [1, 2];
            use_var$f1(x$l1.clone());;
            use_var$f1(x$l1);
        }
    }
    unconstrained fn use_var$f1(_x$l2: [Field; 2]) -> () {
    }
    ");
}

#[test]
fn borrows_on_nested_index() {
    let src = "
    unconstrained fn main(x: Field, y: pub Field) {
        let EXPONENTIATE: [[[Field; 2]; 2]; 2] = [[[1, 1], [0, 0]], [[1, 1], [0, 0]]];
        let mut acc: Field = 0;
        for i in 0..2 {
            for j in 0..2 {
                acc += EXPONENTIATE[i][j][i];
            }
        }
        assert(acc != 0);
    }
    ";

    let program = get_monomorphized(src).unwrap();
    // We expect no clones
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(x$l0: Field, y$l1: pub Field) -> () {
        let EXPONENTIATE$l2 = [[[1, 1], [0, 0]], [[1, 1], [0, 0]]];
        let mut acc$l3 = 0;
        for i$l4 in 0 .. 2 {
            for j$l5 in 0 .. 2 {
                acc$l3 = (acc$l3 + EXPONENTIATE$l2[i$l4][j$l5][i$l4])
            }
        };
        assert((acc$l3 != 0));
    }
    ");
}

#[test]
fn clone_call_array_result() {
    let src = "
    unconstrained fn main(i: u32) -> pub u32 {
        let _a = foo()[1][0][1];
        let _s = foo()[1][0][1];
        i
    }
    unconstrained fn foo() -> [[[[u128; 0]; 2]; 1]; 2] {
        [[[[], []]], [[[], []]]]
    }
    ";

    let program = get_monomorphized(src).unwrap();
    // We expect no clones
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(i$l0: u32) -> pub u32 {
        let _a$l1 = foo$f1()[1][0][1].clone();
        let _s$l2 = foo$f1()[1][0][1].clone();
        i$l0
    }
    unconstrained fn foo$f1() -> [[[[u128; 0]; 2]; 1]; 2] {
        [[[[], []]], [[[], []]]]
    }
    ");
}

#[test]
fn considers_lvalue_index_identifier_in_last_use() {
    let src = "
    unconstrained fn main() {
        let mut b = [true];
        let mut c = [false];
        c = b;
        b[0] = !c[0];
        assert_eq(c[0], true);
    }
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0() -> () {
        let mut b$l0 = [true];
        let mut c$l1 = [false];
        c$l1 = b$l0.clone();
        b$l0[0] = (!c$l1[0]);
        assert((c$l1[0] == true));
    }
    ");
}

#[test]
fn analyzes_expression_before_lvalue_in_assignment() {
    let src = "
    unconstrained fn main() {
        let mut b = [true];
        let mut c = [false];
        b[0] = {
          c = b;
          !c[0]
        };
        assert_eq(c[0], true);
    }
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0() -> () {
        let mut b$l0 = [true];
        let mut c$l1 = [false];
        b$l0[0] = {
            c$l1 = b$l0.clone();
            (!c$l1[0])
        };
        assert((c$l1[0] == true));
    }
    ");
}

#[test]
fn clone_nested_array_used_as_call_arg() {
    let src = "
    unconstrained fn main(i: u32) -> pub bool {
        let G_A: [[bool; 3]; 2] = [[false, false, false], [false, false, false]];
        let result = mutate_array(G_A[i])[1];
        if i != 0 {
            G_A[0][1]
        } else {
            result
        }
    }
    unconstrained fn mutate_array(mut a: [bool; 3]) -> [bool; 3] {
        a[1] = true;
        a
    }
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(i$l0: u32) -> pub bool {
        let G_A$l1 = [[false, false, false], [false, false, false]];
        let result$l2 = mutate_array$f1(G_A$l1[i$l0].clone())[1];
        if (i$l0 != 0) {
            G_A$l1[0][1]
        } else {
            result$l2
        }
    }
    unconstrained fn mutate_array$f1(mut a$l3: [bool; 3]) -> [bool; 3] {
        a$l3[1] = true;
        a$l3
    }
    ");
}

#[test]
fn clone_global_nested_array_used_as_call_arg() {
    let src = "
    global G_A: [[bool; 3]; 2] = [[false, false, false], [false, false, false]];
    unconstrained fn main(i: u32) -> pub bool {
        let result = mutate_array(G_A[i])[1];
        if i != 0 {
            result
        } else {
            G_A[0][1]
        }
    }
    unconstrained fn mutate_array(mut a: [bool; 3]) -> [bool; 3] {
        a[1] = true;
        a
    }
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    global G_A$g0: [[bool; 3]; 2] = [[false, false, false], [false, false, false]];
    unconstrained fn main$f0(i$l0: u32) -> pub bool {
        let result$l1 = mutate_array$f1(G_A$g0[i$l0].clone())[1];
        if (i$l0 != 0) {
            result$l1
        } else {
            G_A$g0[0][1]
        }
    }
    unconstrained fn mutate_array$f1(mut a$l2: [bool; 3]) -> [bool; 3] {
        a$l2[1] = true;
        a$l2
    }
    ");
}

// Regression for issue https://github.com/noir-lang/noir/issues/9907
#[test]
fn regression_9907() {
    let src = "
   unconstrained fn main() -> pub [[Field; 1]; 1] {
        foo([[0xcafebabe]])
    }
    unconstrained fn foo(mut a: [[Field; 1]; 1]) -> [[Field; 1]; 1] {
        let mut b = bar(a)[0];

        let mut x = 0;
        while (x != 0) {}

        b[0] = 0xdeadbeef;
        a
    }
    unconstrained fn bar(mut a: [[Field; 1]; 1]) -> [[Field; 1]; 1] {
        a
    }
    ";

    let program = get_monomorphized(src).unwrap();
    // There are clones on both bar input and output
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0() -> pub [[Field; 1]; 1] {
        foo$f1([[3405691582]])
    }
    unconstrained fn foo$f1(mut a$l0: [[Field; 1]; 1]) -> [[Field; 1]; 1] {
        let mut b$l1 = bar$f2(a$l0.clone())[0].clone();
        let mut x$l2 = 0;
        while (x$l2 != 0) {
        };
        b$l1[0] = 3735928559;
        a$l0
    }
    unconstrained fn bar$f2(mut a$l3: [[Field; 1]; 1]) -> [[Field; 1]; 1] {
        a$l3
    }
    ");
}

#[test]
fn handle_reference_expression_cases() {
    // Each of these cases should delay a clone
    let src = "
        unconstrained fn main(mut a: [Field; 1]) {
            let _ = { a }[0]; // block
            let _ = (*&mut a)[0]; // *

            let tuple = (a, a); // Clones here are expected
            let _ = tuple.0[0];  // but the tuple itself doesn't need to be cloned when getting the
                                 // first element of the tuple.

            let nested = [a, a]; // Clones here are expected
            let _ = nested[0][0];  // index expr, no clone

            let _ = (|x| x)(a)[0]; // other (we should clone)

            let _ = (a, tuple, nested); // ensure each variable is used afterward so each prior use is eligible for a clone
        }
    ";

    let program = get_monomorphized(src).unwrap();
    // There are clones on both bar input and output
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(mut a$l0: [Field; 1]) -> () {
        let _$l1 = {
            a$l0
        }[0];
        let _$l2 = a$l0[0];
        let tuple$l3 = (a$l0.clone(), a$l0.clone());
        let _$l4 = tuple$l3.0[0];
        let nested$l5 = [a$l0.clone(), a$l0.clone()];
        let _$l6 = nested$l5[0][0];
        let _$l9 = lambda$f2(a$l0.clone())[0];
        let _$l10 = (a$l0, tuple$l3, nested$l5)
    }
    unconstrained fn lambda$f1(x$l7: [Field; 1]) -> [Field; 1] {
        x$l7
    }
    unconstrained fn lambda$f2(x$l8: [Field; 1]) -> [Field; 1] {
        x$l8
    }
    ");
}

#[test]
fn clone_nested_array_in_lvalue() {
    let src = "
    unconstrained fn main(i: u32, j: u32) -> pub u32 {
        let mut a = [[1, 2], [3, 4]];
        a[i][j] = 5;
        a[0][0]
    }
    ";

    let program = get_monomorphized(src).unwrap();
    // A clone is inserted in the lvalue position, because the array could be aliased somewhere else,
    // and even if it was cloned, the RC was only increased for the outer array, not the nested one.
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(i$l0: u32, j$l1: u32) -> pub u32 {
        let mut a$l2 = [[1, 2], [3, 4]];
        a$l2[i$l0].clone()[j$l1] = 5;
        a$l2[0][0]
    }
    ");
}

#[test]
fn pure_builtin_args_get_cloned() {
    let src = "
    unconstrained fn main() -> pub u32 {
        let a = [1, 2, 3];
        let x = a.len();
        let y = a.len();
        x + y
    }
    ";

    let program = get_monomorphized_with_stdlib(src, &[stdlib_src::ARRAY_LEN]).unwrap();

    // The ownership pass doesn't know which builtin functions are pure and which ones
    // modifies the arguments, so this optimization is deferred to the SSA generation.
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0() -> pub u32 {
        let a$l0 = [1, 2, 3];
        let x$l1 = len$array_len(a$l0.clone());
        let y$l2 = len$array_len(a$l0);
        (x$l1 + y$l2)
    }
    ");
}

#[test]
fn while_condition_with_array_last_use() {
    // The arrays last use should be in the while condition
    let src = "
    unconstrained fn main() {
        let arr = [1, 2, 3];
        while check(arr) {
            break;
        }
    }

    fn check(a: [Field; 3]) -> bool {
        false
    }
    ";

    let program = get_monomorphized(src).unwrap();
    // `arr` should be cloned in the while condition since it's evaluated multiple times
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0() -> () {
        let arr$l0 = [1, 2, 3];
        while check$f1(arr$l0.clone()) {
            break
        }
    }
    unconstrained fn check$f1(a$l1: [Field; 3]) -> bool {
        false
    }
    ");
}

#[test]
fn dereference_immutable_reference() {
    // Dereferencing an immutable reference should work.
    let src = "
    fn main() {
        let x: u32 = 0;
        let y: &u32 = &x;
        let _: u32 = *y;
    }
    ";

    let options = GetProgramOptions {
        frontend_options: FrontendOptions {
            debug_comptime_in_file: None,
            enabled_unstable_features: &[],
            disable_required_unstable_features: true,
        },
        ..Default::default()
    };

    let program = get_monomorphized_with_options(src, options).unwrap();
    insta::assert_snapshot!(program, @r"
    fn main$f0() -> () {
        let x$l0 = 0;
        let y$l1 = (&x$l0);
        let _$l2 = (*y$l1)
    }
    ");
}

#[test]
fn repeated_array_with_nested_array_element() {
    // For repeated arrays like [a; 3] where `a` is an array,
    // reference counting is handled in SSA via inc_rc instructions,
    // so no clones are inserted at the monomorphization level.
    let src = "
    unconstrained fn main() {
        let a = [1, 2];
        let b = [a; 3];
        use_var(b);
    }

    fn use_var<T>(_x: T) {}
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0() -> () {
        let a$l0 = [1, 2];
        let b$l1 = [a$l0; 3];
        use_var$f1(b$l1);
    }
    unconstrained fn use_var$f1(_x$l2: [[Field; 2]; 3]) -> () {
    }
    ");
}

#[test]
fn repeated_array_with_non_array_element() {
    // For repeated arrays like [x; 3] where `x` is NOT an array,
    // no special handling is needed.
    let src = "
    unconstrained fn main() {
        let x: Field = 42;
        let b = [x; 3];
        use_var(b);
    }

    fn use_var<T>(_x: T) {}
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0() -> () {
        let x$l0 = 42;
        let b$l1 = [x$l0; 3];
        use_var$f1(b$l1);
    }
    unconstrained fn use_var$f1(_x$l2: [Field; 3]) -> () {
    }
    ");
}

#[test]
fn overwrite_at_top_level() {
    let src = "
    unconstrained fn main() {
        let mut v = @[1, 2, 3];
        let w = identity(v);
        v = identity(v);
        v = identity((v, v)).0;
        use_var(v);
    }

    fn use_var<T>(_x: T) {}
    fn identity<T>(x: T) -> T { x }
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0() -> () {
        let mut v$l0 = @[1, 2, 3];
        let w$l1 = identity$f1(v$l0.clone());
        v$l0 = identity$f1(v$l0);
        v$l0 = identity$f2((v$l0.clone(), v$l0)).0;
        use_var$f3(v$l0);
    }
    unconstrained fn identity$f1(x$l2: [Field]) -> [Field] {
        x$l2
    }
    unconstrained fn identity$f2(x$l3: ([Field], [Field])) -> ([Field], [Field]) {
        x$l3
    }
    unconstrained fn use_var$f3(_x$l4: [Field]) -> () {
    }
    ");
}

#[test]
fn overwrite_in_loop() {
    let src = "
    unconstrained fn main() {
        let mut v = @[1, 2, 3];
        for _ in 0 .. 5 {
            v = identity(v);
            use_var(v);
        }
        use_var(v);
    }

    fn use_var<T>(_x: T) {}
    fn identity<T>(x: T) -> T { x }
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0() -> () {
        let mut v$l0 = @[1, 2, 3];
        for _$l1 in 0 .. 5 {
            v$l0 = identity$f1(v$l0);
            use_var$f2(v$l0.clone());
        };
        use_var$f2(v$l0);
    }
    unconstrained fn identity$f1(x$l2: [Field]) -> [Field] {
        x$l2
    }
    unconstrained fn use_var$f2(_x$l3: [Field]) -> () {
    }
    ");
}

#[test]
fn overwrite_conditional() {
    let src = "
    unconstrained fn main(cond: bool) {
        let mut v = @[1, 2, 3];
        if cond {
            v = identity(v);
        } else {
            use_var(v);
        }
        use_var(v);
    }

    fn use_var<T>(_x: T) {}
    fn identity<T>(x: T) -> T { x }
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(cond$l0: bool) -> () {
        let mut v$l1 = @[1, 2, 3];
        if cond$l0 {
            v$l1 = identity$f1(v$l1)
        } else {
            use_var$f2(v$l1.clone());
        };
        use_var$f2(v$l1);
    }
    unconstrained fn identity$f1(x$l2: [Field]) -> [Field] {
        x$l2
    }
    unconstrained fn use_var$f2(_x$l3: [Field]) -> () {
    }
    ");
}

/// Regression: reassigning inside a while loop then using in a for loop.
/// The while loop must restore the variable's loop_index so the for loop
/// correctly sees the variable as defined outside and clones it.
#[test]
fn while_reassign_then_for_loop() {
    let src = "
    unconstrained fn main(cond: bool) {
        let mut v = @[1, 2, 3];
        while cond {
            v = identity(v);
            break;
        }
        for _ in 0..2 {
            use_var(v);
        }
    }

    fn use_var<T>(_x: T) {}
    fn identity<T>(x: T) -> T { x }
    ";

    let program = get_monomorphized(src).unwrap();
    // v must be cloned inside the for loop since it's used across iterations
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(cond$l0: bool) -> () {
        let mut v$l1 = @[1, 2, 3];
        while cond$l0 {
            v$l1 = identity$f1(v$l1);
            break
        };
        for _$l2 in 0 .. 2 {
            use_var$f2(v$l1.clone());
        }
    }
    unconstrained fn identity$f1(x$l3: [Field]) -> [Field] {
        x$l3
    }
    unconstrained fn use_var$f2(_x$l4: [Field]) -> () {
    }
    ");
}

/// Regression test for https://github.com/noir-lang/noir/issues/11574
/// When the reassignment is in the else branch, uses in the then branch
/// should still get cloned if the variable is used after the if/else.
#[test]
fn overwrite_conditional_swapped() {
    let src = "
    unconstrained fn main(cond: bool) {
        let mut v = @[1, 2, 3];
        if cond {
            use_var(v);
        } else {
            v = identity(v);
        }
        use_var(v);
    }

    fn use_var<T>(_x: T) {}
    fn identity<T>(x: T) -> T { x }
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(cond$l0: bool) -> () {
        let mut v$l1 = @[1, 2, 3];
        if cond$l0 {
            use_var$f1(v$l1.clone());
        } else {
            v$l1 = identity$f2(v$l1)
        };
        use_var$f1(v$l1);
    }
    unconstrained fn use_var$f1(_x$l2: [Field]) -> () {
    }
    unconstrained fn identity$f2(x$l3: [Field]) -> [Field] {
        x$l3
    }
    ");
}

#[test]
fn match_with_reassignment_in_one_arm() {
    let src = "
    unconstrained fn main(x: u32) {
        let mut v = @[1, 2, 3];
        let result: () = match x {
            0 => { v = identity(v); },
            1 => { use_var(v); },
            _ => { use_var(v); },
        };
        use_var(v);
    }

    fn use_var<T>(_x: T) {}
    fn identity<T>(x: T) -> T { x }
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(x$l0: u32) -> () {
        let mut v$l1 = @[1, 2, 3];
        let result$l4 = {
            let internal variable$l2 = x$l0;
            match $2 {
                0 => {
                    v$l1 = identity$f1(v$l1)
                },
                1 => {
                    use_var$f2(v$l1.clone());
                },
                _ => {
                    let _$l3 = internal variable$l2;
                    {
                        use_var$f2(v$l1.clone());
                    }
                },
            }
        };
        use_var$f2(v$l1);
    }
    unconstrained fn identity$f1(x$l5: [Field]) -> [Field] {
        x$l5
    }
    unconstrained fn use_var$f2(_x$l6: [Field]) -> () {
    }
    ");
}

#[test]
fn nested_if_reassignment() {
    let src = "
    unconstrained fn main(c1: bool, c2: bool) {
        let mut v = @[1, 2, 3];
        if c1 {
            if c2 {
                v = identity(v);
            }
            use_var(v);
        } else {
            use_var(v);
        }
        use_var(v);
    }

    fn use_var<T>(_x: T) {}
    fn identity<T>(x: T) -> T { x }
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(c1$l0: bool, c2$l1: bool) -> () {
        let mut v$l2 = @[1, 2, 3];
        if c1$l0 {
            if c2$l1 {
                v$l2 = identity$f1(v$l2)
            };
            use_var$f2(v$l2.clone());
        } else {
            use_var$f2(v$l2.clone());
        };
        use_var$f2(v$l2);
    }
    unconstrained fn identity$f1(x$l3: [Field]) -> [Field] {
        x$l3
    }
    unconstrained fn use_var$f2(_x$l4: [Field]) -> () {
    }
    ");
}

#[test]
fn sequential_conditional_reassignments() {
    let src = "
    unconstrained fn main(c1: bool, c2: bool) {
        let mut v = @[1, 2, 3];
        if c1 {
            v = identity(v);
        }
        if c2 {
            v = identity(v);
        }
        use_var(v);
    }

    fn use_var<T>(_x: T) {}
    fn identity<T>(x: T) -> T { x }
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(c1$l0: bool, c2$l1: bool) -> () {
        let mut v$l2 = @[1, 2, 3];
        if c1$l0 {
            v$l2 = identity$f1(v$l2)
        };
        if c2$l1 {
            v$l2 = identity$f1(v$l2)
        };
        use_var$f2(v$l2);
    }
    unconstrained fn identity$f1(x$l3: [Field]) -> [Field] {
        x$l3
    }
    unconstrained fn use_var$f2(_x$l4: [Field]) -> () {
    }
    ");
}

#[test]
fn both_branches_reassign() {
    let src = "
    unconstrained fn main(cond: bool) {
        let mut v = @[1, 2, 3];
        if cond {
            v = identity(v);
        } else {
            v = identity(v);
        }
        use_var(v);
    }

    fn use_var<T>(_x: T) {}
    fn identity<T>(x: T) -> T { x }
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(cond$l0: bool) -> () {
        let mut v$l1 = @[1, 2, 3];
        if cond$l0 {
            v$l1 = identity$f1(v$l1)
        } else {
            v$l1 = identity$f1(v$l1)
        };
        use_var$f2(v$l1);
    }
    unconstrained fn identity$f1(x$l2: [Field]) -> [Field] {
        x$l2
    }
    unconstrained fn use_var$f2(_x$l3: [Field]) -> () {
    }
    ");
}

#[test]
fn loop_with_conditional_reassignment() {
    let src = "
    unconstrained fn main(cond: bool) {
        let mut v = @[1, 2, 3];
        for _ in 0..3 {
            if cond {
                v = identity(v);
            }
            use_var(v);
        }
        use_var(v);
    }

    fn use_var<T>(_x: T) {}
    fn identity<T>(x: T) -> T { x }
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(cond$l0: bool) -> () {
        let mut v$l1 = @[1, 2, 3];
        for _$l2 in 0 .. 3 {
            if cond$l0 {
                v$l1 = identity$f1(v$l1)
            };
            use_var$f2(v$l1.clone());
        };
        use_var$f2(v$l1);
    }
    unconstrained fn identity$f1(x$l3: [Field]) -> [Field] {
        x$l3
    }
    unconstrained fn use_var$f2(_x$l4: [Field]) -> () {
    }
    ");
}

/// `x.0.0` and `x.0` overlap because `x.0` takes the entire sub-tuple that
/// `x.0.0` also reaches into. A clone is genuinely required here.
#[test]
fn clone_needed_when_extract_paths_overlap() {
    let src = "
    unconstrained fn main() {
        let x = (([1], [2]), [3]);
        let _a = x.0.0;
        let _b = x.0;
    }
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0() -> () {
        let x$l0 = (([1], [2]), [3]);
        let _a$l1 = x$l0.0.0.clone();
        let _b$l2 = x$l0.0
    }
    ");
}

/// Single-field struct: extracting the only field is a move — no clone needed
/// since there are no other fields that could alias.
#[test]
fn single_field_struct_extraction_is_optimal() {
    let src = "
    struct Wrapper { inner: [Field; 3] }

    unconstrained fn main() {
        let w = Wrapper { inner: [1, 2, 3] };
        let _x = w.inner;
    }
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0() -> () {
        let w$l1 = {
            let inner$l0 = [1, 2, 3];
            (inner$l0)
        };
        let _x$l2 = w$l1.0
    }
    ");
}

#[test]
fn clones_non_moved_variable_because_of_reference() {
    // Here `let y = arr;` shouldn't be considered a move of `arr` because
    // `z` keeps a reference to `arr`;
    let src = "
    unconstrained fn main(mut arr: [u32; 3], idx: u32) {
        let z: &mut [u32; 3] = &mut arr;
        let y = arr;
        (*z)[idx] = 100;
    }
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(mut arr$l0: [u32; 3], idx$l1: u32) -> () {
        let z$l2 = (&mut arr$l0);
        let y$l3 = arr$l0.clone();
        (*z$l2)[idx$l1] = 100
    }
    ");
}

#[test]
fn clone_inserted_on_index_then_collection() {
    let src = "
    unconstrained fn main() {
        let a = [10];
        foo(a)[bar(a)];
    }
    unconstrained fn foo(a: [u32; 1]) -> [u32; 1] { a }
    unconstrained fn bar(_a: [u32; 1]) -> u32 { 0 }
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0() -> () {
        let a$l0 = [10];
        foo$f1(a$l0)[bar$f2(a$l0.clone())];
    }
    unconstrained fn foo$f1(a$l1: [u32; 1]) -> [u32; 1] {
        a$l1
    }
    unconstrained fn bar$f2(_a$l2: [u32; 1]) -> u32 {
        0
    }
    ");
}

#[test]
fn clone_inserted_on_index_then_collection_in_lvalue() {
    let src = "
    unconstrained fn main() {
        let mut a = [10];
        a[bar(a)] = 20;
    }
    unconstrained fn bar(_a: [u32; 1]) -> u32 { 0 }
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0() -> () {
        let mut a$l0 = [10];
        {
            let i_0$l1 = bar$f1(a$l0.clone());
            a$l0[i_0$l1] = 20
        }
    }
    unconstrained fn bar$f1(_a$l2: [u32; 1]) -> u32 {
        0
    }
    ");
}

/// Nested array index: `arr[0][1]` on a 3D array. When the base variable has
/// no further uses, the indexed element can be moved without cloning.
#[test]
fn nested_array_double_index_is_moved() {
    let src = "
    unconstrained fn main() {
        let arr = [[[1, 2], [3, 4]], [[5, 6], [7, 8]]];
        let _val = arr[0][1];
    }
    ";

    let program = get_monomorphized(src).unwrap();

    // No clone needed — arr is not used again and the intermediate arr[0] is a temporary
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0() -> () {
        let arr$l0 = [[[1, 2], [3, 4]], [[5, 6], [7, 8]]];
        let _val$l1 = arr$l0[0][1]
    }
    ");
}

#[test]
fn clones_non_moved_variable_because_of_reference_even_if_unused() {
    // Similar to the above test, but shows that even when taking a reference
    // to a variable and never using that reference again, the variable will still
    // be cloned instead of moved.
    let src = "
    unconstrained fn main(mut arr: [u32; 3], idx: u32) {
        let _: &mut [u32; 3] = &mut arr;
        let y = arr;
    }
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(mut arr$l0: [u32; 3], idx$l1: u32) -> () {
        let _$l2 = (&mut arr$l0);
        let y$l3 = arr$l0.clone()
    }
    ");
}

#[test]
fn clones_non_moved_variable_because_of_field_reference() {
    // `&mut w2.arr` takes a reference to a field of `w2`, so `w2` is aliased.
    // `let y = w2` must clone `w2` (not move) so that the subsequent write through
    // `z` triggers copy-on-write and leaves `y` unchanged.
    let src = "
    struct Wrapper {
        arr: [u32; 3],
    }
    unconstrained fn main(w: Wrapper, idx: u32) {
        let mut w2: Wrapper = w;
        let z: &mut [u32; 3] = &mut w2.arr;
        let y: Wrapper = w2;
        (*z)[idx] = 100;
    }
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(w$l0: ([u32; 3],), idx$l1: u32) -> () {
        let mut w2$l2 = w$l0;
        let z$l3 = (&mut w2$l2.0);
        let y$l4 = w2$l2.clone();
        (*z$l3)[idx$l1] = 100
    }
    ");
}

#[test]
fn reference_passed_to_call_does_not_prevent_move() {
    // `foo(&mut array)` passes a temporary reference that only lives for the call.
    // After `foo` returns, `array` is no longer aliased, so the final `array` can be
    // moved into `use_array` (no clone needed).
    let src = "
    unconstrained fn main(mut array: [Field; 3]) {
        foo(&mut array);
        use_array(array);
    }

    fn foo(_: &mut [Field; 3]) {}
    fn use_array(_: [Field; 3]) {}
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(mut array$l0: [Field; 3]) -> () {
        foo$f1((&mut array$l0));;
        use_array$f2(array$l0);
    }
    unconstrained fn foo$f1(_$l1: &mut [Field; 3]) -> () {
    }
    unconstrained fn use_array$f2(_$l2: [Field; 3]) -> () {
    }
    ");
}

#[test]
fn reference_passed_to_call_returning_reference_prevents_move() {
    // When a call returns a reference type, the passed `&mut array` might be returned
    // and stored in a binding that outlives the call. So `array` must be treated as
    // aliased and subsequent copies must clone.
    let src = "
    unconstrained fn main(mut array: [Field; 3]) {
        let _r: &mut [Field; 3] = identity(&mut array);
        let y = array;
        use_array(y);
    }

    fn identity(x: &mut [Field; 3]) -> &mut [Field; 3] { x }
    fn use_array(_: [Field; 3]) {}
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(mut array$l0: [Field; 3]) -> () {
        let _r$l1 = identity$f1((&mut array$l0));
        let y$l2 = array$l0.clone();
        use_array$f2(y$l2);
    }
    unconstrained fn identity$f1(x$l3: &mut [Field; 3]) -> &mut [Field; 3] {
        x$l3
    }
    unconstrained fn use_array$f2(_$l4: [Field; 3]) -> () {
    }
    ");
}

#[test]
fn reference_passed_alongside_mut_ref_to_ref_prevents_move() {
    // `call(&mut array, &mut z)` where `z: &mut [Field; 3]` — the second argument has type
    // `&mut &mut [Field; 3]`, so the callee could write `array`'s reference into `*z`,
    // making `array` aliased beyond the call. Therefore `array` must be cloned on copy.
    let src = "
    unconstrained fn main(mut array: [Field; 3]) {
        let mut z = &mut [1, 2, 3];
        call(&mut array, &mut z);
        let y = array;
        use_array(y);
    }

    fn call(x: &mut [Field; 3], y: &mut &mut [Field; 3]) {
        *y = x;
    }
    fn use_array(_: [Field; 3]) {}
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(mut array$l0: [Field; 3]) -> () {
        let mut z$l1 = (&mut [1, 2, 3]);
        call$f1((&mut array$l0), (&mut z$l1));;
        let y$l2 = array$l0.clone();
        use_array$f2(y$l2);
    }
    unconstrained fn call$f1(x$l3: &mut [Field; 3], y$l4: &mut &mut [Field; 3]) -> () {
        *y$l4 = x$l3
    }
    unconstrained fn use_array$f2(_$l5: [Field; 3]) -> () {
    }
    ");
}

#[test]
fn reference_passed_alongside_struct_with_mut_ref_to_ref_prevents_move() {
    // `call(&mut array, container)` where `container: Container` holds a field of type
    // `&mut &mut [Field; 3]`. Even though the second argument is not directly `&mut &mut T`,
    // the callee can reach the inner `&mut &mut [Field; 3]` through the struct field and
    // write `array`'s reference into it. So `array` must be cloned on copy.
    let src = "
    struct Container {
        slot: &mut &mut [Field; 3],
    }
    unconstrained fn main(mut array: [Field; 3]) {
        let mut z: &mut [Field; 3] = &mut [1, 2, 3];
        let container = Container { slot: &mut z };
        call(&mut array, container);
        let y = array;
        use_array(y);
    }

    fn call(x: &mut [Field; 3], c: Container) {
        *(c.slot) = x;
    }
    fn use_array(_: [Field; 3]) {}
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(mut array$l0: [Field; 3]) -> () {
        let mut z$l1 = (&mut [1, 2, 3]);
        let container$l3 = {
            let slot$l2 = (&mut z$l1);
            (slot$l2)
        };
        call$f1((&mut array$l0), container$l3);;
        let y$l4 = array$l0.clone();
        use_array$f2(y$l4);
    }
    unconstrained fn call$f1(x$l5: &mut [Field; 3], c$l6: (&mut &mut [Field; 3],)) -> () {
        *c$l6.0 = x$l5
    }
    unconstrained fn use_array$f2(_$l7: [Field; 3]) -> () {
    }
    ");
}

#[test]
fn call_with_extract_tuple_field_args_does_not_prevent_move() {
    // Mirrors the `try_resize` pattern in UHashMap: `insert(&mut new_map, entry.0, entry.1)`
    // where `entry` is a tuple. The arguments `entry.0` and `entry.1` are `ExtractTupleField`
    // expressions. Even though their types cannot be resolved as Ident/Unary, they are plain
    // Field values — not capable of storing a reference — so the conservative fallback must NOT
    // trigger, and `new_map` must be movable at `*dest = new_map` (no clone).
    let src = "
    unconstrained fn main(mut dest: [Field; 3]) {
        let mut new_map: [Field; 3] = [0, 0, 0];
        let entries: [(Field, Field); 2] = [(1, 2), (3, 4)];
        for i in 0..2 {
            let entry = entries[i];
            insert(&mut new_map, entry.0, entry.1);
        }
        dest = new_map;
    }

    fn insert(map: &mut [Field; 3], key: Field, value: Field) {
        map[0] = key + value;
    }
    ";

    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(mut dest$l0: [Field; 3]) -> () {
        let mut new_map$l1 = [0, 0, 0];
        let entries$l2 = [(1, 2), (3, 4)];
        for i$l3 in 0 .. 2 {
            let entry$l4 = entries$l2[i$l3];
            insert$f1((&mut new_map$l1), entry$l4.0, entry$l4.1);
        };
        dest$l0 = new_map$l1
    }
    unconstrained fn insert$f1(map$l5: &mut [Field; 3], key$l6: Field, value$l7: Field) -> () {
        (*map$l5)[0] = (key$l6 + value$l7)
    }
    ");
}

// Regression tests for incorrect "confirmed moves" on assignments to variables
// declared in an outer loop scope. In all cases below a prior use of `x` must
// produce a clone because the assignment that would "confirm" the move either
// may not execute (loop guard false / conditional) or belongs to a different branch.

#[test]
fn no_confirmed_move_for_assignment_in_dead_loop() {
    // `x = [1,2,3]` is inside a loop that never runs, so the prior `let y = x`
    // must still clone `x`.
    let src = "
    unconstrained fn main(arr: [Field; 3]) {
        let mut x = arr;
        let y = x;
        while (false) {
            x = [1, 2, 3];
        };
        use_var(y);
        use_var(x);
    }

    fn use_var<T>(_x: T) {}
    ";
    let program = get_monomorphized(src).unwrap();
    // `x` in `let y = x` must be cloned because the loop assignment is not guaranteed to execute.
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(arr$l0: [Field; 3]) -> () {
        let mut x$l1 = arr$l0;
        let y$l2 = x$l1.clone();
        while false {
            x$l1 = [1, 2, 3]
        };
        use_var$f1(y$l2);;
        use_var$f1(x$l1);
    }
    unconstrained fn use_var$f1(_x$l3: [Field; 3]) -> () {
    }
    ");
}

#[test]
fn no_confirmed_move_before_loop_when_variable_will_be_used_in_loop() {
    // `x` is first seen inside the loop (in reverse), a fact which should prevent
    // it from being moved into `y` without a clone.
    let src = "
    unconstrained fn main(arr: [Field; 3]) {
        let mut x = arr;
        let mut y = x;
        y[0] = 100;
        for _ in 0 .. 2 {
            use_var(x);
        };
    }

    fn use_var<T>(_x: T) {}
    ";
    let program = get_monomorphized(src).unwrap();
    // `x` in `let y = x` must be cloned otherwise the `x` in the loop would see the modification.
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(arr$l0: [Field; 3]) -> () {
        let mut x$l1 = arr$l0;
        let mut y$l2 = x$l1.clone();
        y$l2[0] = 100;
        for _$l3 in 0 .. 2 {
            use_var$f1(x$l1.clone());
        }
    }
    unconstrained fn use_var$f1(_x$l4: [Field; 3]) -> () {
    }
    ");
}

#[test]
fn no_confirmed_move_for_assignment_in_unreachable_branch() {
    // `x = [1,2,3]` is reachable only through an `if false` branch; `let y = x`
    // must still clone.
    let src = "
    unconstrained fn main(arr: [Field; 3]) {
        let mut x = arr;
        let y = x;
        if (false) {
            while (true) {
                x = [1, 2, 3];
            };
        }
        use_var(y);
        use_var(x);
    }

    fn use_var<T>(_x: T) {}
    ";
    let program = get_monomorphized(src).unwrap();
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(arr$l0: [Field; 3]) -> () {
        let mut x$l1 = arr$l0;
        let y$l2 = x$l1.clone();
        if false {
            while true {
                x$l1 = [1, 2, 3]
            }
        };
        use_var$f1(y$l2);;
        use_var$f1(x$l1);
    }
    unconstrained fn use_var$f1(_x$l3: [Field; 3]) -> () {
    }
    ");
}

#[test]
fn no_confirmed_move_for_assignment_in_other_branch() {
    // `y = x` is in the `if` branch; `x = [1,2,3]` is inside a `while` in the
    // `else` branch.  The two branches are mutually exclusive so the loop
    // assignment must not mark `y = x` as a confirmed move.
    let src = "
    unconstrained fn main(arr: [Field; 3], i: u32) {
        let mut x = arr;
        let mut y = [1, 2, 3];
        if (i > 0) {
            y = x;
        } else {
            let mut j = 0;
            while (j < 1) {
                j += 1;
                x = [1, 2, 3];
            };
        };
        use_var(x);
        use_var(y);
    }

    fn use_var<T>(_x: T) {}
    ";
    let program = get_monomorphized(src).unwrap();
    // `x` in `y = x` must be cloned.
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(arr$l0: [Field; 3], i$l1: u32) -> () {
        let mut x$l2 = arr$l0;
        let mut y$l3 = [1, 2, 3];
        if (i$l1 > 0) {
            y$l3 = x$l2.clone()
        } else {
            let mut j$l4 = 0;
            while (j$l4 < 1) {
                j$l4 = (j$l4 + 1);
                x$l2 = [1, 2, 3]
            }
        };;
        use_var$f1(x$l2);;
        use_var$f1(y$l3);
    }
    unconstrained fn use_var$f1(_x$l5: [Field; 3]) -> () {
    }
    ");
}

#[test]
fn no_confirmed_move_for_outer_use_before_loop_with_self_assignment() {
    // `let y = x` occurs before a loop that contains `x = x`.
    // The prior use `y = x` must clone `x` because the loop may not execute.
    // However, the `x` on the RHS of `x = x` inside the loop is a genuine last
    // use and must NOT be cloned (it is confirmed as a move within the loop body).
    let src = "
    unconstrained fn main(arr: [Field; 3]) {
        let mut x = arr;
        let y = x;
        let mut i = 0;
        while i < 1 {
            i += 1;
            x = x;
        }
        use_var(y);
        use_var(x);
    }

    fn use_var<T>(_x: T) {}
    ";
    let program = get_monomorphized(src).unwrap();
    // `x` in `let y = x` must be cloned; `x` in `x = x` (RHS) must NOT be cloned.
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(arr$l0: [Field; 3]) -> () {
        let mut x$l1 = arr$l0;
        let y$l2 = x$l1.clone();
        let mut i$l3 = 0;
        while (i$l3 < 1) {
            i$l3 = (i$l3 + 1);
            x$l1 = x$l1
        };
        use_var$f1(y$l2);;
        use_var$f1(x$l1);
    }
    unconstrained fn use_var$f1(_x$l4: [Field; 3]) -> () {
    }
    ");
}

#[test]
fn no_confirmed_move_for_dead_code_after_break() {
    // `x = [4,5,6]` is dead code after `break`. The earlier `let mut y = x`
    // must clone `x` because `x = [4,5,6]` never actually executes.
    let src = "
    unconstrained fn main(arr: [Field; 3]) {
        let mut x = arr;
        let mut i = 0;
        while (i < 3) {
            i += 1;
            x = x;
            let mut y = x;
            y[0] = 100;
            use_var(y);
            break;
            x = [4, 5, 6];
        }
        use_var(x);
    }

    fn use_var<T>(_x: T) {}
    ";
    let program = get_monomorphized(src).unwrap();
    // `x` in `let mut y = x` must be cloned — the dead `x = [4,5,6]` after break
    // must NOT cause it to be treated as a confirmed move.
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(arr$l0: [Field; 3]) -> () {
        let mut x$l1 = arr$l0;
        let mut i$l2 = 0;
        while (i$l2 < 3) {
            i$l2 = (i$l2 + 1);
            x$l1 = x$l1;
            let mut y$l3 = x$l1.clone();
            y$l3[0] = 100;
            use_var$f1(y$l3);;
            break;
            x$l1 = [4, 5, 6]
        };
        use_var$f1(x$l1);
    }
    unconstrained fn use_var$f1(_x$l4: [Field; 3]) -> () {
    }
    ");
}

#[test]
fn confirmed_move_for_variable_reassigned_in_the_loop() {
    // Same as no_confirmed_move_for_dead_code_after_break, without the `break`.
    // `x = [4,5,6]` is not dead; the earlier `let mut y = x`
    // doesn't need to clone `x` because `x = [4,5,6]` will execute.
    let src = "
    unconstrained fn main(arr: [Field; 3]) {
        let mut x = arr;
        let mut i = 0;
        while (i < 3) {
            i += 1;
            x = x;
            let mut y = x;
            y[0] = 100;
            use_var(y);
            x = [4, 5, 6];
        }
        use_var(x);
    }

    fn use_var<T>(_x: T) {}
    ";
    let program = get_monomorphized(src).unwrap();
    // `x` in `let mut y = x` doesn't get cloned.
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(arr$l0: [Field; 3]) -> () {
        let mut x$l1 = arr$l0;
        let mut i$l2 = 0;
        while (i$l2 < 3) {
            i$l2 = (i$l2 + 1);
            x$l1 = x$l1;
            let mut y$l3 = x$l1;
            y$l3[0] = 100;
            use_var$f1(y$l3);;
            x$l1 = [4, 5, 6]
        };
        use_var$f1(x$l1);
    }
    unconstrained fn use_var$f1(_x$l4: [Field; 3]) -> () {
    }
    ");
}
