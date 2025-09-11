#![cfg(test)]
//! The easiest way to test this pass is a bit indirect. We have to run
//! ownership in its entirety and look at where the clones are inserted.
//! Testing e.g. the last_use pass directly is difficult since it returns
//! sets of IdentIds which can't be matched to the source code easily.

use crate::test_utils::get_monomorphized_no_emit_test;

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

    let program = get_monomorphized_no_emit_test(src).unwrap();
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

    let program = get_monomorphized_no_emit_test(src).unwrap();
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

    let program = get_monomorphized_no_emit_test(src).unwrap();
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

    let program = get_monomorphized_no_emit_test(src).unwrap();
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
fn moves_call_array_result() {
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

    let program = get_monomorphized_no_emit_test(src).unwrap();
    // We expect no clones
    insta::assert_snapshot!(program, @r"
    unconstrained fn main$f0(i$l0: u32) -> pub u32 {
        let _a$l1 = foo$f1()[1][0][1];
        let _s$l2 = foo$f1()[1][0][1];
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

    let program = get_monomorphized_no_emit_test(src).unwrap();
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

    let program = get_monomorphized_no_emit_test(src).unwrap();
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

    let program = get_monomorphized_no_emit_test(src).unwrap();
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

    let program = get_monomorphized_no_emit_test(src).unwrap();
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