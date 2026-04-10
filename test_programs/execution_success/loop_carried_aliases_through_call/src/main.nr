fn main() -> pub (Field, Field) {
    // Safety: testing context
    unsafe {
        let result_call_input = foo_call_input();
        let result_call_return = foo_call_return();
        (result_call_input, result_call_return)
    }
}

// Issue #12217: loop-carried alias established via a call that stores
// a reference into a double-reference argument.
unconstrained fn foo_call_input() -> Field {
    bar_call_input(&mut 0xcafebabe, 0xdeadbeef)
}

unconstrained fn make_alias(val: &mut Field, container: &mut &mut Field) {
    *container = val;
}

unconstrained fn bar_call_input(mut a: &mut Field, mut b: Field) -> Field {
    let mut i = 0;
    loop {
        if i == 2 {
            break;
        }
        i += 1;

        b = 0xdeadbeef;
        b = *a;

        // Creates loop-carried alias via function call:
        // after this call, a points to b.
        make_alias(&mut b, &mut a);
    }
    b
}

// Issue #12219: loop-carried alias established via a call that returns
// a reference which aliases a call input.
unconstrained fn foo_call_return() -> Field {
    bar_call_return(&mut 0xcafebabe, 0xdeadbeef)
}

unconstrained fn identity(r: &mut Field) -> &mut Field {
    r
}

unconstrained fn bar_call_return(mut a: &mut Field, mut b: Field) -> Field {
    let mut i = 0;
    loop {
        if i == 2 {
            break;
        }
        i += 1;

        b = 0xdeadbeef;
        b = *a;

        // Creates loop-carried alias via function return:
        // identity returns its input, so a now points to b.
        a = identity(&mut b);
    }
    b
}
