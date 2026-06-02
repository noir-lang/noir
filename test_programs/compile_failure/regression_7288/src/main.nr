unconstrained fn foo<T>(x: T) -> T {
    x
}

fn bar<T>(x: T) -> T {
    unsafe {
        foo(x)
    }
}

fn main(x: Field, y: pub Field) {
    let mut y = 0;
    let x = &mut y;
    assert(*(unsafe { bar(x) }) == 0);
}
