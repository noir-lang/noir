#![cfg(test)]
//! The easiest way to test this pass is a bit indirect. We have to run
//! ownership in its entirety and look at where the clones are inserted.
//! Testing e.g. the last_use pass directly is difficult since it returns
//! sets of IdentIds which can't be matched to the source code easily.

use crate::{
    hir::{def_collector::dc_crate::CompilationError, resolution::errors::ResolverError},
    test_utils::{get_monomorphized, get_monomorphized_with_error_filter},
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
        let _$l2 = (*(&mut a$l0))[0];
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
fn array_len_does_not_clone() {
    // Punting the builtin array_len, because these snippets don't have access to stdlib;
    // trying to use `a.len()` would result in a panic.
    let src = "
    #[builtin(array_len)]
    fn len<T, let N: u32>(a: [T; N]) -> u32 { }

    unconstrained fn main() -> pub u32 {
        let a = [1, 2, 3];
        let x = len(a);
        let y = len(a);
        x + y
    }
    ";

    let program = get_monomorphized_with_error_filter(src, |err| {
        matches!(
            err,
            CompilationError::ResolverError(ResolverError::LowLevelFunctionOutsideOfStdlib { .. })
        )
    })
    .unwrap();

    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0() -> pub u32 {
        let a$l0 = [1, 2, 3];
        let x$l1 = len$array_len(a$l0);
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
