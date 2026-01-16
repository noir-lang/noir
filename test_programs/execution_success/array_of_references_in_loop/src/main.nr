fn main() -> pub bool {
    // Safety: testing context
    unsafe {
        foo()
    }
}

unconstrained fn foo() -> bool {
    let mut a: [[&mut bool; 1]; 1] = [[(&mut true)]];
    let mut i: bool = true;
    while i {
        i = false;
        a[0] = [&mut true];
    }
    *a[0][0]
}
