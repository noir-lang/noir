// Regression for https://github.com/noir-lang/noir/issues/12220
fn main() -> pub Field {
    // Safety: testing context
    unsafe {
        foo()
    }
}

unconstrained fn foo() -> Field {
    // idx=0 selects &mut b from the array, creating the loop-carried alias.
    bar(&mut 0xcafebabe, 0xdeadbeef, 0)
}

unconstrained fn bar(mut a: &mut Field, mut b: Field, idx: u32) -> Field {
    let mut i = 0;
    loop {
        if i == 2 {
            break;
        }
        i = i + 1;

        b = 0xdeadbeef;
        b = *a;

        // Build a 2-element array so the size-1 optimisation does not apply,
        // then extract element `idx` (= 0 at runtime) to get &mut b.
        let arr: [&mut Field; 2] = [&mut b, a];
        a = arr[idx];
    }
    b
}
