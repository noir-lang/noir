fn main(mut a: bool) {
    let _: [&bool; 1] = [(&a); 1];
    let _ = foo(a);
}

fn foo(mut a: bool) -> &bool {
    &a
}
