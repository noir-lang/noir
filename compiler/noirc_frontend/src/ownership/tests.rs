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
    unconstrained fn use_var$f2(_x$l4: [i32; 1]) -> () {
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
    unconstrained fn use_var$f1(_x$l2: [i32; 2]) -> () {
    }
    ");
}
