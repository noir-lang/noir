fn main() {
    // Safety: testing context
    unsafe {
        foo()
    }
}
unconstrained fn foo() {
    if *bar([&mut true]) {
        let mut i = true;
        while i {
            i = false;
        }
    };
}
unconstrained fn bar(mut a: [&mut bool; 1]) -> &mut bool {
    let e: (&mut bool,) = (&mut true,);
    if (*a[0]) { a[0] = e.0; };
    a[0]
}
