// Regression for https://github.com/noir-lang/noir/issues/12221
fn main() -> pub Field {
    // Safety: testing context
    unsafe {
        foo()
    }
}

unconstrained fn foo() -> Field {
    bar(&mut 0xcafebabe, 0xdeadbeef, true)
}

unconstrained fn bar(mut a: &mut Field, mut b: Field, do_alias: bool) -> Field {
    let mut i = 0;
    loop {
        if i == 2 {
            break;
        }
        i = i + 1;

        b = 0xdeadbeef;
        b = *a;

        // Conditionally make a point to b (loop-carried alias via conditional branch).
        if do_alias {
            a = &mut b;
        }
    }
    b
}
