unconstrained fn foo<T>(x: T) -> T {
    x
}

fn bar<T>(x: T) -> T {
    // Safety: testing
    unsafe {
        foo(x)
    }
}

fn main(x: Field) {
    let xs = [x, x + 1, x + 2].as_vector();
    // Safety: testing
    unsafe { bar(xs) };
}
